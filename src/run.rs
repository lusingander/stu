use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::Backend, Terminal};
use std::{io::Result, sync::mpsc};

use crate::{
    app::{App, Notification, ViewState},
    event::AppEventType,
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
                match key {
                    key_code!(KeyCode::Esc) | key_code_char!('c', Ctrl) => {
                        return Ok(());
                    }
                    _ => {}
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

                match key {
                    key_code_char!('j') => {
                        app.select_next();
                    }
                    key_code_char!('k') => {
                        app.select_prev();
                    }
                    key_code_char!('g') => {
                        app.select_first();
                    }
                    key_code_char!('G') => {
                        app.select_last();
                    }
                    key_code_char!('f') => {
                        app.select_next_page();
                    }
                    key_code_char!('b') => {
                        app.select_prev_page();
                    }
                    key_code!(KeyCode::Enter) | key_code_char!('m', Ctrl) => {
                        app.move_down();
                    }
                    key_code!(KeyCode::Backspace) | key_code_char!('h', Ctrl) => {
                        app.move_up();
                    }
                    key_code_char!('~') => {
                        app.back_to_bucket_list();
                    }
                    key_code_char!('h') | key_code_char!('l') => {
                        app.select_tabs();
                    }
                    key_code_char!('s') => {
                        app.download();
                    }
                    key_code_char!('p') => {
                        app.preview();
                    }
                    key_code_char!('r') => {
                        app.toggle_copy_details();
                    }
                    key_code_char!('x') => {
                        app.open_management_console();
                    }
                    key_code_char!('?') => {
                        app.toggle_help();
                    }
                    _ => {}
                }
            }
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
