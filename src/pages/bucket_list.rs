use std::rc::Rc;

use laurier::highlight::highlight_matched_text;
use ratatui::{
    crossterm::event::KeyEvent,
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::ListItem,
    Frame,
};

use crate::{
    app::AppContext,
    color::ColorTheme,
    event::{AppEventType, Sender},
    format::format_size_byte,
    handle_user_events, handle_user_events_with_default,
    help::{
        build_help_spans, build_short_help_spans, BuildHelpsItem, BuildShortHelpsItem, Spans,
        SpansWithPriority,
    },
    keys::{UserEvent, UserEventMapper},
    object::{BucketItem, DownloadObjectInfo, ObjectKey},
    widget::{
        BucketListSortDialog, BucketListSortDialogState, BucketListSortType, ConfirmDialog,
        ConfirmDialogState, CopyDetailDialog, CopyDetailDialogState, InputDialog, InputDialogState,
        ScrollList, ScrollListState,
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
    DownloadConfirmDialog(Vec<DownloadObjectInfo>, ConfirmDialogState, bool),
    SaveDialog(InputDialogState, Option<Vec<DownloadObjectInfo>>),
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
                    UserEvent::BucketListSelect if self.non_empty() => {
                        self.move_down();
                    }
                    UserEvent::BucketListDown if self.non_empty() => {
                        self.select_next();
                    }
                    UserEvent::BucketListUp if self.non_empty() => {
                        self.select_prev();
                    }
                    UserEvent::BucketListGoToTop if self.non_empty() => {
                        self.select_first();
                    }
                    UserEvent::BucketListGoToBottom if self.non_empty() => {
                        self.select_last();
                    }
                    UserEvent::BucketListPageDown if self.non_empty() => {
                        self.select_next_page();
                    }
                    UserEvent::BucketListPageUp if self.non_empty() => {
                        self.select_prev_page();
                    }
                    UserEvent::BucketListRefresh if self.non_empty() => {
                        self.tx.send(AppEventType::BucketListRefresh);
                    }
                    UserEvent::BucketListManagementConsole if self.non_empty() => {
                        self.tx.send(AppEventType::BucketListOpenManagementConsole);
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
                    UserEvent::BucketListResetFilter if self.filter_input_state.non_empty() => {
                        self.reset_filter();
                    }
                    UserEvent::BucketListDownloadObject => {
                        self.start_download();
                    }
                    UserEvent::BucketListDownloadObjectAs => {
                        self.start_download_as();
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
            ViewState::DownloadConfirmDialog(_, ref mut state, _) => {
                handle_user_events! { user_events =>
                    UserEvent::SelectDialogClose => {
                        self.close_download_confirm_dialog();
                    }
                    UserEvent::SelectDialogLeft | UserEvent::SelectDialogRight => {
                        state.toggle();
                    }
                    UserEvent::SelectDialogSelect => {
                        self.download();
                    }
                    UserEvent::Help => {
                        self.tx.send(AppEventType::OpenHelp);
                    }
                }
            }
            ViewState::SaveDialog(ref mut state, _) => {
                handle_user_events_with_default! { user_events =>
                    UserEvent::InputDialogClose => {
                        self.close_save_dialog();
                    }
                    UserEvent::InputDialogApply => {
                        let input = state.input().into();
                        self.download_as(input);
                    }
                    UserEvent::Help => {
                        self.tx.send(AppEventType::OpenHelp);
                    }
                    => {
                        state.handle_key_event(key_event);
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

        if let ViewState::DownloadConfirmDialog(objs, state, _) = &mut self.view_state {
            let message_lines = build_download_confirm_message_lines(objs, &self.ctx.theme);
            let download_confirm_dialog = ConfirmDialog::new(message_lines).theme(&self.ctx.theme);
            f.render_stateful_widget(download_confirm_dialog, area, state);
        }

        if let ViewState::SaveDialog(state, _) = &mut self.view_state {
            let save_dialog = InputDialog::default()
                .title("Save As")
                .max_width(40)
                .theme(&self.ctx.theme);
            f.render_stateful_widget(save_dialog, area, state);

            let (cursor_x, cursor_y) = state.cursor();
            f.set_cursor_position((cursor_x, cursor_y));
        }
    }

    pub fn helps(&self, mapper: &UserEventMapper) -> Vec<Spans> {
        #[rustfmt::skip]
        let helps = match self.view_state {
            ViewState::Default => {
                if self.filter_input_state.is_empty() {
                    vec![
                        BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
                        BuildHelpsItem::new(UserEvent::BucketListDown, "Select next item"),
                        BuildHelpsItem::new(UserEvent::BucketListUp, "Select previous item"),
                        BuildHelpsItem::new(UserEvent::BucketListGoToTop, "Go to top"),
                        BuildHelpsItem::new(UserEvent::BucketListGoToBottom, "Go to bottom"),
                        BuildHelpsItem::new(UserEvent::BucketListPageDown, "Scroll page forward"),
                        BuildHelpsItem::new(UserEvent::BucketListPageUp, "Scroll page backward"),
                        BuildHelpsItem::new(UserEvent::BucketListSelect, "Open bucket"),
                        BuildHelpsItem::new(UserEvent::BucketListFilter, "Filter bucket list"),
                        BuildHelpsItem::new(UserEvent::BucketListSort, "Sort bucket list"),
                        BuildHelpsItem::new(UserEvent::BucketListCopyDetails, "Open copy dialog"),
                        BuildHelpsItem::new(UserEvent::BucketListDownloadObject, "Download object"),
                        BuildHelpsItem::new(UserEvent::BucketListDownloadObjectAs, "Download object as"),
                        BuildHelpsItem::new(UserEvent::BucketListRefresh, "Refresh bucket list"),
                        BuildHelpsItem::new(UserEvent::BucketListManagementConsole, "Open management console in browser"),
                    ]
                } else {
                    vec![
                        BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
                        BuildHelpsItem::new(UserEvent::BucketListResetFilter, "Clear filter"),
                        BuildHelpsItem::new(UserEvent::BucketListDown, "Select next item"),
                        BuildHelpsItem::new(UserEvent::BucketListUp, "Select previous item"),
                        BuildHelpsItem::new(UserEvent::BucketListGoToTop, "Go to top"),
                        BuildHelpsItem::new(UserEvent::BucketListGoToBottom, "Go to bottom"),
                        BuildHelpsItem::new(UserEvent::BucketListPageDown, "Scroll page forward"),
                        BuildHelpsItem::new(UserEvent::BucketListPageUp, "Scroll page backward"),
                        BuildHelpsItem::new(UserEvent::BucketListSelect, "Open bucket"),
                        BuildHelpsItem::new(UserEvent::BucketListFilter, "Filter bucket list"),
                        BuildHelpsItem::new(UserEvent::BucketListSort, "Sort bucket list"),
                        BuildHelpsItem::new(UserEvent::BucketListCopyDetails, "Open copy dialog"),
                        BuildHelpsItem::new(UserEvent::BucketListDownloadObject, "Download object"),
                        BuildHelpsItem::new(UserEvent::BucketListDownloadObjectAs, "Download object as"),
                        BuildHelpsItem::new(UserEvent::BucketListRefresh, "Refresh bucket list"),
                        BuildHelpsItem::new(UserEvent::BucketListManagementConsole, "Open management console in browser"),
                    ]
                }
            },
            ViewState::FilterDialog => {
                vec![
                    BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
                    BuildHelpsItem::new(UserEvent::InputDialogClose, "Close filter dialog"),
                    BuildHelpsItem::new(UserEvent::InputDialogApply, "Apply filter"),
                ]
            },
            ViewState::SortDialog => {
                vec![
                    BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
                    BuildHelpsItem::new(UserEvent::SelectDialogClose, "Close sort dialog"),
                    BuildHelpsItem::new(UserEvent::SelectDialogDown, "Select next item"),
                    BuildHelpsItem::new(UserEvent::SelectDialogUp, "Select previous item"),
                    BuildHelpsItem::new(UserEvent::SelectDialogSelect, "Apply sort"),
                ]
            },
            ViewState::CopyDetailDialog(_) => {
                vec![
                    BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
                    BuildHelpsItem::new(UserEvent::SelectDialogClose, "Close copy dialog"),
                    BuildHelpsItem::new(UserEvent::SelectDialogDown, "Select next item"),
                    BuildHelpsItem::new(UserEvent::SelectDialogUp, "Select previous item"),
                    BuildHelpsItem::new(UserEvent::SelectDialogSelect, "Copy selected value to clipboard"),
                ]
            },
            ViewState::DownloadConfirmDialog(_, _, _) => {
                vec![
                    BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
                    BuildHelpsItem::new(UserEvent::SelectDialogClose, "Close confirm dialog"),
                    BuildHelpsItem::new(UserEvent::SelectDialogRight, "Select next"),
                    BuildHelpsItem::new(UserEvent::SelectDialogLeft, "Select previous"),
                    BuildHelpsItem::new(UserEvent::SelectDialogSelect, "Confirm"),
                ]
            }
            ViewState::SaveDialog(_, _) => {
                vec![
                    BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
                    BuildHelpsItem::new(UserEvent::InputDialogClose, "Close save dialog"),
                    BuildHelpsItem::new(UserEvent::InputDialogApply, "Download object"),
                ]
            }
        };
        build_help_spans(helps, mapper, self.ctx.theme.help_key_fg)
    }

    pub fn short_helps(&self, mapper: &UserEventMapper) -> Vec<SpansWithPriority> {
        #[rustfmt::skip]
        let helps = match self.view_state {
            ViewState::Default => {
                if self.filter_input_state.is_empty() {
                    vec![
                        BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
                        BuildShortHelpsItem::group(vec![UserEvent::BucketListDown, UserEvent::BucketListUp], "Select", 1),
                        BuildShortHelpsItem::group(vec![UserEvent::BucketListGoToTop, UserEvent::BucketListGoToBottom], "Top/Bottom", 7),
                        BuildShortHelpsItem::single(UserEvent::BucketListSelect, "Open", 2),
                        BuildShortHelpsItem::single(UserEvent::BucketListFilter, "Filter", 3),
                        BuildShortHelpsItem::single(UserEvent::BucketListSort, "Sort", 4),
                        BuildShortHelpsItem::group(vec![UserEvent::BucketListDownloadObject, UserEvent::BucketListDownloadObjectAs], "Download", 5),
                        BuildShortHelpsItem::single(UserEvent::BucketListRefresh, "Refresh", 6),
                        BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
                    ]
                } else {
                    vec![
                        BuildShortHelpsItem::single(UserEvent::BucketListResetFilter, "Clear filter", 0),
                        BuildShortHelpsItem::group(vec![UserEvent::BucketListDown, UserEvent::BucketListUp], "Select", 1),
                        BuildShortHelpsItem::group(vec![UserEvent::BucketListGoToTop, UserEvent::BucketListGoToBottom], "Top/Bottom", 7),
                        BuildShortHelpsItem::single(UserEvent::BucketListSelect, "Open", 2),
                        BuildShortHelpsItem::single(UserEvent::BucketListFilter, "Filter", 3),
                        BuildShortHelpsItem::single(UserEvent::BucketListSort, "Sort", 4),
                        BuildShortHelpsItem::group(vec![UserEvent::BucketListDownloadObject, UserEvent::BucketListDownloadObjectAs], "Download", 5),
                        BuildShortHelpsItem::single(UserEvent::BucketListRefresh, "Refresh", 6),
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
            ViewState::DownloadConfirmDialog(_, _, _) => {
                vec![
                    BuildShortHelpsItem::single(UserEvent::SelectDialogClose, "Close", 2),
                    BuildShortHelpsItem::group(vec![UserEvent::SelectDialogLeft, UserEvent::SelectDialogRight], "Select", 3),
                    BuildShortHelpsItem::single(UserEvent::SelectDialogSelect, "Confirm", 1),
                    BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
                ]
            },
            ViewState::SaveDialog(_, _) => {
                vec![
                    BuildShortHelpsItem::single(UserEvent::InputDialogClose, "Close", 2),
                    BuildShortHelpsItem::single(UserEvent::InputDialogApply, "Download", 1),
                    BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
                ]
            }
        };
        build_short_help_spans(helps, mapper)
    }
}

impl BucketListPage {
    fn move_down(&self) {
        let object_key = self.current_selected_object_key();
        self.tx.send(AppEventType::BucketListMoveDown(object_key));
    }

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

    pub fn open_download_confirm_dialog(
        &mut self,
        objs: Vec<DownloadObjectInfo>,
        download_as: bool,
    ) {
        let dialog_state = ConfirmDialogState::default();
        self.view_state = ViewState::DownloadConfirmDialog(objs, dialog_state, download_as);
    }

    fn close_download_confirm_dialog(&mut self) {
        self.view_state = ViewState::Default;
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

    fn start_download(&self) {
        let key = self.current_selected_object_key();
        self.tx
            .send(AppEventType::StartLoadAllDownloadObjectList(key, false));
    }

    fn start_download_as(&mut self) {
        let key = self.current_selected_object_key();
        self.tx
            .send(AppEventType::StartLoadAllDownloadObjectList(key, true));
    }

    fn download(&mut self) {
        if let ViewState::DownloadConfirmDialog(objs, state, download_as) = &mut self.view_state {
            if state.is_ok() {
                if *download_as {
                    let objs = std::mem::take(objs);
                    self.open_save_dialog(Some(objs));
                    return;
                }

                let objs = std::mem::take(objs);
                let key = self.current_selected_object_key();
                let bucket = key.bucket_name.clone();
                let dir = key.bucket_name.clone();
                self.tx
                    .send(AppEventType::DownloadObjects(bucket, key, dir, objs));
            }
            self.close_download_confirm_dialog();
        }
    }

    fn download_as(&mut self, input: String) {
        if let ViewState::SaveDialog(_, objs) = &mut self.view_state {
            let input: String = input.trim().into();
            if input.is_empty() {
                return;
            }

            if let Some(objs) = std::mem::take(objs) {
                let key = self.current_selected_object_key();
                let bucket = key.bucket_name.clone();
                let dir = input;
                self.tx
                    .send(AppEventType::DownloadObjects(bucket, key, dir, objs));
            }

            self.close_save_dialog();
        }
    }

    fn open_save_dialog(&mut self, objs: Option<Vec<DownloadObjectInfo>>) {
        self.view_state = ViewState::SaveDialog(InputDialogState::default(), objs);
    }

    fn close_save_dialog(&mut self) {
        self.view_state = ViewState::Default;
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
    use crate::set_cells;

    use super::*;
    use ratatui::{
        backend::TestBackend, buffer::Buffer, crossterm::event::KeyCode, style::Color, Terminal,
    };

    #[tokio::test]
    async fn test_render_without_scroll() -> std::io::Result<()> {
        let ctx = Rc::default();
        let tx = sender();
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

    #[tokio::test]
    async fn test_render_with_scroll() -> std::io::Result<()> {
        let ctx = Rc::default();
        let tx = sender();
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

    #[tokio::test]
    async fn test_render_filter_items() -> std::io::Result<()> {
        let ctx = Rc::default();
        let tx = sender();
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

    #[tokio::test]
    async fn test_render_sort_items() -> std::io::Result<()> {
        let ctx = Rc::default();
        let tx = sender();
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

    #[tokio::test]
    async fn test_filter_items() {
        let ctx = Rc::default();
        let tx = sender();

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

    #[tokio::test]
    async fn test_sort_items() {
        let ctx = Rc::default();
        let tx = sender();

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

    #[tokio::test]
    async fn test_filter_and_sort_items() {
        let ctx = Rc::default();
        let tx = sender();

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

    fn sender() -> Sender {
        let (tx, _) = tokio::sync::mpsc::unbounded_channel();
        Sender::new(tx)
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
