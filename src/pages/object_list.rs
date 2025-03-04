use std::rc::Rc;

use chrono::{DateTime, Local};
use laurier::{highlight::highlight_matched_text, key_code, key_code_char};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::ListItem,
    Frame,
};

use crate::{
    app::AppContext,
    color::ColorTheme,
    config::UiConfig,
    event::{AppEventType, Sender},
    format::{format_datetime, format_size_byte},
    object::{DownloadObjectInfo, ObjectItem, ObjectKey},
    pages::util::{build_helps, build_short_helps},
    widget::{
        ConfirmDialog, ConfirmDialogState, CopyDetailDialog, CopyDetailDialogState, InputDialog,
        InputDialogState, ObjectListSortDialog, ObjectListSortDialogState, ObjectListSortType,
        ScrollList, ScrollListState,
    },
};

const ELLIPSIS: &str = "...";

#[derive(Debug)]
pub struct ObjectListPage {
    object_items: Vec<ObjectItem>,
    object_key: ObjectKey,
    view_indices: Vec<usize>,

    view_state: ViewState,

    list_state: ScrollListState,
    filter_input_state: InputDialogState,
    sort_dialog_state: ObjectListSortDialogState,

    ctx: Rc<AppContext>,
    tx: Sender,
}

#[derive(Debug)]
enum ViewState {
    Default,
    FilterDialog,
    SortDialog,
    CopyDetailDialog(Box<CopyDetailDialogState>),
    DownloadConfirmDialog(Vec<DownloadObjectInfo>, ConfirmDialogState),
}

