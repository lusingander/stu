use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{block::BlockExt, Block, Borders, Padding, Paragraph, StatefulWidget, Widget, Wrap},
};

use crate::{color::ColorTheme, util::digits};

#[derive(Debug, Default)]
enum ScrollEvent {
    #[default]
    None,
    Forward,
    Backward,
    PageForward,
    PageBackward,
    Top,
    End,
    Right,
    Left,
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollLinesOptions {
    pub number: bool,
    pub wrap: bool,
}

impl ScrollLinesOptions {
    pub fn new(number: bool, wrap: bool) -> Self {
        Self { number, wrap }
    }
}

impl Default for ScrollLinesOptions {
    fn default() -> Self {
        Self::new(true, true)
    }
}

#[derive(Debug, Default)]
pub struct ScrollLinesState {
    lines: Vec<Line<'static>>,
    max_digits: usize,
    max_line_width: usize,
    v_offset: usize,
    h_offset: usize,
    options: ScrollLinesOptions,
    scroll_event: ScrollEvent,
}

impl ScrollLinesState {
    pub fn new(lines: Vec<Line<'static>>, options: ScrollLinesOptions) -> Self {
        let max_digits = digits(lines.len());
        let max_line_width = lines.iter().map(Line::width).max().unwrap_or_default();

        Self {
            lines,
            max_digits,
            max_line_width,
            options,
            ..Default::default()
        }
    }

    pub fn scroll_forward(&mut self) {
        self.scroll_event = ScrollEvent::Forward;
    }

    pub fn scroll_backward(&mut self) {
        self.scroll_event = ScrollEvent::Backward;
    }

    pub fn scroll_page_forward(&mut self) {
        self.scroll_event = ScrollEvent::PageForward;
    }

    pub fn scroll_page_backward(&mut self) {
        self.scroll_event = ScrollEvent::PageBackward;
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_event = ScrollEvent::Top;
    }

    pub fn scroll_to_end(&mut self) {
        self.scroll_event = ScrollEvent::End;
    }

    pub fn scroll_right(&mut self) {
        self.scroll_event = ScrollEvent::Right;
    }

    pub fn scroll_left(&mut self) {
        self.scroll_event = ScrollEvent::Left;
    }

    pub fn toggle_wrap(&mut self) {
        self.options.wrap = !self.options.wrap;
        self.h_offset = 0;
    }

    pub fn toggle_number(&mut self) {
        self.options.number = !self.options.number;
    }

    pub fn current_options(&self) -> ScrollLinesOptions {
        self.options
    }
}

#[derive(Debug, Default)]
struct ScrollLinesColor {
    block: Color,
    line_number: Color,
}

impl ScrollLinesColor {
    fn new(theme: &ColorTheme) -> Self {
        Self {
            block: theme.fg,
            line_number: theme.preview_line_number,
        }
    }
}

// fixme: bad implementation for highlighting and displaying the number of lines :(
#[derive(Debug, Default)]
pub struct ScrollLines {
    block: Option<Block<'static>>,
    color: ScrollLinesColor,
}

impl ScrollLines {
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn theme(mut self, theme: &ColorTheme) -> Self {
        self.color = ScrollLinesColor::new(theme);
        self
    }
}

impl StatefulWidget for ScrollLines {
    type State = ScrollLinesState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let content_area = self.block.inner_if_some(area);

        let line_numbers_width = if state.options.number {
            state.max_digits as u16 + 1
        } else {
            0
        };

        let chunks =
            Layout::horizontal([Constraint::Length(line_numbers_width), Constraint::Min(0)])
                .split(content_area);

        let show_lines_count = content_area.height as usize;
        let text_area_width = chunks[1].width as usize - 2 /* padding */;

        // handle scroll events and update the state
        handle_scroll_events(state, text_area_width, show_lines_count);

        let line_numbers_paragraph = build_line_numbers_paragraph(
            state,
            text_area_width,
            show_lines_count,
            self.color.line_number,
        );
        let lines_paragraph = build_lines_paragraph(state, show_lines_count, self.color.block);

        self.block.map(|b| b.fg(self.color.block)).render(area, buf);
        line_numbers_paragraph.render(chunks[0], buf);
        lines_paragraph.render(chunks[1], buf);
    }
}

