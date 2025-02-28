use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::{Color, Stylize},
    widgets::{Block, Padding, Paragraph, Widget},
};

use crate::{color::ColorTheme, constant::APP_NAME, util::prune_strings_to_fit_width};

#[derive(Debug, Default)]
struct HeaderColor {
    block: Color,
    text: Color,
}

impl HeaderColor {
    fn new(theme: &ColorTheme) -> HeaderColor {
        HeaderColor {
            block: theme.fg,
            text: theme.fg,
        }
    }
}

#[derive(Debug, Default)]
pub struct Header {
    breadcrumb: Vec<String>,
    color: HeaderColor,
}

impl Header {
    pub fn new(breadcrumb: Vec<String>) -> Header {
        Header {
            breadcrumb,
            ..Default::default()
        }
    }

    pub fn theme(mut self, theme: &ColorTheme) -> Self {
        self.color = HeaderColor::new(theme);
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

    fn render_header(self, area: Rect, buf: &mut Buffer) {
        let inner_area = area.inner(Margin::new(1, 1));
        let pad = Padding::horizontal(1);
        let max_width = (inner_area.width - pad.left - pad.right) as usize;

        let block_color = self.color.block;
        let text_color = self.color.text;
        let current_key_str = self.build_current_key_str(max_width).fg(text_color);

        let paragraph = Paragraph::new(current_key_str).block(
            Block::bordered()
                .title(APP_NAME)
                .fg(block_color)
                .padding(pad),
        );

        paragraph.render(area, buf);
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
        let theme = ColorTheme::default();
        let breadcrumb = ["bucket", "key01", "key02", "key03"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let header = Header::new(breadcrumb).theme(&theme);
        let mut buf = Buffer::empty(Rect::new(0, 0, 30 + 4, 3));
        header.render(buf.area, &mut buf);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "┌STU─────────────────────────────┐",
            "│ bucket / key01 / key02 / key03 │",
            "└────────────────────────────────┘",
        ]);
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_render_header_with_ellipsis() {
        let theme = ColorTheme::default();
        let breadcrumb = ["bucket", "key01", "key02a", "key03"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let header = Header::new(breadcrumb).theme(&theme);
        let mut buf = Buffer::empty(Rect::new(0, 0, 30 + 4, 3));
        header.render(buf.area, &mut buf);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "┌STU─────────────────────────────┐",
            "│ bucket / ... / key02a / key03  │",
            "└────────────────────────────────┘",
        ]);
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_render_header_empty() {
        let theme = ColorTheme::default();
        let header = Header::new(vec![]).theme(&theme);
        let mut buf = Buffer::empty(Rect::new(0, 0, 30 + 4, 3));
        header.render(buf.area, &mut buf);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "┌STU─────────────────────────────┐",
            "│                                │",
            "└────────────────────────────────┘",
        ]);
        assert_eq!(buf, expected);
    }
}
