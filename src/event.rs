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
    DetailSelectTabs,
    DetailDownload,
    DetailPreview,
    DetailToggleCopyDetails,
    DetailOpenManagementConsole,
    // CopyDetail
    CopyDetailSelectNext,
    CopyDetailSelectPrev,
    CopyDetailCopySelectedValue,
    CopyDetailClose,
    // Preview
    PreviewClose,
    PreviewDownload,
    PreviewToggleCopyDetails,
    // Help
    HelpClose,
    // common
    ToggleHelp,
}

pub struct AppKeyActionManager {
    helps: HashMap<ViewStateTag, Vec<String>>,
    short_helps: HashMap<ViewStateTag, Vec<(String, usize)>>,
}

impl AppKeyActionManager {
    pub fn new() -> AppKeyActionManager {
        AppKeyActionManager {
            helps: AppKeyActionManager::build_helps(),
            short_helps: AppKeyActionManager::build_short_helps(),
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

    fn build_short_helps() -> HashMap<ViewStateTag, Vec<(String, usize)>> {
        // fixme
        vec![
            (ViewStateTag::Initializing, vec![]),
            (
                ViewStateTag::BucketList,
                vec![
                    ("<Esc>: Quit", 0),
                    ("<j/k>: Select", 1),
                    ("<g/G>: Top/Bottom", 3),
                    ("<Enter>: Open", 2),
                    ("<?>: Help", 0),
                ],
            ),
            (
                ViewStateTag::ObjectList,
                vec![
                    ("<Esc>: Quit", 0),
                    ("<j/k>: Select", 3),
                    ("<g/G>: Top/Bottom", 4),
                    ("<Enter>: Open", 1),
                    ("<Backspace>: Go back", 2),
                    ("<?>: Help", 0),
                ],
            ),
            (
                ViewStateTag::Detail,
                vec![
                    ("<Esc>: Quit", 0),
                    ("<h/l>: Select tabs", 3),
                    ("<s>: Download", 1),
                    ("<p>: Preview", 4),
                    ("<Backspace>: Close", 2),
                    ("<?>: Help", 0),
                ],
            ),
            (
                ViewStateTag::CopyDetail,
                vec![
                    ("<Esc>: Quit", 0),
                    ("<j/k>: Select", 3),
                    ("<Enter>: Copy", 1),
                    ("<Backspace>: Close", 2),
                    ("<?>: Help", 0),
                ],
            ),
            (
                ViewStateTag::Preview,
                vec![
                    ("<Esc>: Quit", 0),
                    ("<s>: Download", 2),
                    ("<Backspace>: Close", 1),
                    ("<?>: Help", 0),
                ],
            ),
            (
                ViewStateTag::Help,
                vec![("<Esc>: Quit", 0), ("<?>: Close help", 0)],
            ),
        ]
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().map(|(s, n)| (s.to_owned(), n)).collect()))
        .collect::<HashMap<_, _>>()
    }

