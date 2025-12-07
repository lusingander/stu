use chrono::{DateTime, Local};

pub fn format_size_byte(size_byte: usize) -> String {
    humansize::format_size_i(size_byte, humansize::BINARY)
}

pub fn format_version(version: Option<&str>, fix_dynamic_values: bool) -> &str {
    if fix_dynamic_values {
        // use a fixed version if fix_dynamic_values is true
        return "GeJeVLwoQlknMCcSa";
    }
    match version {
        Some(v) => v,
        None => "-",
    }
}

pub fn format_datetime(
    datetime: &DateTime<Local>,
    format_str: &str,
    fix_dynamic_values: bool,
) -> String {
    if fix_dynamic_values {
        // use a fixed datetime if fix_dynamic_values is true
        return String::from("2024-01-02 13:04:05");
    }
    datetime.format(format_str).to_string()
}