impl ObjectListPage {
    pub fn new(
        object_items: Vec<ObjectItem>,
        object_key: ObjectKey,
        ctx: Rc<AppContext>,
        tx: Sender,
    ) -> Self {
        let items_len = object_items.len();
        let view_indices = (0..items_len).collect();
        Self {
            object_items,
            object_key,
            view_indices,
            view_state: ViewState::Default,
            list_state: ScrollListState::new(items_len),
            filter_input_state: InputDialogState::default(),
            sort_dialog_state: ObjectListSortDialogState::default(),
            ctx,
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
                key_code_char!('R') if self.non_empty() => {
                    self.tx.send(AppEventType::ObjectListRefresh);
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
                key_code_char!('o') => {
                    self.open_sort_dialog();
                }
                key_code_char!('r') => {
                    self.open_copy_detail_dialog();
                }
                key_code_char!('s') if self.non_empty() => {
                    self.tx.send(AppEventType::ObjectListDownloadObject);
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
                    self.filter_view_indices();
                }
            },
            ViewState::SortDialog => match key {
                key_code!(KeyCode::Esc) => {
                    self.close_sort_dialog();
                }
                key_code_char!('j') => {
                    self.select_next_sort_item();
                }
                key_code_char!('k') => {
                    self.select_prev_sort_item();
                }
                key_code!(KeyCode::Enter) => {
                    self.apply_sort();
                }
                key_code_char!('?') => {
                    self.tx.send(AppEventType::OpenHelp);
                }
                _ => {}
            },
            ViewState::CopyDetailDialog(ref mut state) => match key {
                key_code!(KeyCode::Esc) | key_code!(KeyCode::Backspace) => {
                    self.close_copy_detail_dialog();
                }
                key_code!(KeyCode::Enter) => {
                    let (name, value) = state.selected_name_and_value();
                    self.tx.send(AppEventType::CopyToClipboard(name, value));
                }
                key_code_char!('j') => {
                    state.select_next();
                }
                key_code_char!('k') => {
                    state.select_prev();
                }
                key_code_char!('?') => {
                    self.tx.send(AppEventType::OpenHelp);
                }
                _ => {}
            },
            ViewState::DownloadConfirmDialog(_, ref mut state) => match key {
                key_code!(KeyCode::Esc) | key_code!(KeyCode::Backspace) => {
                    self.close_download_confirm_dialog();
                }
                key_code_char!('h') | key_code_char!('l') => {
                    state.toggle();
                }
                key_code!(KeyCode::Enter) => {
                    self.download();
                }
                key_code_char!('?') => {
                    self.tx.send(AppEventType::OpenHelp);
                }
                _ => {}
            },
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let offset = self.list_state.offset;
        let selected = self.list_state.selected;

        let list_items = build_list_items(
            &self.object_items,
            &self.view_indices,
            self.filter_input_state.input(),
            offset,
            selected,
            area,
            &self.ctx.config.ui,
            &self.ctx.theme,
        );

        let list = ScrollList::new(list_items).theme(&self.ctx.theme);
        f.render_stateful_widget(list, area, &mut self.list_state);

        if let ViewState::FilterDialog = self.view_state {
            let filter_dialog = InputDialog::default()
                .title("Filter")
                .max_width(30)
                .theme(&self.ctx.theme);
            f.render_stateful_widget(filter_dialog, area, &mut self.filter_input_state);

            let (cursor_x, cursor_y) = self.filter_input_state.cursor();
            f.set_cursor_position((cursor_x, cursor_y));
        }

        if let ViewState::SortDialog = self.view_state {
            let sort_dialog =
                ObjectListSortDialog::new(self.sort_dialog_state).theme(&self.ctx.theme);
            f.render_widget(sort_dialog, area);
        }

        if let ViewState::CopyDetailDialog(state) = &mut self.view_state {
            let copy_detail_dialog = CopyDetailDialog::default().theme(&self.ctx.theme);
            f.render_stateful_widget(copy_detail_dialog, area, state);
        }

        if let ViewState::DownloadConfirmDialog(objs, state) = &mut self.view_state {
            let message_lines = build_download_confirm_message_lines(objs, &self.ctx.theme);
            let download_confirm_dialog = ConfirmDialog::new(message_lines).theme(&self.ctx.theme);
            f.render_stateful_widget(download_confirm_dialog, area, state);
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
                        (&["/"], "Filter object list"),
                        (&["o"], "Sort object list"),
                        (&["r"], "Open copy dialog"),
                        (&["R"], "Refresh object list"),
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
                        (&["/"], "Filter object list"),
                        (&["o"], "Sort object list"),
                        (&["r"], "Open copy dialog"),
                        (&["R"], "Refresh object list"),
                        (&["x"], "Open management console in browser"),
                    ]
                }
            }
            ViewState::FilterDialog => &[
                (&["Ctrl-c"], "Quit app"),
                (&["Esc"], "Close filter dialog"),
                (&["Enter"], "Apply filter"),
            ],
            ViewState::SortDialog => &[
                (&["Ctrl-c"], "Quit app"),
                (&["Esc"], "Close sort dialog"),
                (&["j/k"], "Select item"),
                (&["Enter"], "Apply sort"),
            ],
            ViewState::CopyDetailDialog(_) => &[
                (&["Ctrl-c"], "Quit app"),
                (&["Esc", "Backspace"], "Close copy dialog"),
                (&["j/k"], "Select item"),
                (&["Enter"], "Copy selected value to clipboard"),
            ],
            ViewState::DownloadConfirmDialog(_, _) => &[
                (&["Ctrl-c"], "Quit app"),
                (&["Esc", "Backspace"], "Close confirm dialog"),
                (&["h/l"], "Select"),
                (&["Enter"], "Confirm"),
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
                        (&["g/G"], "Top/Bottom", 7),
                        (&["Enter"], "Open", 1),
                        (&["Backspace"], "Go back", 2),
                        (&["/"], "Filter", 4),
                        (&["o"], "Sort", 5),
                        (&["R"], "Refresh", 6),
                        (&["?"], "Help", 0),
                    ]
                } else {
                    &[
                        (&["Esc"], "Clear filter", 0),
                        (&["j/k"], "Select", 3),
                        (&["g/G"], "Top/Bottom", 7),
                        (&["Enter"], "Open", 1),
                        (&["Backspace"], "Go back", 2),
                        (&["/"], "Filter", 4),
                        (&["o"], "Sort", 5),
                        (&["R"], "Refresh", 6),
                        (&["?"], "Help", 0),
                    ]
                }
            }
            ViewState::FilterDialog => &[
                (&["Esc"], "Close", 2),
                (&["Enter"], "Filter", 1),
                (&["?"], "Help", 0),
            ],
            ViewState::SortDialog => &[
                (&["Esc"], "Close", 2),
                (&["j/k"], "Select", 3),
                (&["Enter"], "Sort", 1),
                (&["?"], "Help", 0),
            ],
            ViewState::CopyDetailDialog(_) => &[
                (&["Esc"], "Close", 2),
                (&["j/k"], "Select", 3),
                (&["Enter"], "Copy", 1),
                (&["?"], "Help", 0),
            ],
            ViewState::DownloadConfirmDialog(_, _) => &[
                (&["Esc"], "Close", 2),
                (&["h/l"], "Select", 3),
                (&["Enter"], "Confirm", 1),
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

    fn open_sort_dialog(&mut self) {
        self.view_state = ViewState::SortDialog;
    }

    fn close_sort_dialog(&mut self) {
        self.view_state = ViewState::Default;
        self.sort_dialog_state.reset();

        self.sort_view_indices();
    }

    fn open_copy_detail_dialog(&mut self) {
        let item = self.current_selected_item();
        let dialog_state = match item {
            ObjectItem::Dir { .. } => CopyDetailDialogState::object_list_dir(item.clone()),
            ObjectItem::File { .. } => CopyDetailDialogState::object_list_file(item.clone()),
        };
        self.view_state = ViewState::CopyDetailDialog(Box::new(dialog_state));
    }

    fn close_copy_detail_dialog(&mut self) {
        self.view_state = ViewState::Default;
    }

    fn apply_filter(&mut self) {
        self.view_state = ViewState::Default;

        self.filter_view_indices();
    }

    fn reset_filter(&mut self) {
        self.filter_input_state.clear_input();

        self.filter_view_indices();
    }

    fn filter_view_indices(&mut self) {
        let filter = self.filter_input_state.input();
        self.view_indices = self
            .object_items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.name().contains(filter))
            .map(|(idx, _)| idx)
            .collect();
        // reset list state
        self.list_state = ScrollListState::new(self.view_indices.len());

        self.sort_view_indices();
    }

    fn apply_sort(&mut self) {
        self.view_state = ViewState::Default;

        self.sort_view_indices();
    }

    fn select_next_sort_item(&mut self) {
        self.sort_dialog_state.select_next();

        self.sort_view_indices();
    }

    fn select_prev_sort_item(&mut self) {
        self.sort_dialog_state.select_prev();

        self.sort_view_indices();
    }

    fn sort_view_indices(&mut self) {
        let items = &self.object_items;
        let selected = self.sort_dialog_state.selected();

        match selected {
            ObjectListSortType::Default => {
                self.view_indices.sort();
            }
            ObjectListSortType::NameAsc => {
                self.view_indices
                    .sort_by(|a, b| items[*a].name().cmp(items[*b].name()));
            }
            ObjectListSortType::NameDesc => {
                self.view_indices
                    .sort_by(|a, b| items[*b].name().cmp(items[*a].name()));
            }
            ObjectListSortType::LastModifiedAsc => {
                self.view_indices
                    .sort_by(|a, b| items[*a].last_modified().cmp(&items[*b].last_modified()));
            }
            ObjectListSortType::LastModifiedDesc => {
                self.view_indices
                    .sort_by(|a, b| items[*b].last_modified().cmp(&items[*a].last_modified()));
            }
            ObjectListSortType::SizeAsc => {
                self.view_indices
                    .sort_by(|a, b| items[*a].size_byte().cmp(&items[*b].size_byte()));
            }
            ObjectListSortType::SizeDesc => {
                self.view_indices
                    .sort_by(|a, b| items[*b].size_byte().cmp(&items[*a].size_byte()));
            }
        }
    }

    pub fn open_download_confirm_dialog(&mut self, objs: Vec<DownloadObjectInfo>) {
        let dialog_state = ConfirmDialogState::default();
        self.view_state = ViewState::DownloadConfirmDialog(objs, dialog_state);
    }

    fn close_download_confirm_dialog(&mut self) {
        self.view_state = ViewState::Default;
    }

    fn download(&mut self) {
        if let ViewState::DownloadConfirmDialog(objs, state) = &mut self.view_state {
            if state.is_ok() {
                let objs = std::mem::take(objs);
                let bucket = self.object_key.bucket_name.clone();
                let key = self.current_dir_object_key().clone();
                let dir = self.current_selected_item().name().to_string();
                self.tx
                    .send(AppEventType::DownloadObjects(bucket, key, dir, objs));
            }
            self.close_download_confirm_dialog();
        }
    }

    pub fn current_selected_item(&self) -> &ObjectItem {
        let i = self
            .view_indices
            .get(self.list_state.selected)
            .unwrap_or_else(|| {
                panic!(
                    "selected view index {} is out of range {}",
                    self.list_state.selected,
                    self.view_indices.len()
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

    pub fn current_dir_object_key(&self) -> &ObjectKey {
        // not include current selected item
        &self.object_key
    }

    pub fn current_selected_object_key(&self) -> ObjectKey {
        let item = self.current_selected_item();
        let mut object_path = self.object_key.object_path.clone();
        object_path.push(item.name().to_string());
        ObjectKey {
            bucket_name: self.object_key.bucket_name.clone(),
            object_path,
        }
    }

    pub fn object_list(&self) -> Vec<ObjectItem> {
        self.view_indices
            .iter()
            .map(|&original_idx| &self.object_items[original_idx])
            .cloned()
            .collect()
    }

    pub fn list_state(&self) -> ScrollListState {
        self.list_state
    }

    fn non_empty(&self) -> bool {
        !self.view_indices.is_empty()
    }
}

fn build_list_items<'a>(
    current_items: &'a [ObjectItem],
    view_indices: &'a [usize],
    filter: &'a str,
    offset: usize,
    selected: usize,
    area: Rect,
    ui_config: &UiConfig,
    theme: &ColorTheme,
) -> Vec<ListItem<'a>> {
    let show_item_count = (area.height as usize) - 2 /* border */;
    view_indices
        .iter()
        .map(|&original_idx| &current_items[original_idx])
        .skip(offset)
        .take(show_item_count)
        .enumerate()
        .map(|(idx, item)| {
            build_list_item(
                item,
                idx + offset == selected,
                filter,
                area,
                ui_config,
                theme,
            )
        })
        .collect()
}

fn build_list_item<'a>(
    item: &'a ObjectItem,
    selected: bool,
    filter: &'a str,
    area: Rect,
    ui_config: &UiConfig,
    theme: &ColorTheme,
) -> ListItem<'a> {
    let line = match item {
        ObjectItem::Dir { name, .. } => build_object_dir_line(name, filter, area.width, theme),
        ObjectItem::File {
            name,
            size_byte,
            last_modified,
            ..
        } => build_object_file_line(
            name,
            *size_byte,
            last_modified,
            filter,
            area.width,
            ui_config,
            theme,
        ),
    };

    let style = if selected {
        Style::default()
            .bg(theme.list_selected_bg)
            .fg(theme.list_selected_fg)
    } else {
        Style::default()
    };
    ListItem::new(line).style(style)
}

