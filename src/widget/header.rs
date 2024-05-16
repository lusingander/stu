use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    widgets::{Block, Padding, Paragraph, Widget},
};

use crate::{constant::APP_NAME, util::prune_strings_to_fit_width};

pub struct Header {
    breadcrumb: Vec<String>,
}

impl Header {
    pub fn new(breadcrumb: Vec<String>) -> Header {
        Header { breadcrumb }
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
        let inner_area = area.inner(&Margin::new(1, 1));
        let pad = Padding::horizontal(1);
        let max_width = (inner_area.width - pad.left - pad.right) as usize;

        let current_key_str = self.build_current_key_str(max_width);

        let paragraph =
            Paragraph::new(current_key_str).block(Block::bordered().title(APP_NAME).padding(pad));

        paragraph.render(area, buf);
    }

    fn build_current_key_str(self, max_width: usize) -> String {
        if self.breadcrumb.is_empty() {
            return "".to_string();
        }

        let current_key = self.breadcrumb.join(Self::DELIMITER);
        if current_key.len() <= max_width {
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
    use ratatui::assert_buffer_eq;

    use super::*;

    #[test]
    fn test_render_header() {
        let breadcrumb = ["bucket", "key01", "key02", "key03"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let header = Header::new(breadcrumb);
        let mut buf = Buffer::empty(Rect::new(0, 0, 30 + 4, 3));
        header.render(buf.area, &mut buf);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "┌STU─────────────────────────────┐",
            "│ bucket / key01 / key02 / key03 │",
            "└────────────────────────────────┘",
        ]);
        assert_buffer_eq!(buf, expected);
    }

    #[test]
    fn test_render_header_with_ellipsis() {
        let breadcrumb = ["bucket", "key01", "key02a", "key03"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let header = Header::new(breadcrumb);
        let mut buf = Buffer::empty(Rect::new(0, 0, 30 + 4, 3));
        header.render(buf.area, &mut buf);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "┌STU─────────────────────────────┐",
            "│ bucket / ... / key02a / key03  │",
            "└────────────────────────────────┘",
        ]);
        assert_buffer_eq!(buf, expected);
    }

    #[test]
    fn test_render_header_empty() {
        let header = Header::new(vec![]);
        let mut buf = Buffer::empty(Rect::new(0, 0, 30 + 4, 3));
        header.render(buf.area, &mut buf);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "┌STU─────────────────────────────┐",
            "│                                │",
            "└────────────────────────────────┘",
        ]);
        assert_buffer_eq!(buf, expected);
    }
}
