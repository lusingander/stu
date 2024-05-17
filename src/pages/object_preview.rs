use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{block::Title, Block, BorderType, Padding, Paragraph},
    Frame,
};

use crate::{
    event::{AppEventType, AppKeyInput},
    object::FileDetail,
    widget::Dialog,
};

const PREVIEW_LINE_NUMBER_COLOR: Color = Color::DarkGray;

#[derive(Debug)]
pub struct ObjectPreviewPage {
    file_detail: FileDetail,

    preview: Vec<String>,
    preview_max_digits: usize,

    save_dialog_state: Option<SaveDialogState>,
    offset: usize,
}

#[derive(Debug, Default)]
struct SaveDialogState {
    input: String,
    cursor: u16,
}

impl ObjectPreviewPage {
    pub fn new(file_detail: FileDetail, preview: Vec<String>, preview_max_digits: usize) -> Self {
        Self {
            file_detail,
            preview,
            preview_max_digits,
            save_dialog_state: None,
            offset: 0,
        }
    }

    pub fn handle_event(&mut self, event: AppEventType) {
        if let AppEventType::KeyInput(input) = event {
            self.handle_key_input(input);
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let content_area = area.inner(&Margin::new(1, 1)); // border

        let preview_max_digits = self.preview_max_digits;
        let show_lines_count = content_area.height as usize;
        let content_max_width = (content_area.width as usize) - preview_max_digits - 3 /* pad */;

        let content: Vec<Line> = ((self.offset + 1)..)
            .zip(self.preview.iter().skip(self.offset))
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

        let title = format!("Preview [{}]", &self.file_detail.name);

        let paragraph = Paragraph::new(content).block(
            Block::bordered()
                .title(title)
                .padding(Padding::horizontal(1)),
        );

        f.render_widget(paragraph, area);

        if let Some(state) = &self.save_dialog_state {
            let dialog_width = (area.width - 4).min(40);
            let dialog_height = 1 + 2 /* border */;
            let area = calc_centered_dialog_rect(area, dialog_width, dialog_height);

            let max_width = dialog_width - 2 /* border */- 2/* pad */;
            let input_width = state.input.len().saturating_sub(max_width as usize);
            let input_view: &str = &state.input[input_width..];

            let title = Title::from("Save As");
            let dialog_content = Paragraph::new(input_view).block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title(title)
                    .padding(Padding::horizontal(1)),
            );
            let dialog = Dialog::new(Box::new(dialog_content));
            f.render_widget_ref(dialog, area);

            let cursor_x = area.x + state.cursor.min(max_width) + 1 /* border */ + 1/* pad */;
            let cursor_y = area.y + 1;
            f.set_cursor(cursor_x, cursor_y);
        }
    }

    pub fn open_save_dialog(&mut self) {
        self.save_dialog_state = Some(SaveDialogState::default());
    }

    pub fn close_save_dialog(&mut self) {
        self.save_dialog_state = None;
    }

