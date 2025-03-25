use std::{rc::Rc, sync::Arc};

use ratatui::{crossterm::event::KeyEvent, layout::Rect, Frame};

use crate::{
    app::AppContext,
    environment::ImagePicker,
    event::{AppEventType, Sender},
    handle_user_events, handle_user_events_with_default,
    help::{
        build_help_spans, build_short_help_spans, BuildHelpsItem, BuildShortHelpsItem, Spans,
        SpansWithPriority,
    },
    keys::{UserEvent, UserEventMapper},
    object::{FileDetail, RawObject},
    widget::{
        self, EncodingDialog, EncodingDialogState, ImagePreview, ImagePreviewState, InputDialog,
        InputDialogState, TextPreview, TextPreviewState,
    },
};

#[derive(Debug)]
pub struct ObjectPreviewPage {
    preview_type: PreviewType,

    file_detail: FileDetail,
    file_version_id: Option<String>,
    object: Arc<RawObject>,

    view_state: ViewState,
    encoding_dialog_state: EncodingDialogState,

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
    EncodingDialog,
}

impl ObjectPreviewPage {
    pub fn new(
        file_detail: FileDetail,
        file_version_id: Option<String>,
        object: RawObject,
        ctx: Rc<AppContext>,
        tx: Sender,
    ) -> Self {
        let encoding_dialog_state = EncodingDialogState::new(&ctx.config.preview.encodings);

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
                encoding_dialog_state.selected(),
            );
            if let Some(msg) = msg {
                tx.send(AppEventType::NotifyWarn(msg));
            }
            PreviewType::Text(state)
        };

        Self {
            preview_type,
            object: Arc::new(object),
            file_detail,
            file_version_id,
            view_state: ViewState::Default,
            encoding_dialog_state,
            ctx,
            tx,
        }
    }

    pub fn handle_key(&mut self, user_events: Vec<UserEvent>, key_event: KeyEvent) {
        match (&mut self.view_state, &mut self.preview_type) {
            (ViewState::Default, PreviewType::Text(state)) => {
                handle_user_events! { user_events =>
                    UserEvent::ObjectPreviewBack => {
                        self.tx.send(AppEventType::CloseCurrentPage);
                    }
                    UserEvent::ObjectPreviewDown => {
                        state.scroll_lines_state.scroll_forward();
                    }
                    UserEvent::ObjectPreviewUp => {
                        state.scroll_lines_state.scroll_backward();
                    }
                    UserEvent::ObjectPreviewPageDown => {
                        state.scroll_lines_state.scroll_page_forward();
                    }
                    UserEvent::ObjectPreviewPageUp => {
                        state.scroll_lines_state.scroll_page_backward();
                    }
                    UserEvent::ObjectPreviewGoToTop => {
                        state.scroll_lines_state.scroll_to_top();
                    }
                    UserEvent::ObjectPreviewGoToBottom => {
                        state.scroll_lines_state.scroll_to_end();
                    }
                    UserEvent::ObjectPreviewLeft => {
                        state.scroll_lines_state.scroll_left();
                    }
                    UserEvent::ObjectPreviewRight => {
                        state.scroll_lines_state.scroll_right();
                    }
                    UserEvent::ObjectPreviewToggleWrap => {
                        state.scroll_lines_state.toggle_wrap();
                    }
                    UserEvent::ObjectPreviewToggleNumber => {
                        state.scroll_lines_state.toggle_number();
                    }
                    UserEvent::ObjectPreviewDownload => {
                        self.download();
                    }
                    UserEvent::ObjectPreviewDownloadAs => {
                        self.open_save_dialog();
                    }
                    UserEvent::ObjectPreviewEncoding => {
                        self.open_encoding_dialog();
                    }
                    UserEvent::Help => {
                        self.tx.send(AppEventType::OpenHelp);
                    }
                }
            }
            (ViewState::Default, PreviewType::Image(_)) => {
                handle_user_events! { user_events =>
                    UserEvent::ObjectPreviewBack => {
                        self.tx.send(AppEventType::CloseCurrentPage);
                    }
                    UserEvent::ObjectPreviewDownload => {
                        self.download();
                        self.disable_image_render();
                    }
                    UserEvent::ObjectPreviewDownloadAs => {
                        self.open_save_dialog();
                        self.disable_image_render();
                    }
                    UserEvent::Help => {
                        self.tx.send(AppEventType::OpenHelp);
                    }
                }
            }
            (ViewState::SaveDialog(state), _) => {
                handle_user_events_with_default! { user_events =>
                    UserEvent::InputDialogClose => {
                        self.close_save_dialog();
                        self.enable_image_render();
                    }
                    UserEvent::InputDialogApply => {
                        let input = state.input().into();
                        self.download_as(input);
                        // enable_image_render is called after download is completed
                    }
                    UserEvent::Help => {
                        self.tx.send(AppEventType::OpenHelp);
                    }
                    => {
                        state.handle_key_event(key_event);
                    }
                }
            }
            (ViewState::EncodingDialog, _) => {
                handle_user_events! { user_events =>
                    UserEvent::SelectDialogClose => {
                        self.close_encoding_dialog();
                    }
                    UserEvent::SelectDialogDown => {
                        self.encoding_dialog_state.select_next();
                    }
                    UserEvent::SelectDialogUp => {
                        self.encoding_dialog_state.select_prev();
                    }
                    UserEvent::SelectDialogSelect => {
                        self.apply_encoding();
                    }
                    UserEvent::Help => {
                        self.tx.send(AppEventType::OpenHelp);
                    }
                }
            }
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

        if let ViewState::EncodingDialog = &mut self.view_state {
            let encoding_dialog =
                EncodingDialog::new(&self.encoding_dialog_state).theme(&self.ctx.theme);
            f.render_widget(encoding_dialog, area);
        }
    }

    pub fn helps(&self, mapper: &UserEventMapper) -> Vec<Spans> {
        #[rustfmt::skip]
        let helps = match (&self.view_state, &self.preview_type) {
            (ViewState::Default, PreviewType::Text(_)) => {
                vec![
                    BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
                    BuildHelpsItem::new(UserEvent::ObjectPreviewDown, "Scroll forward"),
                    BuildHelpsItem::new(UserEvent::ObjectPreviewUp, "Scroll backward"),
                    BuildHelpsItem::new(UserEvent::ObjectPreviewPageDown, "Scroll page forward"),
                    BuildHelpsItem::new(UserEvent::ObjectPreviewPageUp, "Scroll page backward"),
                    BuildHelpsItem::new(UserEvent::ObjectPreviewGoToTop, "Scroll to top"),
                    BuildHelpsItem::new(UserEvent::ObjectPreviewGoToBottom, "Scroll to end"),
                    BuildHelpsItem::new(UserEvent::ObjectPreviewLeft, "Scroll left"),
                    BuildHelpsItem::new(UserEvent::ObjectPreviewRight, "Scroll right"),
                    BuildHelpsItem::new(UserEvent::ObjectPreviewToggleWrap, "Toggle wrap"),
                    BuildHelpsItem::new(UserEvent::ObjectPreviewToggleNumber, "Toggle number"),
                    BuildHelpsItem::new(UserEvent::ObjectPreviewBack, "Close preview"),
                    BuildHelpsItem::new(UserEvent::ObjectPreviewDownload, "Download object"),
                    BuildHelpsItem::new(UserEvent::ObjectPreviewDownloadAs, "Download object as"),
                    BuildHelpsItem::new(UserEvent::ObjectPreviewEncoding, "Open encoding dialog"),
                ]
            },
            (ViewState::Default, PreviewType::Image(_)) => {
                vec![
                    BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
                    BuildHelpsItem::new(UserEvent::ObjectPreviewBack, "Close preview"),
                    BuildHelpsItem::new(UserEvent::ObjectPreviewDownload, "Download object"),
                    BuildHelpsItem::new(UserEvent::ObjectPreviewDownloadAs, "Download object as"),
                ]
            },
            (ViewState::SaveDialog(_), _) => {
                vec![
                    BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
                    BuildHelpsItem::new(UserEvent::InputDialogClose, "Close save dialog"),
                    BuildHelpsItem::new(UserEvent::InputDialogApply, "Download object"),
                ]
            },
            (ViewState::EncodingDialog, _) => {
                vec![
                    BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
                    BuildHelpsItem::new(UserEvent::SelectDialogClose, "Close encoding dialog"),
                    BuildHelpsItem::new(UserEvent::SelectDialogDown, "Select next item"),
                    BuildHelpsItem::new(UserEvent::SelectDialogUp, "Select previous item"),
                    BuildHelpsItem::new(UserEvent::SelectDialogSelect, "Reopen with encoding"),
                ]
            },
        };
        build_help_spans(helps, mapper, self.ctx.theme.help_key_fg)
    }

    pub fn short_helps(&self, mapper: &UserEventMapper) -> Vec<SpansWithPriority> {
        #[rustfmt::skip]
        let helps = match (&self.view_state, &self.preview_type) {
            (ViewState::Default, PreviewType::Text(_)) => {
                vec![
                    BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
                    BuildShortHelpsItem::group(vec![UserEvent::ObjectPreviewDown, UserEvent::ObjectPreviewUp], "Scroll", 2),
                    BuildShortHelpsItem::group(vec![UserEvent::ObjectPreviewGoToTop, UserEvent::ObjectPreviewGoToBottom], "Top/End", 5),
                    BuildShortHelpsItem::group(vec![UserEvent::ObjectPreviewDownload, UserEvent::ObjectPreviewDownloadAs], "Download", 3),
                    BuildShortHelpsItem::single(UserEvent::ObjectPreviewEncoding, "Encoding", 4),
                    BuildShortHelpsItem::single(UserEvent::ObjectPreviewBack, "Close", 1),
                    BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
                ]
            },
            (ViewState::Default, PreviewType::Image(_)) => {
                vec![
                    BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
                    BuildShortHelpsItem::group(vec![UserEvent::ObjectPreviewDownload, UserEvent::ObjectPreviewDownloadAs], "Download", 2),
                    BuildShortHelpsItem::single(UserEvent::ObjectPreviewBack, "Close", 1),
                    BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
                ]
            },
            (ViewState::SaveDialog(_), _) => {
                vec![
                    BuildShortHelpsItem::single(UserEvent::InputDialogClose, "Close", 2),
                    BuildShortHelpsItem::single(UserEvent::InputDialogApply, "Download", 1),
                    BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
                ]
            },
             (ViewState::EncodingDialog, _) => {
                vec![
                    BuildShortHelpsItem::single(UserEvent::SelectDialogClose, "Close", 2),
                    BuildShortHelpsItem::group(vec![UserEvent::SelectDialogDown, UserEvent::SelectDialogUp], "Select", 3),
                    BuildShortHelpsItem::single(UserEvent::SelectDialogSelect, "Encode", 1),
                    BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
                ]
             },
        };
        build_short_help_spans(helps, mapper)
    }
}

