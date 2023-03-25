use std::{sync::mpsc, thread};

use crossterm::event::KeyEvent;

use crate::{
    client::Client,
    config::Config,
    error::{AppError, Result},
    item::{FileDetail, FileVersion, Item},
};

pub enum AppEventType {
    Key(KeyEvent),
    Resize(u16, u16),
    Initialize(Config, Client),
    LoadObjects,
    CompleteLoadObjects(Result<CompleteLoadObjectsResult>),
    LoadObject,
    CompleteLoadObject(Result<CompleteLoadObjectResult>),
    DownloadObject,
    CompleteDownloadObject(Result<CompleteDownloadObjectResult>),
    Info(String),
    Error(AppError),
}

pub struct CompleteLoadObjectsResult {
    pub items: Vec<Item>,
}

impl CompleteLoadObjectsResult {
    pub fn new(items: Result<Vec<Item>>) -> Result<CompleteLoadObjectsResult> {
        let items = items?;
        Ok(CompleteLoadObjectsResult { items })
    }
}

pub struct CompleteLoadObjectResult {
    pub detail: FileDetail,
    pub versions: Vec<FileVersion>,
    pub map_key: String,
}

impl CompleteLoadObjectResult {
    pub fn new(
        detail: Result<FileDetail>,
        versions: Result<Vec<FileVersion>>,
        map_key: String,
    ) -> Result<CompleteLoadObjectResult> {
        let detail = detail?;
        let versions = versions?;
        Ok(CompleteLoadObjectResult {
            detail,
            versions,
            map_key,
        })
    }
}

pub struct CompleteDownloadObjectResult {
    pub bytes: Vec<u8>,
    pub path: String,
}

impl CompleteDownloadObjectResult {
    pub fn new(bytes: Result<Vec<u8>>, path: String) -> Result<CompleteDownloadObjectResult> {
        let bytes = bytes?;
        Ok(CompleteDownloadObjectResult { bytes, path })
    }
}

pub fn new() -> (mpsc::Sender<AppEventType>, mpsc::Receiver<AppEventType>) {
    let (tx, rx) = mpsc::channel();

    let event_tx = tx.clone();
    thread::spawn(move || loop {
        match crossterm::event::read() {
            Ok(e) => match e {
                crossterm::event::Event::Key(key) => {
                    event_tx.send(AppEventType::Key(key)).unwrap();
                }
                crossterm::event::Event::Resize(w, h) => {
                    event_tx.send(AppEventType::Resize(w, h)).unwrap();
                }
                _ => {}
            },
            Err(e) => {
                let e = AppError::new("Failed to read event", e);
                event_tx.send(AppEventType::Error(e)).unwrap();
            }
        }
    });

    (tx, rx)
}
