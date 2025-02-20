use std::rc::Rc;

use laurier::{key_code, key_code_char};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    Frame,
};

use crate::{
    app::AppContext,
    environment::ImagePicker,
    event::{AppEventType, Sender},
    object::{FileDetail, ObjectKey, RawObject},
    pages::util::{build_helps, build_short_helps},
    widget::{
        self, ImagePreview, ImagePreviewState, InputDialog, InputDialogState, TextPreview,
        TextPreviewState,
    },
};

#[derive(Debug)]
pub struct ObjectPreviewPage {
    preview_type: PreviewType,

    file_detail: FileDetail,
    file_version_id: Option<String>,
    object: RawObject,
    path: String,
    object_key: ObjectKey,

    view_state: ViewState,

    ctx: Rc<AppContext>,
    tx: Sender,
}

#[derive(Debug)]
enum PreviewType {
    Text(TextPreviewState),
    Image(ImagePreviewState),
}

#[derive(Debug, Default)]
enum ViewState {
    #[default]
    Default,
    SaveDialog(InputDialogState),
}

impl ObjectPreviewPage {
    pub fn new(
        file_detail: FileDetail,
        file_version_id: Option<String>,
        object: RawObject,
        path: String,
        object_key: ObjectKey,
        ctx: Rc<AppContext>,
        tx: Sender,
    ) -> Self {
        let preview_type = if infer::is_image(&object.bytes) {
            let (state, msg) =
                ImagePreviewState::new(&object.bytes, ctx.env.image_picker.clone().into());
            if let Some(msg) = msg {
                tx.send(AppEventType::NotifyWarn(msg));
            }
            PreviewType::Image(state)
        } else {
            let (state, msg) = TextPreviewState::new(
                &file_detail,
                &object,
                ctx.config.preview.highlight,
                &ctx.config.preview.highlight_theme,
            );
            if let Some(msg) = msg {
                tx.send(AppEventType::NotifyWarn(msg));
            }
            PreviewType::Text(state)
        };

        Self {
            preview_type,
            object,
            file_detail,
            file_version_id,
            path,
            object_key,
            view_state: ViewState::Default,
            ctx,
            tx,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match (&mut self.view_state, &mut self.preview_type) {
            (ViewState::Default, PreviewType::Text(state)) => match key {
                key_code!(KeyCode::Esc) => {
                    self.tx.send(AppEventType::Quit);
                }
                key_code!(KeyCode::Backspace) | key_code!(KeyCode::Left) => {
                    self.tx.send(AppEventType::CloseCurrentPage);
                }
                key_code_char!('j') => {
                    state.scroll_lines_state.scroll_forward();
                }
                key_code_char!('k') => {
                    state.scroll_lines_state.scroll_backward();
                }
                key_code_char!('f') => {
                    state.scroll_lines_state.scroll_page_forward();
                }
                key_code_char!('b') => {
                    state.scroll_lines_state.scroll_page_backward();
                }
                key_code_char!('g') => {
                    state.scroll_lines_state.scroll_to_top();
                }
                key_code_char!('G') => {
                    state.scroll_lines_state.scroll_to_end();
                }
                key_code_char!('h') => {
                    state.scroll_lines_state.scroll_left();
                }
                key_code_char!('l') => {
                    state.scroll_lines_state.scroll_right();
                }
                key_code_char!('w') => {
                    state.scroll_lines_state.toggle_wrap();
                }
                key_code_char!('n') => {
                    state.scroll_lines_state.toggle_number();
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
            (ViewState::Default, PreviewType::Image(_)) => match key {
                key_code!(KeyCode::Esc) => {
                    self.tx.send(AppEventType::Quit);
                }
                key_code!(KeyCode::Backspace) | key_code!(KeyCode::Left) => {
                    self.tx.send(AppEventType::CloseCurrentPage);
                }
                key_code_char!('s') => {
                    self.download();
                }
                key_code_char!('S') => {
                    self.open_save_dialog();
                    self.disable_image_render();
                }
                key_code_char!('?') => {
                    self.tx.send(AppEventType::OpenHelp);
                }
                _ => {}
            },
            (ViewState::SaveDialog(state), _) => match key {
                key_code!(KeyCode::Esc) => {
                    self.close_save_dialog();
                    self.enable_image_render();
                }
                key_code!(KeyCode::Enter) | key_code!(KeyCode::Right) => {
                    let input = state.input().into();
                    self.download_as(input);
                    // enable_image_render is called after download is completed
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
        match self.preview_type {
            PreviewType::Text(ref mut state) => {
                let preview = TextPreview::new(
                    self.file_detail.name.as_str(),
                    self.file_version_id.as_deref(),
                    &self.ctx.theme,
                );
                f.render_stateful_widget(preview, area, state);
            }
            PreviewType::Image(ref mut state) => {
                let preview = ImagePreview::new(
                    self.file_detail.name.as_str(),
                    self.file_version_id.as_deref(),
                );
                f.render_stateful_widget(preview, area, state);
            }
        }

        if let ViewState::SaveDialog(state) = &mut self.view_state {
            let save_dialog = InputDialog::default()
                .title("Save As")
                .max_width(40)
                .theme(&self.ctx.theme);
            f.render_stateful_widget(save_dialog, area, state);

            let (cursor_x, cursor_y) = state.cursor();
            f.set_cursor_position((cursor_x, cursor_y));
        }
    }

    pub fn helps(&self) -> Vec<String> {
        let helps: &[(&[&str], &str)] = match (&self.view_state, &self.preview_type) {
            (ViewState::Default, PreviewType::Text(_)) => &[
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
            (ViewState::Default, PreviewType::Image(_)) => &[
                (&["Esc", "Ctrl-c"], "Quit app"),
                (&["Backspace"], "Close preview"),
                (&["s"], "Download object"),
                (&["S"], "Download object as"),
            ],
            (ViewState::SaveDialog(_), _) => &[
                (&["Ctrl-c"], "Quit app"),
                (&["Esc"], "Close save dialog"),
                (&["Enter"], "Download object"),
            ],
        };

        build_helps(helps)
    }

    pub fn short_helps(&self) -> Vec<(String, usize)> {
        let helps: &[(&[&str], &str, usize)] = match (&self.view_state, &self.preview_type) {
            (ViewState::Default, PreviewType::Text(_)) => &[
                (&["Esc"], "Quit", 0),
                (&["j/k"], "Scroll", 2),
                (&["g/G"], "Top/End", 4),
                (&["s/S"], "Download", 3),
                (&["Backspace"], "Close", 1),
                (&["?"], "Help", 0),
            ],
            (ViewState::Default, PreviewType::Image(_)) => &[
                (&["Esc"], "Quit", 0),
                (&["s/S"], "Download", 2),
                (&["Backspace"], "Close", 1),
                (&["?"], "Help", 0),
            ],
            (ViewState::SaveDialog(_), _) => &[
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
        self.view_state = ViewState::SaveDialog(InputDialogState::default());
    }

    pub fn close_save_dialog(&mut self) {
        self.view_state = ViewState::Default;
    }

    pub fn enable_image_render(&mut self) {
        if let PreviewType::Image(state) = &mut self.preview_type {
            state.set_render(true);
        }
    }

    pub fn disable_image_render(&mut self) {
        if let PreviewType::Image(state) = &mut self.preview_type {
            state.set_render(false);
        }
    }

    pub fn is_image_preview(&self) -> bool {
        matches!(self.preview_type, PreviewType::Image(_))
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

    pub fn current_object_key(&self) -> &ObjectKey {
        &self.object_key
    }
}

impl From<ImagePicker> for widget::ImagePicker {
    fn from(value: ImagePicker) -> Self {
        match value {
            ImagePicker::Disabled => widget::ImagePicker::Disabled,
            ImagePicker::Ok(picker) => widget::ImagePicker::Ok(picker),
            ImagePicker::Error(e) => widget::ImagePicker::Error(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{event, set_cells};

    use super::*;
    use chrono::{DateTime, Local, NaiveDateTime};
    use ratatui::{backend::TestBackend, buffer::Buffer, style::Color, Terminal};

    fn object(ss: &[&str]) -> RawObject {
        RawObject {
            bytes: ss.join("\n").as_bytes().to_vec(),
        }
    }

    #[test]
    fn test_render_without_scroll() -> std::io::Result<()> {
        let ctx = Rc::default();
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
            let object_key = ObjectKey {
                bucket_name: "test-bucket".to_string(),
                object_path: vec![file_path.clone()],
            };
            let mut page =
                ObjectPreviewPage::new(file_detail, None, object, file_path, object_key, ctx, tx);
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
        let ctx = Rc::default();
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let file_detail = file_detail();
            let preview = ["Hello, world!"; 20];
            let object = object(&preview);
            let file_path = "file.txt".to_string();
            let object_key = ObjectKey {
                bucket_name: "test-bucket".to_string(),
                object_path: vec![file_path.clone()],
            };
            let mut page =
                ObjectPreviewPage::new(file_detail, None, object, file_path, object_key, ctx, tx);
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
        let ctx = Rc::default();
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
            let object_key = ObjectKey {
                bucket_name: "test-bucket".to_string(),
                object_path: vec![file_path.clone()],
            };
            let mut page =
                ObjectPreviewPage::new(file_detail, None, object, file_path, object_key, ctx, tx);
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
