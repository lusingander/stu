use std::{collections::HashMap, sync::mpsc, thread};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use enum_tag::EnumTag;

use crate::{
    app::{ViewState, ViewStateTag},
    client::Client,
    config::Config,
    error::{AppError, Result},
    item::{BucketItem, FileDetail, FileVersion, Object, ObjectItem, ObjectKey},
    key_code, key_code_char,
};

pub enum AppEventType {
    Key(KeyEvent),
    KeyAction(AppKeyAction),
    Resize(u16, u16),
    Initialize(Config, Client),
    CompleteInitialize(Result<CompleteInitializeResult>),
    LoadObjects,
    CompleteLoadObjects(Result<CompleteLoadObjectsResult>),
    LoadObject,
    CompleteLoadObject(Result<CompleteLoadObjectResult>),
    DownloadObject,
    CompleteDownloadObject(Result<CompleteDownloadObjectResult>),
    PreviewObject,
    CompletePreviewObject(Result<CompletePreviewObjectResult>),
    CopyToClipboard(String, String),
    Info(String),
    Error(AppError),
}

// fixme: split into actions for each state
pub enum AppKeyAction {
    SelectNext,
    SelectPrev,
    SelectFirst,
    SelectLast,
    SelectNextPage,
    SelectPrevPage,
    MoveDown,
    MoveUp,
    BackToBucketList,
    SelectTabs,
    Download,
    Preview,
    ToggleCopyDetails,
    OpenManagementConsole,
    ToggleHelp,
}

pub struct AppKeyActionManager {
    helps: HashMap<ViewStateTag, Vec<String>>,
}

impl AppKeyActionManager {
    pub fn new() -> AppKeyActionManager {
        AppKeyActionManager {
            helps: AppKeyActionManager::build_helps(),
        }
    }

    fn build_helps() -> HashMap<ViewStateTag, Vec<String>> {
        // fixme
        vec![
            (ViewStateTag::Initializing, vec![]),
            (
                ViewStateTag::BucketList,
                vec![
                    "<Esc> <Ctrl-c>: Quit app",
                    "<j/k>: Select item",
                    "<g/G>: Go to top/bottom",
                    "<f>: Scroll page forward",
                    "<b>: Scroll page backward",
                    "<Enter>: Open bucket",
                    "<x>: Open management console in browser",
                ],
            ),
            (
                ViewStateTag::ObjectList,
                vec![
                    "<Esc> <Ctrl-c>: Quit app",
                    "<j/k>: Select item",
                    "<g/G>: Go to top/bottom",
                    "<f>: Scroll page forward",
                    "<b>: Scroll page backward",
                    "<Enter>: Open file or folder",
                    "<Backspace>: Go back to prev folder",
                    "<~>: Go back to bucket list",
                    "<x>: Open management console in browser",
                ],
            ),
            (
                ViewStateTag::Detail,
                vec![
                    "<Esc> <Ctrl-c>: Quit app",
                    "<h/l>: Select tabs",
                    "<Backspace>: Close detail panel",
                    "<r>: Open copy dialog",
                    "<s>: Download object",
                    "<p>: Preview object",
                    "<x>: Open management console in browser",
                ],
            ),
            (
                ViewStateTag::CopyDetail,
                vec![
                    "<Esc> <Ctrl-c>: Quit app",
                    "<j/k>: Select item",
                    "<Enter>: Copy selected value to clipboard",
                    "<Backspace> <r>: Close copy dialog",
                ],
            ),
            (
                ViewStateTag::Preview,
                vec![
                    "<Esc> <Ctrl-c>: Quit app",
                    "<Backspace>: Close preview",
                    "<s>: Download object",
                ],
            ),
            (ViewStateTag::Help, vec![]),
        ]
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().map(|s| s.to_owned()).collect()))
        .collect::<HashMap<_, _>>()
    }

    pub fn key_to_action(&self, key: KeyEvent) -> Option<AppKeyAction> {
        // fixme
        match key {
            key_code_char!('j') => Some(AppKeyAction::SelectNext),
            key_code_char!('k') => Some(AppKeyAction::SelectPrev),
            key_code_char!('g') => Some(AppKeyAction::SelectFirst),
            key_code_char!('G') => Some(AppKeyAction::SelectLast),
            key_code_char!('f') => Some(AppKeyAction::SelectNextPage),
            key_code_char!('b') => Some(AppKeyAction::SelectPrevPage),
            key_code!(KeyCode::Enter) | key_code_char!('m', Ctrl) => Some(AppKeyAction::MoveDown),
            key_code!(KeyCode::Backspace) | key_code_char!('h', Ctrl) => Some(AppKeyAction::MoveUp),
            key_code_char!('~') => Some(AppKeyAction::BackToBucketList),
            key_code_char!('h') | key_code_char!('l') => Some(AppKeyAction::SelectTabs),
            key_code_char!('s') => Some(AppKeyAction::Download),
            key_code_char!('p') => Some(AppKeyAction::Preview),
            key_code_char!('r') => Some(AppKeyAction::ToggleCopyDetails),
            key_code_char!('x') => Some(AppKeyAction::OpenManagementConsole),
            key_code_char!('?') => Some(AppKeyAction::ToggleHelp),
            _ => None,
        }
    }

    pub fn helps(&self, vs: &ViewState) -> &Vec<String> {
        self.helps.get(&vs.tag()).unwrap()
    }
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
    pub path: String,
}

impl CompletePreviewObjectResult {
    pub fn new(obj: Result<Object>, path: String) -> Result<CompletePreviewObjectResult> {
        let obj = obj?;
        Ok(CompletePreviewObjectResult { obj, path })
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
