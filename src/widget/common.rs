use ratatui::layout::{Constraint, Layout, Rect};

pub fn calc_centered_dialog_rect(r: Rect, dialog_width: u16, dialog_height: u16) -> Rect {
    let vertical_pad = r.height.saturating_sub(dialog_height) / 2;
    let vertical_layout = Layout::vertical(Constraint::from_lengths([
        vertical_pad,
        dialog_height,
        vertical_pad,
    ]))
    .split(r);

    let horizontal_pad = r.width.saturating_sub(dialog_width) / 2;
    Layout::horizontal(Constraint::from_lengths([
        horizontal_pad,
        dialog_width,
        horizontal_pad,
    ]))
    .split(vertical_layout[1])[1]
}
