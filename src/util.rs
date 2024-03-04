pub fn to_preview_string(bytes: &[u8], _content_type: &str) -> String {
    // fixme: consider content_type
    String::from_utf8_lossy(bytes).into()
}
