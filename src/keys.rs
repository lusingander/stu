use indexmap::IndexMap;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::Config;

const DEFAULT_KEYBINDINGS: &str = include_str!("../assets/keybindings.toml");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserEvent {
    Quit,
    Help,
    DumpApp,
    BucketListDown,
    BucketListUp,
    BucketListGoToTop,
    BucketListGoToBottom,
    BucketListPageDown,
    BucketListPageUp,
    BucketListSelect,
    BucketListDownloadObject,
    BucketListDownloadObjectAs,
    BucketListFilter,
    BucketListSort,
    BucketListCopyDetails,
    BucketListRefresh,
    BucketListResetFilter,
    BucketListManagementConsole,
    ObjectListDown,
    ObjectListUp,
    ObjectListGoToTop,
    ObjectListGoToBottom,
    ObjectListPageDown,
    ObjectListPageUp,
    ObjectListSelect,
    ObjectListBack,
    ObjectListBucketList,
    ObjectListDownloadObject,
    ObjectListDownloadObjectAs,
    ObjectListFilter,
    ObjectListSort,
    ObjectListCopyDetails,
    ObjectListRefresh,
    ObjectListResetFilter,
    ObjectListManagementConsole,
    ObjectDetailDown,
    ObjectDetailUp,
    ObjectDetailRight,
    ObjectDetailLeft,
    ObjectDetailGoToTop,
    ObjectDetailGoToBottom,
    ObjectDetailBack,
    ObjectDetailDownload,
    ObjectDetailDownloadAs,
    ObjectDetailPreview,
    ObjectDetailCopyDetails,
    ObjectDetailManagementConsole,
    ObjectPreviewDown,
    ObjectPreviewUp,
    ObjectPreviewRight,
    ObjectPreviewLeft,
    ObjectPreviewGoToTop,
    ObjectPreviewGoToBottom,
    ObjectPreviewPageDown,
    ObjectPreviewPageUp,
    ObjectPreviewBack,
    ObjectPreviewDownload,
    ObjectPreviewDownloadAs,
    ObjectPreviewEncoding,
    ObjectPreviewToggleWrap,
    ObjectPreviewToggleNumber,
    HelpClose,
    InputDialogClose,
    InputDialogApply,
    SelectDialogDown,
    SelectDialogUp,
    SelectDialogRight,
    SelectDialogLeft,
    SelectDialogClose,
    SelectDialogSelect,
}

#[derive(Debug, Default)]
pub struct UserEventMapper {
    map: IndexMap<KeyEvent, Vec<UserEvent>>,
}

