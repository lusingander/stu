use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    config::PreviewConfig,
    event::{AppEventType, Sender},
    key_code, key_code_char,
    object::{FileDetail, RawObject},
    pages::util::{build_helps, build_short_helps},
    widget::{SaveDialog, SaveDialogState, TextPreview, TextPreviewState},
};

#[derive(Debug)]
pub struct ObjectPreviewPage {
    state: TextPreviewState,

    file_detail: FileDetail,
    file_version_id: Option<String>,
    object: RawObject,
    path: String,

    view_state: ViewState,

    tx: Sender,
}

#[derive(Debug, Default)]
enum ViewState {
    #[default]
    Default,
    SaveDialog(SaveDialogState),
}

impl ObjectPreviewPage {
    pub fn new(
        file_detail: FileDetail,
        file_version_id: Option<String>,
        object: RawObject,
        path: String,
        preview_config: PreviewConfig,
        tx: Sender,
    ) -> Self {
        let (state, msg) = TextPreviewState::new(&file_detail, &object, preview_config.highlight);
        if let Some(msg) = msg {
            tx.send(AppEventType::NotifyWarn(msg));
        }

        Self {
            state,
            object,
            file_detail,
            file_version_id,
            path,
            view_state: ViewState::Default,
            tx,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.view_state {
            ViewState::Default => match key {
                key_code!(KeyCode::Esc) => {
                    self.tx.send(AppEventType::Quit);
                }
                key_code!(KeyCode::Backspace) => {
                    self.tx.send(AppEventType::CloseCurrentPage);
                }
                key_code_char!('j') => {
                    self.state.scroll_lines_state.scroll_forward();
                }
                key_code_char!('k') => {
                    self.state.scroll_lines_state.scroll_backward();
                }
                key_code_char!('f') => {
                    self.state.scroll_lines_state.scroll_page_forward();
                }
                key_code_char!('b') => {
                    self.state.scroll_lines_state.scroll_page_backward();
                }
                key_code_char!('g') => {
                    self.state.scroll_lines_state.scroll_to_top();
                }
                key_code_char!('G') => {
                    self.state.scroll_lines_state.scroll_to_end();
                }
                key_code_char!('h') => {
                    self.state.scroll_lines_state.scroll_left();
                }
                key_code_char!('l') => {
                    self.state.scroll_lines_state.scroll_right();
                }
                key_code_char!('w') => {
                    self.state.scroll_lines_state.toggle_wrap();
                }
                key_code_char!('n') => {
                    self.state.scroll_lines_state.toggle_number();
                }
                key_code_char!('s') => {
                    self.download();
                }
                key_code_char!('S') => {
                    self.open_save_dialog();
                }
                key_code_char!('?') => {
                    self.tx.send(AppEventType::OpenHelp);
                }
                _ => {}
            },
            ViewState::SaveDialog(ref mut state) => match key {
                key_code!(KeyCode::Esc) => {
                    self.close_save_dialog();
                }
                key_code!(KeyCode::Enter) => {
                    let input = state.input().into();
                    self.download_as(input);
                }
                key_code_char!('?') => {
                    self.tx.send(AppEventType::OpenHelp);
                }
                _ => {
                    state.handle_key_event(key);
                }
            },
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let preview = TextPreview::new(
            self.file_detail.name.as_str(),
            self.file_version_id.as_deref(),
        );
        f.render_stateful_widget(preview, area, &mut self.state);

        if let ViewState::SaveDialog(state) = &mut self.view_state {
            let save_dialog = SaveDialog::default();
            f.render_stateful_widget(save_dialog, area, state);

            let (cursor_x, cursor_y) = state.cursor();
            f.set_cursor(cursor_x, cursor_y);
        }
    }

    pub fn helps(&self) -> Vec<String> {
        let helps: &[(&[&str], &str)] = match self.view_state {
            ViewState::Default => &[
                (&["Esc", "Ctrl-c"], "Quit app"),
                (&["j/k"], "Scroll forward/backward"),
                (&["f/b"], "Scroll page forward/backward"),
                (&["g/G"], "Scroll to top/end"),
                (&["h/l"], "Scroll left/right"),
                (&["w"], "Toggle wrap"),
                (&["n"], "Toggle number"),
                (&["Backspace"], "Close preview"),
                (&["s"], "Download object"),
                (&["S"], "Download object as"),
            ],
            ViewState::SaveDialog(_) => &[
                (&["Ctrl-c"], "Quit app"),
                (&["Esc"], "Close save dialog"),
                (&["Enter"], "Download object"),
            ],
        };

        build_helps(helps)
    }

    pub fn short_helps(&self) -> Vec<(String, usize)> {
        let helps: &[(&[&str], &str, usize)] = match self.view_state {
            ViewState::Default => &[
                (&["Esc"], "Quit", 0),
                (&["j/k"], "Scroll", 2),
                (&["g/G"], "Top/End", 4),
                (&["s/S"], "Download", 3),
                (&["Backspace"], "Close", 2),
                (&["?"], "Help", 0),
            ],
            ViewState::SaveDialog(_) => &[
                (&["Esc"], "Close", 2),
                (&["Enter"], "Download", 1),
                (&["?"], "Help", 0),
            ],
        };

        build_short_helps(helps)
    }
}

impl ObjectPreviewPage {
    fn open_save_dialog(&mut self) {
        self.view_state = ViewState::SaveDialog(SaveDialogState::default());
    }

    pub fn close_save_dialog(&mut self) {
        self.view_state = ViewState::Default;
    }

    fn download(&self) {
        // object has been already downloaded, so send completion event to save file
        let obj = self.object.clone();
        let path = self.path.clone();
        self.tx.send(AppEventType::PreviewDownloadObject(obj, path));
    }

    fn download_as(&self, input: String) {
        let input: String = input.trim().into();
        if input.is_empty() {
            return;
        }

        let file_detail = self.file_detail.clone();
        let version_id = self.file_version_id.clone();
        self.tx.send(AppEventType::PreviewDownloadObjectAs(
            file_detail,
            input,
            version_id,
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::{event, set_cells};

    use super::*;
    use chrono::{DateTime, Local, NaiveDateTime};
    use itertools::Itertools;
    use ratatui::{backend::TestBackend, buffer::Buffer, style::Color, Terminal};

    fn object(ss: &[&str]) -> RawObject {
        RawObject {
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
            let preview_config = PreviewConfig::default();
            let mut page =
                ObjectPreviewPage::new(file_detail, None, object, file_path, preview_config, tx);
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
            ([2], [1, 2, 3, 5]) => fg: Color::DarkGray,
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
            let preview_config = PreviewConfig::default();
            let mut page =
                ObjectPreviewPage::new(file_detail, None, object, file_path, preview_config, tx);
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
            let preview_config = PreviewConfig::default();
            let mut page =
                ObjectPreviewPage::new(file_detail, None, object, file_path, preview_config, tx);
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
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
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
            last_modified: parse_datetime("2024-01-02 13:01:02"),
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
