use std::{collections::HashMap, sync::mpsc, thread};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use enum_tag::EnumTag;

use crate::{
    app::{ViewState, ViewStateTag},
    client::Client,
    config::Config,
    error::{AppError, Result},
    item::{BucketItem, FileDetail, FileVersion, Object, ObjectItem, ObjectKey},
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

#[derive(Clone, Copy)]
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
    DetailDownloadObject,
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
    PreviewDownloadObject,
    // Help
    HelpClose,
    // common
    ToggleHelp,
}

pub struct AppKeyActionManager {
    key_action_map: HashMap<ViewStateTag, HashMap<AppKeyInput, AppKeyAction>>,
    helps: HashMap<ViewStateTag, Vec<String>>,
    short_helps: HashMap<ViewStateTag, Vec<(String, usize)>>,
}

type AppKeyInput = (KeyCode, KeyModifiers);

impl AppKeyActionManager {
    pub fn new() -> AppKeyActionManager {
        let key_maps = key_maps();
        AppKeyActionManager {
            key_action_map: AppKeyActionManager::build_key_action_map(&key_maps),
            helps: AppKeyActionManager::build_helps(),
            short_helps: AppKeyActionManager::build_short_helps(),
        }
    }

    fn build_key_action_map(
        key_maps: &[(ViewStateTag, KeyCode, KeyModifiers, AppKeyAction)],
    ) -> HashMap<ViewStateTag, HashMap<AppKeyInput, AppKeyAction>> {
        let grouped = key_maps.iter().fold(
            HashMap::<ViewStateTag, Vec<(KeyCode, KeyModifiers, AppKeyAction)>>::new(),
            |mut acc, (s, c, m, a)| {
                acc.entry(*s).or_default().push((*c, *m, *a));
                acc
            },
        );
        grouped
            .into_iter()
            .map(|(k, vec)| (k, vec.into_iter().map(|(c, m, a)| ((c, m), a)).collect()))
            .collect()
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
        self.key_action_map
            .get(&vs.tag())
            .and_then(|m| m.get(&(key.code, key.modifiers)))
            .copied()
    }

    pub fn helps(&self, vs: &ViewState) -> &Vec<String> {
        self.helps.get(&vs.tag()).unwrap()
    }

    pub fn short_helps(&self, vs: &ViewState) -> &Vec<(String, usize)> {
        self.short_helps.get(&vs.tag()).unwrap()
    }
}

