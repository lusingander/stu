use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::Backend, Terminal};
use std::{io::Result, sync::mpsc};

use crate::{
    app::{App, Notification, ViewState},
    event::{AppEventType, AppKeyAction},
    key_code, key_code_char, ui,
};

pub async fn run<B: Backend>(
    app: &mut App,
    terminal: &mut Terminal<B>,
    rx: mpsc::Receiver<AppEventType>,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;
        match rx.recv().unwrap() {
            AppEventType::Key(key) => {
                if matches!(key, key_code!(KeyCode::Esc) | key_code_char!('c', Ctrl)) {
                    // Exit regardless of status
                    return Ok(());
                }

                if app.app_view_state.is_loading {
                    continue;
                }

                match app.app_view_state.notification {
                    Notification::Error(_) => {
                        if matches!(app.app_view_state.view_state, ViewState::Initializing) {
                            return Ok(());
                        }
                        app.app_view_state.notification = Notification::None;
                        continue;
                    }
                    Notification::Info(_) => {
                        app.app_view_state.notification = Notification::None;
                    }
                    Notification::None => {}
                }

                if let Some(action) = app
                    .action_manager
                    .key_to_action(key, &app.app_view_state.view_state)
                {
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
                AppKeyAction::DetailSelectTabs => {
                    app.detail_select_tabs();
                }
                AppKeyAction::DetailDownloadObject => {
                    app.detail_download_object();
                }
                AppKeyAction::DetailPreview => {
                    app.detail_preview();
                }
                AppKeyAction::DetailToggleCopyDetails => {
                    app.detail_open_copy_details();
                }
                AppKeyAction::DetailOpenManagementConsole => {
                    app.detail_open_management_console();
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
                AppKeyAction::PreviewClose => {
                    app.preview_close();
                }
                AppKeyAction::PreviewDownloadObject => {
                    app.preview_download_object();
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
            AppEventType::Resize(_, height) => {
                app.resize(height as usize);
            }
            AppEventType::Initialize(config, client) => {
                app.initialize(config, client);
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
            AppEventType::DownloadObject => {
                app.download_object();
            }
            AppEventType::CompleteDownloadObject(result) => {
                app.complete_download_object(result);
            }
            AppEventType::PreviewObject => {
                app.preview_object();
            }
            AppEventType::CompletePreviewObject(result) => {
                app.complete_preview_object(result);
            }
            AppEventType::CopyToClipboard(name, value) => {
                app.copy_to_clipboard(name, value);
            }
            AppEventType::Info(msg) => {
                app.app_view_state.notification = Notification::Info(msg);
            }
            AppEventType::Error(e) => {
                app.save_error(&e);
                app.app_view_state.notification = Notification::Error(e.msg);
            }
        }
    }
}
