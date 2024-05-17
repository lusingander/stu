use std::{sync::mpsc, thread};

use crossterm::event::KeyEvent;

use crate::{
    client::Client,
    config::Config,
    error::{AppError, Result},
    object::{BucketItem, FileDetail, FileVersion, Object, ObjectItem, ObjectKey},
};

pub enum AppEventType {
    Key(KeyEvent),
    KeyAction(AppKeyAction),
    Resize(usize, usize),
    Initialize(Config, Client, Option<String>),
    CompleteInitialize(Result<CompleteInitializeResult>),
    LoadObjects,
    CompleteLoadObjects(Result<CompleteLoadObjectsResult>),
    LoadObject,
    CompleteLoadObject(Result<CompleteLoadObjectResult>),
    DownloadObject(FileDetail),
    DownloadObjectAs(FileDetail, String),
    CompleteDownloadObject(Result<CompleteDownloadObjectResult>),
    PreviewObject(FileDetail),
    CompletePreviewObject(Result<CompletePreviewObjectResult>),
    CopyToClipboard(String, String),
    KeyInput(AppKeyInput),
    NotifyInfo(String),
    NotifySuccess(String),
    NotifyError(AppError),
}

#[derive(Clone, Copy)]
pub enum AppKeyInput {
    Char(char),
    Backspace,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AppKeyAction {
    // Initializing
    // BucketList
    BucketListSelectNext,
    BucketListSelectPrev,
    BucketListSelectFirst,
    BucketListSelectLast,
    BucketListSelectNextPage,
    BucketListSelectPrevPage,
    BucketListMoveDown,
    BucketListOpenManagementConsole,
    // ObjectList
    ObjectListSelectNext,
    ObjectListSelectPrev,
    ObjectListSelectFirst,
    ObjectListSelectLast,
    ObjectListSelectNextPage,
    ObjectListSelectPrevPage,
    ObjectListMoveDown,
    ObjectListMoveUp,
    ObjectListBackToBucketList,
    ObjectListOpenManagementConsole,
    // Detail
    DetailClose,
    DetailSelectNext,
    DetailSelectPrev,
    DetailDownloadObject,
    DetailPreview,
    DetailOpenDownloadObjectAs,
    DetailOpenCopyDetails,
    DetailOpenManagementConsole,
    // DetailSave
    DetailSaveDownloadObjectAs,
    // CopyDetail
    CopyDetailSelectNext,
    CopyDetailSelectPrev,
    CopyDetailCopySelectedValue,
    CopyDetailClose,
    // Preview
    PreviewScrollForward,
    PreviewScrollBackward,
    PreviewScrollToTop,
    PreviewScrollToEnd,
    PreviewClose,
    PreviewDownloadObject,
    PreviewOpenDownloadObjectAs,
    // PreviewSave
    PreviewSaveDownloadObjectAs,
    // Help
    HelpClose,
    // common
    ToggleHelp,
}

pub struct CompleteInitializeResult {
    pub buckets: Vec<BucketItem>,
}

impl CompleteInitializeResult {
    pub fn new(buckets: Result<Vec<BucketItem>>) -> Result<CompleteInitializeResult> {
        let buckets = buckets?;
        Ok(CompleteInitializeResult { buckets })
    }
}

pub struct CompleteLoadObjectsResult {
    pub items: Vec<ObjectItem>,
}

impl CompleteLoadObjectsResult {
    pub fn new(items: Result<Vec<ObjectItem>>) -> Result<CompleteLoadObjectsResult> {
        let items = items?;
        Ok(CompleteLoadObjectsResult { items })
    }
}

pub struct CompleteLoadObjectResult {
    pub detail: Box<FileDetail>, // to avoid "warning: large size difference between variants" for AppEventType
    pub versions: Vec<FileVersion>,
    pub map_key: ObjectKey,
}

impl CompleteLoadObjectResult {
    pub fn new(
        detail: Result<FileDetail>,
        versions: Result<Vec<FileVersion>>,
        map_key: ObjectKey,
    ) -> Result<CompleteLoadObjectResult> {
        let detail = Box::new(detail?);
        let versions = versions?;
        Ok(CompleteLoadObjectResult {
            detail,
            versions,
            map_key,
        })
    }
}

pub struct CompleteDownloadObjectResult {
    pub obj: Object,
    pub path: String,
}

impl CompleteDownloadObjectResult {
    pub fn new(obj: Result<Object>, path: String) -> Result<CompleteDownloadObjectResult> {
        let obj = obj?;
        Ok(CompleteDownloadObjectResult { obj, path })
    }
}

pub struct CompletePreviewObjectResult {
    pub obj: Object,
    pub file_detail: FileDetail,
    pub path: String,
}

impl CompletePreviewObjectResult {
    pub fn new(
        obj: Result<Object>,
        file_detail: FileDetail,
        path: String,
    ) -> Result<CompletePreviewObjectResult> {
        let obj = obj?;
        Ok(CompletePreviewObjectResult {
            obj,
            file_detail,
            path,
        })
    }
}

#[derive(Clone)]
pub struct Sender {
    tx: mpsc::Sender<AppEventType>,
}

impl Sender {
    pub fn send(&self, event: AppEventType) {
        self.tx.send(event).unwrap();
    }
}

pub struct Receiver {
    rx: mpsc::Receiver<AppEventType>,
}

impl Receiver {
    pub fn recv(&self) -> AppEventType {
        self.rx.recv().unwrap()
    }
}

pub fn new() -> (Sender, Receiver) {
    let (tx, rx) = mpsc::channel();
    let tx = Sender { tx };
    let rx = Receiver { rx };

    let event_tx = tx.clone();
    thread::spawn(move || loop {
        match crossterm::event::read() {
            Ok(e) => match e {
                crossterm::event::Event::Key(key) => {
                    event_tx.send(AppEventType::Key(key));
                }
                crossterm::event::Event::Resize(w, h) => {
                    event_tx.send(AppEventType::Resize(w as usize, h as usize));
                }
                _ => {}
            },
            Err(e) => {
                let e = AppError::new("Failed to read event", e);
                event_tx.send(AppEventType::NotifyError(e));
            }
        }
    });

    (tx, rx)
}
