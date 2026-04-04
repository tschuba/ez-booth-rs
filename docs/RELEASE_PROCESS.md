# Release Process

This repository publishes stable releases from semantic version tags like `v0.1.0`.

## Overview

- release artifacts are built by GitHub Actions from the tagged commit
- the tagged commit must already contain the matching workspace version in `Cargo.toml`
- release bundles include the platform launcher, the full WASM app bundle, and a usage README
- GitHub Releases also include `checksums.txt` for SHA256 verification

## Versioning

Use stable semantic versioning.

- bump `MAJOR` for intentional breaking changes
- bump `MINOR` for backward-compatible features and operator-visible improvements
- bump `PATCH` for backward-compatible fixes and release-only corrections

This workflow does not publish pre-releases. Tags like `v1.0.0-beta.1` are rejected.

## Before A Release

1. Merge the intended work into `main` through pull requests.
2. Sync local `main` with `origin/main`.
3. Run `./scripts/validate-release.sh`.
4. Decide the next version number.
5. Prepare any optional release notes you want to prepend to the generated GitHub notes.

## Create A Release

Use the helper script from `main`:

```bash
./scripts/create-release.sh 0.1.0
```

What the script does:

1. verifies you are on a clean, up-to-date local `main`
2. updates `[workspace.package] version` in `Cargo.toml`
3. creates a release commit on `main`
4. creates an annotated tag like `v0.1.0`
5. pushes `main` and the tag to `origin`

The annotated tag message is optional. If you provide one, GitHub prepends it to the generated release notes.

## GitHub Actions Release Flow

The release workflow in `.github/workflows/release.yml` runs when a tag matching `v*.*.*` is pushed.

It performs these steps:

1. validates that the tag is stable semantic versioning
2. validates that `Cargo.toml` already matches the tag version
3. builds the launcher for Windows, macOS, and Linux
4. builds the production WASM bundle
5. packages complete per-platform archives
6. generates SHA256 checksums
7. creates a GitHub release with generated notes and uploaded assets

## Release Assets

Each GitHub release contains:

- `ez-booth-windows-vX.Y.Z.zip`
- `ez-booth-macos-vX.Y.Z.tar.gz`
- `ez-booth-linux-vX.Y.Z.tar.gz`
- `checksums.txt`

Each platform archive includes:

- the platform launcher binary
- `index.html`
- built `.js`, `.css`, and `.wasm` files
- `README.txt` copied from `crates/ez-booth-app/ARTIFACT_README.md`

## Verify Downloads

On macOS or Linux:

```bash
shasum -a 256 -c checksums.txt
```

On Windows PowerShell:

```powershell
Get-FileHash .\ez-booth-windows-v0.1.0.zip -Algorithm SHA256
```

Compare the reported hash with the matching line in `checksums.txt`.

## Troubleshooting

### Tag rejected by workflow

The release workflow rejects:

- non-semver tags
- pre-release tags
- tags whose version does not match `Cargo.toml`

Fix the version on `main`, create a new tag, and push again.

### Release build fails

Inspect the failed GitHub Actions job.

- launcher build failures are usually platform-specific toolchain issues
- WASM build failures are usually frontend dependency or `trunk` build issues
- packaging failures usually mean an expected artifact name changed

### Publish job fails with "not a git repository"

The `publish` job runs `gh release create --verify-tag`, which requires a checked-out repository so the CLI can verify the tag against the repo remote.

If this error appears:

- ensure the `publish` job starts with `actions/checkout`
- ensure the checkout step runs before downloading artifacts or invoking `gh`

This is a workflow configuration issue, not a damaged repository or invalid release tag.

### Wrong release notes

Delete the release in GitHub, update the annotated tag locally if repository settings allow it, and recreate the tag. If release immutability is enabled, create a new patch release instead.

### Emergency follow-up release

1. fix the issue on a new `fix/...` branch
2. merge through a pull request
3. run `./scripts/validate-release.sh`
4. create the next patch release with `./scripts/create-release.sh X.Y.Z`

## Important Constraint

The release workflow validates the version instead of changing it. This keeps the tagged source, the published assets, and the repository history aligned.
