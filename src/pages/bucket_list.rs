use std::cmp::Ordering;

use laurier::{key_code, key_code_char};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::ListItem,
    Frame,
};

use crate::{
    color::ColorTheme,
    event::{AppEventType, Sender},
    object::{BucketItem, ObjectKey},
    pages::util::{build_helps, build_short_helps},
    util::split_str,
    widget::{
        BucketListSortDialog, BucketListSortDialogState, BucketListSortType, CopyDetailDialog,
        CopyDetailDialogState, InputDialog, InputDialogState, ScrollList, ScrollListState,
    },
};

#[derive(Debug)]
pub struct BucketListPage {
    bucket_items: Vec<BucketItem>,
    view_indices: Vec<usize>,

    view_state: ViewState,

    list_state: ScrollListState,
    filter_input_state: InputDialogState,
    sort_dialog_state: BucketListSortDialogState,

    theme: ColorTheme,
    tx: Sender,
}

#[derive(Debug)]
enum ViewState {
    Default,
    FilterDialog,
    SortDialog,
    CopyDetailDialog(Box<CopyDetailDialogState>),
}

impl BucketListPage {
    pub fn new(bucket_items: Vec<BucketItem>, theme: ColorTheme, tx: Sender) -> Self {
        let items_len = bucket_items.len();
        let view_indices = (0..items_len).collect();
        Self {
            bucket_items,
            view_indices,
            view_state: ViewState::Default,
            list_state: ScrollListState::new(items_len),
            filter_input_state: InputDialogState::default(),
            sort_dialog_state: BucketListSortDialogState::default(),
            theme,
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
                    self.tx.send(AppEventType::BucketListMoveDown);
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
                    self.tx.send(AppEventType::BucketListRefresh);
                }
                key_code_char!('x') if self.non_empty() => {
                    self.tx.send(AppEventType::BucketListOpenManagementConsole);
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
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let offset = self.list_state.offset;
        let selected = self.list_state.selected;

        let list_items = build_list_items(
            &self.bucket_items,
            &self.view_indices,
            self.filter_input_state.input(),
            &self.theme,
            offset,
            selected,
            area,
        );

        let list = ScrollList::new(list_items).theme(&self.theme);
        f.render_stateful_widget(list, area, &mut self.list_state);

        if let ViewState::FilterDialog = self.view_state {
            let filter_dialog = InputDialog::default()
                .title("Filter")
                .max_width(30)
                .theme(&self.theme);
            f.render_stateful_widget(filter_dialog, area, &mut self.filter_input_state);

            let (cursor_x, cursor_y) = self.filter_input_state.cursor();
            f.set_cursor_position((cursor_x, cursor_y));
        }

        if let ViewState::SortDialog = self.view_state {
            let sort_dialog = BucketListSortDialog::new(self.sort_dialog_state).theme(&self.theme);
            f.render_widget(sort_dialog, area);
        }

        if let ViewState::CopyDetailDialog(state) = &mut self.view_state {
            let copy_detail_dialog = CopyDetailDialog::default().theme(&self.theme);
            f.render_stateful_widget(copy_detail_dialog, area, state);
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
                        (&["Enter"], "Open bucket"),
                        (&["/"], "Filter bucket list"),
                        (&["o"], "Sort bucket list"),
                        (&["r"], "Open copy dialog"),
                        (&["R"], "Refresh bucket list"),
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
                        (&["Enter"], "Open bucket"),
                        (&["/"], "Filter bucket list"),
                        (&["o"], "Sort bucket list"),
                        (&["r"], "Open copy dialog"),
                        (&["R"], "Refresh bucket list"),
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
        };
        build_helps(helps)
    }

    pub fn short_helps(&self) -> Vec<(String, usize)> {
        let helps: &[(&[&str], &str, usize)] = match self.view_state {
            ViewState::Default => {
                if self.filter_input_state.input().is_empty() {
                    &[
                        (&["Esc"], "Quit", 0),
                        (&["j/k"], "Select", 1),
                        (&["g/G"], "Top/Bottom", 6),
                        (&["Enter"], "Open", 2),
                        (&["/"], "Filter", 3),
                        (&["o"], "Sort", 4),
                        (&["R"], "Refresh", 5),
                        (&["?"], "Help", 0),
                    ]
                } else {
                    &[
                        (&["Esc"], "Clear filter", 0),
                        (&["j/k"], "Select", 1),
                        (&["g/G"], "Top/Bottom", 6),
                        (&["Enter"], "Open", 2),
                        (&["/"], "Filter", 3),
                        (&["o"], "Sort", 4),
                        (&["R"], "Refresh", 5),
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
        };
        build_short_helps(helps)
    }
}

impl BucketListPage {
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
        self.view_state =
            ViewState::CopyDetailDialog(Box::new(CopyDetailDialogState::bucket_list(item.clone())));
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
            .bucket_items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.name.contains(filter))
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
        let items = &self.bucket_items;
        let selected = self.sort_dialog_state.selected();

        #[allow(clippy::type_complexity)]
        let sort_func: Box<dyn FnMut(&usize, &usize) -> Ordering> = match selected {
            BucketListSortType::Default => Box::new(|a, b| a.cmp(b)),
            BucketListSortType::NameAsc => Box::new(|a, b| items[*a].name.cmp(&items[*b].name)),
            BucketListSortType::NameDesc => Box::new(|a, b| items[*b].name.cmp(&items[*a].name)),
        };

        self.view_indices.sort_by(sort_func);
    }

    pub fn current_selected_item(&self) -> &BucketItem {
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
        self.bucket_items.get(*i).unwrap_or_else(|| {
            panic!(
                "selected index {} is out of range {}",
                i,
                self.bucket_items.len()
            )
        })
    }

    pub fn current_selected_object_key(&self) -> ObjectKey {
        let item = self.current_selected_item();
        ObjectKey {
            bucket_name: item.name.clone(),
            object_path: Vec::new(),
        }
    }

    fn non_empty(&self) -> bool {
        !self.view_indices.is_empty()
    }
}

fn build_list_items<'a>(
    current_items: &'a [BucketItem],
    view_indices: &'a [usize],
    filter: &'a str,
    theme: &'a ColorTheme,
    offset: usize,
    selected: usize,
    area: Rect,
) -> Vec<ListItem<'a>> {
    let show_item_count = (area.height as usize) - 2 /* border */;
    view_indices
        .iter()
        .map(|&original_idx| &current_items[original_idx])
        .skip(offset)
        .take(show_item_count)
        .enumerate()
        .map(|(idx, item)| {
            let selected = idx + offset == selected;
            build_list_item(&item.name, selected, filter, theme)
        })
        .collect()
}

