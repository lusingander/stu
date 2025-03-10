use ratatui::{backend::Backend, Terminal};
use std::io::Result;

use crate::{
    app::{App, Notification},
    event::{AppEventType, Receiver},
    handle_user_events,
    keys::UserEvent,
    pages::page::Page,
};

pub async fn run<B: Backend>(
    app: &mut App,
    terminal: &mut Terminal<B>,
    rx: Receiver,
) -> Result<()> {
    loop {
        terminal.draw(|f| app.render(f))?;

        let event = rx.recv();
        tracing::debug!("event received: {:?}", event);

        match event {
            AppEventType::Key(key_event) => {
                let user_events = app.mapper.find_events(key_event);

                handle_user_events! { user_events =>
                    UserEvent::Quit => {
                        // Exit regardless of status
                        return Ok(());
                    }
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

                handle_user_events! { user_events =>
                    UserEvent::DumpApp => {
                        app.dump_app();
                        continue;
                    }
                }

                app.page_stack
                    .current_page_mut()
                    .handle_user_events(user_events, key_event);
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
            AppEventType::LoadAllDownloadObjectList(key) => {
                app.load_all_download_objects(key);
            }
            AppEventType::CompleteLoadAllDownloadObjectList(result) => {
                app.complete_load_all_download_objects(result);
            }
            AppEventType::StartDownloadObject(object_name, size_byte, version_id) => {
                app.start_download_object(object_name, size_byte, version_id);
            }
            AppEventType::DownloadObject(object_name, size_byte, version_id) => {
                app.download_object(object_name, size_byte, version_id);
            }
            AppEventType::StartDownloadObjectAs(size_byte, input, version_id) => {
                app.start_download_object_as(size_byte, input, version_id);
            }
            AppEventType::DownloadObjectAs(size_byte, input, version_id) => {
                app.download_object_as(size_byte, input, version_id);
            }
            AppEventType::CompleteDownloadObject(result) => {
                app.complete_download_object(result);
            }
            AppEventType::DownloadObjects(bucket, key, dir, objs) => {
                app.download_objects(bucket, key, dir, objs);
            }
            AppEventType::CompleteDownloadObjects(result) => {
                app.complete_download_objects(result);
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
            AppEventType::ObjectListDownloadObject => {
                app.object_list_download_object();
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
