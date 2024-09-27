use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};

// implemented independently to calculate based on offset position
pub struct ScrollBar {
    lines_len: usize,
    offset: usize,
    bar_char: char,
    color: Color,
}

impl ScrollBar {
    pub fn new(lines_len: usize, offset: usize) -> ScrollBar {
        ScrollBar {
            lines_len,
            offset,
            bar_char: '│', // use '┃' or '║' instead...?
            color: Color::default(),
        }
    }

    pub fn color(mut self, color: Color) -> ScrollBar {
        self.color = color;
        self
    }
}

impl Widget for ScrollBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_scroll_bar(area, buf);
    }
}

impl ScrollBar {
    fn render_scroll_bar(&self, area: Rect, buf: &mut Buffer) {
        let scrollbar_height = self.calc_scrollbar_height(area);
        let scrollbar_top = self.calc_scrollbar_top(area, scrollbar_height);

        let x = area.x;
        for h in 0..scrollbar_height {
            let y = scrollbar_top + h;
            buf[(x, y)].set_char(self.bar_char).set_fg(self.color);
        }
    }

    fn calc_scrollbar_height(&self, area: Rect) -> u16 {
        let area_h = area.height as f64;
        let lines_len = self.lines_len as f64;
        let height = area_h * (area_h / lines_len);
        (height as u16).max(1)
    }

    fn calc_scrollbar_top(&self, area: Rect, scrollbar_height: u16) -> u16 {
        let area_h = area.height as f64;
        let scrollbar_h = scrollbar_height as f64;
        let offset = self.offset as f64;
        let lines_len = self.lines_len as f64;
        let top = ((area_h - scrollbar_h) * offset) / (lines_len - area_h);
        area.y + (top as u16)
    }
}
