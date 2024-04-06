use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    widgets::{Clear, Widget, WidgetRef},
};

pub struct Dialog<'a> {
    content: Box<dyn WidgetRef + 'a>,
}

impl<'a> Dialog<'a> {
    pub fn new(content: Box<dyn WidgetRef + 'a>) -> Dialog<'a> {
        Dialog { content }
    }
}

impl<'a> WidgetRef for Dialog<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.render_dialog(area, buf);
    }
}

impl<'a> Dialog<'a> {
    fn render_dialog(&self, area: Rect, buf: &mut Buffer) {
        Clear.render(outer_rect(area, &Margin::new(1, 0)), buf);
        self.content.render_ref(area, buf);
    }
}

fn outer_rect(r: Rect, margin: &Margin) -> Rect {
    let doubled_margin_horizontal = margin.horizontal.saturating_mul(2);
    let doubled_margin_vertical = margin.vertical.saturating_mul(2);
    Rect {
        x: r.x.saturating_sub(margin.horizontal),
        y: r.y.saturating_sub(margin.vertical),
        width: r.width.saturating_add(doubled_margin_horizontal),
        height: r.height.saturating_add(doubled_margin_vertical),
    }
}
