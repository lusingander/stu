use std::{
    fmt::{self, Debug, Formatter},
    path::PathBuf,
    sync::{mpsc, Arc},
    thread,
};

use ratatui::crossterm::event::KeyEvent;

use crate::{
    client::Client,
    error::{AppError, Result},
    object::{
        BucketItem, DownloadObjectInfo, FileDetail, FileVersion, ObjectItem, ObjectKey, RawObject,
    },
};

#[derive(Debug)]
pub enum AppEventType {
    Key(KeyEvent),
    Resize(usize, usize),
    Initialize(Client, Option<String>),
    CompleteInitialize(Result<CompleteInitializeResult>),
    ReloadBuckets,
    CompleteReloadBuckets(Result<CompleteReloadBucketsResult>),
    LoadObjects,
    CompleteLoadObjects(Result<CompleteLoadObjectsResult>),
    ReloadObjects,
    CompleteReloadObjects(Result<CompleteReloadObjectsResult>),
    LoadObjectDetail,
    CompleteLoadObjectDetail(Result<CompleteLoadObjectDetailResult>),
    LoadObjectVersions,
    CompleteLoadObjectVersions(Result<CompleteLoadObjectVersionsResult>),
    StartLoadAllDownloadObjectList(ObjectKey, bool),
    LoadAllDownloadObjectList(ObjectKey, bool),
    CompleteLoadAllDownloadObjectList(Result<CompleteLoadAllDownloadObjectListResult>),
    StartDownloadObject(String, usize, Option<String>),
    DownloadObject(String, usize, Option<String>),
    StartDownloadObjectAs(ObjectKey, usize, String, Option<String>),
    DownloadObjectAs(ObjectKey, usize, String, Option<String>),
    CompleteDownloadObject(Result<CompleteDownloadObjectResult>),
    DownloadObjects(String, ObjectKey, String, Vec<DownloadObjectInfo>),
    CompleteDownloadObjects(Result<CompleteDownloadObjectsResult>),
    PreviewObject(ObjectKey, FileDetail, Option<String>),
    CompletePreviewObject(Result<CompletePreviewObjectResult>),
    StartSaveObject(String, Arc<RawObject>),
    SaveObject(String, Arc<RawObject>),
    CompleteSaveObject(Result<CompleteSaveObjectResult>),
    BucketListMoveDown,
    BucketListRefresh,
    ObjectListMoveDown,
    ObjectListMoveUp,
    ObjectListRefresh,
    BackToBucketList,
    OpenObjectVersionsTab,
    OpenPreview(ObjectKey, FileDetail, Option<String>),
    PreviewRerenderImage,
    BucketListOpenManagementConsole,
    ObjectListOpenManagementConsole,
    ObjectDetailOpenManagementConsole,
    CloseCurrentPage,
    OpenHelp,
    CopyToClipboard(String, String),
    NotifyInfo(String),
    NotifySuccess(String),
    NotifyWarn(String),
    NotifyError(AppError),
}

#[derive(Debug)]
pub struct CompleteInitializeResult {
    pub buckets: Vec<BucketItem>,
}

impl CompleteInitializeResult {
    pub fn new(buckets: Result<Vec<BucketItem>>) -> Result<CompleteInitializeResult> {
        let buckets = buckets?;
        Ok(CompleteInitializeResult { buckets })
    }
}

impl From<CompleteReloadBucketsResult> for CompleteInitializeResult {
    fn from(result: CompleteReloadBucketsResult) -> Self {
        CompleteInitializeResult {
            buckets: result.buckets,
        }
    }
}

#[derive(Debug)]
pub struct CompleteReloadBucketsResult {
    pub buckets: Vec<BucketItem>,
}

impl CompleteReloadBucketsResult {
    pub fn new(buckets: Result<Vec<BucketItem>>) -> Result<CompleteReloadBucketsResult> {
        let buckets = buckets?;
        Ok(CompleteReloadBucketsResult { buckets })
    }
}

#[derive(Debug)]
pub struct CompleteLoadObjectsResult {
    pub items: Vec<ObjectItem>,
}

impl CompleteLoadObjectsResult {
    pub fn new(items: Result<Vec<ObjectItem>>) -> Result<CompleteLoadObjectsResult> {
        let items = items?;
        Ok(CompleteLoadObjectsResult { items })
    }
}