fn build_object_dir_line<'a>(
    name: &'a str,
    filter: &'a str,
    width: u16,
    theme: &ColorTheme,
) -> Line<'a> {
    let name = format!("{}/", name);
    let name_w = (width as usize) - 2 /* spaces */ - 4 /* border + pad */ - 1 /* slash */;
    let pad_name =
        console::pad_str(&name, name_w, console::Alignment::Left, Some(ELLIPSIS)).to_string();

    if filter.is_empty() {
        Line::from(vec![" ".into(), pad_name.bold(), " ".into()])
    } else {
        let i = name.find(filter).unwrap();
        let mut spans = highlight_matched_text(pad_name)
            .ellipsis(ELLIPSIS)
            .matched_range(i, i + filter.len())
            .not_matched_style(Style::default().bold())
            .matched_style(Style::default().fg(theme.list_filter_match).bold())
            .into_spans();
        spans.insert(0, " ".into());
        spans.push(" ".into());
        Line::from(spans)
    }
}

fn build_object_file_line<'a>(
    name: &'a str,
    size_byte: usize,
    last_modified: &'a DateTime<Local>,
    filter: &'a str,
    width: u16,
    ui_config: &UiConfig,
    theme: &ColorTheme,
) -> Line<'a> {
    let size = format_size_byte(size_byte);
    let date = format_datetime(last_modified, &ui_config.object_list.date_format);
    let date_w: usize = ui_config.object_list.date_width;
    let size_w: usize = 10;
    let name_w: usize = (width as usize) - date_w - size_w - 10 /* spaces */ - 4 /* border + pad */;

    let pad_name =
        console::pad_str(name, name_w, console::Alignment::Left, Some(ELLIPSIS)).to_string();
    let pad_date = console::pad_str(&date, date_w, console::Alignment::Left, None).to_string();
    let pad_size = console::pad_str(&size, size_w, console::Alignment::Right, None).to_string();

    if filter.is_empty() {
        Line::from(vec![
            " ".into(),
            pad_name.into(),
            "    ".into(),
            pad_date.into(),
            "    ".into(),
            pad_size.into(),
            " ".into(),
        ])
    } else {
        let i = name.find(filter).unwrap();
        let mut spans = highlight_matched_text(pad_name)
            .ellipsis(ELLIPSIS)
            .matched_range(i, i + filter.len())
            .not_matched_style(Style::default())
            .matched_style(Style::default().fg(theme.list_filter_match))
            .into_spans();
        spans.insert(0, " ".into());
        spans.push("    ".into());
        spans.push(pad_date.into());
        spans.push("    ".into());
        spans.push(pad_size.into());
        spans.push(" ".into());
        Line::from(spans)
    }
}

