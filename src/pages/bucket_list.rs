use std::rc::Rc;

use laurier::highlight::highlight_matched_text;
use ratatui::{
    crossterm::event::KeyEvent, layout::Rect, style::Style, text::Line, widgets::ListItem, Frame,
};

use crate::{
    app::AppContext,
    color::ColorTheme,
    event::{AppEventType, Sender},
    handle_user_events, handle_user_events_with_default,
    help::{build_short_help_spans, BuildShortHelpsItem, SpansWithPriority},
    keys::{UserEvent, UserEventMapper},
    object::{BucketItem, ObjectKey},
    pages::util::build_helps,
    widget::{
        BucketListSortDialog, BucketListSortDialogState, BucketListSortType, CopyDetailDialog,
        CopyDetailDialogState, InputDialog, InputDialogState, ScrollList, ScrollListState,
    },
};

const ELLIPSIS: &str = "...";

#[derive(Debug)]
pub struct BucketListPage {
    bucket_items: Vec<BucketItem>,
    view_indices: Vec<usize>,

    view_state: ViewState,

    list_state: ScrollListState,
    filter_input_state: InputDialogState,
    sort_dialog_state: BucketListSortDialogState,

    ctx: Rc<AppContext>,
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
    pub fn new(bucket_items: Vec<BucketItem>, ctx: Rc<AppContext>, tx: Sender) -> Self {
        let items_len = bucket_items.len();
        let view_indices = (0..items_len).collect();
        Self {
            bucket_items,
            view_indices,
            view_state: ViewState::Default,
            list_state: ScrollListState::new(items_len),
            filter_input_state: InputDialogState::default(),
            sort_dialog_state: BucketListSortDialogState::default(),
            ctx,
            tx,
        }
    }