    pub fn key_to_action(&self, key: KeyEvent, vs: &ViewState) -> Option<AppKeyAction> {
        match vs {
            ViewState::Initializing => None,
            ViewState::BucketList => match key {
                key_code_char!('j') => Some(AppKeyAction::BucketListSelectNext),
                key_code_char!('k') => Some(AppKeyAction::BucketListSelectPrev),
                key_code_char!('g') => Some(AppKeyAction::BucketListSelectFirst),
                key_code_char!('G') => Some(AppKeyAction::BucketListSelectLast),
                key_code_char!('f') => Some(AppKeyAction::BucketListSelectNextPage),
                key_code_char!('b') => Some(AppKeyAction::BucketListSelectPrevPage),
                key_code!(KeyCode::Enter) => Some(AppKeyAction::BucketListMoveDown),
                key_code_char!('m', Ctrl) => Some(AppKeyAction::BucketListMoveDown),
                key_code_char!('x') => Some(AppKeyAction::BucketListOpenManagementConsole),
                key_code_char!('?') => Some(AppKeyAction::ToggleHelp),
                _ => None,
            },
            ViewState::ObjectList => match key {
                key_code_char!('j') => Some(AppKeyAction::ObjectListSelectNext),
                key_code_char!('k') => Some(AppKeyAction::ObjectListSelectPrev),
                key_code_char!('g') => Some(AppKeyAction::ObjectListSelectFirst),
                key_code_char!('G') => Some(AppKeyAction::ObjectListSelectLast),
                key_code_char!('f') => Some(AppKeyAction::ObjectListSelectNextPage),
                key_code_char!('b') => Some(AppKeyAction::ObjectListSelectPrevPage),
                key_code!(KeyCode::Enter) => Some(AppKeyAction::ObjectListMoveDown),
                key_code_char!('m', Ctrl) => Some(AppKeyAction::ObjectListMoveDown),
                key_code!(KeyCode::Backspace) => Some(AppKeyAction::ObjectListMoveUp),
                key_code_char!('h', Ctrl) => Some(AppKeyAction::ObjectListMoveUp),
                key_code_char!('~') => Some(AppKeyAction::ObjectListBackToBucketList),
                key_code_char!('x') => Some(AppKeyAction::ObjectListOpenManagementConsole),
                key_code_char!('?') => Some(AppKeyAction::ToggleHelp),
                _ => None,
            },
            ViewState::Detail(_) => match key {
                key_code!(KeyCode::Backspace) => Some(AppKeyAction::DetailClose),
                key_code_char!('h', Ctrl) => Some(AppKeyAction::DetailClose),
                key_code_char!('h') => Some(AppKeyAction::DetailSelectTabs),
                key_code_char!('l') => Some(AppKeyAction::DetailSelectTabs),
                key_code_char!('s') => Some(AppKeyAction::DetailDownload),
                key_code_char!('p') => Some(AppKeyAction::DetailPreview),
                key_code_char!('r') => Some(AppKeyAction::DetailToggleCopyDetails),
                key_code_char!('x') => Some(AppKeyAction::DetailOpenManagementConsole),
                key_code_char!('?') => Some(AppKeyAction::ToggleHelp),
                _ => None,
            },
            ViewState::CopyDetail(_) => match key {
                key_code_char!('j') => Some(AppKeyAction::CopyDetailSelectNext),
                key_code_char!('k') => Some(AppKeyAction::CopyDetailSelectPrev),
                key_code!(KeyCode::Enter) => Some(AppKeyAction::CopyDetailCopySelectedValue),
                key_code_char!('m', Ctrl) => Some(AppKeyAction::CopyDetailCopySelectedValue),
                key_code!(KeyCode::Backspace) => Some(AppKeyAction::CopyDetailClose),
                key_code_char!('h', Ctrl) => Some(AppKeyAction::CopyDetailClose),
                key_code_char!('?') => Some(AppKeyAction::ToggleHelp),
                _ => None,
            },
            ViewState::Preview(_) => match key {
                key_code!(KeyCode::Backspace) => Some(AppKeyAction::PreviewClose),
                key_code_char!('h', Ctrl) => Some(AppKeyAction::PreviewClose),
                key_code_char!('s') => Some(AppKeyAction::PreviewDownload),
                key_code_char!('r') => Some(AppKeyAction::PreviewToggleCopyDetails),
                key_code_char!('?') => Some(AppKeyAction::ToggleHelp),
                _ => None,
            },
            ViewState::Help(_) => match key {
                key_code!(KeyCode::Backspace) => Some(AppKeyAction::HelpClose),
                key_code_char!('h', Ctrl) => Some(AppKeyAction::HelpClose),
                key_code_char!('?') => Some(AppKeyAction::ToggleHelp),
                _ => None,
            },
        }
    }

    pub fn helps(&self, vs: &ViewState) -> &Vec<String> {
        self.helps.get(&vs.tag()).unwrap()
    }

    pub fn short_helps(&self, vs: &ViewState) -> &Vec<(String, usize)> {
        self.short_helps.get(&vs.tag()).unwrap()
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
