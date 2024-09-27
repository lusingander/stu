use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    text::Line,
    widgets::Widget,
};

#[derive(Debug)]
pub struct Divider {
    char: &'static str,
    color: Color,
}

impl Divider {
    pub fn new(color: Color) -> Self {
        Self { char: "─", color }
    }
}

impl Widget for Divider {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = Line::from(self.char.repeat(area.width as usize)).fg(self.color);
        line.render(area, buf);
    }
}