fn build_list_item<'a>(
    name: &'a str,
    selected: bool,
    filter: &'a str,
    theme: &'a ColorTheme,
) -> ListItem<'a> {
    let line = if filter.is_empty() {
        Line::from(vec![" ".into(), name.into(), " ".into()])
    } else {
        let (before, highlighted, after) = split_str(name, filter).unwrap();
        Line::from(vec![
            " ".into(),
            before.into(),
            highlighted.fg(theme.list_filter_match),
            after.into(),
            " ".into(),
        ])
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

#[cfg(test)]
mod tests {
    use crate::{event, set_cells};

    use super::*;
    use ratatui::{backend::TestBackend, buffer::Buffer, style::Color, Terminal};

    #[test]
    fn test_render_without_scroll() -> std::io::Result<()> {
        let theme = ColorTheme::default();
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let items = ["bucket1", "bucket2", "bucket3"]
                .into_iter()
                .map(bucket_item)
                .collect();
            let mut page = BucketListPage::new(items, theme, tx);
            let area = Rect::new(0, 0, 30, 10);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────── 1 / 3 ┐",
            "│  bucket1                   │",
            "│  bucket2                   │",
            "│  bucket3                   │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "└────────────────────────────┘",
        ]);
        set_cells! { expected =>
            (2..28, [1]) => bg: Color::Cyan, fg: Color::Black,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn test_render_with_scroll() -> std::io::Result<()> {
        let theme = ColorTheme::default();
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let items = (0..16)
                .map(|i| bucket_item(&format!("bucket{}", i + 1)))
                .collect();
            let mut page = BucketListPage::new(items, theme, tx);
            let area = Rect::new(0, 0, 30, 10);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────  1 / 16 ┐",
            "│  bucket1                  ││",
            "│  bucket2                  ││",
            "│  bucket3                  ││",
            "│  bucket4                  ││",
            "│  bucket5                   │",
            "│  bucket6                   │",
            "│  bucket7                   │",
            "│  bucket8                   │",
            "└────────────────────────────┘",
        ]);
        set_cells! { expected =>
            // selected item
            (2..28, [1]) => bg: Color::Cyan, fg: Color::Black,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn test_render_filter_items() -> std::io::Result<()> {
        let theme = ColorTheme::default();
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        let items = ["foo", "bar", "baz", "qux", "foobar"]
            .into_iter()
            .map(bucket_item)
            .collect();
        let mut page = BucketListPage::new(items, theme, tx);
        let area = Rect::new(0, 0, 30, 10);

        page.handle_key(KeyEvent::from(KeyCode::Char('/')));
        page.handle_key(KeyEvent::from(KeyCode::Char('b')));

        terminal.draw(|f| {
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────── 1 / 3 ┐",
            "│  bar                       │",
            "│  baz                       │",
            "│ ╭Filter──────────────────╮ │",
            "│ │ b                      │ │",
            "│ ╰────────────────────────╯ │",
            "│                            │",
            "│                            │",
            "│                            │",
            "└────────────────────────────┘",
        ]);
        set_cells! { expected =>
            // selected item
            (2..28, [1]) => bg: Color::Cyan, fg: Color::Black,
            // match
            ([3], [1]) => fg: Color::Red,
            ([3], [2]) => fg: Color::Red,
        }

        terminal.backend().assert_buffer(&expected);

        page.handle_key(KeyEvent::from(KeyCode::Char('a')));
        page.handle_key(KeyEvent::from(KeyCode::Enter));

        terminal.draw(|f| {
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────── 1 / 3 ┐",
            "│  bar                       │",
            "│  baz                       │",
            "│  foobar                    │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "└────────────────────────────┘",
        ]);
        set_cells! { expected =>
            // selected item
            (2..28, [1]) => bg: Color::Cyan, fg: Color::Black,
            // match
            ([3, 4], [1]) => fg: Color::Red,
            ([3, 4], [2]) => fg: Color::Red,
            ([6, 7], [3]) => fg: Color::Red,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn test_render_sort_items() -> std::io::Result<()> {
        let theme = ColorTheme::default();
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        let items = ["foo", "bar", "baz", "qux", "foobar"]
            .into_iter()
            .map(bucket_item)
            .collect();
        let mut page = BucketListPage::new(items, theme, tx);
        let area = Rect::new(0, 0, 30, 10);

        page.handle_key(KeyEvent::from(KeyCode::Char('o')));
        page.handle_key(KeyEvent::from(KeyCode::Char('j')));
        page.handle_key(KeyEvent::from(KeyCode::Char('j')));

        terminal.draw(|f| {
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────── 1 / 5 ┐",
            "│  qux                       │",
            "│ ╭Sort────────────────────╮ │",
            "│ │ Default                │ │",
            "│ │ Name (Asc)             │ │",
            "│ │ Name (Desc)            │ │",
            "│ ╰────────────────────────╯ │",
            "│                            │",
            "│                            │",
            "└────────────────────────────┘",
        ]);
        set_cells! { expected =>
            // selected item
            (2..28, [1]) => bg: Color::Cyan, fg: Color::Black,
            // selected sort item
            (4..26, [5]) => fg: Color::Cyan,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn test_filter_items() {
        let theme = ColorTheme::default();
        let (tx, _) = event::new();

        let items = ["foo", "bar", "baz", "qux", "foobar"]
            .into_iter()
            .map(bucket_item)
            .collect();
        let mut page = BucketListPage::new(items, theme, tx);

        page.handle_key(KeyEvent::from(KeyCode::Char('/')));
        page.handle_key(KeyEvent::from(KeyCode::Char('b')));
        page.handle_key(KeyEvent::from(KeyCode::Char('a')));

        assert_eq!(page.view_indices, vec![1, 2, 4]);

        page.handle_key(KeyEvent::from(KeyCode::Char('r')));

        assert_eq!(page.view_indices, vec![1, 4]);

        page.handle_key(KeyEvent::from(KeyCode::Char('r')));

        assert!(page.view_indices.is_empty());

        page.handle_key(KeyEvent::from(KeyCode::Backspace));
        page.handle_key(KeyEvent::from(KeyCode::Backspace));

        assert_eq!(page.view_indices, vec![1, 2, 4]);

        page.handle_key(KeyEvent::from(KeyCode::Esc));

        assert_eq!(page.view_indices, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_sort_items() {
        let theme = ColorTheme::default();
        let (tx, _) = event::new();

        let items = ["foo", "bar", "baz", "qux", "foobar"]
            .into_iter()
            .map(bucket_item)
            .collect();
        let mut page = BucketListPage::new(items, theme, tx);

        page.handle_key(KeyEvent::from(KeyCode::Char('o')));

        page.handle_key(KeyEvent::from(KeyCode::Char('j'))); // select NameAsc

        assert_eq!(page.view_indices, vec![1, 2, 0, 4, 3]);

        page.handle_key(KeyEvent::from(KeyCode::Char('j'))); // select NameDesc
        page.handle_key(KeyEvent::from(KeyCode::Enter));

        assert_eq!(page.view_indices, vec![3, 4, 0, 2, 1]);

        page.handle_key(KeyEvent::from(KeyCode::Char('o')));
        page.handle_key(KeyEvent::from(KeyCode::Char('k'))); // select NameAsc

        assert_eq!(page.view_indices, vec![1, 2, 0, 4, 3]);

        page.handle_key(KeyEvent::from(KeyCode::Esc));

        assert_eq!(page.view_indices, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_filter_and_sort_items() {
        let theme = ColorTheme::default();
        let (tx, _) = event::new();

        let items = ["foo", "bar", "baz", "qux", "foobar"]
            .into_iter()
            .map(bucket_item)
            .collect();
        let mut page = BucketListPage::new(items, theme, tx);

        page.handle_key(KeyEvent::from(KeyCode::Char('/')));
        page.handle_key(KeyEvent::from(KeyCode::Char('b')));
        page.handle_key(KeyEvent::from(KeyCode::Char('a')));
        page.handle_key(KeyEvent::from(KeyCode::Enter));

        assert_eq!(page.view_indices, vec![1, 2, 4]);

        page.handle_key(KeyEvent::from(KeyCode::Char('o')));
        page.handle_key(KeyEvent::from(KeyCode::Char('j')));
        page.handle_key(KeyEvent::from(KeyCode::Char('j')));
        page.handle_key(KeyEvent::from(KeyCode::Enter));

        assert_eq!(page.view_indices, vec![4, 2, 1]);

        page.handle_key(KeyEvent::from(KeyCode::Esc));

        assert_eq!(page.view_indices, vec![3, 4, 0, 2, 1]);

        page.handle_key(KeyEvent::from(KeyCode::Char('/')));
        page.handle_key(KeyEvent::from(KeyCode::Char('f')));
        page.handle_key(KeyEvent::from(KeyCode::Char('o')));
        page.handle_key(KeyEvent::from(KeyCode::Enter));

        assert_eq!(page.view_indices, vec![4, 0]);

        page.handle_key(KeyEvent::from(KeyCode::Char('o')));
        page.handle_key(KeyEvent::from(KeyCode::Esc));

        assert_eq!(page.view_indices, vec![0, 4]);
    }

    fn setup_terminal() -> std::io::Result<Terminal<TestBackend>> {
        let backend = TestBackend::new(30, 10);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(terminal)
    }

    fn bucket_item(name: &str) -> BucketItem {
        BucketItem {
            name: name.to_string(),
            s3_uri: "".to_string(),
            arn: "".to_string(),
            object_url: "".to_string(),
        }
    }
}
