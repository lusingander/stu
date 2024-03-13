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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
    DetailOpenCopyDetails,
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
            helps: AppKeyActionManager::build_helps(&key_maps),
            short_helps: AppKeyActionManager::build_short_helps(&key_maps),
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

    #[rustfmt::skip]
    fn build_helps(
        key_maps: &[(ViewStateTag, KeyCode, KeyModifiers, AppKeyAction)],
    ) -> HashMap<ViewStateTag, Vec<String>> {
        use AppKeyAction::*;
        vec![
            (
                ViewStateTag::Initializing,
                vec![],
            ),
            (
                ViewStateTag::BucketList,
                AppKeyActionManager::build_help_vec(
                    key_maps,
                    ViewStateTag::BucketList,
                    &[
                        ("Select item", true, &[BucketListSelectNext, BucketListSelectPrev]),
                        ("Go to top/bottom", true, &[BucketListSelectFirst, BucketListSelectLast]),
                        ("Scroll page forward", false, &[BucketListSelectNextPage]),
                        ("Scroll page backward", false, &[BucketListSelectPrevPage]),
                        ("Open bucket", false, &[BucketListMoveDown]),
                        ("Open management console in browser", false, &[BucketListOpenManagementConsole]),
                    ],
                ),
            ),
            (
                ViewStateTag::ObjectList,
                AppKeyActionManager::build_help_vec(
                    key_maps,
                    ViewStateTag::ObjectList,
                    &[
                        ("Select item", true, &[ObjectListSelectNext, ObjectListSelectPrev]),
                        ("Go to top/bottom", true, &[ObjectListSelectFirst, ObjectListSelectLast]),
                        ("Scroll page forward", false, &[ObjectListSelectNextPage]),
                        ("Scroll page backward", false, &[ObjectListSelectPrevPage]),
                        ("Open file or folder", false, &[ObjectListMoveDown]),
                        ("Go back to prev folder", false, &[ObjectListMoveUp]),
                        ("Go back to bucket list", false, &[ObjectListBackToBucketList]),
                        ("Open management console in browser", false, &[ObjectListOpenManagementConsole]),
                    ],
                ),
            ),
            (
                ViewStateTag::Detail,
                AppKeyActionManager::build_help_vec(
                    key_maps,
                    ViewStateTag::Detail,
                    &[
                        ("Select tabs", true, &[DetailSelectNext, DetailSelectPrev]),
                        ("Close detail panel", false, &[DetailClose]),
                        ("Open copy dialog", false, &[DetailOpenCopyDetails]),
                        ("Download object", false, &[DetailDownloadObject]),
                        ("Preview object", false, &[DetailPreview]),
                        ("Open management console in browser", false, &[DetailOpenManagementConsole]),
                    ],
                ),
            ),
            (
                ViewStateTag::CopyDetail,
                AppKeyActionManager::build_help_vec(
                    key_maps,
                    ViewStateTag::CopyDetail,
                    &[
                        ("Select item", true, &[CopyDetailSelectNext, CopyDetailSelectPrev]),
                        ("Copy selected value to clipboard", false, &[CopyDetailCopySelectedValue]),
                        ("Close copy dialog", false, &[CopyDetailClose]),
                    ],
                ),
            ),
            (
                ViewStateTag::Preview,
                AppKeyActionManager::build_help_vec(
                    key_maps,
                    ViewStateTag::Preview,
                    &[
                        ("Close preview", false, &[PreviewClose]),
                        ("Download object", false, &[PreviewDownloadObject]),
                    ],
                ),
            ),
            (
                ViewStateTag::Help,
                vec![],
            ),
        ]
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().map(|s| s.to_owned()).collect()))
        .collect::<HashMap<_, _>>()
    }

    #[rustfmt::skip]
    fn build_short_helps(
        key_maps: &[(ViewStateTag, KeyCode, KeyModifiers, AppKeyAction)],
    ) -> HashMap<ViewStateTag, Vec<(String, usize)>> {
        use AppKeyAction::*;
        vec![
            (
                ViewStateTag::Initializing,
                vec![],
            ),
            (
                ViewStateTag::BucketList,
                AppKeyActionManager::build_short_help_with_priority_vec(
                    key_maps,
                    ViewStateTag::BucketList,
                    &[
                        ("Select", 1, true, &[BucketListSelectNext, BucketListSelectPrev]),
                        ("Top/Bottom", 3, true, &[BucketListSelectFirst, BucketListSelectLast]),
                        ("Open", 2, false, &[BucketListMoveDown]),
                        ("Help", 0, false, &[ToggleHelp]),
                    ],
                ),
            ),
            (
                ViewStateTag::ObjectList,
                AppKeyActionManager::build_short_help_with_priority_vec(
                    key_maps,
                    ViewStateTag::ObjectList,
                    &[
                        ("Select", 3, true, &[ObjectListSelectNext, ObjectListSelectPrev]),
                        ("Top/Bottom", 4, true, &[ObjectListSelectFirst, ObjectListSelectLast]),
                        ("Open", 1, false, &[ObjectListMoveDown]),
                        ("Go back", 2, false, &[ObjectListMoveUp]),
                        ("Help", 0, false, &[ToggleHelp]),
                    ],
                ),
            ),
            (
                ViewStateTag::Detail,
                AppKeyActionManager::build_short_help_with_priority_vec(
                    key_maps,
                    ViewStateTag::Detail,
                    &[
                        ("Select tabs", 3, true, &[DetailSelectNext, DetailSelectPrev]),
                        ("Download", 1, false, &[DetailDownloadObject]),
                        ("Preview", 4, false, &[DetailPreview]),
                        ("Close", 2, false, &[DetailClose]),
                        ("Help", 0, false, &[ToggleHelp]),
                    ],
                ),
            ),
            (
                ViewStateTag::CopyDetail,
                AppKeyActionManager::build_short_help_with_priority_vec(
                    key_maps,
                    ViewStateTag::CopyDetail,
                    &[
                        ("Select", 3, true, &[CopyDetailSelectNext, CopyDetailSelectPrev]),
                        ("Copy", 1, false, &[CopyDetailCopySelectedValue]),
                        ("Close", 2, false, &[CopyDetailClose]),
                        ("Help", 0, false, &[ToggleHelp]),
                    ],
                ),
            ),
            (
                ViewStateTag::Preview,
                AppKeyActionManager::build_short_help_with_priority_vec(
                    key_maps,
                    ViewStateTag::Preview,
                    &[
                        ("Download", 2, true, &[PreviewDownloadObject]),
                        ("Close", 1, false, &[PreviewClose]),
                        ("Help", 0, false, &[ToggleHelp]),
                    ],
                ),
            ),
            (
                ViewStateTag::Help,
                AppKeyActionManager::build_short_help_with_priority_vec(
                    key_maps,
                    ViewStateTag::Help,
                    &[
                        ("Help", 0, false, &[ToggleHelp]),
                    ],
                ),
            ),
        ]
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().map(|(s, n)| (s.to_owned(), n)).collect()))
        .collect::<HashMap<_, _>>()
    }

    fn build_help_vec(
        key_maps: &[(ViewStateTag, KeyCode, KeyModifiers, AppKeyAction)],
        target_vs: ViewStateTag,
        xs: &[(&str, bool, &[AppKeyAction])],
    ) -> Vec<String> {
        let mut vec = Vec::new();
        vec.push(String::from("<Esc> <Ctrl-c>: Quit app"));
        for (desc, with_slash, actions) in xs {
            let maps = AppKeyActionManager::find_key_maps(key_maps, target_vs, actions);
            let keys = if *with_slash {
                let keys = maps
                    .into_iter()
                    .map(|(_, code, modifier, _)| to_key_input_str(code, modifier))
                    .collect::<Vec<String>>()
                    .join("/");
                format!("<{}>", keys)
            } else {
                maps.into_iter()
                    .map(|(_, code, modifier, _)| to_key_input_str(code, modifier))
                    .map(|key| format!("<{}>", key))
                    .collect::<Vec<String>>()
                    .join(" ")
            };
            let s = format!("{}: {}", keys, desc);
            vec.push(s);
        }
        vec
    }

    fn build_short_help_with_priority_vec(
        key_maps: &[(ViewStateTag, KeyCode, KeyModifiers, AppKeyAction)],
        target_vs: ViewStateTag,
        xs: &[(&str, usize, bool, &[AppKeyAction])],
    ) -> Vec<(String, usize)> {
        let mut vec = Vec::new();
        vec.push((String::from("<Esc>: Quit"), 0));
        for (desc, priority, with_slash, actions) in xs {
            let maps = AppKeyActionManager::find_key_maps(key_maps, target_vs, actions);
            let keys = if *with_slash {
                let maps = maps.into_iter().fold(
                    Vec::<(AppKeyAction, KeyCode, KeyModifiers)>::new(),
                    |mut acc, (_, c, m, a)| {
                        if !acc.iter().any(|(aa, _, _)| a == *aa) {
                            acc.push((a, c, m));
                        }
                        acc
                    },
                );
                let keys = maps
                    .into_iter()
                    .map(|(_, code, modifier)| to_key_input_str(code, modifier))
                    .collect::<Vec<String>>()
                    .join("/");
                format!("<{}>", keys)
            } else {
                let (_, code, modifier, _) = maps.first().unwrap();
                let key = to_key_input_str(*code, *modifier);
                format!("<{}>", key)
            };
            let s = format!("{}: {}", keys, desc);
            vec.push((s, *priority));
        }
        vec
    }

    fn find_key_maps(
        key_maps: &[(ViewStateTag, KeyCode, KeyModifiers, AppKeyAction)],
        target_vs: ViewStateTag,
        target_actions: &[AppKeyAction],
    ) -> Vec<(ViewStateTag, KeyCode, KeyModifiers, AppKeyAction)> {
        key_maps
            .iter()
            .filter(|(vs, _, _, _)| target_vs == *vs)
            .filter(|(_, _, _, action)| target_actions.contains(action))
            .copied()
            .collect()
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

fn to_key_input_str(code: KeyCode, modifier: KeyModifiers) -> String {
    let s = match code {
        KeyCode::F(n) => format!("F{}", n),
        KeyCode::Char(c) => format!("{}", c),
        _ => format!("{:?}", code),
    };
    match modifier {
        KeyModifiers::CONTROL => format!("Ctrl-{}", s),
        _ => s,
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
            KeyCode::Backspace,
            KeyModifiers::NONE,
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
            KeyModifiers::NONE,
            AppKeyAction::DetailSelectPrev,
        ),
        (
            ViewStateTag::Detail,
            KeyCode::Char('l'),
            KeyModifiers::NONE,
            AppKeyAction::DetailSelectNext,
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
            AppKeyAction::DetailOpenCopyDetails,
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
            KeyCode::Backspace,
            KeyModifiers::NONE,
            AppKeyAction::CopyDetailClose,
        ),
        (
            ViewStateTag::CopyDetail,
            KeyCode::Char('r'),
            KeyModifiers::NONE,
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
