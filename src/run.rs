use laurier::{key_code, key_code_char};
use ratatui::{backend::Backend, crossterm::event::KeyCode, Terminal};
use std::io::Result;

use crate::{
    app::{App, Notification},
    event::{AppEventType, Receiver},
    pages::page::Page,
    render::render,
};

pub async fn run<B: Backend>(
    app: &mut App,
    terminal: &mut Terminal<B>,
    rx: Receiver,
) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;

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

                if app.loading() {
                    // Ignore key inputs while loading (except quit)
                    continue;
                }

                if matches!(app.current_notification(), Notification::Error(_)) {
                    if matches!(app.page_stack.current_page(), Page::Initializing(_)) {
                        return Ok(());
                    }
                    // Clear message and cancel key input
                    app.clear_notification();
                    continue;
                }

                if matches!(
                    app.current_notification(),
                    Notification::Info(_) | Notification::Success(_) | Notification::Warn(_)
                ) {
                    // Clear message and pass key input as is
                    app.clear_notification();
                }

                if matches!(key, key_code!(KeyCode::F(12))) {
                    app.dump_app();
                    continue;
                }

                app.page_stack.current_page_mut().handle_key(key);
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
            AppEventType::ReloadBuckets => {
                app.reload_buckets();
            }
            AppEventType::CompleteReloadBuckets(result) => {
                app.complete_reload_buckets(result);
            }
            AppEventType::LoadObjects => {
                app.load_objects();
            }
            AppEventType::CompleteLoadObjects(result) => {
                app.complete_load_objects(result);
            }
            AppEventType::ReloadObjects => {
                app.reload_objects();
            }
            AppEventType::CompleteReloadObjects(result) => {
                app.complete_reload_objects(result);
            }
            AppEventType::LoadObjectDetail => {
                app.load_object_detail();
            }
            AppEventType::CompleteLoadObjectDetail(result) => {
                app.complete_load_object_detail(result);
            }
            AppEventType::LoadObjectVersions => {
                app.load_object_versions();
            }
            AppEventType::CompleteLoadObjectVersions(result) => {
                app.complete_load_object_versions(result);
            }
            AppEventType::DownloadObject(file_detail, version_id) => {
                app.download_object(file_detail, version_id);
            }
            AppEventType::DownloadObjectAs(file_detail, input, version_id) => {
                app.download_object_as(file_detail, input, version_id);
            }
            AppEventType::CompleteDownloadObject(result) => {
                app.complete_download_object(result);
            }
            AppEventType::PreviewObject(file_detail, version_id) => {
                app.preview_object(file_detail, version_id);
            }
            AppEventType::CompletePreviewObject(result) => {
                app.complete_preview_object(result);
            }
            AppEventType::BucketListMoveDown => {
                app.bucket_list_move_down();
            }
            AppEventType::BucketListRefresh => {
                app.bucket_list_refresh();
            }
            AppEventType::ObjectListMoveDown => {
                app.object_list_move_down();
            }
            AppEventType::ObjectListMoveUp => {
                app.object_list_move_up();
            }
            AppEventType::ObjectListRefresh => {
                app.object_list_refresh();
            }
            AppEventType::BackToBucketList => {
                app.back_to_bucket_list();
            }
            AppEventType::OpenObjectVersionsTab => {
                app.open_object_versions_tab();
            }
            AppEventType::OpenPreview(file_detail, version_id) => {
                app.open_preview(file_detail, version_id);
            }
            AppEventType::DetailDownloadObject(file_detail, version_id) => {
                app.detail_download_object(file_detail, version_id);
            }
            AppEventType::DetailDownloadObjectAs(file_detail, input, version_id) => {
                app.detail_download_object_as(file_detail, input, version_id);
            }
            AppEventType::PreviewDownloadObject(obj, path) => {
                app.preview_download_object(obj, path);
            }
            AppEventType::PreviewDownloadObjectAs(file_detail, input, version_id) => {
                app.preview_download_object_as(file_detail, input, version_id);
            }
            AppEventType::PreviewRerenderImage => {
                app.preview_rerender_image();
            }
            AppEventType::BucketListOpenManagementConsole => {
                app.bucket_list_open_management_console();
            }
            AppEventType::ObjectListOpenManagementConsole => {
                app.object_list_open_management_console();
            }
            AppEventType::ObjectDetailOpenManagementConsole => {
                app.object_detail_open_management_console();
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
