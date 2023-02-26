use std::{sync::mpsc, thread};

use crossterm::event::KeyEvent;

pub enum AppEventType {
    Key(KeyEvent),
    Error(String),
}

pub struct AppEvent {
    rx: mpsc::Receiver<AppEventType>,
    _tx: mpsc::Sender<AppEventType>,
}

impl AppEvent {
    pub fn new() -> AppEvent {
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

        AppEvent { rx, _tx: tx }
    }

    pub fn receive(&self) -> AppEventType {
        self.rx.recv().unwrap()
    }
}