impl UserEventMapper {
    pub fn load(config: &Config) -> anyhow::Result<UserEventMapper> {
        let path = config.keybindings_file_path()?;
        let custom_bindings_str = std::fs::read_to_string(path).unwrap_or_default();
        build_user_event_mapper(DEFAULT_KEYBINDINGS, &custom_bindings_str)
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub fn find_events(&self, e: KeyEvent) -> Vec<UserEvent> {
        self.map.get(&e).cloned().unwrap_or_default()
    }

    pub fn find_keys(&self, e: UserEvent) -> Vec<KeyEvent> {
        self.map
            .iter()
            .filter_map(|(k, v)| if v.contains(&e) { Some(*k) } else { None })
            .collect()
    }

    pub fn find_first_key(&self, e: UserEvent) -> Option<KeyEvent> {
        self.map
            .iter()
            .find_map(|(k, v)| if v.contains(&e) { Some(*k) } else { None })
    }
}

#[rustfmt::skip]
fn build_user_event_mapper(
    default_bindings_str: &str,
    custom_bindings_str: &str
) -> Result<UserEventMapper, String> {
    let bindings = deserialize_and_merge_bindings(default_bindings_str, custom_bindings_str)?;
    let mut map = IndexMap::new();

    set_event_to_map(&mut map, &bindings, "common", "quit", UserEvent::Quit)?;
    set_event_to_map(&mut map, &bindings, "common", "help", UserEvent::Help)?;
    set_event_to_map(&mut map, &bindings, "common", "dump", UserEvent::DumpApp)?;

    set_event_to_map(&mut map, &bindings, "bucket_list", "down", UserEvent::BucketListDown)?;
    set_event_to_map(&mut map, &bindings, "bucket_list", "up", UserEvent::BucketListUp)?;
    set_event_to_map(&mut map, &bindings, "bucket_list", "go_to_top", UserEvent::BucketListGoToTop)?;
    set_event_to_map(&mut map, &bindings, "bucket_list", "go_to_bottom", UserEvent::BucketListGoToBottom)?;
    set_event_to_map(&mut map, &bindings, "bucket_list", "page_down", UserEvent::BucketListPageDown)?;
    set_event_to_map(&mut map, &bindings, "bucket_list", "page_up", UserEvent::BucketListPageUp)?;
    set_event_to_map(&mut map, &bindings, "bucket_list", "select", UserEvent::BucketListSelect)?;
    set_event_to_map(&mut map, &bindings, "bucket_list", "download", UserEvent::BucketListDownloadObject)?;
    set_event_to_map(&mut map, &bindings, "bucket_list", "download_as", UserEvent::BucketListDownloadObjectAs)?;
    set_event_to_map(&mut map, &bindings, "bucket_list", "filter", UserEvent::BucketListFilter)?;
    set_event_to_map(&mut map, &bindings, "bucket_list", "sort", UserEvent::BucketListSort)?;
    set_event_to_map(&mut map, &bindings, "bucket_list", "copy_details", UserEvent::BucketListCopyDetails)?;
    set_event_to_map(&mut map, &bindings, "bucket_list", "refresh", UserEvent::BucketListRefresh)?;
    set_event_to_map(&mut map, &bindings, "bucket_list", "reset_filter", UserEvent::BucketListResetFilter)?;
    set_event_to_map(&mut map, &bindings, "bucket_list", "management_console", UserEvent::BucketListManagementConsole)?;

    set_event_to_map(&mut map, &bindings, "object_list", "down", UserEvent::ObjectListDown)?;
    set_event_to_map(&mut map, &bindings, "object_list", "up", UserEvent::ObjectListUp)?;
    set_event_to_map(&mut map, &bindings, "object_list", "go_to_top", UserEvent::ObjectListGoToTop)?;
    set_event_to_map(&mut map, &bindings, "object_list", "go_to_bottom", UserEvent::ObjectListGoToBottom)?;
    set_event_to_map(&mut map, &bindings, "object_list", "page_down", UserEvent::ObjectListPageDown)?;
    set_event_to_map(&mut map, &bindings, "object_list", "page_up", UserEvent::ObjectListPageUp)?;
    set_event_to_map(&mut map, &bindings, "object_list", "select", UserEvent::ObjectListSelect)?;
    set_event_to_map(&mut map, &bindings, "object_list", "back", UserEvent::ObjectListBack)?;
    set_event_to_map(&mut map, &bindings, "object_list", "bucket_list", UserEvent::ObjectListBucketList)?;
    set_event_to_map(&mut map, &bindings, "object_list", "download", UserEvent::ObjectListDownloadObject)?;
    set_event_to_map(&mut map, &bindings, "object_list", "download_as", UserEvent::ObjectListDownloadObjectAs)?;
    set_event_to_map(&mut map, &bindings, "object_list", "filter", UserEvent::ObjectListFilter)?;
    set_event_to_map(&mut map, &bindings, "object_list", "sort", UserEvent::ObjectListSort)?;
    set_event_to_map(&mut map, &bindings, "object_list", "copy_details", UserEvent::ObjectListCopyDetails)?;
    set_event_to_map(&mut map, &bindings, "object_list", "refresh", UserEvent::ObjectListRefresh)?;
    set_event_to_map(&mut map, &bindings, "object_list", "reset_filter", UserEvent::ObjectListResetFilter)?;
    set_event_to_map(&mut map, &bindings, "object_list", "management_console", UserEvent::ObjectListManagementConsole)?;
    
    set_event_to_map(&mut map, &bindings, "object_detail", "down", UserEvent::ObjectDetailDown)?;
    set_event_to_map(&mut map, &bindings, "object_detail", "up", UserEvent::ObjectDetailUp)?;
    set_event_to_map(&mut map, &bindings, "object_detail", "right", UserEvent::ObjectDetailRight)?;
    set_event_to_map(&mut map, &bindings, "object_detail", "left", UserEvent::ObjectDetailLeft)?;
    set_event_to_map(&mut map, &bindings, "object_detail", "go_to_top", UserEvent::ObjectDetailGoToTop)?;
    set_event_to_map(&mut map, &bindings, "object_detail", "go_to_bottom", UserEvent::ObjectDetailGoToBottom)?;
    set_event_to_map(&mut map, &bindings, "object_detail", "back", UserEvent::ObjectDetailBack)?;
    set_event_to_map(&mut map, &bindings, "object_detail", "download", UserEvent::ObjectDetailDownload)?;
    set_event_to_map(&mut map, &bindings, "object_detail", "download_as", UserEvent::ObjectDetailDownloadAs)?;
    set_event_to_map(&mut map, &bindings, "object_detail", "preview", UserEvent::ObjectDetailPreview)?;
    set_event_to_map(&mut map, &bindings, "object_detail", "copy_details", UserEvent::ObjectDetailCopyDetails)?;
    set_event_to_map(&mut map, &bindings, "object_detail", "management_console", UserEvent::ObjectDetailManagementConsole)?;

    set_event_to_map(&mut map, &bindings, "object_preview", "down", UserEvent::ObjectPreviewDown)?;
    set_event_to_map(&mut map, &bindings, "object_preview", "up", UserEvent::ObjectPreviewUp)?;
    set_event_to_map(&mut map, &bindings, "object_preview", "right", UserEvent::ObjectPreviewRight)?;
    set_event_to_map(&mut map, &bindings, "object_preview", "left", UserEvent::ObjectPreviewLeft)?;
    set_event_to_map(&mut map, &bindings, "object_preview", "go_to_top", UserEvent::ObjectPreviewGoToTop)?;
    set_event_to_map(&mut map, &bindings, "object_preview", "go_to_bottom", UserEvent::ObjectPreviewGoToBottom)?;
    set_event_to_map(&mut map, &bindings, "object_preview", "page_down", UserEvent::ObjectPreviewPageDown)?;
    set_event_to_map(&mut map, &bindings, "object_preview", "page_up", UserEvent::ObjectPreviewPageUp)?;
    set_event_to_map(&mut map, &bindings, "object_preview", "back", UserEvent::ObjectPreviewBack)?;
    set_event_to_map(&mut map, &bindings, "object_preview", "download", UserEvent::ObjectPreviewDownload)?;
    set_event_to_map(&mut map, &bindings, "object_preview", "download_as", UserEvent::ObjectPreviewDownloadAs)?;
    set_event_to_map(&mut map, &bindings, "object_preview", "encoding", UserEvent::ObjectPreviewEncoding)?;
    set_event_to_map(&mut map, &bindings, "object_preview", "toggle_wrap", UserEvent::ObjectPreviewToggleWrap)?;
    set_event_to_map(&mut map, &bindings, "object_preview", "toggle_number", UserEvent::ObjectPreviewToggleNumber)?;

    set_event_to_map(&mut map, &bindings, "help", "close", UserEvent::HelpClose)?;

    set_event_to_map(&mut map, &bindings, "input_dialog", "close", UserEvent::InputDialogClose)?;
    set_event_to_map(&mut map, &bindings, "input_dialog", "apply", UserEvent::InputDialogApply)?;

    set_event_to_map(&mut map, &bindings, "select_dialog", "down", UserEvent::SelectDialogDown)?;
    set_event_to_map(&mut map, &bindings, "select_dialog", "up", UserEvent::SelectDialogUp)?;
    set_event_to_map(&mut map, &bindings, "select_dialog", "right", UserEvent::SelectDialogRight)?;
    set_event_to_map(&mut map, &bindings, "select_dialog", "left", UserEvent::SelectDialogLeft)?;
    set_event_to_map(&mut map, &bindings, "select_dialog", "close", UserEvent::SelectDialogClose)?;
    set_event_to_map(&mut map, &bindings, "select_dialog", "select", UserEvent::SelectDialogSelect)?;

    Ok(UserEventMapper { map })
}

fn set_event_to_map(
    map: &mut IndexMap<KeyEvent, Vec<UserEvent>>,
    bindings: &KeyMap,
    section: &str,
    event: &str,
    user_event: UserEvent,
) -> Result<(), String> {
    let keys = bindings
        .get(section)
        .and_then(|b| b.get(event))
        .ok_or_else(|| {
            format!("No keybindings found for section '{section}' and event '{event}'")
        })?;
    for key_event in parse_key_events(keys)? {
        map.entry(key_event).or_default().push(user_event);
    }
    Ok(())
}

type KeyMap = IndexMap<String, IndexMap<String, Vec<String>>>;

fn deserialize_and_merge_bindings(
    default_bindings_str: &str,
    custom_bindings_str: &str,
) -> Result<KeyMap, String> {
    let default_bindings: KeyMap = toml::from_str(default_bindings_str)
        .map_err(|e| format!("failed to parse default bindings: {}", e))?;
    let mut custom_bindings: KeyMap = toml::from_str(custom_bindings_str)
        .map_err(|e| format!("failed to parse custom bindings: {}", e))?;

    let mut bindings: KeyMap = IndexMap::new();
    for (section, mut default_section_bindings) in default_bindings {
        if let Some(custom_section_bindings) = custom_bindings.swap_remove(&section) {
            for (event, keys) in custom_section_bindings {
                default_section_bindings.insert(event, keys);
            }
        }
        bindings.insert(section, default_section_bindings);
    }

    Ok(bindings)
}

fn parse_key_events(raws: &[String]) -> Result<Vec<KeyEvent>, String> {
    raws.iter()
        .map(|raw| parse_key_event(raw))
        .collect::<Result<Vec<_>, String>>()
}

fn parse_key_event(raw: &str) -> Result<KeyEvent, String> {
    let raw_lower = raw.to_ascii_lowercase().replace(' ', "");
    let (remaining, modifiers) = extract_modifiers(&raw_lower);
    parse_key_code_with_modifiers(remaining, modifiers)
}

fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
    let mut modifiers = KeyModifiers::empty();
    let mut current = raw;
    loop {
        match current {
            rest if rest.starts_with("ctrl-") => {
                modifiers.insert(KeyModifiers::CONTROL);
                current = &rest[5..];
            }
            rest if rest.starts_with("alt-") => {
                modifiers.insert(KeyModifiers::ALT);
                current = &rest[4..];
            }
            rest if rest.starts_with("shift-") => {
                modifiers.insert(KeyModifiers::SHIFT);
                current = &rest[6..];
            }
            _ => break, // break out of the loop if no known prefix is detected
        };
    }
    (current, modifiers)
}