    pub fn handle_key(&mut self, user_events: Vec<UserEvent>, key_event: KeyEvent) {
        match self.view_state {
            ViewState::Default => {
                handle_user_events! { user_events =>
                    UserEvent::BucketListSelect => {
                        if self.non_empty() {
                            self.tx.send(AppEventType::BucketListMoveDown);
                        }
                    }
                    UserEvent::BucketListDown => {
                        if self.non_empty() {
                            self.select_next();
                        }
                    }
                    UserEvent::BucketListUp => {
                        if self.non_empty() {
                            self.select_prev();
                        }
                    }
                    UserEvent::BucketListGoToTop => {
                        if self.non_empty() {
                            self.select_first();
                        }
                    }
                    UserEvent::BucketListGoToBottom => {
                        if self.non_empty() {
                            self.select_last();
                        }
                    }
                    UserEvent::BucketListPageDown => {
                        if self.non_empty() {
                            self.select_next_page();
                        }
                    }
                    UserEvent::BucketListPageUp => {
                        if self.non_empty() {
                            self.select_prev_page();
                        }
                    }
                    UserEvent::BucketListRefresh => {
                        if self.non_empty() {
                            self.tx.send(AppEventType::BucketListRefresh);
                        }
                    }
                    UserEvent::BucketListManagementConsole => {
                        if self.non_empty() {
                            self.tx.send(AppEventType::BucketListOpenManagementConsole);
                        }
                    }
                    UserEvent::BucketListFilter => {
                        self.open_filter_dialog();
                    }
                    UserEvent::BucketListSort => {
                        self.open_sort_dialog();
                    }
                    UserEvent::BucketListCopyDetails => {
                        self.open_copy_detail_dialog();
                    }
                    UserEvent::BucketListResetFilter => {
                        if !self.filter_input_state.input().is_empty() {
                            self.reset_filter();
                        }
                    }
                    UserEvent::Help => {
                        self.tx.send(AppEventType::OpenHelp);
                    }
                }
            }
            ViewState::FilterDialog => {
                handle_user_events_with_default! { user_events =>
                    UserEvent::InputDialogApply => {
                        self.apply_filter();
                    }
                    UserEvent::InputDialogClose => {
                        self.close_filter_dialog();
                    }
                    UserEvent::Help => {
                        self.tx.send(AppEventType::OpenHelp);
                    }
                    => {
                        self.filter_input_state.handle_key_event(key_event);
                        self.filter_view_indices();
                    }
                }
            }
            ViewState::SortDialog => {
                handle_user_events! { user_events =>
                    UserEvent::SelectDialogClose => {
                        self.close_sort_dialog();
                    }
                    UserEvent::SelectDialogDown => {
                        self.select_next_sort_item();
                    }
                    UserEvent::SelectDialogUp => {
                        self.select_prev_sort_item();
                    }
                    UserEvent::SelectDialogSelect => {
                        self.apply_sort();
                    }
                    UserEvent::Help => {
                        self.tx.send(AppEventType::OpenHelp);
                    }
                }
            }
            ViewState::CopyDetailDialog(ref mut state) => {
                handle_user_events! { user_events =>
                        UserEvent::SelectDialogClose => {
                            self.close_copy_detail_dialog();
                        }
                        UserEvent::SelectDialogDown => {
                            state.select_next();
                        }
                        UserEvent::SelectDialogUp => {
                            state.select_prev();
                        }
                        UserEvent::SelectDialogSelect => {
                            let (name, value) = state.selected_name_and_value();
                            self.tx.send(AppEventType::CopyToClipboard(name, value));
                        }
                        UserEvent::Help => {
                            self.tx.send(AppEventType::OpenHelp);
                        }
                }
            }
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let offset = self.list_state.offset;
        let selected = self.list_state.selected;

        let list_items = build_list_items(
            &self.bucket_items,
            &self.view_indices,
            self.filter_input_state.input(),
            &self.ctx.theme,
            offset,
            selected,
            area,
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
                BucketListSortDialog::new(self.sort_dialog_state).theme(&self.ctx.theme);
            f.render_widget(sort_dialog, area);
        }

        if let ViewState::CopyDetailDialog(state) = &mut self.view_state {
            let copy_detail_dialog = CopyDetailDialog::default().theme(&self.ctx.theme);
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

    pub fn short_helps(&self, mapper: &UserEventMapper) -> Vec<SpansWithPriority> {
        #[rustfmt::skip]
        let helps = match self.view_state {
            ViewState::Default => {
                if self.filter_input_state.input().is_empty() {
                    vec![
                        BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
                        BuildShortHelpsItem::group(vec![UserEvent::BucketListDown, UserEvent::BucketListUp], "Select", 1),
                        BuildShortHelpsItem::group(vec![UserEvent::BucketListGoToTop, UserEvent::BucketListGoToBottom], "Top/Bottom", 6),
                        BuildShortHelpsItem::single(UserEvent::BucketListSelect, "Open", 2),
                        BuildShortHelpsItem::single(UserEvent::BucketListFilter, "Filter", 3),
                        BuildShortHelpsItem::single(UserEvent::BucketListSort, "Sort", 4),
                        BuildShortHelpsItem::single(UserEvent::BucketListRefresh, "Refresh", 5),
                        BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
                    ]
                } else {
                    vec![
                        BuildShortHelpsItem::single(UserEvent::BucketListResetFilter, "Clear filter", 0),
                        BuildShortHelpsItem::group(vec![UserEvent::BucketListDown, UserEvent::BucketListUp], "Select", 1),
                        BuildShortHelpsItem::group(vec![UserEvent::BucketListGoToTop, UserEvent::BucketListGoToBottom], "Top/Bottom", 6),
                        BuildShortHelpsItem::single(UserEvent::BucketListSelect, "Open", 2),
                        BuildShortHelpsItem::single(UserEvent::BucketListFilter, "Filter", 3),
                        BuildShortHelpsItem::single(UserEvent::BucketListSort, "Sort", 4),
                        BuildShortHelpsItem::single(UserEvent::BucketListRefresh, "Refresh", 5),
                        BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
                    ]
                }
            }
            ViewState::FilterDialog => {
                vec![
                    BuildShortHelpsItem::single(UserEvent::InputDialogClose, "Close", 2),
                    BuildShortHelpsItem::single(UserEvent::InputDialogApply, "Filter", 1),
                    BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
                ]
            },
            ViewState::SortDialog => {
                vec![
                    BuildShortHelpsItem::single(UserEvent::SelectDialogClose, "Close", 2),
                    BuildShortHelpsItem::group(vec![UserEvent::SelectDialogDown, UserEvent::SelectDialogUp], "Select", 3),
                    BuildShortHelpsItem::single(UserEvent::SelectDialogSelect, "Sort", 1),
                    BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
                ]
            },
            ViewState::CopyDetailDialog(_) => {
                vec![
                    BuildShortHelpsItem::single(UserEvent::SelectDialogClose, "Close", 2),
                    BuildShortHelpsItem::group(vec![UserEvent::SelectDialogDown, UserEvent::SelectDialogUp], "Select", 3),
                    BuildShortHelpsItem::single(UserEvent::SelectDialogSelect, "Copy", 1),
                    BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
                ]
            },
        };
        build_short_help_spans(helps, mapper)
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

        match selected {
            BucketListSortType::Default => {
                self.view_indices.sort();
            }
            BucketListSortType::NameAsc => {
                self.view_indices
                    .sort_by(|a, b| items[*a].name.cmp(&items[*b].name));
            }
            BucketListSortType::NameDesc => {
                self.view_indices
                    .sort_by(|a, b| items[*b].name.cmp(&items[*a].name));
            }
        }
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
            build_list_item(&item.name, selected, filter, area.width, theme)
        })
        .collect()
}

fn build_list_item<'a>(
    name: &'a str,
    selected: bool,
    filter: &'a str,
    width: u16,
    theme: &'a ColorTheme,
) -> ListItem<'a> {
    let name_w = (width as usize) - 4 /* border + pad */;
    let pad_name =
        console::pad_str(name, name_w, console::Alignment::Left, Some(ELLIPSIS)).to_string();

