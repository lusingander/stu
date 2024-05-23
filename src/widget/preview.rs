use ansi_to_tui::IntoText;
use once_cell::sync::Lazy;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap},
};
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};

use crate::{
    object::{FileDetail, RawObject},
    util::{digits, extension_from_file_name, to_preview_string},
};

const PREVIEW_LINE_NUMBER_COLOR: Color = Color::DarkGray;

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

#[derive(Debug)]
pub struct PreviewState {
    file_name: String,
    lines: Vec<Line<'static>>,
    original_lines: Vec<String>,
    max_digits: usize,
    offset: usize,
}

impl PreviewState {
    pub fn new(file_detail: &FileDetail, object: &RawObject, highlight: bool) -> Self {
        let s = to_preview_string(&object.bytes);
        let s = if s.ends_with('\n') {
            s.trim_end()
        } else {
            s.as_str()
        };

        let original_lines: Vec<String> = s.split('\n').map(|s| s.to_string()).collect();

        let lines: Vec<Line<'static>> = if highlight {
            let extension = extension_from_file_name(&file_detail.name);
            let syntax = SYNTAX_SET.find_syntax_by_extension(&extension).unwrap();
            let mut h = HighlightLines::new(syntax, &THEME_SET.themes["base16-ocean.dark"]);
            let s = LinesWithEndings::from(s)
                .map(|line| {
                    let ranges: Vec<(syntect::highlighting::Style, &str)> =
                        h.highlight_line(line, &SYNTAX_SET).unwrap();
                    as_24_bit_terminal_escaped(&ranges[..], false)
                })
                .collect::<Vec<String>>()
                .join("");
            s.into_text().unwrap().into_iter().collect()
        } else {
            original_lines
                .iter()
                .map(|s| Line::raw(s.clone()))
                .collect()
        };

        let max_digits = digits(lines.len());

        Self {
            file_name: file_detail.name.clone(),
            lines,
            original_lines,
            max_digits,
            offset: 0,
        }
    }

    pub fn scroll_forward(&mut self) {
        if self.offset < self.lines.len() - 1 {
            self.offset = self.offset.saturating_add(1);
        }
    }

    pub fn scroll_backward(&mut self) {
        if self.offset > 0 {
            self.offset = self.offset.saturating_sub(1);
        }
    }

    pub fn scroll_to_top(&mut self) {
        self.offset = 0;
    }

    pub fn scroll_to_end(&mut self) {
        self.offset = self.lines.len() - 1;
    }
}

// fixme: bad implementation for highlighting and displaying the number of lines :(
pub struct Preview<'a> {
    state: &'a PreviewState,
}

impl<'a> Preview<'a> {
    pub fn new(state: &'a PreviewState) -> Self {
        Self { state }
    }
}

impl Widget for Preview<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content_area = area.inner(&Margin::new(1, 1)); // border

        let title = format!("Preview [{}]", self.state.file_name);
        let block = Block::bordered().title(title);

        let chunks = Layout::horizontal([
            Constraint::Length(self.state.max_digits as u16 + 1),
            Constraint::Min(0),
        ])
        .split(content_area);

        let show_lines_count = content_area.height as usize;

        // may not be correct because the wrap of the text is calculated separately...
        let line_heights = self
            .state
            .original_lines
            .iter()
            .skip(self.state.offset)
            .take(show_lines_count)
            .map(|line| {
                let lines = textwrap::wrap(line, chunks[1].width as usize - 2);
                lines.len()
            });
        let lines_count = self.state.original_lines.len();
        let line_numbers_content: Vec<Line> = ((self.state.offset + 1)..)
            .zip(line_heights)
            .flat_map(|(line, line_height)| {
                if line > lines_count {
                    vec![Line::raw("")]
                } else {
                    let line_number = format!("{:>width$}", line, width = self.state.max_digits);
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
            .state
            .lines
            .iter()
            .skip(self.state.offset)
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