impl ObjectPreviewPage {
    fn open_save_dialog(&mut self) {
        self.view_state = ViewState::SaveDialog(InputDialogState::default());
    }

    fn close_save_dialog(&mut self) {
        self.view_state = ViewState::Default;
    }

    fn open_encoding_dialog(&mut self) {
        if let PreviewType::Text(_) = &mut self.preview_type {
            self.view_state = ViewState::EncodingDialog;
        }
    }

    fn close_encoding_dialog(&mut self) {
        self.view_state = ViewState::Default;
        self.encoding_dialog_state.reset();
    }

    fn apply_encoding(&mut self) {
        if let ViewState::EncodingDialog = &self.view_state {
            if let PreviewType::Text(state) = &mut self.preview_type {
                state.set_encoding(self.encoding_dialog_state.selected());
                state.update_lines(
                    &self.file_detail,
                    &self.object,
                    self.ctx.config.preview.highlight,
                    &self.ctx.config.preview.highlight_theme,
                );
            }
        }
        self.close_encoding_dialog();
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
        self.tx.send(AppEventType::StartSaveObject(
            self.file_detail.name.clone(),
            Arc::clone(&self.object),
        ));
    }

    fn download_as(&mut self, input: String) {
        let input: String = input.trim().into();
        if input.is_empty() {
            return;
        }

        self.tx.send(AppEventType::StartSaveObject(
            input,
            Arc::clone(&self.object),
        ));

        self.close_save_dialog();
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
    use crate::set_cells;

    use super::*;
    use chrono::{DateTime, Local, NaiveDateTime};
    use ratatui::{backend::TestBackend, buffer::Buffer, style::Color, Terminal};

    fn object(ss: &[&str]) -> RawObject {
        RawObject {
            bytes: ss.join("\n").as_bytes().to_vec(),
        }
    }

    #[tokio::test]
    async fn test_render_without_scroll() -> std::io::Result<()> {
        let ctx = Rc::default();
        let tx = sender();
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
            let mut page = ObjectPreviewPage::new(file_detail, None, object, ctx, tx);
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

    #[tokio::test]
    async fn test_render_with_scroll() -> std::io::Result<()> {
        let ctx = Rc::default();
        let tx = sender();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let file_detail = file_detail();
            let preview = ["Hello, world!"; 20];
            let object = object(&preview);
            let mut page = ObjectPreviewPage::new(file_detail, None, object, ctx, tx);
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

    #[tokio::test]
    async fn test_render_save_dialog_without_scroll() -> std::io::Result<()> {
        let ctx = Rc::default();
        let tx = sender();
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
            let mut page = ObjectPreviewPage::new(file_detail, None, object, ctx, tx);
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

    fn sender() -> Sender {
        let (tx, _) = tokio::sync::mpsc::unbounded_channel();
        Sender::new(tx)
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
