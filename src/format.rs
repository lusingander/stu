use chrono::{DateTime, Local};

pub fn format_size_byte(size_byte: usize) -> String {
    humansize::format_size_i(size_byte, humansize::BINARY)
}

#[cfg(not(feature = "imggen"))]
pub fn format_version(version: &str) -> &str {
    version
}

#[cfg(feature = "imggen")]
pub fn format_version(_version: &str) -> &str {
    "GeJeVLwoQlknMCcSa"
}

#[cfg(not(feature = "imggen"))]
pub fn format_datetime(datetime: &DateTime<Local>, format_str: &str) -> String {
    datetime.format(format_str).to_string()
}

#[cfg(feature = "imggen")]
pub fn format_datetime(_datetime: &DateTime<Local>, _: &str) -> String {
    String::from("2024-01-02 13:04:05")
}
