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

                if let Some(action) = app.action_manager.key_to_action(key) {
                    app.send_app_key_action(action);
                }
            }
            AppEventType::KeyAction(action) => match action {
                AppKeyAction::SelectNext => {
                    app.select_next();
                }
                AppKeyAction::SelectPrev => {
                    app.select_prev();
                }
                AppKeyAction::SelectFirst => {
                    app.select_first();
                }
                AppKeyAction::SelectLast => {
                    app.select_last();
                }
                AppKeyAction::SelectNextPage => {
                    app.select_next_page();
                }
                AppKeyAction::SelectPrevPage => {
                    app.select_prev_page();
                }
                AppKeyAction::MoveDown => {
                    app.move_down();
                }
                AppKeyAction::MoveUp => {
                    app.move_up();
                }
                AppKeyAction::BackToBucketList => {
                    app.back_to_bucket_list();
                }
                AppKeyAction::SelectTabs => {
                    app.select_tabs();
                }
                AppKeyAction::Download => {
                    app.download();
                }
                AppKeyAction::Preview => {
                    app.preview();
                }
                AppKeyAction::ToggleCopyDetails => {
                    app.toggle_copy_details();
                }
                AppKeyAction::OpenManagementConsole => {
                    app.open_management_console();
                }
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
