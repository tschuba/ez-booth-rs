use std::borrow::Cow;
use std::convert::Infallible;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use sysinfo::{Pid, ProcessesToUpdate, System};
use warp::http::header::CONTENT_TYPE;
use warp::http::{Response, StatusCode};
use warp::{Filter, Reply};

const APP_NAME: &str = "EZ Booth";
const LOCK_FILE_NAME: &str = "launcher.lock";
const PORT_RANGE: std::ops::RangeInclusive<u16> = 8080..=8089;

enum LockState {
    Acquired(LockFile),
    ActiveInstance(PathBuf),
    Warning(anyhow::Error),
}

struct LockFile {
    path: PathBuf,
}

impl LockFile {
    fn try_acquire() -> LockState {
        match Self::lock_path() {
            Ok(path) => match Self::acquire_at(path) {
                Ok(lock) => LockState::Acquired(lock),
                Err(LockAcquireError::ActiveInstance(path)) => LockState::ActiveInstance(path),
                Err(LockAcquireError::Warning(err)) => LockState::Warning(err),
            },
            Err(err) => LockState::Warning(err),
        }
    }

    fn lock_path() -> Result<PathBuf> {
        let project_dirs = ProjectDirs::from("", "", "ez-booth")
            .context("could not determine a user config directory for ez-booth")?;
        Ok(project_dirs.config_dir().join(LOCK_FILE_NAME))
    }

    fn acquire_at(path: PathBuf) -> Result<Self, LockAcquireError> {
        let parent = path
            .parent()
            .context("lock file path is missing a parent directory")
            .map_err(LockAcquireError::Warning)?;
        fs::create_dir_all(parent)
            .with_context(|| format!("could not create config directory at {}", parent.display()))
            .map_err(LockAcquireError::Warning)?;

        Self::create_lock_file(&path)?;

        Ok(Self { path })
    }

    fn create_lock_file(path: &Path) -> Result<(), LockAcquireError> {
        let pid = process::id().to_string();

        loop {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
            {
                Ok(mut file) => {
                    use std::io::Write;

                    file.write_all(pid.as_bytes())
                        .with_context(|| format!("could not write lock file at {}", path.display()))
                        .map_err(LockAcquireError::Warning)?;
                    return Ok(());
                }
                Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                    Self::validate_or_cleanup_existing(path)?;
                }
                Err(err) => {
                    return Err(LockAcquireError::Warning(anyhow::Error::new(err).context(
                        format!("could not create lock file at {}", path.display()),
                    )));
                }
            }
        }
    }

    fn validate_or_cleanup_existing(path: &Path) -> Result<(), LockAcquireError> {
        if !path.exists() {
            return Ok(());
        }

        let lock_contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(()),
            Err(err) => {
                return Err(LockAcquireError::Warning(anyhow::Error::new(err).context(
                    format!("could not read lock file at {}", path.display()),
                )));
            }
        };

        let pid = match lock_contents.trim().parse::<u32>() {
            Ok(pid) => pid,
            Err(_) => {
                println!("Cleaned up invalid lock file from previous session.");
                fs::remove_file(path)
                    .with_context(|| {
                        format!("could not remove invalid lock file at {}", path.display())
                    })
                    .map_err(LockAcquireError::Warning)?;
                return Ok(());
            }
        };

        if process_is_active(pid) {
            return Err(LockAcquireError::ActiveInstance(path.to_path_buf()));
        }

        println!("Cleaned up stale lock file from previous session.");
        fs::remove_file(path)
            .with_context(|| format!("could not remove stale lock file at {}", path.display()))
            .map_err(LockAcquireError::Warning)?;

        Ok(())
    }

    fn cleanup(&self) {
        let _ = fs::remove_file(&self.path);
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        self.cleanup();
    }
}

enum LockAcquireError {
    ActiveInstance(PathBuf),
    Warning(anyhow::Error),
}

fn process_is_active(pid: u32) -> bool {
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);

    system
        .process(Pid::from_u32(pid))
        .map(|process| {
            process
                .name()
                .to_string_lossy()
                .to_ascii_lowercase()
                .contains("ez-booth")
        })
        .unwrap_or(false)
}

