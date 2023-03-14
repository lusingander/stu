use std::{sync::mpsc, thread};

use crossterm::event::KeyEvent;

use crate::{client::Client, config::Config, error::AppError};

pub enum AppEventType {
    Key(KeyEvent),
    Initialize(Config, Client),
    LoadObjects,
    LoadObject,
    DownloadObject,
    Info(String),
    Error(String, String),
}

impl AppEventType {
    pub fn error(e: AppError) -> AppEventType {
        AppEventType::Error(e.msg, format!("{:?}", e.e))
    }

    fn error_with_msg<E: std::error::Error>(msg: impl Into<String>, e: E) -> AppEventType {
        AppEventType::Error(msg.into(), format!("{:?}", e))
    }
}

pub fn new() -> (mpsc::Sender<AppEventType>, mpsc::Receiver<AppEventType>) {
    let (tx, rx) = mpsc::channel();

    let event_tx = tx.clone();
    thread::spawn(move || loop {
        match crossterm::event::read() {
            Ok(e) => {
                if let crossterm::event::Event::Key(key) = e {
                    event_tx.send(AppEventType::Key(key)).unwrap();
                }
            }
            Err(e) => {
                let e = AppEventType::error_with_msg("Failed to read event", e);
                event_tx.send(e).unwrap();
            }
        }
    });

    (tx, rx)
}
