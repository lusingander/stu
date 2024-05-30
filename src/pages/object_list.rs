use chrono::{DateTime, Local};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::ListItem,
    Frame,
};

use crate::{
    event::{AppEventType, Sender},
    key_code, key_code_char,
    object::ObjectItem,
    pages::util::{build_helps, build_short_helps},
    ui::common::{format_datetime, format_size_byte},
    util::split_str,
    widget::{InputDialog, InputDialogState, ScrollList, ScrollListState},
};

const SELECTED_COLOR: Color = Color::Cyan;
const SELECTED_ITEM_TEXT_COLOR: Color = Color::Black;
const HIGHLIGHTED_ITEM_TEXT_COLOR: Color = Color::Red;

#[derive(Debug)]
pub struct ObjectListPage {
    object_items: Vec<ObjectItem>,
    filtered_indices: Vec<usize>,

    view_state: ViewState,

    list_state: ScrollListState,
    filter_input_state: InputDialogState,
    tx: Sender,
}

#[derive(Debug)]
enum ViewState {
    Default,
    FilterDialog,
}

impl ObjectListPage {
    pub fn new(object_items: Vec<ObjectItem>, tx: Sender) -> Self {
        let items_len = object_items.len();
        let filtered_indices = (0..items_len).collect();
        Self {
            object_items,
            filtered_indices,
            view_state: ViewState::Default,
            list_state: ScrollListState::new(items_len),
            filter_input_state: InputDialogState::default(),
            tx,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.view_state {
            ViewState::Default => match key {
                key_code!(KeyCode::Esc) => {
                    if self.filter_input_state.input().is_empty() {
                        self.tx.send(AppEventType::Quit);
                    } else {
                        self.reset_filter();
                    }
                }
                key_code!(KeyCode::Enter) if self.non_empty() => {
                    self.tx.send(AppEventType::ObjectListMoveDown);
                }
                key_code!(KeyCode::Backspace) => {
                    self.tx.send(AppEventType::ObjectListMoveUp);
                }
                key_code_char!('j') if self.non_empty() => {
                    self.select_next();
                }
                key_code_char!('k') if self.non_empty() => {
                    self.select_prev();
                }
                key_code_char!('g') if self.non_empty() => {
                    self.select_first();
                }
                key_code_char!('G') if self.non_empty() => {
                    self.select_last();
                }
                key_code_char!('f') if self.non_empty() => {
                    self.select_next_page();
                }
                key_code_char!('b') if self.non_empty() => {
                    self.select_prev_page();
                }
                key_code_char!('~') => {
                    self.tx.send(AppEventType::BackToBucketList);
                }
                key_code_char!('x') if self.non_empty() => {
                    self.tx.send(AppEventType::ObjectListOpenManagementConsole);
                }
                key_code_char!('/') => {
                    self.open_filter_dialog();
                }
                key_code_char!('?') => {
                    self.tx.send(AppEventType::OpenHelp);
                }
                _ => {}
            },
            ViewState::FilterDialog => match key {
                key_code!(KeyCode::Esc) => {
                    self.close_filter_dialog();
                }
                key_code!(KeyCode::Enter) => {
                    self.apply_filter();
                }
                key_code_char!('?') => {
                    self.tx.send(AppEventType::OpenHelp);
                }
                _ => {
                    self.filter_input_state.handle_key_event(key);
                    self.update_filtered_indices();
                }
            },
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let offset = self.list_state.offset;
        let selected = self.list_state.selected;

        let list_items = build_list_items(
            &self.object_items,
            &self.filtered_indices,
            self.filter_input_state.input(),
            offset,
            selected,
            area,
        );

        let list = ScrollList::new(list_items);
        f.render_stateful_widget(list, area, &mut self.list_state);

        if let ViewState::FilterDialog = self.view_state {
            let filter_dialog = InputDialog::default().title("Filter").max_width(30);
            f.render_stateful_widget(filter_dialog, area, &mut self.filter_input_state);

            let (cursor_x, cursor_y) = self.filter_input_state.cursor();
            f.set_cursor(cursor_x, cursor_y);
        }
    }

    pub fn helps(&self) -> Vec<String> {
        let helps: &[(&[&str], &str)] = match self.view_state {
            ViewState::Default => {
                if self.filter_input_state.input().is_empty() {
                    &[
                        (&["Esc", "Ctrl-c"], "Quit app"),
                        (&["j/k"], "Select item"),
                        (&["g/G"], "Go to top/bottom"),
                        (&["f"], "Scroll page forward"),
                        (&["b"], "Scroll page backward"),
                        (&["Enter"], "Open file or folder"),
                        (&["Backspace"], "Go back to prev folder"),
                        (&["~"], "Go back to bucket list"),
                        (&["/"], "Filter bucket list"),
                        (&["x"], "Open management console in browser"),
                    ]
                } else {
                    &[
                        (&["Ctrl-c"], "Quit app"),
                        (&["Esc"], "Clear filter"),
                        (&["j/k"], "Select item"),
                        (&["g/G"], "Go to top/bottom"),
                        (&["f"], "Scroll page forward"),
                        (&["b"], "Scroll page backward"),
                        (&["Enter"], "Open file or folder"),
                        (&["Backspace"], "Go back to prev folder"),
                        (&["~"], "Go back to bucket list"),
                        (&["/"], "Filter bucket list"),
                        (&["x"], "Open management console in browser"),
                    ]
                }
            }
            ViewState::FilterDialog => &[
                (&["Ctrl-c"], "Quit app"),
                (&["Esc"], "Close filter dialog"),
                (&["Enter"], "Apply filter"),
            ],
        };
        build_helps(helps)
    }

    pub fn short_helps(&self) -> Vec<(String, usize)> {
        let helps: &[(&[&str], &str, usize)] = match self.view_state {
            ViewState::Default => {
                if self.filter_input_state.input().is_empty() {
                    &[
                        (&["Esc"], "Quit", 0),
                        (&["j/k"], "Select", 3),
                        (&["g/G"], "Top/Bottom", 5),
                        (&["Enter"], "Open", 1),
                        (&["Backspace"], "Go back", 2),
                        (&["/"], "Filter", 4),
                        (&["?"], "Help", 0),
                    ]
                } else {
                    &[
                        (&["Esc"], "Clear filter", 0),
                        (&["j/k"], "Select", 3),
                        (&["g/G"], "Top/Bottom", 5),
                        (&["Enter"], "Open", 1),
                        (&["Backspace"], "Go back", 2),
                        (&["/"], "Filter", 4),
                        (&["?"], "Help", 0),
                    ]
                }
            }
            ViewState::FilterDialog => &[
                (&["Esc"], "Close", 2),
                (&["Enter"], "Filter", 1),
                (&["?"], "Help", 0),
            ],
        };
        build_short_helps(helps)
    }
}

impl ObjectListPage {
    fn select_next(&mut self) {
        self.list_state.select_next();
    }

    fn select_prev(&mut self) {
        self.list_state.select_prev();
    }

    fn select_first(&mut self) {
        self.list_state.select_first();
    }

    fn select_last(&mut self) {
        self.list_state.select_last();
    }

    fn select_next_page(&mut self) {
        self.list_state.select_next_page();
    }

    fn select_prev_page(&mut self) {
        self.list_state.select_prev_page();
    }

    fn open_filter_dialog(&mut self) {
        self.view_state = ViewState::FilterDialog;
    }

    fn close_filter_dialog(&mut self) {
        self.view_state = ViewState::Default;
        self.reset_filter();
    }

    fn apply_filter(&mut self) {
        self.view_state = ViewState::Default;

        self.update_filtered_indices();
    }

    fn reset_filter(&mut self) {
        self.filter_input_state.clear_input();

        self.update_filtered_indices();
    }

    fn update_filtered_indices(&mut self) {
        let filter = self.filter_input_state.input();
        self.filtered_indices = self
            .object_items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.name().contains(filter))
            .map(|(idx, _)| idx)
            .collect();
        // reset list state
        self.list_state = ScrollListState::new(self.filtered_indices.len());
    }

    pub fn current_selected_item(&self) -> &ObjectItem {
        let i = self
            .filtered_indices
            .get(self.list_state.selected)
            .unwrap_or_else(|| {
                panic!(
                    "selected filtered index {} is out of range {}",
                    self.list_state.selected,
                    self.filtered_indices.len()
                )
            });
        self.object_items.get(*i).unwrap_or_else(|| {
            panic!(
                "selected index {} is out of range {}",
                i,
                self.object_items.len()
            )
        })
    }

    pub fn object_list(&self) -> Vec<ObjectItem> {
        self.object_items
            .iter()
            .enumerate()
            .filter(|(i, _)| self.filtered_indices.contains(i))
            .map(|(_, item)| item)
            .cloned()
            .collect()
    }

    pub fn list_state(&self) -> ScrollListState {
        self.list_state
    }

    fn non_empty(&self) -> bool {
        !self.filtered_indices.is_empty()
    }
}

fn build_list_items<'a>(
    current_items: &'a [ObjectItem],
    filter_indices: &'a [usize],
    filter: &'a str,
    offset: usize,
    selected: usize,
    area: Rect,
) -> Vec<ListItem<'a>> {
    let show_item_count = (area.height as usize) - 2 /* border */;
    current_items
        .iter()
        .enumerate()
        .filter(|(original_idx, _)| filter_indices.contains(original_idx))
        .skip(offset)
        .take(show_item_count)
        .enumerate()
        .map(|(idx, (_, item))| build_list_item(item, idx + offset == selected, filter, area))
        .collect()
}

