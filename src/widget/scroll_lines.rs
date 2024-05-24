use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, Borders, Padding, Paragraph, StatefulWidget, Widget, Wrap},
};

use crate::util::digits;

const PREVIEW_LINE_NUMBER_COLOR: Color = Color::DarkGray;

#[derive(Debug, Default)]
enum ScrollEvent {
    #[default]
    None,
    PageForward,
    PageBackward,
}

#[derive(Debug, Clone)]
pub struct ScrollLinesOptions {
    pub number: bool,
    pub wrap: bool,
}

impl Default for ScrollLinesOptions {
    fn default() -> Self {
        Self {
            number: true,
            wrap: true,
        }
    }
}

#[derive(Debug, Default)]
pub struct ScrollLinesState {
    lines: Vec<Line<'static>>,
    original_lines: Vec<String>,
    max_digits: usize,
    max_line_width: usize,
    v_offset: usize,
    h_offset: usize,
    options: ScrollLinesOptions,
    title: String,
    scroll_event: ScrollEvent,
}

impl ScrollLinesState {
    pub fn new(
        lines: Vec<Line<'static>>,
        original_lines: Vec<String>,
        title: String,
        options: ScrollLinesOptions,
    ) -> Self {
        let max_digits = digits(lines.len());
        let max_line_width = lines.iter().map(Line::width).max().unwrap_or_default();

        Self {
            lines,
            original_lines,
            max_digits,
            max_line_width,
            options,
            title,
            ..Default::default()
        }
    }

    pub fn scroll_forward(&mut self) {
        if self.v_offset < self.lines.len().saturating_sub(1) {
            self.v_offset = self.v_offset.saturating_add(1);
        }
    }

    pub fn scroll_backward(&mut self) {
        if self.v_offset > 0 {
            self.v_offset = self.v_offset.saturating_sub(1);
        }
    }

    pub fn scroll_page_forward(&mut self) {
        self.scroll_event = ScrollEvent::PageForward;
    }

    pub fn scroll_page_backward(&mut self) {
        self.scroll_event = ScrollEvent::PageBackward;
    }

    pub fn scroll_to_top(&mut self) {
        self.v_offset = 0;
    }

    pub fn scroll_to_end(&mut self) {
        self.v_offset = self.lines.len().saturating_sub(1);
    }

    pub fn scroll_right(&mut self) {
        if self.h_offset < self.max_line_width.saturating_sub(1) {
            self.h_offset = self.h_offset.saturating_add(1);
        }
    }

    pub fn scroll_left(&mut self) {
        if self.h_offset > 0 {
            self.h_offset = self.h_offset.saturating_sub(1);
        }
    }

    pub fn toggle_wrap(&mut self) {
        self.options.wrap = !self.options.wrap;
        self.h_offset = 0;
    }

    pub fn toggle_number(&mut self) {
        self.options.number = !self.options.number;
    }
}

// fixme: bad implementation for highlighting and displaying the number of lines :(
#[derive(Debug, Default)]
pub struct ScrollLines {}

impl StatefulWidget for ScrollLines {
    type State = ScrollLinesState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let content_area = area.inner(&Margin::new(1, 1)); // border

        let block = Block::bordered().title(state.title.clone());

        let line_numbers_width = if state.options.number {
            state.max_digits as u16 + 1
        } else {
            0
        };

        let chunks =
            Layout::horizontal([Constraint::Length(line_numbers_width), Constraint::Min(0)])
                .split(content_area);

        let show_lines_count = content_area.height as usize;

        // handle scroll events and update the state
        match state.scroll_event {
            ScrollEvent::PageForward => {
                let line_heights = wrapped_line_width_iter(
                    &state.original_lines,
                    state.v_offset,
                    chunks[1].width as usize - 2,
                    show_lines_count,
                    state.options.wrap,
                );
                let mut add_offset = 0;
                let mut total_h = 0;
                for h in line_heights {
                    add_offset += 1;
                    total_h += h;
                    if total_h >= show_lines_count {
                        state.v_offset += add_offset;
                        if total_h > show_lines_count {
                            // if the last line is wrapped, the offset should be decreased by 1
                            state.v_offset -= 1;
                        }
                        break;
                    }
                }
                if total_h < show_lines_count {
                    state.scroll_to_end();
                }
                state.scroll_event = ScrollEvent::None;
            }
            ScrollEvent::PageBackward => {
                let line_heights = wrapped_reversed_line_width_iter(
                    &state.original_lines,
                    state.v_offset,
                    chunks[1].width as usize - 2,
                    show_lines_count,
                    state.options.wrap,
                );
                let mut sub_offset = 0;
                let mut total_h = 0;
                for h in line_heights {
                    sub_offset += 1;
                    total_h += h;
                    if total_h >= show_lines_count {
                        state.v_offset -= sub_offset;
                        if total_h > show_lines_count {
                            // if the first line is wrapped, the offset should be increased by 1
                            state.v_offset += 1;
                        }
                        break;
                    }
                }
                if total_h < show_lines_count {
                    state.scroll_to_top();
                }
                state.scroll_event = ScrollEvent::None;
            }
            _ => {}
        }

        // may not be correct because the wrap of the text is calculated separately...
        let line_heights = wrapped_line_width_iter(
            &state.original_lines,
            state.v_offset,
            chunks[1].width as usize - 2,
            show_lines_count,
            state.options.wrap,
        );
        let lines_count = state.original_lines.len();
        let line_numbers_content: Vec<Line> = ((state.v_offset + 1)..)
            .zip(line_heights)
            .flat_map(|(line, line_height)| {
                if line > lines_count {
                    vec![Line::raw("")]
                } else {
                    let line_number = format!("{:>width$}", line, width = state.max_digits);
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

        let lines_content: Vec<Line> = state
            .lines
            .iter()
            .skip(state.v_offset)
            .take(show_lines_count)
            .cloned()
            .collect();

        let mut lines_paragraph = Paragraph::new(lines_content).block(
            Block::default()
                .borders(Borders::NONE)
                .padding(Padding::horizontal(1)),
        );

        lines_paragraph = if state.options.wrap {
            lines_paragraph.wrap(Wrap { trim: false })
        } else {
            lines_paragraph.scroll((0, state.h_offset as u16))
        };

        block.render(area, buf);
        line_numbers_paragraph.render(chunks[0], buf);
        lines_paragraph.render(chunks[1], buf);
    }
}

fn wrapped_line_width_iter(
    lines: &[String],
    offset: usize,
    width: usize,
    height: usize,
    wrap: bool,
) -> impl Iterator<Item = usize> + '_ {
    lines.iter().skip(offset).take(height).map(move |line| {
        if wrap {
            let lines = textwrap::wrap(line, width);
            lines.len()
        } else {
            1
        }
    })
}

fn wrapped_reversed_line_width_iter(
    lines: &[String],
    offset: usize,
    width: usize,
    height: usize,
    wrap: bool,
) -> impl Iterator<Item = usize> + '_ {
    lines
        .iter()
        .take(offset)
        .rev()
        .take(height)
        .map(move |line| {
            if wrap {
                let lines = textwrap::wrap(line, width);
                lines.len()
            } else {
                1
            }
        })
}
