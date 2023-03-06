use std::{sync::mpsc, thread};

use crossterm::event::KeyEvent;

use crate::client::Client;

pub enum AppEventType {
    Key(KeyEvent),
    ClientInitialized(Client),
    LoadObjects,
    LoadObject,
    DownloadObject,
    Info(String),
    Error(String),
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
                event_tx.send(AppEventType::Error(e.to_string())).unwrap();
            }
        }
    });

    (tx, rx)
}
