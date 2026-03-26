/// Format an error message for user display, truncating if too long
pub fn format_error_message<E: std::fmt::Debug>(error: &E) -> String {
    const MAX_LEN: usize = 140;
    let mut formatted = format!("{:?}", error).replace(['\n', '\r'], " ");

    if formatted.len() > MAX_LEN {
        formatted.truncate(MAX_LEN - 3);
        formatted.push_str("...");
    }

    formatted
}
