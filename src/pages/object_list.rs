use std::rc::Rc;

use chrono::{DateTime, Local};
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
    config::UiConfig,
    event::{AppEventType, Sender},
    format::{format_datetime, format_size_byte},
    handle_user_events, handle_user_events_with_default,
    help::{
        build_help_spans, build_short_help_spans, BuildHelpsItem, BuildShortHelpsItem, Spans,
        SpansWithPriority,
    },
    keys::{UserEvent, UserEventMapper},
    object::{DownloadObjectInfo, ObjectItem, ObjectKey},
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
    DownloadConfirmDialog(Vec<DownloadObjectInfo>, ConfirmDialogState, bool),
    SaveDialog(InputDialogState, Option<Vec<DownloadObjectInfo>>),
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

    pub fn handle_key(&mut self, user_events: Vec<UserEvent>, key_event: KeyEvent) {
        match self.view_state {
            ViewState::Default => {
                handle_user_events! { user_events =>
                    UserEvent::ObjectListSelect if self.non_empty() => {
                        self.tx.send(AppEventType::ObjectListMoveDown);
                    }
                    UserEvent::ObjectListBack => {
                        self.tx.send(AppEventType::ObjectListMoveUp);
                    }
                    UserEvent::ObjectListDown if self.non_empty() => {
                        self.select_next();
                    }
                    UserEvent::ObjectListUp if self.non_empty() => {
                        self.select_prev();
                    }
                    UserEvent::ObjectListGoToTop if self.non_empty() => {
                        self.select_first();
                    }
                    UserEvent::ObjectListGoToBottom if self.non_empty() => {
                        self.select_last();
                    }
                    UserEvent::ObjectListPageDown if self.non_empty() => {
                        self.select_next_page();
                    }
                    UserEvent::ObjectListPageUp if self.non_empty() => {
                        self.select_prev_page();
                    }
                    UserEvent::ObjectListRefresh if self.non_empty() => {
                        self.tx.send(AppEventType::ObjectListRefresh);
                    }
                    UserEvent::ObjectListBucketList => {
                        self.tx.send(AppEventType::BackToBucketList);
                    }
                    UserEvent::ObjectListManagementConsole if self.non_empty() => {
                        self.open_management_console();
                    }
                    UserEvent::ObjectListFilter => {
                        self.open_filter_dialog();
                    }
                    UserEvent::ObjectListSort => {
                        self.open_sort_dialog();
                    }
                    UserEvent::ObjectListCopyDetails if self.non_empty() => {
                        self.open_copy_detail_dialog();
                    }
                    UserEvent::ObjectListDownloadObject if self.non_empty() => {
                        self.start_download();
                    }
                    UserEvent::ObjectListDownloadObjectAs if self.non_empty() => {
                        self.start_download_as();
                    }
                    UserEvent::Help => {
                        self.tx.send(AppEventType::OpenHelp);
                    }
                    UserEvent::ObjectListResetFilter => {
                        self.reset_filter();
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
                        BuildHelpsItem::new(UserEvent::ObjectListDown, "Select next item"),
                        BuildHelpsItem::new(UserEvent::ObjectListUp, "Select previous item"),
                        BuildHelpsItem::new(UserEvent::ObjectListGoToTop, "Go to top"),
                        BuildHelpsItem::new(UserEvent::ObjectListGoToBottom, "Go to bottom"),
                        BuildHelpsItem::new(UserEvent::ObjectListPageDown, "Scroll page forward"),
                        BuildHelpsItem::new(UserEvent::ObjectListPageUp, "Scroll page backward"),
                        BuildHelpsItem::new(UserEvent::ObjectListSelect, "Open file or folder"),
                        BuildHelpsItem::new(UserEvent::ObjectListBack, "Go back to prev folder"),
                        BuildHelpsItem::new(UserEvent::ObjectListBucketList, "Go back to bucket list"),
                        BuildHelpsItem::new(UserEvent::ObjectListFilter, "Filter object list"),
                        BuildHelpsItem::new(UserEvent::ObjectListDownloadObject, "Download object"),
                        BuildHelpsItem::new(UserEvent::ObjectListDownloadObjectAs, "Download object as"),
                        BuildHelpsItem::new(UserEvent::ObjectListSort, "Sort object list"),
                        BuildHelpsItem::new(UserEvent::ObjectListCopyDetails, "Open copy dialog"),
                        BuildHelpsItem::new(UserEvent::ObjectListRefresh, "Refresh object list"),
                        BuildHelpsItem::new(UserEvent::ObjectListManagementConsole, "Open management console in browser"),
                    ]
                } else {
                    vec![
                        BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
                        BuildHelpsItem::new(UserEvent::ObjectListResetFilter, "Clear filter"),
                        BuildHelpsItem::new(UserEvent::ObjectListDown, "Select next item"),
                        BuildHelpsItem::new(UserEvent::ObjectListUp, "Select previous item"),
                        BuildHelpsItem::new(UserEvent::ObjectListGoToTop, "Go to top"),
                        BuildHelpsItem::new(UserEvent::ObjectListGoToBottom, "Go to bottom"),
                        BuildHelpsItem::new(UserEvent::ObjectListPageDown, "Scroll page forward"),
                        BuildHelpsItem::new(UserEvent::ObjectListPageUp, "Scroll page backward"),
                        BuildHelpsItem::new(UserEvent::ObjectListSelect, "Open file or folder"),
                        BuildHelpsItem::new(UserEvent::ObjectListBack, "Go back to prev folder"),
                        BuildHelpsItem::new(UserEvent::ObjectListBucketList, "Go back to bucket list"),
                        BuildHelpsItem::new(UserEvent::ObjectListFilter, "Filter object list"),
                        BuildHelpsItem::new(UserEvent::ObjectListDownloadObject, "Download object"),
                        BuildHelpsItem::new(UserEvent::ObjectListDownloadObjectAs, "Download object as"),
                        BuildHelpsItem::new(UserEvent::ObjectListSort, "Sort object list"),
                        BuildHelpsItem::new(UserEvent::ObjectListCopyDetails, "Open copy dialog"),
                        BuildHelpsItem::new(UserEvent::ObjectListRefresh, "Refresh object list"),
                        BuildHelpsItem::new(UserEvent::ObjectListManagementConsole, "Open management console in browser"),
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
                        BuildShortHelpsItem::group(vec![UserEvent::ObjectListDown, UserEvent::ObjectListUp], "Select", 3),
                        BuildShortHelpsItem::group(vec![UserEvent::ObjectListGoToTop, UserEvent::ObjectListGoToBottom], "Top/Bottom", 8),
                        BuildShortHelpsItem::single(UserEvent::ObjectListSelect, "Open", 1),
                        BuildShortHelpsItem::single(UserEvent::ObjectListBack, "Go back", 2),
                        BuildShortHelpsItem::single(UserEvent::ObjectListFilter, "Filter", 4),
                        BuildShortHelpsItem::group(vec![UserEvent::ObjectListDownloadObject, UserEvent::ObjectListDownloadObjectAs], "Download", 5),
                        BuildShortHelpsItem::single(UserEvent::ObjectListSort, "Sort", 6),
                        BuildShortHelpsItem::single(UserEvent::ObjectListRefresh, "Refresh", 7),
                        BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
                    ]
                } else {
                    vec![
                        BuildShortHelpsItem::single(UserEvent::ObjectListResetFilter, "Clear filter", 0),
                        BuildShortHelpsItem::group(vec![UserEvent::ObjectListDown, UserEvent::ObjectListUp], "Select", 3),
                        BuildShortHelpsItem::group(vec![UserEvent::ObjectListGoToTop, UserEvent::ObjectListGoToBottom], "Top/Bottom", 8),
                        BuildShortHelpsItem::single(UserEvent::ObjectListSelect, "Open", 1),
                        BuildShortHelpsItem::single(UserEvent::ObjectListBack, "Go back", 2),
                        BuildShortHelpsItem::single(UserEvent::ObjectListFilter, "Filter", 4),
                        BuildShortHelpsItem::group(vec![UserEvent::ObjectListDownloadObject, UserEvent::ObjectListDownloadObjectAs], "Download", 5),
                        BuildShortHelpsItem::single(UserEvent::ObjectListSort, "Sort", 6),
                        BuildShortHelpsItem::single(UserEvent::ObjectListRefresh, "Refresh", 7),
                        BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
                    ]
                }
            },
            ViewState::FilterDialog => {
                vec![
                    BuildShortHelpsItem::single(UserEvent::InputDialogClose, "Close", 2),
                    BuildShortHelpsItem::single(UserEvent::InputDialogApply, "Filter", 1),
                    BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
                ]
            }
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

    fn start_download(&self) {
        match self.current_selected_item() {
            ObjectItem::Dir { .. } => {
                let key = self.current_selected_object_key();
                self.tx
                    .send(AppEventType::StartLoadAllDownloadObjectList(key, false));
            }
            ObjectItem::File {
                name, size_byte, ..
            } => {
                let object_key = self.current_selected_object_key();
                let object_name = name.clone();
                self.tx.send(AppEventType::StartDownloadObject(
                    object_key,
                    object_name,
                    *size_byte,
                    None,
                ));
            }
        }
    }

    fn start_download_as(&mut self) {
        match self.current_selected_item() {
            ObjectItem::Dir { .. } => {
                let key = self.current_selected_object_key();
                self.tx
                    .send(AppEventType::StartLoadAllDownloadObjectList(key, true));
            }
            ObjectItem::File { .. } => {
                self.open_save_dialog(None);
            }
        }
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
                let bucket = self.object_key.bucket_name.clone();
                let key = self.current_selected_object_key();
                let dir = self.current_selected_item().name().to_string();
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

            match std::mem::take(objs) {
                Some(objs) => {
                    let bucket = self.object_key.bucket_name.clone();
                    let key = self.current_selected_object_key();
                    let dir = input;
                    self.tx
                        .send(AppEventType::DownloadObjects(bucket, key, dir, objs));
                }
                None => {
                    let object_key = self.current_selected_object_key();
                    let size_byte = self.current_selected_item().size_byte().unwrap();
                    self.tx.send(AppEventType::StartDownloadObjectAs(
                        object_key, size_byte, input, None,
                    ));
                }
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

    fn open_management_console(&self) {
        let object_key = self.current_dir_object_key().clone();
        self.tx
            .send(AppEventType::ObjectListOpenManagementConsole(object_key));
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
    use crate::set_cells;

    use super::*;
    use chrono::NaiveDateTime;
    use ratatui::{
        backend::TestBackend,
        buffer::Buffer,
        crossterm::event::KeyCode,
        style::{Color, Modifier},
        Terminal,
    };

    #[tokio::test]
    async fn test_render_without_scroll() -> std::io::Result<()> {
        let ctx = Rc::default();
        let tx = sender();
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

    #[tokio::test]
    async fn test_render_with_scroll() -> std::io::Result<()> {
        let ctx = Rc::default();
        let tx = sender();
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

    #[tokio::test]
    async fn test_render_with_config() -> std::io::Result<()> {
        let tx = sender();
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

    #[tokio::test]
    async fn test_sort_items() {
        let ctx = Rc::default();
        let tx = sender();
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

        page.handle_key(
            vec![UserEvent::ObjectListSort],
            KeyEvent::from(KeyCode::Char('o')),
        );
        // select NameAsc
        page.handle_key(
            vec![UserEvent::SelectDialogDown],
            KeyEvent::from(KeyCode::Char('j')),
        );

        assert_eq!(page.view_indices, vec![4, 2, 1, 0, 3]);

        // select NameDesc
        page.handle_key(
            vec![UserEvent::SelectDialogDown],
            KeyEvent::from(KeyCode::Char('j')),
        );

        assert_eq!(page.view_indices, vec![3, 0, 1, 2, 4]);

        page.handle_key(
            vec![UserEvent::SelectDialogDown],
            KeyEvent::from(KeyCode::Char('j')),
        ); // select LastModifiedAsc

        assert_eq!(page.view_indices, vec![0, 2, 4, 3, 1]);

        // select LastModifiedDesc
        page.handle_key(
            vec![UserEvent::SelectDialogDown],
            KeyEvent::from(KeyCode::Char('j')),
        );

        assert_eq!(page.view_indices, vec![1, 3, 4, 0, 2]);

        // select SizeAsc
        page.handle_key(
            vec![UserEvent::SelectDialogDown],
            KeyEvent::from(KeyCode::Char('j')),
        );

        assert_eq!(page.view_indices, vec![0, 2, 4, 1, 3]);

        // select SizeDesc
        page.handle_key(
            vec![UserEvent::SelectDialogDown],
            KeyEvent::from(KeyCode::Char('j')),
        );

        assert_eq!(page.view_indices, vec![3, 1, 4, 0, 2]);
    }

    fn setup_terminal() -> std::io::Result<Terminal<TestBackend>> {
        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(terminal)
    }

    fn sender() -> Sender {
        let (tx, _) = tokio::sync::mpsc::unbounded_channel();
        Sender::new(tx)
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