fn build_download_confirm_message_lines<'a>(
    objs: &[DownloadObjectInfo],
    theme: &ColorTheme,
) -> Vec<Line<'a>> {
    let total_size = format_size_byte(objs.iter().map(|obj| obj.size_byte).sum());
    let total_count = objs.len();
    let size_message = format!("{} objects (Total size: {})", total_count, total_size);

    vec![
        Line::from("You are about to download the following files:".fg(theme.fg)),
        Line::from(""),
        Line::from(size_message.fg(theme.fg).bold()),
        Line::from(""),
        Line::from("This operation may take some time. Do you want to proceed?".fg(theme.fg)),
    ]
}

#[cfg(test)]
mod tests {
    use crate::{event, set_cells};

    use super::*;
    use chrono::NaiveDateTime;
    use ratatui::{
        backend::TestBackend,
        buffer::Buffer,
        style::{Color, Modifier},
        Terminal,
    };

    #[test]
    fn test_render_without_scroll() -> std::io::Result<()> {
        let ctx = Rc::default();
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let items = vec![
                object_dir_item("dir1"),
                object_dir_item("dir2"),
                object_file_item("file1", 1024 + 10, "2024-01-02 13:01:02"),
                object_file_item("file2", 1024 * 999, "2023-12-31 09:00:00"),
            ];
            let object_key = ObjectKey {
                bucket_name: "test-bucket".to_string(),
                object_path: vec!["path".to_string(), "to".to_string()],
            };
            let mut page = ObjectListPage::new(items, object_key, ctx, tx);
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
            (3..56, [1, 2]) => modifier: Modifier::BOLD,
            // selected item
            (2..58, [1]) => bg: Color::Cyan, fg: Color::Black,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn test_render_with_scroll() -> std::io::Result<()> {
        let ctx = Rc::default();
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let items = (0..32)
                .map(|i| object_file_item(&format!("file{}", i + 1), 1024, "2024-01-02 13:01:02"))
                .collect();
            let object_key = ObjectKey {
                bucket_name: "test-bucket".to_string(),
                object_path: vec!["path".to_string(), "to".to_string()],
            };
            let mut page = ObjectListPage::new(items, object_key, ctx, tx);
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

    #[test]
    fn test_render_with_config() -> std::io::Result<()> {
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let items = vec![
                object_dir_item("dir1"),
                object_dir_item("dir2"),
                object_file_item("file1", 1024 + 10, "2024-01-02 13:01:02"),
                object_file_item("file2", 1024 * 999, "2023-12-31 09:00:00"),
            ];
            let object_key = ObjectKey {
                bucket_name: "test-bucket".to_string(),
                object_path: vec!["path".to_string(), "to".to_string()],
            };
            let mut ctx = AppContext::default();
            ctx.config.ui.object_list.date_format = "%Y/%m/%d".to_string();
            ctx.config.ui.object_list.date_width = 10;
            let mut page = ObjectListPage::new(items, object_key, Rc::new(ctx), tx);
            let area = Rect::new(0, 0, 60, 10);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌─────────────────────────────────────────────────── 1 / 4 ┐",
            "│  dir1/                                                   │",
            "│  dir2/                                                   │",
            "│  file1                         2024/01/02      1.01 KiB  │",
            "│  file2                         2023/12/31       999 KiB  │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "└──────────────────────────────────────────────────────────┘",
        ]);
        set_cells! { expected =>
            // dir items
            (3..56, [1, 2]) => modifier: Modifier::BOLD,
            // selected item
            (2..58, [1]) => bg: Color::Cyan, fg: Color::Black,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn test_sort_items() {
        let ctx = Rc::default();
        let (tx, _) = event::new();
        let items = vec![
            object_dir_item("rid"),
            object_file_item("file", 1024, "2024-01-02 13:01:02"),
            object_dir_item("dir"),
            object_file_item("xyz", 1024 * 1024, "2023-12-31 23:59:59"),
            object_file_item("abc", 0, "-2000-01-01 00:00:00"),
        ];
        let object_key = ObjectKey {
            bucket_name: "test-bucket".to_string(),
            object_path: vec!["path".to_string(), "to".to_string()],
        };
        let mut page = ObjectListPage::new(items, object_key, ctx, tx);

        page.handle_key(KeyEvent::from(KeyCode::Char('o')));
        page.handle_key(KeyEvent::from(KeyCode::Char('j'))); // select NameAsc

        assert_eq!(page.view_indices, vec![4, 2, 1, 0, 3]);

        page.handle_key(KeyEvent::from(KeyCode::Char('j'))); // select NameDesc

        assert_eq!(page.view_indices, vec![3, 0, 1, 2, 4]);

        page.handle_key(KeyEvent::from(KeyCode::Char('j'))); // select LastModifiedAsc

        assert_eq!(page.view_indices, vec![0, 2, 4, 3, 1]);

        page.handle_key(KeyEvent::from(KeyCode::Char('j'))); // select LastModifiedDesc

        assert_eq!(page.view_indices, vec![1, 3, 4, 0, 2]);

        page.handle_key(KeyEvent::from(KeyCode::Char('j'))); // select SizeAsc

        assert_eq!(page.view_indices, vec![0, 2, 4, 1, 3]);

        page.handle_key(KeyEvent::from(KeyCode::Char('j'))); // select SizeDesc

        assert_eq!(page.view_indices, vec![3, 1, 4, 0, 2]);
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

    fn object_dir_item(name: &str) -> ObjectItem {
        ObjectItem::Dir {
            name: name.to_string(),
            key: "".to_string(),
            s3_uri: "".to_string(),
            object_url: "".to_string(),
        }
    }

    fn object_file_item(name: &str, size_byte: usize, last_modified: &str) -> ObjectItem {
        ObjectItem::File {
            name: name.to_string(),
            size_byte,
            last_modified: parse_datetime(last_modified),
            key: "".to_string(),
            s3_uri: "".to_string(),
            arn: "".to_string(),
            object_url: "".to_string(),
            e_tag: "".to_string(),
        }
    }
}
