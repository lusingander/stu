use ratatui::{layout::Rect, Frame};

pub trait Page {
    fn render(&mut self, f: &mut Frame, area: Rect);

    fn render_header() -> bool {
        true
    }
}