    pub fn scroll_forward(&mut self) {
        if self.offset < self.preview.len() - 1 {
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
        self.offset = self.preview.len() - 1;
    }

    fn handle_key_input(&mut self, input: AppKeyInput) {
        if let Some(ref mut state) = self.save_dialog_state {
            match input {
                AppKeyInput::Char(c) => {
                    if c == '?' {
                        return;
                    }
                    state.input.push(c);
                    state.cursor = state.cursor.saturating_add(1);
                }
                AppKeyInput::Backspace => {
                    state.input.pop();
                    state.cursor = state.cursor.saturating_sub(1);
                }
            }
        }
    }

    pub fn file_detail(&self) -> &FileDetail {
        &self.file_detail
    }

    pub fn save_dialog_key_input(&self) -> Option<String> {
        self.save_dialog_state.as_ref().map(|s| s.input.clone())
    }
}

fn calc_centered_dialog_rect(r: Rect, dialog_width: u16, dialog_height: u16) -> Rect {
    let vertical_pad = (r.height - dialog_height) / 2;
    let vertical_layout = Layout::vertical(Constraint::from_lengths([
        vertical_pad,
        dialog_height,
        vertical_pad,
    ]))
    .split(r);

    let horizontal_pad = (r.width - dialog_width) / 2;
    Layout::horizontal(Constraint::from_lengths([
        horizontal_pad,
        dialog_width,
        horizontal_pad,
    ]))
    .split(vertical_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Local};
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

    #[test]
    fn test_render_without_scroll() -> std::io::Result<()> {
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let file_detail = file_detail();
            let preview = [
                "Hello, world!",
                "This is a test file.",
                "This file is used for testing.",
                "Thank you!",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect();
            let preview_max_digits = 1;
            let mut page = ObjectPreviewPage::new(file_detail, preview, preview_max_digits);
            let area = Rect::new(0, 0, 30, 10);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌Preview [file.txt]──────────┐",
            "│ 1 Hello, world!            │",
            "│ 2 This is a test file.     │",
            "│ 3 This file is used for    │",
            "│   testing.                 │",
            "│ 4 Thank you!               │",
            "│                            │",
            "│                            │",
            "│                            │",
            "└────────────────────────────┘",
        ]);
        for y in 1..6 {
            expected.get_mut(2, y).set_fg(Color::DarkGray);
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn test_render_with_scroll() -> std::io::Result<()> {
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let file_detail = file_detail();
            let preview = vec!["Hello, world!".to_string(); 20];
            let preview_max_digits = 2;
            let mut page = ObjectPreviewPage::new(file_detail, preview, preview_max_digits);
            let area = Rect::new(0, 0, 30, 10);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌Preview [file.txt]──────────┐",
            "│  1 Hello, world!           │",
            "│  2 Hello, world!           │",
            "│  3 Hello, world!           │",
            "│  4 Hello, world!           │",
            "│  5 Hello, world!           │",
            "│  6 Hello, world!           │",
            "│  7 Hello, world!           │",
            "│  8 Hello, world!           │",
            "└────────────────────────────┘",
        ]);
        for y in 1..9 {
            for x in 2..4 {
                expected.get_mut(x, y).set_fg(Color::DarkGray);
            }
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn test_render_save_dialog_without_scroll() -> std::io::Result<()> {
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let file_detail = file_detail();
            let preview = [
                "Hello, world!",
                "This is a test file.",
                "This file is used for testing.",
                "Thank you!",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect();
            let preview_max_digits = 1;
            let mut page = ObjectPreviewPage::new(file_detail, preview, preview_max_digits);
            page.open_save_dialog();
            let area = Rect::new(0, 0, 30, 10);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌Preview [file.txt]──────────┐",
            "│ 1 Hello, world!            │",
            "│ 2 This is a test file.     │",
            "│ ╭Save As─────────────────╮ │",
            "│ │                        │ │",
            "│ ╰────────────────────────╯ │",
            "│                            │",
            "│                            │",
            "│                            │",
            "└────────────────────────────┘",
        ]);
        for y in 1..3 {
            expected.get_mut(2, y).set_fg(Color::DarkGray);
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    fn parse_datetime(s: &str) -> DateTime<Local> {
        DateTime::parse_from_rfc3339(s)
            .unwrap()
            .with_timezone(&Local)
    }

    fn setup_terminal() -> std::io::Result<Terminal<TestBackend>> {
        let backend = TestBackend::new(30, 10);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(terminal)
    }

    fn file_detail() -> FileDetail {
        FileDetail {
            name: "file.txt".to_string(),
            size_byte: 1024 + 10,
            last_modified: parse_datetime("2024-01-02T13:01:02+09:00"),
            e_tag: "bef684de-a260-48a4-8178-8a535ecccadb".to_string(),
            content_type: "text/plain".to_string(),
            storage_class: "STANDARD".to_string(),
            key: "file.txt".to_string(),
            s3_uri: "s3://bucket-1/file.txt".to_string(),
            arn: "arn:aws:s3:::bucket-1/file.txt".to_string(),
            object_url: "https://bucket-1.s3.ap-northeast-1.amazonaws.com/file.txt".to_string(),
        }
    }
}