fn key_maps() -> Vec<(ViewStateTag, KeyCode, KeyModifiers, AppKeyAction)> {
    vec![
        (
            ViewStateTag::BucketList,
            KeyCode::Char('j'),
            KeyModifiers::NONE,
            AppKeyAction::BucketListSelectNext,
        ),
        (
            ViewStateTag::BucketList,
            KeyCode::Char('k'),
            KeyModifiers::NONE,
            AppKeyAction::BucketListSelectPrev,
        ),
        (
            ViewStateTag::BucketList,
            KeyCode::Char('g'),
            KeyModifiers::NONE,
            AppKeyAction::BucketListSelectFirst,
        ),
        (
            ViewStateTag::BucketList,
            KeyCode::Char('G'),
            KeyModifiers::SHIFT,
            AppKeyAction::BucketListSelectLast,
        ),
        (
            ViewStateTag::BucketList,
            KeyCode::Char('f'),
            KeyModifiers::NONE,
            AppKeyAction::BucketListSelectNextPage,
        ),
        (
            ViewStateTag::BucketList,
            KeyCode::Char('b'),
            KeyModifiers::NONE,
            AppKeyAction::BucketListSelectPrevPage,
        ),
        (
            ViewStateTag::BucketList,
            KeyCode::Enter,
            KeyModifiers::NONE,
            AppKeyAction::BucketListMoveDown,
        ),
        (
            ViewStateTag::BucketList,
            KeyCode::Char('m'),
            KeyModifiers::CONTROL,
            AppKeyAction::BucketListMoveDown,
        ),
        (
            ViewStateTag::BucketList,
            KeyCode::Char('x'),
            KeyModifiers::NONE,
            AppKeyAction::BucketListOpenManagementConsole,
        ),
        (
            ViewStateTag::BucketList,
            KeyCode::Char('?'),
            KeyModifiers::NONE,
            AppKeyAction::ToggleHelp,
        ),
        (
            ViewStateTag::ObjectList,
            KeyCode::Char('j'),
            KeyModifiers::NONE,
            AppKeyAction::ObjectListSelectNext,
        ),
        (
            ViewStateTag::ObjectList,
            KeyCode::Char('k'),
            KeyModifiers::NONE,
            AppKeyAction::ObjectListSelectPrev,
        ),
        (
            ViewStateTag::ObjectList,
            KeyCode::Char('g'),
            KeyModifiers::NONE,
            AppKeyAction::ObjectListSelectFirst,
        ),
        (
            ViewStateTag::ObjectList,
            KeyCode::Char('G'),
            KeyModifiers::SHIFT,
            AppKeyAction::ObjectListSelectLast,
        ),
        (
            ViewStateTag::ObjectList,
            KeyCode::Char('f'),
            KeyModifiers::NONE,
            AppKeyAction::ObjectListSelectNextPage,
        ),
        (
            ViewStateTag::ObjectList,
            KeyCode::Char('b'),
            KeyModifiers::NONE,
            AppKeyAction::ObjectListSelectPrevPage,
        ),
        (
            ViewStateTag::ObjectList,
            KeyCode::Enter,
            KeyModifiers::NONE,
            AppKeyAction::ObjectListMoveDown,
        ),
        (
            ViewStateTag::ObjectList,
            KeyCode::Char('m'),
            KeyModifiers::CONTROL,
            AppKeyAction::ObjectListMoveDown,
        ),
        (
            ViewStateTag::ObjectList,
            KeyCode::Backspace,
            KeyModifiers::NONE,
            AppKeyAction::ObjectListMoveUp,
        ),
        (
            ViewStateTag::ObjectList,
            KeyCode::Char('h'),
            KeyModifiers::CONTROL,
            AppKeyAction::ObjectListMoveUp,
        ),
        (
            ViewStateTag::ObjectList,
            KeyCode::Char('~'),
            KeyModifiers::NONE,
            AppKeyAction::ObjectListBackToBucketList,
        ),
        (
            ViewStateTag::ObjectList,
            KeyCode::Char('x'),
            KeyModifiers::NONE,
            AppKeyAction::ObjectListOpenManagementConsole,
        ),
        (
            ViewStateTag::ObjectList,
            KeyCode::Char('?'),
            KeyModifiers::NONE,
            AppKeyAction::ToggleHelp,
        ),
        (
            ViewStateTag::Detail,
            KeyCode::Backspace,
            KeyModifiers::NONE,
            AppKeyAction::DetailClose,
        ),
        (
            ViewStateTag::Detail,
            KeyCode::Char('h'),
            KeyModifiers::CONTROL,
            AppKeyAction::DetailClose,
        ),
        (
            ViewStateTag::Detail,
            KeyCode::Char('h'),
            KeyModifiers::NONE,
            AppKeyAction::DetailSelectTabs,
        ),
        (
            ViewStateTag::Detail,
            KeyCode::Char('l'),
            KeyModifiers::NONE,
            AppKeyAction::DetailSelectTabs,
        ),
        (
            ViewStateTag::Detail,
            KeyCode::Char('s'),
            KeyModifiers::NONE,
            AppKeyAction::DetailDownloadObject,
        ),
        (
            ViewStateTag::Detail,
            KeyCode::Char('p'),
            KeyModifiers::NONE,
            AppKeyAction::DetailPreview,
        ),
        (
            ViewStateTag::Detail,
            KeyCode::Char('r'),
            KeyModifiers::NONE,
            AppKeyAction::DetailToggleCopyDetails,
        ),
        (
            ViewStateTag::Detail,
            KeyCode::Char('x'),
            KeyModifiers::NONE,
            AppKeyAction::DetailOpenManagementConsole,
        ),
        (
            ViewStateTag::Detail,
            KeyCode::Char('?'),
            KeyModifiers::NONE,
            AppKeyAction::ToggleHelp,
        ),
        (
            ViewStateTag::CopyDetail,
            KeyCode::Char('j'),
            KeyModifiers::NONE,
            AppKeyAction::CopyDetailSelectNext,
        ),
        (
            ViewStateTag::CopyDetail,
            KeyCode::Char('k'),
            KeyModifiers::NONE,
            AppKeyAction::CopyDetailSelectPrev,
        ),
        (
            ViewStateTag::CopyDetail,
            KeyCode::Enter,
            KeyModifiers::NONE,
            AppKeyAction::CopyDetailCopySelectedValue,
        ),
        (
            ViewStateTag::CopyDetail,
            KeyCode::Char('m'),
            KeyModifiers::CONTROL,
            AppKeyAction::CopyDetailCopySelectedValue,
        ),
        (
            ViewStateTag::CopyDetail,
            KeyCode::Char('r'),
            KeyModifiers::NONE,
            AppKeyAction::CopyDetailClose,
        ),
        (
            ViewStateTag::CopyDetail,
            KeyCode::Backspace,
            KeyModifiers::NONE,
            AppKeyAction::CopyDetailClose,
        ),
        (
            ViewStateTag::CopyDetail,
            KeyCode::Char('h'),
            KeyModifiers::CONTROL,
            AppKeyAction::CopyDetailClose,
        ),
        (
            ViewStateTag::CopyDetail,
            KeyCode::Char('?'),
            KeyModifiers::NONE,
            AppKeyAction::ToggleHelp,
        ),
        (
            ViewStateTag::Preview,
            KeyCode::Backspace,
            KeyModifiers::NONE,
            AppKeyAction::PreviewClose,
        ),
        (
            ViewStateTag::Preview,
            KeyCode::Char('h'),
            KeyModifiers::CONTROL,
            AppKeyAction::PreviewClose,
        ),
        (
            ViewStateTag::Preview,
            KeyCode::Char('s'),
            KeyModifiers::NONE,
            AppKeyAction::PreviewDownloadObject,
        ),
        (
            ViewStateTag::Preview,
            KeyCode::Char('?'),
            KeyModifiers::NONE,
            AppKeyAction::ToggleHelp,
        ),
        (
            ViewStateTag::Help,
            KeyCode::Backspace,
            KeyModifiers::NONE,
            AppKeyAction::HelpClose,
        ),
        (
            ViewStateTag::Help,
            KeyCode::Char('h'),
            KeyModifiers::CONTROL,
            AppKeyAction::HelpClose,
        ),
        (
            ViewStateTag::Help,
            KeyCode::Char('?'),
            KeyModifiers::NONE,
            AppKeyAction::ToggleHelp,
        ),
    ]
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