    let line = if filter.is_empty() {
        Line::from(vec![" ".into(), pad_name.into(), " ".into()])
    } else {
        let i = name.find(filter).unwrap();
        let mut spans = highlight_matched_text(pad_name)
            .ellipsis(ELLIPSIS)
            .matched_range(i, i + filter.len())
            .not_matched_style(Style::default())
            .matched_style(Style::default().fg(theme.list_filter_match))
            .into_spans();
        spans.insert(0, " ".into());
        spans.push(" ".into());
        Line::from(spans)
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
    use ratatui::{
        backend::TestBackend, buffer::Buffer, crossterm::event::KeyCode, style::Color, Terminal,
    };

    #[test]
    fn test_render_without_scroll() -> std::io::Result<()> {
        let ctx = Rc::default();
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let items = ["bucket1", "bucket2", "bucket3"]
                .into_iter()
                .map(bucket_item)
                .collect();
            let mut page = BucketListPage::new(items, ctx, tx);
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
        let ctx = Rc::default();
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let items = (0..16)
                .map(|i| bucket_item(&format!("bucket{}", i + 1)))
                .collect();
            let mut page = BucketListPage::new(items, ctx, tx);
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
        let ctx = Rc::default();
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        let items = ["foo", "bar", "baz", "qux", "foobar"]
            .into_iter()
            .map(bucket_item)
            .collect();
        let mut page = BucketListPage::new(items, ctx, tx);
        let area = Rect::new(0, 0, 30, 10);

        page.handle_key(
            vec![UserEvent::BucketListFilter],
            KeyEvent::from(KeyCode::Char('/')),
        );
        page.handle_key(vec![], KeyEvent::from(KeyCode::Char('b')));

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

        page.handle_key(vec![], KeyEvent::from(KeyCode::Char('a')));
        page.handle_key(
            vec![UserEvent::InputDialogApply],
            KeyEvent::from(KeyCode::Enter),
        );

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
        let ctx = Rc::default();
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        let items = ["foo", "bar", "baz", "qux", "foobar"]
            .into_iter()
            .map(bucket_item)
            .collect();
        let mut page = BucketListPage::new(items, ctx, tx);
        let area = Rect::new(0, 0, 30, 10);

        page.handle_key(
            vec![UserEvent::BucketListSort],
            KeyEvent::from(KeyCode::Char('o')),
        );
        page.handle_key(
            vec![UserEvent::SelectDialogDown],
            KeyEvent::from(KeyCode::Char('j')),
        );
        page.handle_key(
            vec![UserEvent::SelectDialogDown],
            KeyEvent::from(KeyCode::Char('j')),
        );

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
        let ctx = Rc::default();
        let (tx, _) = event::new();

        let items = ["foo", "bar", "baz", "qux", "foobar"]
            .into_iter()
            .map(bucket_item)
            .collect();
        let mut page = BucketListPage::new(items, ctx, tx);

        page.handle_key(
            vec![UserEvent::BucketListFilter],
            KeyEvent::from(KeyCode::Char('/')),
        );
        page.handle_key(vec![], KeyEvent::from(KeyCode::Char('b')));
        page.handle_key(vec![], KeyEvent::from(KeyCode::Char('a')));

        assert_eq!(page.view_indices, vec![1, 2, 4]);

        page.handle_key(vec![], KeyEvent::from(KeyCode::Char('r')));

        assert_eq!(page.view_indices, vec![1, 4]);

        page.handle_key(vec![], KeyEvent::from(KeyCode::Char('r')));

        assert!(page.view_indices.is_empty());

        page.handle_key(vec![], KeyEvent::from(KeyCode::Backspace));
        page.handle_key(vec![], KeyEvent::from(KeyCode::Backspace));

        assert_eq!(page.view_indices, vec![1, 2, 4]);

        page.handle_key(
            vec![UserEvent::InputDialogClose],
            KeyEvent::from(KeyCode::Esc),
        );

        assert_eq!(page.view_indices, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_sort_items() {
        let ctx = Rc::default();
        let (tx, _) = event::new();

        let items = ["foo", "bar", "baz", "qux", "foobar"]
            .into_iter()
            .map(bucket_item)
            .collect();
        let mut page = BucketListPage::new(items, ctx, tx);

        page.handle_key(
            vec![UserEvent::BucketListSort],
            KeyEvent::from(KeyCode::Char('o')),
        );

        page.handle_key(
            vec![UserEvent::SelectDialogDown],
            KeyEvent::from(KeyCode::Char('j')),
        ); // select NameAsc

        assert_eq!(page.view_indices, vec![1, 2, 0, 4, 3]);

        page.handle_key(
            vec![UserEvent::SelectDialogDown],
            KeyEvent::from(KeyCode::Char('j')),
        ); // select NameDesc
        page.handle_key(
            vec![UserEvent::SelectDialogSelect],
            KeyEvent::from(KeyCode::Enter),
        );

        assert_eq!(page.view_indices, vec![3, 4, 0, 2, 1]);

        page.handle_key(
            vec![UserEvent::BucketListSort],
            KeyEvent::from(KeyCode::Char('o')),
        );
        page.handle_key(
            vec![UserEvent::SelectDialogUp],
            KeyEvent::from(KeyCode::Char('k')),
        ); // select NameAsc

        assert_eq!(page.view_indices, vec![1, 2, 0, 4, 3]);

        page.handle_key(
            vec![UserEvent::SelectDialogClose],
            KeyEvent::from(KeyCode::Esc),
        );

        assert_eq!(page.view_indices, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_filter_and_sort_items() {
        let ctx = Rc::default();
        let (tx, _) = event::new();

        let items = ["foo", "bar", "baz", "qux", "foobar"]
            .into_iter()
            .map(bucket_item)
            .collect();
        let mut page = BucketListPage::new(items, ctx, tx);

        page.handle_key(
            vec![UserEvent::BucketListFilter],
            KeyEvent::from(KeyCode::Char('/')),
        );
        page.handle_key(vec![], KeyEvent::from(KeyCode::Char('b')));
        page.handle_key(vec![], KeyEvent::from(KeyCode::Char('a')));
        page.handle_key(
            vec![UserEvent::InputDialogApply],
            KeyEvent::from(KeyCode::Enter),
        );

        assert_eq!(page.view_indices, vec![1, 2, 4]);

        page.handle_key(
            vec![UserEvent::BucketListSort],
            KeyEvent::from(KeyCode::Char('o')),
        );
        page.handle_key(
            vec![UserEvent::SelectDialogDown],
            KeyEvent::from(KeyCode::Char('j')),
        );
        page.handle_key(
            vec![UserEvent::SelectDialogDown],
            KeyEvent::from(KeyCode::Char('j')),
        );
        page.handle_key(
            vec![UserEvent::SelectDialogSelect],
            KeyEvent::from(KeyCode::Enter),
        );

        assert_eq!(page.view_indices, vec![4, 2, 1]);

        page.handle_key(
            vec![UserEvent::BucketListResetFilter],
            KeyEvent::from(KeyCode::Esc),
        );

        assert_eq!(page.view_indices, vec![3, 4, 0, 2, 1]);

        page.handle_key(
            vec![UserEvent::BucketListFilter],
            KeyEvent::from(KeyCode::Char('/')),
        );
        page.handle_key(vec![], KeyEvent::from(KeyCode::Char('f')));
        page.handle_key(vec![], KeyEvent::from(KeyCode::Char('o')));
        page.handle_key(
            vec![UserEvent::InputDialogApply],
            KeyEvent::from(KeyCode::Enter),
        );

        assert_eq!(page.view_indices, vec![4, 0]);

        page.handle_key(
            vec![UserEvent::BucketListSort],
            KeyEvent::from(KeyCode::Char('o')),
        );
        page.handle_key(
            vec![UserEvent::SelectDialogClose],
            KeyEvent::from(KeyCode::Esc),
        );

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