#[tokio::main]
async fn main() -> Result<()> {
    let _lock = match LockFile::try_acquire() {
        LockState::Acquired(lock) => Some(lock),
        LockState::ActiveInstance(path) => {
            eprintln!(
                "Error: Another ez-booth instance is already running.\n\nPlease close the existing instance before starting a new one.\nIf you believe this is an error, delete the lock file at:\n  {}",
                path.display()
            );
            process::exit(1);
        }
        LockState::Warning(err) => {
            eprintln!(
                "Warning: {}.\nMultiple instances may run simultaneously.\n\nContinuing anyway...",
                err
            );
            None
        }
    };

    let port = find_available_port()?;
    let url = format!("http://127.0.0.1:{port}");

    println!("{APP_NAME} is starting...");
    println!("Opening browser at: {url}");
    println!("\nPress Ctrl+C to stop the server.\n");

    if let Err(err) = webbrowser::open(&url) {
        eprintln!("Could not open the browser automatically: {err}");
        eprintln!("Open this URL manually: {url}");
    }

    serve_app(port).await
}

fn find_available_port() -> Result<u16> {
    for port in PORT_RANGE {
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Ok(port);
        }
    }

    Err(anyhow::anyhow!(
        "Could not find an available port (tried 8080-8089).\n\nThis may indicate another ez-booth instance is running,\nor these ports are in use by other applications."
    ))
}

async fn serve_app(port: u16) -> Result<()> {
    let app_dir = app_dir()?;
    let routes =
        warp::get()
            .and(warp::path::full())
            .and_then(move |full_path: warp::path::FullPath| {
                let app_dir = app_dir.clone();
                async move { Ok::<_, Infallible>(serve_path(&app_dir, full_path.as_str()).await) }
            });

    let (_, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], port), async {
            let _ = tokio::signal::ctrl_c().await;
        });

    server.await;
    println!("{APP_NAME} stopped.");
    Ok(())
}

fn app_dir() -> Result<PathBuf> {
    let executable = std::env::current_exe().context("could not determine launcher path")?;
    executable
        .parent()
        .map(Path::to_path_buf)
        .context("launcher path does not have a parent directory")
}

async fn serve_path(app_dir: &Path, request_path: &str) -> impl Reply {
    match read_response(app_dir, request_path).await {
        Ok(response) => response,
        Err(err) => text_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to serve the application: {err}"),
            "text/plain; charset=utf-8",
        ),
    }
}

async fn read_response(app_dir: &Path, request_path: &str) -> Result<Response<Vec<u8>>> {
    let sanitized = sanitize_request_path(request_path);
    let asset_path = app_dir.join(sanitized.as_ref());

    if let Some(response) = file_response_if_exists(&asset_path).await? {
        return Ok(response);
    }

    if !has_extension(sanitized.as_ref()) {
        let fallback = app_dir.join("index.html");
        if let Some(response) = file_response_if_exists(&fallback).await? {
            return Ok(response);
        }
    }

    Ok(text_response(
        StatusCode::NOT_FOUND,
        "File not found".to_string(),
        "text/plain; charset=utf-8",
    ))
}

fn sanitize_request_path(request_path: &str) -> Cow<'_, str> {
    let trimmed = request_path.trim_start_matches('/');
    if trimmed.is_empty() {
        return Cow::Borrowed("index.html");
    }

    let cleaned: Vec<&str> = trimmed
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != "." && *segment != "..")
        .collect();

    if cleaned.is_empty() {
        Cow::Borrowed("index.html")
    } else {
        Cow::Owned(cleaned.join("/"))
    }
}

fn has_extension(path: &str) -> bool {
    Path::new(path).extension().is_some()
}

async fn file_response_if_exists(path: &Path) -> Result<Option<Response<Vec<u8>>>> {
    match tokio::fs::read(path).await {
        Ok(bytes) => Ok(Some(bytes_response(bytes, content_type(path)))),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("could not read {}", path.display())),
    }
}

fn bytes_response(body: Vec<u8>, content_type: &'static str) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type)
        .body(body)
        .expect("response builder should not fail")
}

fn text_response(
    status: StatusCode,
    body: String,
    content_type: &'static str,
) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, content_type)
        .body(body.into_bytes())
        .expect("response builder should not fail")
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("mjs") => "application/javascript; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("json") => "application/json",
        Some("txt") => "text/plain; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}
