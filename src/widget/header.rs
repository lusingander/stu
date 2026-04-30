use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Margin, Rect},
    style::{Color, Stylize},
    widgets::{Block, Padding, Paragraph, Widget},
};

use crate::{color::Theme, util::prune_strings_to_fit_width};

#[derive(Debug, Default)]
struct HeaderColor {
    block: Color,
    text: Color,
}

impl HeaderColor {
    fn new(theme: &Theme) -> HeaderColor {
        HeaderColor {
            block: theme.fg,
            text: theme.fg,
        }
    }
}

#[derive(Debug, Default)]
pub struct Header {
    breadcrumb: Vec<String>,
    region: Option<String>,
    color: HeaderColor,
}

impl Header {
    pub fn new(breadcrumb: Vec<String>) -> Header {
        Header {
            breadcrumb,
            ..Default::default()
        }
    }

    pub fn theme(mut self, theme: &Theme) -> Self {
        self.color = HeaderColor::new(theme);
        self
    }

    pub fn region(mut self, region: Option<String>) -> Self {
        self.region = region.filter(|s| !s.is_empty());
        self
    }
}

impl Widget for Header {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_header(area, buf);
    }
}

impl Header {
    const DELIMITER: &'static str = " / ";
    const ELLIPSIS: &'static str = "...";
    const REGION_GAP: usize = 2;

    fn render_header(self, area: Rect, buf: &mut Buffer) {
        if self.region.is_some() {
            self.render_header_with_region(area, buf);
            return;
        }

        let inner_area = area.inner(Margin::new(1, 1));
        let pad = Padding::horizontal(1);
        let max_width = (inner_area.width - pad.left - pad.right) as usize;

        let block_color = self.color.block;
        let text_color = self.color.text;
        let current_key_str = self.build_current_key_str(max_width).fg(text_color);

        let paragraph =
            Paragraph::new(current_key_str).block(Block::bordered().fg(block_color).padding(pad));

        paragraph.render(area, buf);
    }

    fn render_header_with_region(self, area: Rect, buf: &mut Buffer) {
        let pad = Padding::horizontal(1);
        let block_color = self.color.block;
        let text_color = self.color.text;

        let block = Block::bordered().fg(block_color).padding(pad);
        let inner_area = block.inner(area);
        block.render(area, buf);

        if inner_area.width == 0 || inner_area.height == 0 {
            return;
        }

        let total_width = inner_area.width as usize;

        // Region is guaranteed to be Some here; format and measure it.
        let region_text = format!("[{}]", self.region.as_deref().unwrap());
        let region_width = console::measure_text_width(&region_text);

        // If the region label doesn't fit, fall back to the breadcrumb-only layout.
        if region_width + Self::REGION_GAP >= total_width {
            let breadcrumb_str = self.build_current_key_str(total_width).fg(text_color);
            Paragraph::new(breadcrumb_str).render(inner_area, buf);
            return;
        }

        let breadcrumb_width = total_width - region_width - Self::REGION_GAP;

        let breadcrumb_str = self.build_current_key_str(breadcrumb_width).fg(text_color);
        let breadcrumb_area = Rect {
            x: inner_area.x,
            y: inner_area.y,
            width: breadcrumb_width as u16,
            height: inner_area.height,
        };
        Paragraph::new(breadcrumb_str).render(breadcrumb_area, buf);

        let region_area = Rect {
            x: inner_area.x + (total_width - region_width) as u16,
            y: inner_area.y,
            width: region_width as u16,
            height: inner_area.height,
        };
        Paragraph::new(region_text.fg(text_color))
            .alignment(Alignment::Right)
            .render(region_area, buf);
    }

    fn build_current_key_str(self, max_width: usize) -> String {
        if self.breadcrumb.is_empty() {
            return "".to_string();
        }

        let current_key = self.breadcrumb.join(Self::DELIMITER);
        if console::measure_text_width(&current_key) <= max_width {
            return current_key;
        }

        //   string: <bucket> / ... / s1 / s2 / s3 / s4 / s5
        // priority:        1 /   0 /  4 /  3 /  2 /  1 /  0
        let bl = self.breadcrumb.len();
        let mut bs: Vec<(String, usize)> = self
            .breadcrumb
            .into_iter()
            .enumerate()
            .map(|(i, p)| (p, bl - i - 1))
            .collect();
        bs.insert(1, (Self::ELLIPSIS.to_string(), 0));
        bs.first_mut().unwrap().1 = 1;
        bs.last_mut().unwrap().1 = 0;

        let keys = prune_strings_to_fit_width(&bs, max_width, Self::DELIMITER);
        keys.join(Self::DELIMITER)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_header() {
        let theme = Theme::default();
        let breadcrumb = ["bucket", "key01", "key02", "key03"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let header = Header::new(breadcrumb).theme(&theme).region(None);
        let mut buf = Buffer::empty(Rect::new(0, 0, 30 + 4, 3));
        header.render(buf.area, &mut buf);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "┌────────────────────────────────┐",
            "│ bucket / key01 / key02 / key03 │",
            "└────────────────────────────────┘",
        ]);
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_render_header_with_ellipsis() {
        let theme = Theme::default();
        let breadcrumb = ["bucket", "key01", "key02a", "key03"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let header = Header::new(breadcrumb).theme(&theme);
        let mut buf = Buffer::empty(Rect::new(0, 0, 30 + 4, 3));
        header.render(buf.area, &mut buf);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "┌────────────────────────────────┐",
            "│ bucket / ... / key02a / key03  │",
            "└────────────────────────────────┘",
        ]);
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_render_header_empty() {
        let theme = Theme::default();
        let header = Header::new(vec![]).theme(&theme);
        let mut buf = Buffer::empty(Rect::new(0, 0, 30 + 4, 3));
        header.render(buf.area, &mut buf);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "┌────────────────────────────────┐",
            "│                                │",
            "└────────────────────────────────┘",
        ]);
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_render_header_with_region() {
        let theme = Theme::default();
        let breadcrumb = ["bucket", "key01"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let header = Header::new(breadcrumb)
            .theme(&theme)
            .region(Some("eu-central-1".to_string()));
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 3));
        header.render(buf.area, &mut buf);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "┌──────────────────────────────────────┐",
            "│ bucket / key01        [eu-central-1] │",
            "└──────────────────────────────────────┘",
        ]);
        assert_eq!(buf, expected);
    }
}
