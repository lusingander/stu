use crossterm::event::KeyCode;
use ratatui::{backend::Backend, Terminal};
use std::io::Result;

use crate::{
    app::{App, Notification},
    event::{AppEventType, Receiver},
    key_code, key_code_char,
    pages::page::Page,
    ui,
};

pub async fn run<B: Backend>(
    app: &mut App,
    terminal: &mut Terminal<B>,
    rx: Receiver,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        let event = rx.recv();
        tracing::debug!("event received: {:?}", event);

        match event {
            AppEventType::Quit => {
                return Ok(());
            }
            AppEventType::Key(key) => {
                if matches!(key, key_code_char!('c', Ctrl)) {
                    // Exit regardless of status
                    return Ok(());
                }

                if app.app_view_state.is_loading {
                    // Ignore key inputs while loading (except quit)
                    continue;
                }

                if matches!(app.app_view_state.notification, Notification::Error(_)) {
                    if matches!(app.page_stack.current_page(), Page::Initializing(_)) {
                        return Ok(());
                    }
                    // Clear message and cancel key input
                    app.clear_notification();
                    continue;
                }

                if matches!(
                    app.app_view_state.notification,
                    Notification::Info(_) | Notification::Success(_) | Notification::Warn(_)
                ) {
                    // Clear message and pass key input as is
                    app.clear_notification();
                }

                if matches!(key, key_code!(KeyCode::F(12))) {
                    app.dump_app();
                    continue;
                }

                match app.page_stack.current_page_mut() {
                    Page::Initializing(page) => page.handle_key(key),
                    Page::BucketList(page) => page.handle_key(key),
                    Page::ObjectList(page) => page.handle_key(key),
                    Page::ObjectDetail(page) => page.handle_key(key),
                    Page::ObjectPreview(page) => page.handle_key(key),
                    Page::Help(page) => page.handle_key(key),
                }
            }
            AppEventType::Resize(width, height) => {
                app.resize(width, height);
            }
            AppEventType::Initialize(client, bucket) => {
                app.initialize(client, bucket);
            }
            AppEventType::CompleteInitialize(result) => {
                app.complete_initialize(result);
            }
            AppEventType::LoadObjects => {
                app.load_objects();
            }
            AppEventType::CompleteLoadObjects(result) => {
                app.complete_load_objects(result);
            }
            AppEventType::LoadObject => {
                app.load_object();
            }
            AppEventType::CompleteLoadObject(result) => {
                app.complete_load_object(result);
            }
            AppEventType::DownloadObject(file_detail) => {
                app.download_object(file_detail);
            }
            AppEventType::DownloadObjectAs(file_detail, input) => {
                app.download_object_as(file_detail, input);
            }
            AppEventType::CompleteDownloadObject(result) => {
                app.complete_download_object(result);
            }
            AppEventType::PreviewObject(file_detail) => {
                app.preview_object(file_detail);
            }
            AppEventType::CompletePreviewObject(result) => {
                app.complete_preview_object(result);
            }
            AppEventType::BucketListMoveDown => {
                app.bucket_list_move_down();
            }
            AppEventType::ObjectListMoveDown => {
                app.object_list_move_down();
            }
            AppEventType::ObjectListMoveUp => {
                app.object_list_move_up();
            }
            AppEventType::BackToBucketList => {
                app.back_to_bucket_list();
            }
            AppEventType::OpenPreview(file_detail) => {
                app.open_preview(file_detail);
            }
            AppEventType::DetailDownloadObject(file_detail) => {
                app.detail_download_object(file_detail);
            }
            AppEventType::DetailDownloadObjectAs(file_detail, input) => {
                app.detail_download_object_as(file_detail, input);
            }
            AppEventType::PreviewDownloadObject(obj, path) => {
                app.preview_download_object(obj, path);
            }
            AppEventType::PreviewDownloadObjectAs(file_detail, input) => {
                app.preview_download_object_as(file_detail, input);
            }
            AppEventType::BucketListOpenManagementConsole => {
                app.bucket_list_open_management_console();
            }
            AppEventType::ObjectListOpenManagementConsole => {
                app.object_list_open_management_console();
            }
            AppEventType::ObjectDetailOpenManagementConsole(name) => {
                app.object_detail_open_management_console(name);
            }
            AppEventType::CloseCurrentPage => {
                app.close_current_page();
            }
            AppEventType::OpenHelp => {
                app.open_help();
            }
            AppEventType::CopyToClipboard(name, value) => {
                app.copy_to_clipboard(name, value);
            }
            AppEventType::NotifyInfo(msg) => {
                app.info_notification(msg);
            }
            AppEventType::NotifySuccess(msg) => {
                app.success_notification(msg);
            }
            AppEventType::NotifyWarn(msg) => {
                app.warn_notification(msg);
            }
            AppEventType::NotifyError(e) => {
                app.error_notification(e);
            }
        }
    }
}
