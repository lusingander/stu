use std::{
    fmt::{self, Debug, Formatter},
    path::PathBuf,
    sync::Arc,
};

use futures::{FutureExt, StreamExt};
use ratatui::crossterm::event::KeyEvent;
use tokio::{select, spawn, sync::mpsc};

use crate::{
    error::{AppError, Result},
    object::{
        BucketItem, DownloadObjectInfo, FileDetail, FileVersion, ObjectItem, ObjectKey, RawObject,
    },
};

#[derive(Debug)]
pub enum AppEventType {
    Key(KeyEvent),
    Resize,
    Initialize(Option<String>),
    CompleteInitialize(Result<CompleteInitializeResult>),
    ReloadBuckets,
    CompleteReloadBuckets(Result<CompleteReloadBucketsResult>),
    LoadObjects(ObjectKey),
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
    StartDownloadObject(ObjectKey, String, usize, Option<String>),
    DownloadObject(ObjectKey, String, usize, Option<String>),
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
    BucketListMoveDown(ObjectKey),
    BucketListRefresh,
    ObjectListMoveDown,
    ObjectListMoveUp,
    ObjectListRefresh,
    BackToBucketList,
    OpenObjectVersionsTab,
    OpenPreview(ObjectKey, FileDetail, Option<String>),
    PreviewRerenderImage,
    BucketListOpenManagementConsole,
    ObjectListOpenManagementConsole(ObjectKey),
    ObjectDetailOpenManagementConsole(ObjectKey),
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
    pub object_key: ObjectKey,
}

impl CompleteLoadObjectsResult {
    pub fn new(
        items: Result<Vec<ObjectItem>>,
        object_key: ObjectKey,
    ) -> Result<CompleteLoadObjectsResult> {
        let items = items?;
        Ok(CompleteLoadObjectsResult { items, object_key })
    }
}

impl From<CompleteReloadObjectsResult> for CompleteLoadObjectsResult {
    fn from(result: CompleteReloadObjectsResult) -> Self {
        CompleteLoadObjectsResult {
            items: result.items,
            object_key: result.object_key,
        }
    }
}

#[derive(Debug)]
pub struct CompleteReloadObjectsResult {
    pub items: Vec<ObjectItem>,
    pub object_key: ObjectKey,
}

impl CompleteReloadObjectsResult {
    pub fn new(
        items: Result<Vec<ObjectItem>>,
        object_key: ObjectKey,
    ) -> Result<CompleteReloadObjectsResult> {
        let items = items?;
        Ok(CompleteReloadObjectsResult { items, object_key })
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
    tx: mpsc::UnboundedSender<AppEventType>,
}

impl Debug for Sender {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Sender")
    }
}

impl Sender {
    pub fn new(tx: mpsc::UnboundedSender<AppEventType>) -> Self {
        Self { tx }
    }

    pub fn send(&self, event: AppEventType) {
        self.tx.send(event).unwrap();
    }
}

pub struct Receiver {
    rx: mpsc::UnboundedReceiver<AppEventType>,
}

impl Receiver {
    pub fn new(rx: mpsc::UnboundedReceiver<AppEventType>) -> Self {
        Self { rx }
    }

    pub async fn recv(&mut self) -> AppEventType {
        self.rx.recv().await.unwrap()
    }
}

pub fn new() -> (Sender, Receiver) {
    let (tx, rx) = mpsc::unbounded_channel();
    let tx = Sender::new(tx);
    let rx = Receiver::new(rx);

    let mut reader = ratatui::crossterm::event::EventStream::new();
    let event_tx = tx.clone();
    spawn(async move {
        loop {
            let event = reader.next().fuse();
            select! {
                _ = event_tx.tx.closed() => {
                    break;
                }
                Some(Ok(e)) = event => {
                    match e {
                        ratatui::crossterm::event::Event::Key(key) => {
                            event_tx.send(AppEventType::Key(key));
                        }
                        ratatui::crossterm::event::Event::Resize(_, _) => {
                            event_tx.send(AppEventType::Resize);
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    (tx, rx)
}
