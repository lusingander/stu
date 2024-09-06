use chrono::{DateTime, Local};
use ratatui::layout::{Constraint, Layout, Rect};

pub fn calc_centered_dialog_rect(r: Rect, dialog_width: u16, dialog_height: u16) -> Rect {
    let vertical_pad = (r.height - dialog_height) / 2;
    let vertical_layout = Layout::vertical(Constraint::from_lengths([
        vertical_pad,
        dialog_height,
        vertical_pad,
    ]))
    .split(r);

    let horizontal_pad = (r.width - dialog_width) / 2;
    Layout::horizontal(Constraint::from_lengths([
        horizontal_pad,
        dialog_width,
        horizontal_pad,
    ]))
    .split(vertical_layout[1])[1]
}

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
