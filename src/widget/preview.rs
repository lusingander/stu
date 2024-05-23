use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap},
};

const PREVIEW_LINE_NUMBER_COLOR: Color = Color::DarkGray;

// fixme: bad implementation for highlighting and displaying the number of lines :(
pub struct Preview<'a> {
    file_name: &'a str,
    lines: &'a Vec<Line<'static>>,
    original_lines: &'a Vec<String>,
    max_digits: usize,
    offset: usize,
}

impl<'a> Preview<'a> {
    pub fn new(
        file_name: &'a str,
        lines: &'a Vec<Line<'static>>,
        original_lines: &'a Vec<String>,
        max_digits: usize,
        offset: usize,
    ) -> Self {
        Self {
            file_name,
            lines,
            original_lines,
            max_digits,
            offset,
        }
    }
}

impl Widget for Preview<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content_area = area.inner(&Margin::new(1, 1)); // border

        let title = format!("Preview [{}]", self.file_name);
        let block = Block::bordered().title(title);

        let chunks = Layout::horizontal([
            Constraint::Length(self.max_digits as u16 + 1),
            Constraint::Min(0),
        ])
        .split(content_area);

        let show_lines_count = content_area.height as usize;

        // may not be correct because the wrap of the text is calculated separately...
        let line_heights = self
            .original_lines
            .iter()
            .skip(self.offset)
            .take(show_lines_count)
            .map(|line| {
                let lines = textwrap::wrap(line, chunks[1].width as usize - 2);
                lines.len()
            });
        let lines_count = self.original_lines.len();
        let line_numbers_content: Vec<Line> = ((self.offset + 1)..)
            .zip(line_heights)
            .flat_map(|(line, line_height)| {
                if line > lines_count {
                    vec![Line::raw("")]
                } else {
                    let line_number = format!("{:>width$}", line, width = self.max_digits);
                    let number_line: Line = line_number.fg(PREVIEW_LINE_NUMBER_COLOR).into();
                    let empty_lines = (0..(line_height - 1)).map(|_| Line::raw(""));
                    std::iter::once(number_line).chain(empty_lines).collect()
                }
            })
            .take(show_lines_count)
            .collect();

        let line_numbers_paragraph = Paragraph::new(line_numbers_content).block(
            Block::default()
                .borders(Borders::NONE)
                .padding(Padding::left(1)),
        );

        let lines_content: Vec<Line> = self
            .lines
            .iter()
            .skip(self.offset)
            .take(show_lines_count)
            .cloned()
            .collect();

        let lines_paragraph = Paragraph::new(lines_content)
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .padding(Padding::horizontal(1)),
            )
            .wrap(Wrap { trim: false });

        block.render(area, buf);
        line_numbers_paragraph.render(chunks[0], buf);
        lines_paragraph.render(chunks[1], buf);
    }
}