fn build_line_numbers_paragraph(
    state: &ScrollLinesState,
    text_area_width: usize,
    show_lines_count: usize,
    line_number_color: Color,
) -> Paragraph {
    // may not be correct because the wrap of the text is calculated separately...
    let line_heights = wrapped_line_width_iter(
        &state.lines,
        state.v_offset,
        text_area_width,
        show_lines_count,
        state.options.wrap,
    );
    let lines_count = state.lines.len();
    let line_numbers_content: Vec<Line> = ((state.v_offset + 1)..)
        .zip(line_heights)
        .flat_map(|(line, line_height)| {
            if line > lines_count {
                vec![Line::raw("")]
            } else {
                let line_number = format!("{:>width$}", line, width = state.max_digits);
                let number_line: Line = line_number.fg(line_number_color).into();
                let empty_lines = (0..(line_height - 1)).map(|_| Line::raw(""));
                std::iter::once(number_line).chain(empty_lines).collect()
            }
        })
        .take(show_lines_count)
        .collect();

    Paragraph::new(line_numbers_content).block(
        Block::default()
            .borders(Borders::NONE)
            .padding(Padding::left(1)),
    )
}

fn build_lines_paragraph(
    state: &ScrollLinesState,
    show_lines_count: usize,
    block_color: Color,
) -> Paragraph {
    let lines_content: Vec<Line> = state
        .lines
        .iter()
        .skip(state.v_offset)
        .take(show_lines_count)
        .cloned()
        .collect();

    let lines_paragraph = Paragraph::new(lines_content).block(
        Block::default()
            .borders(Borders::NONE)
            .padding(Padding::horizontal(1))
            .fg(block_color),
    );

    if state.options.wrap {
        lines_paragraph.wrap(Wrap { trim: false })
    } else {
        lines_paragraph.scroll((0, state.h_offset as u16))
    }
}

fn handle_scroll_events(state: &mut ScrollLinesState, width: usize, height: usize) {
    match state.scroll_event {
        ScrollEvent::None => {}
        ScrollEvent::Forward => {
            if state.v_offset < state.lines.len().saturating_sub(1) {
                state.v_offset = state.v_offset.saturating_add(1);
            }
        }
        ScrollEvent::Backward => {
            if state.v_offset > 0 {
                state.v_offset = state.v_offset.saturating_sub(1);
            }
        }
        ScrollEvent::PageForward => {
            let line_heights = wrapped_line_width_iter(
                &state.lines,
                state.v_offset,
                width,
                height,
                state.options.wrap,
            );
            let mut add_offset = 0;
            let mut total_h = 0;
            for h in line_heights {
                add_offset += 1;
                total_h += h;
                if total_h >= height {
                    state.v_offset += add_offset;
                    if total_h > height {
                        // if the last line is wrapped, the offset should be decreased by 1
                        state.v_offset -= 1;
                    }
                    break;
                }
            }
            if total_h < height {
                // scroll to the end
                state.v_offset = state.lines.len().saturating_sub(1);
            }
        }
        ScrollEvent::PageBackward => {
            let line_heights = wrapped_reversed_line_width_iter(
                &state.lines,
                state.v_offset,
                width,
                height,
                state.options.wrap,
            );
            let mut sub_offset = 0;
            let mut total_h = 0;
            for h in line_heights {
                sub_offset += 1;
                total_h += h;
                if total_h >= height {
                    state.v_offset -= sub_offset;
                    if total_h > height {
                        // if the first line is wrapped, the offset should be increased by 1
                        state.v_offset += 1;
                    }
                    break;
                }
            }
            if total_h < height {
                // scroll to the top
                state.v_offset = 0;
            }
        }
        ScrollEvent::Top => {
            state.v_offset = 0;
        }
        ScrollEvent::End => {
            state.v_offset = state.lines.len().saturating_sub(1);
        }
        ScrollEvent::Right => {
            if state.h_offset < state.max_line_width.saturating_sub(1) {
                state.h_offset = state.h_offset.saturating_add(1);
            }
        }
        ScrollEvent::Left => {
            if state.h_offset > 0 {
                state.h_offset = state.h_offset.saturating_sub(1);
            }
        }
    }
    // reset the scroll event
    state.scroll_event = ScrollEvent::None;
}

fn wrapped_line_width_iter<'a>(
    lines: &'a [Line],
    offset: usize,
    width: usize,
    height: usize,
    wrap: bool,
) -> impl Iterator<Item = usize> + 'a {
    lines.iter().skip(offset).take(height).map(move |line| {
        if wrap {
            let line_str = line_to_string(line);
            let lines = textwrap::wrap(&line_str, width);
            lines.len()
        } else {
            1
        }
    })
}

fn wrapped_reversed_line_width_iter<'a>(
    lines: &'a [Line],
    offset: usize,
    width: usize,
    height: usize,
    wrap: bool,
) -> impl Iterator<Item = usize> + 'a {
    lines
        .iter()
        .take(offset)
        .rev()
        .take(height)
        .map(move |line| {
            if wrap {
                let line_str = line_to_string(line);
                let lines = textwrap::wrap(&line_str, width);
                lines.len()
            } else {
                1
            }
        })
}