fn build_list_item<'a>(
    item: &'a ObjectItem,
    selected: bool,
    filter: &'a str,
    area: Rect,
) -> ListItem<'a> {
    let line = match item {
        ObjectItem::Dir { name, .. } => build_object_dir_line(name, filter),
        ObjectItem::File {
            name,
            size_byte,
            last_modified,
            ..
        } => build_object_file_line(name, *size_byte, last_modified, filter, area.width),
    };

    let style = if selected {
        Style::default()
            .bg(SELECTED_COLOR)
            .fg(SELECTED_ITEM_TEXT_COLOR)
    } else {
        Style::default()
    };
    ListItem::new(line).style(style)
}

fn build_object_dir_line<'a>(name: &'a str, filter: &'a str) -> Line<'a> {
    if filter.is_empty() {
        Line::from(vec![" ".into(), name.bold(), "/".bold(), " ".into()])
    } else {
        let (before, highlighted, after) = split_str(name, filter).unwrap();
        Line::from(vec![
            " ".into(),
            before.bold(),
            highlighted.fg(HIGHLIGHTED_ITEM_TEXT_COLOR).bold(),
            after.bold(),
            "/".bold(),
            " ".into(),
        ])
    }
}

fn build_object_file_line<'a>(
    name: &'a str,
    size_byte: usize,
    last_modified: &'a DateTime<Local>,
    filter: &'a str,
    width: u16,
) -> Line<'a> {
    let size = format_size_byte(size_byte);
    let date = format_datetime(last_modified);
    let date_w: usize = 19;
    let size_w: usize = 10;
    let name_w: usize = (width as usize) - date_w - size_w - 10 /* spaces */ - 4 /* border + pad */;

    let name = format!("{:<name_w$}", name, name_w = name_w);
    let date = format!("{:<date_w$}", date, date_w = date_w);
    let size = format!("{:>size_w$}", size, size_w = size_w);

    if filter.is_empty() {
        Line::from(vec![
            " ".into(),
            name.into(),
            "    ".into(),
            date.into(),
            "    ".into(),
            size.into(),
            " ".into(),
        ])
    } else {
        let (before, highlighted, after) = split_str(&name, filter).unwrap();
        Line::from(vec![
            " ".into(),
            before.into(),
            highlighted.fg(HIGHLIGHTED_ITEM_TEXT_COLOR),
            after.into(),
            "    ".into(),
            date.into(),
            "    ".into(),
            size.into(),
            " ".into(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use crate::{event, set_cells};

    use super::*;
    use chrono::NaiveDateTime;
    use ratatui::{backend::TestBackend, buffer::Buffer, style::Modifier, Terminal};

    #[test]
    fn test_render_without_scroll() -> std::io::Result<()> {
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let items = vec![
                ObjectItem::Dir {
                    name: "dir1".to_string(),
                },
                ObjectItem::Dir {
                    name: "dir2".to_string(),
                },
                ObjectItem::File {
                    name: "file1".to_string(),
                    size_byte: 1024 + 10,
                    last_modified: parse_datetime("2024-01-02 13:01:02"),
                },
                ObjectItem::File {
                    name: "file2".to_string(),
                    size_byte: 1024 * 999,
                    last_modified: parse_datetime("2023-12-31 09:00:00"),
                },
            ];
            let mut page = ObjectListPage::new(items, tx);
            let area = Rect::new(0, 0, 60, 10);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌─────────────────────────────────────────────────── 1 / 4 ┐",
            "│  dir1/                                                   │",
            "│  dir2/                                                   │",
            "│  file1                2024-01-02 13:01:02      1.01 KiB  │",
            "│  file2                2023-12-31 09:00:00       999 KiB  │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "└──────────────────────────────────────────────────────────┘",
        ]);
        set_cells! { expected =>
            // dir items
            (3..8, [1, 2]) => modifier: Modifier::BOLD,
            // selected item
            (2..58, [1]) => bg: Color::Cyan, fg: Color::Black,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn test_render_with_scroll() -> std::io::Result<()> {
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let items = (0..32)
                .map(|i| ObjectItem::File {
                    name: format!("file{}", i + 1),
                    size_byte: 1024,
                    last_modified: parse_datetime("2024-01-02 13:01:02"),
                })
                .collect();
            let mut page = ObjectListPage::new(items, tx);
            let area = Rect::new(0, 0, 60, 10);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
                "┌─────────────────────────────────────────────────  1 / 32 ┐",
                "│  file1                2024-01-02 13:01:02         1 KiB ││",
                "│  file2                2024-01-02 13:01:02         1 KiB ││",
                "│  file3                2024-01-02 13:01:02         1 KiB  │",
                "│  file4                2024-01-02 13:01:02         1 KiB  │",
                "│  file5                2024-01-02 13:01:02         1 KiB  │",
                "│  file6                2024-01-02 13:01:02         1 KiB  │",
                "│  file7                2024-01-02 13:01:02         1 KiB  │",
                "│  file8                2024-01-02 13:01:02         1 KiB  │",
                "└──────────────────────────────────────────────────────────┘",
        ]);
        set_cells! { expected =>
            // selected item
            (2..58, [1]) => bg: Color::Cyan, fg: Color::Black,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    fn setup_terminal() -> std::io::Result<Terminal<TestBackend>> {
        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(terminal)
    }

    fn parse_datetime(s: &str) -> DateTime<Local> {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
    }
}
