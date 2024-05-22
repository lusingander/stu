use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    text::Line,
    widgets::Widget,
};

const DIVIDER_COLOR: Color = Color::DarkGray;

#[derive(Debug)]
pub struct Divider {
    char: &'static str,
    color: Color,
}

impl Default for Divider {
    fn default() -> Self {
        Self {
            char: "─",
            color: DIVIDER_COLOR,
        }
    }
}

impl Widget for Divider {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = Line::from(self.char.repeat(area.width as usize)).fg(self.color);
        line.render(area, buf);
    }
}