fn line_to_string(line: &Line) -> String {
    line.styled_graphemes(Style::default())
        .map(|g| g.symbol)
        .collect::<Vec<&str>>()
        .concat()
}

#[cfg(test)]
mod tests {
    use crate::set_cells;

    use super::*;

    #[test]
    fn test_scroll_lines_scroll() {
        let mut state = state(true, true);

        let buf = render_scroll_lines(&mut state);

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌TITLE─────────────┐",
            "│  1 aaa bbb ccc   │",
            "│    ddd           │",
            "│  2 aaa bbb ccc   │",
            "│  3 aaa           │",
            "│  4 aaa bbb       │",
            "└──────────────────┘",
        ]);
        set_cells! { expected =>
            ([2, 3], [1, 3, 4, 5]) => fg: Color::DarkGray,
        }

        assert_eq!(buf, expected);

        state.scroll_forward();

        let buf = render_scroll_lines(&mut state);

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌TITLE─────────────┐",
            "│  2 aaa bbb ccc   │",
            "│  3 aaa           │",
            "│  4 aaa bbb       │",
            "│  5 aaa bbb ccc   │",
            "│    ddd eee       │",
            "└──────────────────┘",
        ]);
        set_cells! { expected =>
            ([2, 3], [1, 2, 3, 4]) => fg: Color::DarkGray,
        }

        assert_eq!(buf, expected);

        state.scroll_forward();

        let buf = render_scroll_lines(&mut state);

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌TITLE─────────────┐",
            "│  3 aaa           │",
            "│  4 aaa bbb       │",
            "│  5 aaa bbb ccc   │",
            "│    ddd eee       │",
            "│  6 aaaaaaaa      │",
            "└──────────────────┘",
        ]);
        set_cells! { expected =>
            ([2, 3], [1, 2, 3, 5]) => fg: Color::DarkGray,
        }

        assert_eq!(buf, expected);

        state.scroll_page_forward();

        let buf = render_scroll_lines(&mut state);

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌TITLE─────────────┐",
            "│  6 aaaaaaaa      │",
            "│    bbbbbbbb      │",
            "│  7               │",
            "│  8 0123456789012 │",
            "│    3456789       │",
            "└──────────────────┘",
        ]);
        set_cells! { expected =>
            ([2, 3], [1, 3, 4]) => fg: Color::DarkGray,
        }

        assert_eq!(buf, expected);

        state.scroll_page_forward();

        let buf = render_scroll_lines(&mut state);

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌TITLE─────────────┐",
            "│  9 a             │",
            "│ 10 b             │",
            "│ 11 c             │",
            "│ 12 d             │",
            "│ 13 e             │",
            "└──────────────────┘",
        ]);
        set_cells! { expected =>
            ([2, 3], [1, 2, 3, 4, 5]) => fg: Color::DarkGray,
        }

        assert_eq!(buf, expected);

        state.scroll_to_end();

        let buf = render_scroll_lines(&mut state);

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌TITLE─────────────┐",
            "│ 16 g             │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "└──────────────────┘",
        ]);
        set_cells! { expected =>
            ([2, 3], [1]) => fg: Color::DarkGray,
        }

        assert_eq!(buf, expected);

        state.scroll_page_backward();

        let buf = render_scroll_lines(&mut state);

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌TITLE─────────────┐",
            "│ 13 e             │",
            "│ 14 aaa bbb ccc   │",
            "│    ddd eee fff   │",
            "│    ggg           │",
            "│ 15 f             │",
            "└──────────────────┘",
        ]);
        set_cells! { expected =>
            ([2, 3], [1, 2, 5]) => fg: Color::DarkGray,
        }

        assert_eq!(buf, expected);

        state.scroll_page_backward();

        let buf = render_scroll_lines(&mut state);

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌TITLE─────────────┐",
            "│  9 a             │",
            "│ 10 b             │",
            "│ 11 c             │",
            "│ 12 d             │",
            "│ 13 e             │",
            "└──────────────────┘",
        ]);
        set_cells! { expected =>
            ([2, 3], [1, 2, 3, 4, 5]) => fg: Color::DarkGray,
        }

        assert_eq!(buf, expected);

        state.scroll_backward();

        let buf = render_scroll_lines(&mut state);

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌TITLE─────────────┐",
            "│  8 0123456789012 │",
            "│    3456789       │",
            "│  9 a             │",
            "│ 10 b             │",
            "│ 11 c             │",
            "└──────────────────┘",
        ]);
        set_cells! { expected =>
            ([2, 3], [1, 3, 4, 5]) => fg: Color::DarkGray,
        }

        assert_eq!(buf, expected);

        state.scroll_to_top();

        let buf = render_scroll_lines(&mut state);

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌TITLE─────────────┐",
            "│  1 aaa bbb ccc   │",
            "│    ddd           │",
            "│  2 aaa bbb ccc   │",
            "│  3 aaa           │",
            "│  4 aaa bbb       │",
            "└──────────────────┘",
        ]);
        set_cells! { expected =>
            ([2, 3], [1, 3, 4, 5]) => fg: Color::DarkGray,
        }

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_scroll_lines_options() {
        let mut state = state(true, true);

        let buf = render_scroll_lines(&mut state);

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌TITLE─────────────┐",
            "│  1 aaa bbb ccc   │",
            "│    ddd           │",
            "│  2 aaa bbb ccc   │",
            "│  3 aaa           │",
            "│  4 aaa bbb       │",
            "└──────────────────┘",
        ]);
        set_cells! { expected =>
            ([2, 3], [1, 3, 4, 5]) => fg: Color::DarkGray,
        }

        assert_eq!(buf, expected);

        state.toggle_number();

        let buf = render_scroll_lines(&mut state);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "┌TITLE─────────────┐",
            "│ aaa bbb ccc ddd  │",
            "│ aaa bbb ccc      │",
            "│ aaa              │",
            "│ aaa bbb          │",
            "│ aaa bbb ccc ddd  │",
            "└──────────────────┘",
        ]);

        assert_eq!(buf, expected);

        state.toggle_number();
        state.toggle_wrap();

        let buf = render_scroll_lines(&mut state);

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌TITLE─────────────┐",
            "│  1 aaa bbb ccc d │",
            "│  2 aaa bbb ccc   │",
            "│  3 aaa           │",
            "│  4 aaa bbb       │",
            "│  5 aaa bbb ccc d │",
            "└──────────────────┘",
        ]);
        set_cells! { expected =>
            ([2, 3], [1, 2, 3, 4, 5]) => fg: Color::DarkGray,
        }

        assert_eq!(buf, expected);

        state.scroll_right();

        let buf = render_scroll_lines(&mut state);

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌TITLE─────────────┐",
            "│  1 aa bbb ccc dd │",
            "│  2 aa bbb ccc    │",
            "│  3 aa            │",
            "│  4 aa bbb        │",
            "│  5 aa bbb ccc dd │",
            "└──────────────────┘",
        ]);
        set_cells! { expected =>
            ([2, 3], [1, 2, 3, 4, 5]) => fg: Color::DarkGray,
        }

        assert_eq!(buf, expected);

        state.toggle_number();

        let buf = render_scroll_lines(&mut state);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "┌TITLE─────────────┐",
            "│ aa bbb ccc ddd   │",
            "│ aa bbb ccc       │",
            "│ aa               │",
            "│ aa bbb           │",
            "│ aa bbb ccc ddd e │",
            "└──────────────────┘",
        ]);

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_scroll_lines_no_block() {
        let mut state = state(true, true);

        let buf = render_scroll_lines_no_block(&mut state);

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "  1 aaa bbb ccc ddd ",
            "  2 aaa bbb ccc     ",
            "  3 aaa             ",
            "  4 aaa bbb         ",
            "  5 aaa bbb ccc ddd ",
            "    eee             ",
            "  6 aaaaaaaa        ",
        ]);
        set_cells! { expected =>
            ([1, 2], [0, 1, 2, 3, 4, 6]) => fg: Color::DarkGray,
        }

        assert_eq!(buf, expected);
    }

    fn state(number: bool, wrap: bool) -> ScrollLinesState {
        let lines: Vec<Line> = [
            "aaa bbb ccc ddd",
            "aaa bbb ccc",
            "aaa",
            "aaa bbb ",
            "aaa bbb ccc ddd eee",
            "aaaaaaaa bbbbbbbb",
            "",
            "01234567890123456789",
            "a",
            "b",
            "c",
            "d",
            "e",
            "aaa bbb ccc ddd eee fff ggg",
            "f",
            "g",
        ]
        .iter()
        .cloned()
        .map(Line::raw)
        .collect();
        let options = ScrollLinesOptions { number, wrap };
        ScrollLinesState::new(lines, options)
    }

    fn render_scroll_lines(state: &mut ScrollLinesState) -> Buffer {
        let theme = ColorTheme::default();
        let scroll_lines = ScrollLines::default()
            .block(Block::bordered().title("TITLE"))
            .theme(&theme);
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 5 + 2));
        scroll_lines.render(buf.area, &mut buf, state);
        buf
    }

    fn render_scroll_lines_no_block(state: &mut ScrollLinesState) -> Buffer {
        let theme = ColorTheme::default();
        let scroll_lines = ScrollLines::default().theme(&theme);
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 5 + 2));
        scroll_lines.render(buf.area, &mut buf, state);
        buf
    }
}
