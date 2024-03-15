use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use enum_tag::EnumTag;

use crate::{
    app::{ViewState, ViewStateTag},
    event::{AppKeyAction, AppKeyInput},
    key_code, key_code_char,
    util::{group_map, to_map},
};

const QUIT_HELP_STR: &str = "<Esc> <Ctrl-c>: Quit app";
const QUIT_SHORT_HELP_STR: &str = "<Esc>: Quit";

pub struct AppKeyActionManager {
    key_action_map: HashMap<ViewStateTag, HashMap<KeyCodeModifiers, AppKeyAction>>,
    helps: HashMap<ViewStateTag, Vec<String>>,
    short_helps: HashMap<ViewStateTag, Vec<(String, usize)>>,
}

type KeyCodeModifiers = (KeyCode, KeyModifiers);

type KeyMapEntry = (ViewStateTag, KeyCode, KeyModifiers, AppKeyAction);

type HelpMsgDef = (&'static str, bool, &'static [AppKeyAction]);
type ShortHelpMsgDef = (&'static str, usize, bool, &'static [AppKeyAction]);

impl AppKeyActionManager {
    pub fn new() -> AppKeyActionManager {
        let key_maps = default_key_maps();
        AppKeyActionManager {
            key_action_map: AppKeyActionManager::build_key_action_map(&key_maps),
            helps: AppKeyActionManager::build_helps(&key_maps),
            short_helps: AppKeyActionManager::build_short_helps(&key_maps),
        }
    }

    fn build_key_action_map(
        key_maps: &[KeyMapEntry],
    ) -> HashMap<ViewStateTag, HashMap<KeyCodeModifiers, AppKeyAction>> {
        let grouped = group_map(key_maps, |(s, _, _, _)| *s, |(_, c, m, a)| (*c, *m, *a));
        grouped
            .into_iter()
            .map(|(k, vec)| (k, to_map(vec, |(c, m, a)| ((c, m), a))))
            .collect()
    }