impl From<CompleteReloadObjectsResult> for CompleteLoadObjectsResult {
    fn from(result: CompleteReloadObjectsResult) -> Self {
        CompleteLoadObjectsResult {
            items: result.items,
        }
    }
}

#[derive(Debug)]
pub struct CompleteReloadObjectsResult {
    pub items: Vec<ObjectItem>,
}

impl CompleteReloadObjectsResult {
    pub fn new(items: Result<Vec<ObjectItem>>) -> Result<CompleteReloadObjectsResult> {
        let items = items?;
        Ok(CompleteReloadObjectsResult { items })
    }
}

#[derive(Debug)]
pub struct CompleteLoadObjectDetailResult {
    pub detail: Box<FileDetail>, // to avoid "warning: large size difference between variants" for AppEventType
    pub map_key: ObjectKey,
}

impl CompleteLoadObjectDetailResult {
    pub fn new(
        detail: Result<FileDetail>,
        map_key: ObjectKey,
    ) -> Result<CompleteLoadObjectDetailResult> {
        let detail = Box::new(detail?);
        Ok(CompleteLoadObjectDetailResult { detail, map_key })
    }
}

#[derive(Debug)]
pub struct CompleteLoadObjectVersionsResult {
    pub versions: Vec<FileVersion>,
    pub map_key: ObjectKey,
}

impl CompleteLoadObjectVersionsResult {
    pub fn new(
        versions: Result<Vec<FileVersion>>,
        map_key: ObjectKey,
    ) -> Result<CompleteLoadObjectVersionsResult> {
        let versions = versions?;
        Ok(CompleteLoadObjectVersionsResult { versions, map_key })
    }
}

#[derive(Debug)]
pub struct CompleteLoadAllDownloadObjectListResult {
    pub objs: Vec<DownloadObjectInfo>,
    pub download_as: bool,
}

impl CompleteLoadAllDownloadObjectListResult {
    pub fn new(
        objs: Result<Vec<DownloadObjectInfo>>,
        download_as: bool,
    ) -> Result<CompleteLoadAllDownloadObjectListResult> {
        let objs = objs?;
        Ok(CompleteLoadAllDownloadObjectListResult { objs, download_as })
    }
}

#[derive(Debug)]
pub struct CompleteDownloadObjectResult {
    pub path: PathBuf,
}

impl CompleteDownloadObjectResult {
    pub fn new(result: Result<()>, path: PathBuf) -> Result<CompleteDownloadObjectResult> {
        result?;
        Ok(CompleteDownloadObjectResult { path })
    }
}

#[derive(Debug)]
pub struct CompleteDownloadObjectsResult {
    pub download_dir: PathBuf,
}

impl CompleteDownloadObjectsResult {
    pub fn new(download_dir: PathBuf) -> Result<CompleteDownloadObjectsResult> {
        Ok(CompleteDownloadObjectsResult { download_dir })
    }
}

#[derive(Debug)]
pub struct CompletePreviewObjectResult {
    pub obj: RawObject,
    pub file_detail: FileDetail,
    pub file_version_id: Option<String>,
}

impl CompletePreviewObjectResult {
    pub fn new(
        obj: Result<RawObject>,
        file_detail: FileDetail,
        file_version_id: Option<String>,
    ) -> Result<CompletePreviewObjectResult> {
        let obj = obj?;
        Ok(CompletePreviewObjectResult {
            obj,
            file_detail,
            file_version_id,
        })
    }
}

#[derive(Debug)]
pub struct CompleteSaveObjectResult {
    pub path: PathBuf,
}

impl CompleteSaveObjectResult {
    pub fn new(result: Result<()>, path: PathBuf) -> Result<CompleteSaveObjectResult> {
        result?;
        Ok(CompleteSaveObjectResult { path })
    }
}

#[derive(Clone)]
pub struct Sender {
    tx: mpsc::Sender<AppEventType>,
}

impl Debug for Sender {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Sender")
    }
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
        match ratatui::crossterm::event::read() {
            Ok(e) => match e {
                ratatui::crossterm::event::Event::Key(key) => {
                    event_tx.send(AppEventType::Key(key));
                }
                ratatui::crossterm::event::Event::Resize(w, h) => {
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
