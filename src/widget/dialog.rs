use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::{Color, Stylize},
    widgets::{Block, Clear, Widget},
};

type DialogContent<'a> = Box<dyn FnOnce(Rect, &mut Buffer) + 'a>;

pub struct Dialog<'a> {
    content: DialogContent<'a>,
    margin: Margin,
    bg: Color,
}

impl<'a> Dialog<'a> {
    pub fn new<W: Widget + 'a>(content: W) -> Dialog<'a> {
        Dialog {
            content: Box::new(move |area, buf| content.render(area, buf)),
            margin: Margin::default(),
            bg: Color::default(),
        }
    }

    pub fn margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = color;
        self
    }
}

impl Widget for Dialog<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_dialog(area, buf);
    }
}

impl Dialog<'_> {
    fn render_dialog(self, area: Rect, buf: &mut Buffer) {
        let outer = outer_rect(area, self.margin);
        Clear.render(outer, buf);
        Block::default().bg(self.bg).render(outer, buf);
        (self.content)(area, buf);
    }
}

fn outer_rect(r: Rect, margin: Margin) -> Rect {
    let doubled_margin_horizontal = margin.horizontal.saturating_mul(2);
    let doubled_margin_vertical = margin.vertical.saturating_mul(2);
    Rect {
        x: r.x.saturating_sub(margin.horizontal),
        y: r.y.saturating_sub(margin.vertical),
        width: r.width.saturating_add(doubled_margin_horizontal),
        height: r.height.saturating_add(doubled_margin_vertical),
    }
}
