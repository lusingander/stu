use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, Padding, Paragraph, Widget},
};

const PREVIEW_LINE_NUMBER_COLOR: Color = Color::DarkGray;

pub struct Preview<'a> {
    file_name: &'a str,
    lines: &'a Vec<String>,
    max_digits: usize,
    offset: usize,
}

impl<'a> Preview<'a> {
    pub fn new(
        file_name: &'a str,
        lines: &'a Vec<String>,
        max_digits: usize,
        offset: usize,
    ) -> Self {
        Self {
            file_name,
            lines,
            max_digits,
            offset,
        }
    }
}

impl Widget for Preview<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content_area = area.inner(&Margin::new(1, 1)); // border

        let preview_max_digits = self.max_digits;
        let show_lines_count = content_area.height as usize;
        let content_max_width = (content_area.width as usize) - preview_max_digits - 3 /* pad */;

        let content: Vec<Line> = ((self.offset + 1)..)
            .zip(self.lines.iter().skip(self.offset))
            .flat_map(|(n, s)| {
                let ss = textwrap::wrap(s, content_max_width);
                ss.into_iter().enumerate().map(move |(i, s)| {
                    let line_number = if i == 0 {
                        format!("{:>preview_max_digits$}", n)
                    } else {
                        " ".repeat(preview_max_digits)
                    };
                    Line::from(vec![
                        line_number.fg(PREVIEW_LINE_NUMBER_COLOR),
                        " ".into(),
                        s.into(),
                    ])
                })
            })
            .take(show_lines_count)
            .collect();

        let title = format!("Preview [{}]", self.file_name);

        let paragraph = Paragraph::new(content).block(
            Block::bordered()
                .title(title)
                .padding(Padding::horizontal(1)),
        );
        paragraph.render(area, buf);
    }
}
