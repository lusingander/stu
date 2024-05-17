use crossterm::event::KeyCode;
use ratatui::{backend::Backend, Terminal};
use std::io::Result;

use crate::{
    app::{App, Notification, ViewState},
    event::{AppEventType, AppKeyAction, Receiver},
    key_code, key_code_char, ui,
};

pub async fn run<B: Backend>(
    app: &mut App,
    terminal: &mut Terminal<B>,
    rx: Receiver,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;
        match rx.recv() {
            AppEventType::Key(key) => {
                if matches!(key, key_code!(KeyCode::Esc) | key_code_char!('c', Ctrl)) {
                    // Exit regardless of status
                    return Ok(());
                }

                if app.app_view_state.is_loading {
                    // Ignore key inputs while loading (except quit)
                    continue;
                }

                if matches!(app.app_view_state.notification, Notification::Error(_)) {
                    if matches!(app.app_view_state.view_state, ViewState::Initializing) {
                        return Ok(());
                    }
                    // Clear message and cancel key input
                    app.clear_notification();
                    continue;
                }

                if matches!(
                    app.app_view_state.notification,
                    Notification::Info(_) | Notification::Success(_)
                ) {
                    // Clear message and pass key input as is
                    app.clear_notification();
                }

                let vs = &app.app_view_state.view_state;
                if let Some(input) = app.action_manager.key_to_input(key, vs) {
                    app.send_app_key_input(input)
                }
                if let Some(action) = app.action_manager.key_to_action(key, vs) {
                    app.send_app_key_action(action);
                }
            }
            AppEventType::KeyAction(action) => match action {
                // Initializing
                // BucketList
                AppKeyAction::BucketListSelectNext => {
                    app.bucket_list_select_next();
                }
                AppKeyAction::BucketListSelectPrev => {
                    app.bucket_list_select_prev();
                }
                AppKeyAction::BucketListSelectFirst => {
                    app.bucket_list_select_first();
                }
                AppKeyAction::BucketListSelectLast => {
                    app.bucket_list_select_last();
                }
                AppKeyAction::BucketListSelectNextPage => {
                    app.bucket_list_select_next_page();
                }
                AppKeyAction::BucketListSelectPrevPage => {
                    app.bucket_list_select_prev_page();
                }
                AppKeyAction::BucketListMoveDown => {
                    app.bucket_list_move_down();
                }
                AppKeyAction::BucketListOpenManagementConsole => {
                    app.bucket_list_open_management_console();
                }
                // ObjectList
                AppKeyAction::ObjectListSelectNext => {
                    app.object_list_select_next();
                }
                AppKeyAction::ObjectListSelectPrev => {
                    app.object_list_select_prev();
                }
                AppKeyAction::ObjectListSelectFirst => {
                    app.object_list_select_first();
                }
                AppKeyAction::ObjectListSelectLast => {
                    app.object_list_select_last();
                }
                AppKeyAction::ObjectListSelectNextPage => {
                    app.object_list_select_next_page();
                }
                AppKeyAction::ObjectListSelectPrevPage => {
                    app.object_list_select_prev_page();
                }
                AppKeyAction::ObjectListMoveDown => {
                    app.object_list_move_down();
                }
                AppKeyAction::ObjectListMoveUp => {
                    app.object_list_move_up();
                }
                AppKeyAction::ObjectListBackToBucketList => {
                    app.object_list_back_to_bucket_list();
                }
                AppKeyAction::ObjectListOpenManagementConsole => {
                    app.object_list_open_management_console();
                }
                // Detail
                AppKeyAction::DetailClose => {
                    app.detail_close();
                }
                AppKeyAction::DetailSelectNext => {
                    app.detail_select_tabs();
                }
                AppKeyAction::DetailSelectPrev => {
                    app.detail_select_tabs();
                }
                AppKeyAction::DetailDownloadObject => {
                    app.detail_download_object();
                }
                AppKeyAction::DetailOpenDownloadObjectAs => {
                    app.detail_open_download_object_as();
                }
                AppKeyAction::DetailPreview => {
                    app.detail_preview();
                }
                AppKeyAction::DetailOpenCopyDetails => {
                    app.detail_open_copy_details();
                }
                AppKeyAction::DetailOpenManagementConsole => {
                    app.detail_open_management_console();
                }
                // DetailSave
                AppKeyAction::DetailSaveDownloadObjectAs => {
                    app.detail_save_download_object_as();
                }
                // CopyDetail
                AppKeyAction::CopyDetailSelectNext => {
                    app.copy_detail_select_next();
                }
                AppKeyAction::CopyDetailSelectPrev => {
                    app.copy_detail_select_prev();
                }
                AppKeyAction::CopyDetailCopySelectedValue => {
                    app.copy_detail_copy_selected_value();
                }
                AppKeyAction::CopyDetailClose => {
                    app.copy_detail_close();
                }
                // Preview
                AppKeyAction::PreviewScrollForward => {
                    app.preview_scroll_forward();
                }
                AppKeyAction::PreviewScrollBackward => {
                    app.preview_scroll_backward();
                }
                AppKeyAction::PreviewScrollToTop => {
                    app.preview_scroll_to_top();
                }
                AppKeyAction::PreviewScrollToEnd => {
                    app.preview_scroll_to_end();
                }
                AppKeyAction::PreviewClose => {
                    app.preview_close();
                }
                AppKeyAction::PreviewDownloadObject => {
                    app.preview_download_object();
                }
                AppKeyAction::PreviewOpenDownloadObjectAs => {
                    app.preview_open_download_object_as();
                }
                // PreviewSave
                AppKeyAction::PreviewSaveDownloadObjectAs => {
                    app.preview_save_download_object_as();
                }
                // Help
                AppKeyAction::HelpClose => {
                    app.help_close();
                }
                // common
                AppKeyAction::ToggleHelp => {
                    app.toggle_help();
                }
            },
            AppEventType::Resize(width, height) => {
                app.resize(width, height);
            }
            AppEventType::Initialize(config, client, bucket) => {
                app.initialize(config, client, bucket);
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
            AppEventType::CopyToClipboard(name, value) => {
                app.copy_to_clipboard(name, value);
            }
            AppEventType::KeyInput(input) => {
                app.key_input(input);
            }
            AppEventType::NotifyInfo(msg) => {
                app.info_notification(msg);
            }
            AppEventType::NotifySuccess(msg) => {
                app.success_notification(msg);
            }
            AppEventType::NotifyError(e) => {
                app.error_notification(e);
            }
        }
    }
}
