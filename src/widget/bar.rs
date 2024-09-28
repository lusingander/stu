use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

#[derive(Debug)]
pub struct Bar {
    char: &'static str,
    color: Color,
}

impl Default for Bar {
    fn default() -> Self {
        Self {
            char: "┃",
            color: Color::default(),
        }
    }
}

impl Bar {
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Widget for Bar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for y in area.top()..area.bottom() {
            buf.set_string(area.left(), y, self.char, Style::default().fg(self.color));
        }
    }
}