fn parse_key_code_with_modifiers(
    raw: &str,
    mut modifiers: KeyModifiers,
) -> Result<KeyEvent, String> {
    let c = match raw {
        "esc" => KeyCode::Esc,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "backtab" => {
            modifiers.insert(KeyModifiers::SHIFT);
            KeyCode::BackTab
        }
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        "space" => KeyCode::Char(' '),
        "hyphen" => KeyCode::Char('-'),
        "minus" => KeyCode::Char('-'),
        "tab" => KeyCode::Tab,
        c if c.len() == 1 => {
            let mut c = c.chars().next().unwrap();
            if modifiers.contains(KeyModifiers::SHIFT) {
                c = c.to_ascii_uppercase();
            }
            KeyCode::Char(c)
        }
        _ => return Err(format!("Unable to parse {raw}")),
    };
    Ok(KeyEvent::new(c, modifiers))
}

pub fn key_event_to_string(key: KeyEvent, short: bool) -> String {
    if let KeyCode::Char(c) = key.code {
        if key.modifiers == KeyModifiers::SHIFT {
            return c.to_ascii_uppercase().into();
        }
    }

    let char;
    let key_code = match key.code {
        KeyCode::Backspace => {
            if short {
                "BS"
            } else {
                "Backspace"
            }
        }
        KeyCode::Enter => "Enter",
        KeyCode::Left => "Left",
        KeyCode::Right => "Right",
        KeyCode::Up => "Up",
        KeyCode::Down => "Down",
        KeyCode::Home => "Home",
        KeyCode::End => "End",
        KeyCode::PageUp => "PageUp",
        KeyCode::PageDown => "PageDown",
        KeyCode::Tab => "Tab",
        KeyCode::BackTab => "BackTab",
        KeyCode::Delete => {
            if short {
                "Del"
            } else {
                "Delete"
            }
        }
        KeyCode::Insert => {
            if short {
                "Ins"
            } else {
                "Insert"
            }
        }
        KeyCode::F(n) => {
            char = format!("F{n}");
            &char
        }
        KeyCode::Char(' ') => "Space",
        KeyCode::Char(c) => {
            char = c.to_string();
            &char
        }
        KeyCode::Esc => "Esc",
        KeyCode::Null => "",
        KeyCode::CapsLock => "",
        KeyCode::Menu => "",
        KeyCode::ScrollLock => "",
        KeyCode::Media(_) => "",
        KeyCode::NumLock => "",
        KeyCode::PrintScreen => "",
        KeyCode::Pause => "",
        KeyCode::KeypadBegin => "",
        KeyCode::Modifier(_) => "",
    };

    let mut modifiers = Vec::with_capacity(3);

    if key.modifiers.intersects(KeyModifiers::CONTROL) {
        if short {
            modifiers.push("C");
        } else {
            modifiers.push("Ctrl");
        }
    }

    if key.modifiers.intersects(KeyModifiers::SHIFT) {
        if short {
            modifiers.push("S");
        } else {
            modifiers.push("Shift");
        }
    }

    if key.modifiers.intersects(KeyModifiers::ALT) {
        if short {
            modifiers.push("A");
        } else {
            modifiers.push("Alt");
        }
    }

    let mut key = modifiers.join("-");

    if !key.is_empty() {
        key.push('-');
    }
    key.push_str(key_code);

    key
}