    #[rustfmt::skip]
    fn build_helps(
        key_maps: &[KeyMapEntry],
    ) -> HashMap<ViewStateTag, Vec<String>> {
        use AppKeyAction::*;
        let v: Vec<(ViewStateTag, Vec<HelpMsgDef>)> = vec![
            (
                ViewStateTag::Initializing,
                vec![],
            ),
            (
                ViewStateTag::BucketList,
                vec![
                    ("Select item", true, &[BucketListSelectNext, BucketListSelectPrev]),
                    ("Go to top/bottom", true, &[BucketListSelectFirst, BucketListSelectLast]),
                    ("Scroll page forward", false, &[BucketListSelectNextPage]),
                    ("Scroll page backward", false, &[BucketListSelectPrevPage]),
                    ("Open bucket", false, &[BucketListMoveDown]),
                    ("Open management console in browser", false, &[BucketListOpenManagementConsole]),
                ],
            ),
            (
                ViewStateTag::ObjectList,
                vec![
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
            (
                ViewStateTag::Detail,
                vec![
                    ("Select tabs", true, &[DetailSelectNext, DetailSelectPrev]),
                    ("Close detail panel", false, &[DetailClose]),
                    ("Open copy dialog", false, &[DetailOpenCopyDetails]),
                    ("Download object", false, &[DetailDownloadObject]),
                    ("Download object as", false, &[DetailOpenDownloadObjectAs]),
                    ("Preview object", false, &[DetailPreview]),
                    ("Open management console in browser", false, &[DetailOpenManagementConsole]),
                ],
            ),
            (
                ViewStateTag::DetailSave,
                vec![
                    ("Download object", false, &[DetailSaveDownloadObjectAs]),
                ],
            ),
            (
                ViewStateTag::CopyDetail,
                vec![
                    ("Select item", true, &[CopyDetailSelectNext, CopyDetailSelectPrev]),
                    ("Copy selected value to clipboard", false, &[CopyDetailCopySelectedValue]),
                    ("Close copy dialog", false, &[CopyDetailClose]),
                ],
            ),
            (
                ViewStateTag::Preview,
                vec![
                    ("Close preview", false, &[PreviewClose]),
                    ("Download object", false, &[PreviewDownloadObject]),
                ],
            ),
            (
                ViewStateTag::Help,
                vec![],
            ),
        ];
        to_map(v, |(k, v)| { (k, AppKeyActionManager::build_help_vec(key_maps, k, &v)) })
    }

    #[rustfmt::skip]
    fn build_short_helps(key_maps: &[KeyMapEntry]) -> HashMap<ViewStateTag, Vec<(String, usize)>> {
        use AppKeyAction::*;
        let v: Vec<(ViewStateTag, Vec<ShortHelpMsgDef>)> = vec![
            (
                ViewStateTag::Initializing,
                vec![],
            ),
            (
                ViewStateTag::BucketList,
                vec![
                    ("Select", 1, true, &[BucketListSelectNext, BucketListSelectPrev]),
                    ("Top/Bottom", 3, true, &[BucketListSelectFirst, BucketListSelectLast]),
                    ("Open", 2, false, &[BucketListMoveDown]),
                    ("Help", 0, false, &[ToggleHelp]),
                ],
            ),
            (
                ViewStateTag::ObjectList,
                vec![
                    ("Select", 3, true, &[ObjectListSelectNext, ObjectListSelectPrev]),
                    ("Top/Bottom", 4, true, &[ObjectListSelectFirst, ObjectListSelectLast]),
                    ("Open", 1, false, &[ObjectListMoveDown]),
                    ("Go back", 2, false, &[ObjectListMoveUp]),
                    ("Help", 0, false, &[ToggleHelp]),
                ],
            ),
            (
                ViewStateTag::Detail,
                vec![
                    ("Select tabs", 3, true, &[DetailSelectNext, DetailSelectPrev]),
                    ("Download", 1, false, &[DetailDownloadObject]),
                    ("Preview", 4, false, &[DetailPreview]),
                    ("Close", 2, false, &[DetailClose]),
                    ("Help", 0, false, &[ToggleHelp]),
                ],
            ),
            (
                ViewStateTag::DetailSave,
                vec![
                    ("Download", 1, false, &[DetailSaveDownloadObjectAs]),
                    ("Help", 0, false, &[ToggleHelp]),
                ],
            ),
            (
                ViewStateTag::CopyDetail,
                vec![
                    ("Select", 3, true, &[CopyDetailSelectNext, CopyDetailSelectPrev]),
                    ("Copy", 1, false, &[CopyDetailCopySelectedValue]),
                    ("Close", 2, false, &[CopyDetailClose]),
                    ("Help", 0, false, &[ToggleHelp]),
                ],
            ),
            (
                ViewStateTag::Preview,
                vec![
                    ("Download", 2, true, &[PreviewDownloadObject]),
                    ("Close", 1, false, &[PreviewClose]),
                    ("Help", 0, false, &[ToggleHelp]),
                ],
            ),
            (
                ViewStateTag::Help,
                vec![
                    ("Close help", 0, false, &[ToggleHelp]),
                ],
            ),
        ];
        to_map(v, |(k, v)| {
            (k, AppKeyActionManager::build_short_help_with_priority_vec(key_maps, k, &v))
        })
    }

    fn build_help_vec(
        key_maps: &[KeyMapEntry],
        target_vs: ViewStateTag,
        xs: &[HelpMsgDef],
    ) -> Vec<String> {
        let mut vec = Vec::new();
        vec.push(String::from(QUIT_HELP_STR));
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
        key_maps: &[KeyMapEntry],
        target_vs: ViewStateTag,
        xs: &[ShortHelpMsgDef],
    ) -> Vec<(String, usize)> {
        let mut vec = Vec::new();
        vec.push((String::from(QUIT_SHORT_HELP_STR), 0));
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
        key_maps: &[KeyMapEntry],
        target_vs: ViewStateTag,
        target_actions: &[AppKeyAction],
    ) -> Vec<KeyMapEntry> {
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

    pub fn key_to_input(&self, key: KeyEvent, vs: &ViewState) -> Option<AppKeyInput> {
        if let ViewState::DetailSave(_) = vs {
            match key {
                key_code_char!(c) => Some(AppKeyInput::Char(c)),
                key_code!(KeyCode::Backspace) => Some(AppKeyInput::Backspace),
                _ => None,
            }
        } else {
            None
        }
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

#[rustfmt::skip]
fn default_key_maps() -> Vec<KeyMapEntry> {
    use AppKeyAction::*;
    let v = vec![
        (ViewStateTag::BucketList, KeyCode::Char('j'), KeyModifiers::NONE,  BucketListSelectNext),
        (ViewStateTag::BucketList, KeyCode::Char('k'), KeyModifiers::NONE,  BucketListSelectPrev),
        (ViewStateTag::BucketList, KeyCode::Char('g'), KeyModifiers::NONE,  BucketListSelectFirst),
        (ViewStateTag::BucketList, KeyCode::Char('G'), KeyModifiers::SHIFT, BucketListSelectLast),
        (ViewStateTag::BucketList, KeyCode::Char('f'), KeyModifiers::NONE,  BucketListSelectNextPage),
        (ViewStateTag::BucketList, KeyCode::Char('b'), KeyModifiers::NONE,  BucketListSelectPrevPage),
        (ViewStateTag::BucketList, KeyCode::Enter,     KeyModifiers::NONE,  BucketListMoveDown),
        (ViewStateTag::BucketList, KeyCode::Char('x'), KeyModifiers::NONE,  BucketListOpenManagementConsole),
        (ViewStateTag::BucketList, KeyCode::Char('?'), KeyModifiers::NONE,  ToggleHelp),
        (ViewStateTag::ObjectList, KeyCode::Char('j'), KeyModifiers::NONE,  ObjectListSelectNext),
        (ViewStateTag::ObjectList, KeyCode::Char('k'), KeyModifiers::NONE,  ObjectListSelectPrev),
        (ViewStateTag::ObjectList, KeyCode::Char('g'), KeyModifiers::NONE,  ObjectListSelectFirst),
        (ViewStateTag::ObjectList, KeyCode::Char('G'), KeyModifiers::SHIFT, ObjectListSelectLast),
        (ViewStateTag::ObjectList, KeyCode::Char('f'), KeyModifiers::NONE,  ObjectListSelectNextPage),
        (ViewStateTag::ObjectList, KeyCode::Char('b'), KeyModifiers::NONE,  ObjectListSelectPrevPage),
        (ViewStateTag::ObjectList, KeyCode::Enter,     KeyModifiers::NONE,  ObjectListMoveDown),
        (ViewStateTag::ObjectList, KeyCode::Backspace, KeyModifiers::NONE,  ObjectListMoveUp),
        (ViewStateTag::ObjectList, KeyCode::Char('~'), KeyModifiers::NONE,  ObjectListBackToBucketList),
        (ViewStateTag::ObjectList, KeyCode::Char('x'), KeyModifiers::NONE,  ObjectListOpenManagementConsole),
        (ViewStateTag::ObjectList, KeyCode::Char('?'), KeyModifiers::NONE,  ToggleHelp),
        (ViewStateTag::Detail,     KeyCode::Backspace, KeyModifiers::NONE,  DetailClose),
        (ViewStateTag::Detail,     KeyCode::Char('h'), KeyModifiers::NONE,  DetailSelectPrev),
        (ViewStateTag::Detail,     KeyCode::Char('l'), KeyModifiers::NONE,  DetailSelectNext),
        (ViewStateTag::Detail,     KeyCode::Char('s'), KeyModifiers::NONE,  DetailDownloadObject),
        (ViewStateTag::Detail,     KeyCode::Char('S'), KeyModifiers::SHIFT, DetailOpenDownloadObjectAs),
        (ViewStateTag::Detail,     KeyCode::Char('p'), KeyModifiers::NONE,  DetailPreview),
        (ViewStateTag::Detail,     KeyCode::Char('r'), KeyModifiers::NONE,  DetailOpenCopyDetails),
        (ViewStateTag::Detail,     KeyCode::Char('x'), KeyModifiers::NONE,  DetailOpenManagementConsole),
        (ViewStateTag::Detail,     KeyCode::Char('?'), KeyModifiers::NONE,  ToggleHelp),
        (ViewStateTag::DetailSave, KeyCode::Enter,     KeyModifiers::NONE,  DetailSaveDownloadObjectAs),
        (ViewStateTag::DetailSave, KeyCode::Char('?'), KeyModifiers::NONE,  ToggleHelp),
        (ViewStateTag::CopyDetail, KeyCode::Char('j'), KeyModifiers::NONE,  CopyDetailSelectNext),
        (ViewStateTag::CopyDetail, KeyCode::Char('k'), KeyModifiers::NONE,  CopyDetailSelectPrev),
        (ViewStateTag::CopyDetail, KeyCode::Enter,     KeyModifiers::NONE,  CopyDetailCopySelectedValue),
        (ViewStateTag::CopyDetail, KeyCode::Backspace, KeyModifiers::NONE,  CopyDetailClose),
        (ViewStateTag::CopyDetail, KeyCode::Char('?'), KeyModifiers::NONE,  ToggleHelp),
        (ViewStateTag::Preview,    KeyCode::Backspace, KeyModifiers::NONE,  PreviewClose),
        (ViewStateTag::Preview,    KeyCode::Char('s'), KeyModifiers::NONE,  PreviewDownloadObject),
        (ViewStateTag::Preview,    KeyCode::Char('?'), KeyModifiers::NONE,  ToggleHelp),
        (ViewStateTag::Help,       KeyCode::Backspace, KeyModifiers::NONE,  HelpClose),
        (ViewStateTag::Help,       KeyCode::Char('?'), KeyModifiers::NONE,  ToggleHelp),
    ];
    // (ViewStateTag, AppKeyAction) must not be duplicated
    debug_assert!(group_map(&v, |t| (t.0, t.3), |t| (t.0, t.3)).iter().all(|(_, v)| v.len() == 1));
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;

    #[rustfmt::skip]
    #[test]
    fn test_build_key_action_map() {
        use AppKeyAction::*;
        let key_maps = vec![
            (ViewStateTag::BucketList, KeyCode::Char('j'), KeyModifiers::NONE,  BucketListSelectNext),
            (ViewStateTag::BucketList, KeyCode::Char('k'), KeyModifiers::NONE,  BucketListSelectPrev),
            (ViewStateTag::BucketList, KeyCode::Down,      KeyModifiers::NONE,  BucketListSelectPrev),
            (ViewStateTag::ObjectList, KeyCode::Char('j'), KeyModifiers::NONE,  ObjectListSelectNext),
            (ViewStateTag::ObjectList, KeyCode::Enter,     KeyModifiers::NONE,  ObjectListMoveDown),
            (ViewStateTag::CopyDetail, KeyCode::Char('j'), KeyModifiers::NONE,  CopyDetailSelectNext),
        ];
        let actual = AppKeyActionManager::build_key_action_map(&key_maps);
        let expected = hashmap! {
            ViewStateTag::BucketList => hashmap! {
                (KeyCode::Char('j'), KeyModifiers::NONE) => BucketListSelectNext,
                (KeyCode::Char('k'), KeyModifiers::NONE) => BucketListSelectPrev,
                (KeyCode::Down,      KeyModifiers::NONE) => BucketListSelectPrev,
            },
            ViewStateTag::ObjectList => hashmap! {
                (KeyCode::Char('j'), KeyModifiers::NONE) => ObjectListSelectNext,
                (KeyCode::Enter,     KeyModifiers::NONE) => ObjectListMoveDown,
            },
            ViewStateTag::CopyDetail => hashmap! {
                (KeyCode::Char('j'), KeyModifiers::NONE) => CopyDetailSelectNext,
            },
        };
        assert_eq!(actual, expected);
    }

    #[rustfmt::skip]
    #[test]
    fn test_build_help_vec() {
        use AppKeyAction::*;
        let key_maps = vec![
            (ViewStateTag::BucketList, KeyCode::Char('j'), KeyModifiers::NONE,  BucketListSelectNext),
            (ViewStateTag::BucketList, KeyCode::Char('k'), KeyModifiers::NONE,  BucketListSelectPrev),
            (ViewStateTag::BucketList, KeyCode::Down,      KeyModifiers::NONE,  BucketListSelectPrev),
            (ViewStateTag::ObjectList, KeyCode::Char('j'), KeyModifiers::NONE,  ObjectListSelectNext),
            (ViewStateTag::ObjectList, KeyCode::Char('k'), KeyModifiers::NONE,  ObjectListSelectPrev),
            (ViewStateTag::CopyDetail, KeyCode::Char('j'), KeyModifiers::NONE,  CopyDetailSelectNext),
        ];
        let xs: &[HelpMsgDef] = &[
            ("abc", false, &[BucketListSelectNext]),
            ("def", false, &[BucketListSelectPrev]),
        ];
        let actual = AppKeyActionManager::build_help_vec(&key_maps, ViewStateTag::BucketList, xs);
        let expected = vec!["<Esc> <Ctrl-c>: Quit app", "<j>: abc", "<k> <Down>: def"];
        assert_eq!(actual, expected);

        let xs: &[HelpMsgDef] = &[
            ("foo bar", false, &[ObjectListSelectNext, ObjectListSelectPrev]),
            ("123", true, &[ObjectListSelectNext, ObjectListSelectPrev]),
        ];
        let actual = AppKeyActionManager::build_help_vec(&key_maps, ViewStateTag::ObjectList, xs);
        let expected = vec!["<Esc> <Ctrl-c>: Quit app", "<j> <k>: foo bar", "<j/k>: 123"];
        assert_eq!(actual, expected);
    }

    #[rustfmt::skip]
    #[test]
    fn test_build_short_help_with_priority_vec() {
        use AppKeyAction::*;
        let key_maps = vec![
            (ViewStateTag::BucketList, KeyCode::Char('j'), KeyModifiers::NONE,  BucketListSelectNext),
            (ViewStateTag::BucketList, KeyCode::Char('k'), KeyModifiers::NONE,  BucketListSelectPrev),
            (ViewStateTag::BucketList, KeyCode::Down,      KeyModifiers::NONE,  BucketListSelectPrev),
            (ViewStateTag::ObjectList, KeyCode::Char('j'), KeyModifiers::NONE,  ObjectListSelectNext),
            (ViewStateTag::ObjectList, KeyCode::Char('k'), KeyModifiers::NONE,  ObjectListSelectPrev),
            (ViewStateTag::CopyDetail, KeyCode::Char('j'), KeyModifiers::NONE,  CopyDetailSelectNext),
        ];
        let xs: &[ShortHelpMsgDef] = &[
            ("abc", 2, false, &[BucketListSelectNext]),
            ("def", 1, false, &[BucketListSelectPrev]),
        ];
        let actual = AppKeyActionManager::build_short_help_with_priority_vec(&key_maps, ViewStateTag::BucketList, xs);
        let expected: Vec<(String, usize)> = vec![
            ("<Esc>: Quit".to_string(), 0),
            ("<j>: abc".to_string(), 2),
            ("<k>: def".to_string(), 1),
        ];
        assert_eq!(actual, expected);

        let xs: &[ShortHelpMsgDef] = &[
            ("foo bar", 1, false, &[ObjectListSelectNext, ObjectListSelectPrev]),
            ("123", 2, true, &[ObjectListSelectNext, ObjectListSelectPrev]),
        ];
        let actual = AppKeyActionManager::build_short_help_with_priority_vec(&key_maps, ViewStateTag::ObjectList, xs);
        let expected: Vec<(String, usize)> = vec![
            ("<Esc>: Quit".to_string(), 0),
            ("<j>: foo bar".to_string(), 1),
            ("<j/k>: 123".to_string(), 2),
        ];
        assert_eq!(actual, expected);
    }
}
