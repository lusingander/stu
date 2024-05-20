use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, Padding, Paragraph},
    Frame,
};

use crate::{
    event::{AppEventType, Sender},
    key_code, key_code_char,
    object::{FileDetail, Object},
    pages::util::{build_helps, build_short_helps},
    util::{digits, to_preview_string},
    widget::{SaveDialog, SaveDialogState},
};

const PREVIEW_LINE_NUMBER_COLOR: Color = Color::DarkGray;

#[derive(Debug)]
pub struct ObjectPreviewPage {
    file_detail: FileDetail,

    preview: Vec<String>,
    preview_max_digits: usize,
    object: Object,
    path: String,

    save_dialog_state: Option<SaveDialogState>,
    offset: usize,
    tx: Sender,
}

impl ObjectPreviewPage {
    pub fn new(file_detail: FileDetail, object: Object, path: String, tx: Sender) -> Self {
        let s = to_preview_string(&object.bytes, &object.content_type);
        let s = if s.ends_with('\n') {
            s.trim_end()
        } else {
            s.as_str()
        };
        let preview: Vec<String> = s.split('\n').map(|s| s.to_string()).collect();
        let preview_len = preview.len();
        let preview_max_digits = digits(preview_len);

        Self {
            file_detail,
            preview,
            preview_max_digits,
            object,
            path,
            save_dialog_state: None,
            offset: 0,
            tx,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.save_dialog_state.is_some() {
            match key {
                key_code!(KeyCode::Esc) => {
                    // todo: should not quit
                    self.tx.send(AppEventType::Quit);
                }
                key_code!(KeyCode::Enter) => {
                    self.tx.send(AppEventType::PreviewDownloadObjectAs);
                }
                key_code_char!('?') => {
                    self.tx.send(AppEventType::OpenHelp);
                }
                key_code_char!(c) => {
                    // todo: fix
                    self.add_char_to_input(c);
                }
                key_code!(KeyCode::Backspace) => {
                    // todo: fix
                    self.delete_char_from_input();
                }
                _ => {}
            }
            return;
        }

        match key {
            key_code!(KeyCode::Esc) => {
                self.tx.send(AppEventType::Quit);
            }
            key_code!(KeyCode::Backspace) => {
                self.tx.send(AppEventType::CloseCurrentPage);
            }
            key_code_char!('j') => {
                self.scroll_forward();
            }
            key_code_char!('k') => {
                self.scroll_backward();
            }
            key_code_char!('g') => {
                self.scroll_to_top();
            }
            key_code_char!('G') => {
                self.scroll_to_end();
            }
            key_code_char!('s') => {
                self.tx.send(AppEventType::PreviewDownloadObject);
            }
            key_code_char!('S') => {
                self.open_save_dialog();
            }
            key_code_char!('?') => {
                self.tx.send(AppEventType::OpenHelp);
            }
            _ => {}
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

        if let Some(state) = &mut self.save_dialog_state {
            let save_dialog = SaveDialog::default();
            f.render_stateful_widget(save_dialog, area, state);

            let (cursor_x, cursor_y) = state.cursor();
            f.set_cursor(cursor_x, cursor_y);
        }
    }

    pub fn helps(&self) -> Vec<String> {
        if self.save_dialog_state.is_some() {
            let helps: &[(&[&str], &str)] = &[
                (&["Esc", "Ctrl-c"], "Quit app"),
                (&["Enter"], "Download object"),
            ];
            return build_helps(helps);
        }

        let helps: &[(&[&str], &str)] = &[
            (&["Esc", "Ctrl-c"], "Quit app"),
            (&["j/k"], "Scroll forward/backward"),
            (&["g/G"], "Scroll to top/end"),
            (&["Backspace"], "Close preview"),
            (&["s"], "Download object"),
            (&["S"], "Download object as"),
        ];
        build_helps(helps)
    }

    pub fn short_helps(&self) -> Vec<(String, usize)> {
        if self.save_dialog_state.is_some() {
            let helps: &[(&[&str], &str, usize)] = &[
                (&["Esc"], "Quit", 0),
                (&["Enter"], "Download", 1),
                (&["?"], "Help", 0),
            ];
            return build_short_helps(helps);
        }

        let helps: &[(&[&str], &str, usize)] = &[
            (&["Esc"], "Quit", 0),
            (&["j/k"], "Scroll", 2),
            (&["g/G"], "Top/End", 4),
            (&["s/S"], "Download", 3),
            (&["Backspace"], "Close", 2),
            (&["?"], "Help", 0),
        ];
        build_short_helps(helps)
    }
}

impl ObjectPreviewPage {
    fn open_save_dialog(&mut self) {
        self.save_dialog_state = Some(SaveDialogState::default());
    }

    pub fn close_save_dialog(&mut self) {
        self.save_dialog_state = None;
    }

    fn scroll_forward(&mut self) {
        if self.offset < self.preview.len() - 1 {
            self.offset = self.offset.saturating_add(1);
        }
    }

    fn scroll_backward(&mut self) {
        if self.offset > 0 {
            self.offset = self.offset.saturating_sub(1);
        }
    }

    fn scroll_to_top(&mut self) {
        self.offset = 0;
    }

    fn scroll_to_end(&mut self) {
        self.offset = self.preview.len() - 1;
    }

    fn add_char_to_input(&mut self, c: char) {
        if let Some(ref mut state) = self.save_dialog_state {
            state.add_char(c);
        }
    }

    fn delete_char_from_input(&mut self) {
        if let Some(ref mut state) = self.save_dialog_state {
            state.delete_char();
        }
    }

    pub fn file_detail(&self) -> &FileDetail {
        &self.file_detail
    }

    pub fn save_dialog_key_input(&self) -> Option<String> {
        self.save_dialog_state.as_ref().map(|s| s.input().into())
    }

    pub fn object(&self) -> &Object {
        &self.object
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use crate::{event, set_cells};

    use super::*;
    use chrono::{DateTime, Local};
    use itertools::Itertools;
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

    fn object(ss: &[&str]) -> Object {
        Object {
            content_type: "text/plain".to_string(),
            bytes: ss.iter().join("\n").as_bytes().to_vec(),
        }
    }

    #[test]
    fn test_render_without_scroll() -> std::io::Result<()> {
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let file_detail = file_detail();
            let preview = [
                "Hello, world!",
                "This is a test file.",
                "This file is used for testing.",
                "Thank you!",
            ];
            let object = object(&preview);
            let file_path = "file.txt".to_string();
            let mut page = ObjectPreviewPage::new(file_detail, object, file_path, tx);
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
        set_cells! { expected =>
            ([2], 1..6) => fg: Color::DarkGray,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn test_render_with_scroll() -> std::io::Result<()> {
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let file_detail = file_detail();
            let preview = ["Hello, world!"; 20];
            let object = object(&preview);
            let file_path = "file.txt".to_string();
            let mut page = ObjectPreviewPage::new(file_detail, object, file_path, tx);
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
        set_cells! { expected =>
            (2..4, 1..9) => fg: Color::DarkGray,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn test_render_save_dialog_without_scroll() -> std::io::Result<()> {
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let file_detail = file_detail();
            let preview = [
                "Hello, world!",
                "This is a test file.",
                "This file is used for testing.",
                "Thank you!",
            ];
            let object = object(&preview);
            let file_path = "file.txt".to_string();
            let mut page = ObjectPreviewPage::new(file_detail, object, file_path, tx);
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
        set_cells! { expected =>
            ([2], 1..3) => fg: Color::DarkGray,
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
