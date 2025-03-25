use std::rc::Rc;

use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyEvent,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, ListItem, Padding, Paragraph, StatefulWidget, Tabs, Widget},
    Frame,
};

use crate::{
    app::AppContext,
    color::ColorTheme,
    config::UiConfig,
    event::{AppEventType, Sender},
    format::{format_datetime, format_size_byte, format_version},
    handle_user_events, handle_user_events_with_default,
    help::{
        build_help_spans, build_short_help_spans, BuildHelpsItem, BuildShortHelpsItem, Spans,
        SpansWithPriority,
    },
    keys::{UserEvent, UserEventMapper},
    object::{FileDetail, FileVersion, ObjectItem, ObjectKey},
    widget::{
        Bar, CopyDetailDialog, CopyDetailDialogState, Divider, InputDialog, InputDialogState,
        ScrollLines, ScrollLinesOptions, ScrollLinesState, ScrollList, ScrollListState,
    },
};

#[derive(Debug)]
pub struct ObjectDetailPage {
    file_detail: FileDetail,
    file_versions: Vec<FileVersion>,
    object_key: ObjectKey,

    tab: Tab,
    view_state: ViewState,

    object_items: Vec<ObjectItem>,
    list_state: ScrollListState,

    ctx: Rc<AppContext>,
    tx: Sender,
}

#[derive(Debug)]
enum Tab {
    Detail(DetailTabState),
    Version(VersionTabState),
}

impl Tab {
    fn val(&self) -> usize {
        match self {
            Tab::Detail(_) => 0,
            Tab::Version(_) => 1,
        }
    }
}

#[derive(Debug)]
enum ViewState {
    Default,
    SaveDialog(InputDialogState),
    CopyDetailDialog(Box<CopyDetailDialogState>),
}

impl ObjectDetailPage {
    pub fn new(
        file_detail: FileDetail,
        object_items: Vec<ObjectItem>,
        object_key: ObjectKey,
        list_state: ScrollListState,
        ctx: Rc<AppContext>,
        tx: Sender,
    ) -> Self {
        let detail_tab_state = DetailTabState::new(&file_detail, &ctx.config.ui);
        Self {
            file_detail,
            file_versions: Vec::new(),
            object_key,
            tab: Tab::Detail(detail_tab_state),
            view_state: ViewState::Default,
            object_items,
            list_state,
            ctx,
            tx,
        }
    }

    pub fn handle_key(&mut self, user_events: Vec<UserEvent>, key_event: KeyEvent) {
        match self.view_state {
            ViewState::Default => {
                handle_user_events! { user_events =>
                    UserEvent::ObjectDetailBack => {
                        self.tx.send(AppEventType::CloseCurrentPage);
                    }
                    UserEvent::ObjectDetailRight | UserEvent::ObjectDetailLeft => {
                        self.toggle_tab();
                    }
                    UserEvent::ObjectDetailDown => {
                        match self.tab {
                            Tab::Detail(ref mut state) => {
                                state.scroll_lines_state.scroll_forward();
                            }
                            Tab::Version(ref mut state) => {
                                state.select_next();
                            }
                        }
                    }
                    UserEvent::ObjectDetailUp => {
                        match self.tab {
                            Tab::Detail(ref mut state) => {
                                state.scroll_lines_state.scroll_backward();
                            }
                            Tab::Version(ref mut state) => {
                                state.select_prev();
                            }
                        }
                    }
                    UserEvent::ObjectDetailGoToTop => {
                        if let Tab::Version(ref mut state) = self.tab {
                            state.select_first();
                        }
                    }
                    UserEvent::ObjectDetailGoToBottom => {
                        if let Tab::Version(ref mut state) = self.tab {
                            state.select_last();
                        }
                    }
                    UserEvent::ObjectDetailDownload => {
                        self.download();
                    }
                    UserEvent::ObjectDetailDownloadAs => {
                        self.open_save_dialog();
                    }
                    UserEvent::ObjectDetailPreview => {
                        self.preview();
                    }
                    UserEvent::ObjectDetailCopyDetails => {
                        self.open_copy_detail_dialog();
                    }
                    UserEvent::ObjectDetailManagementConsole => {
                        self.open_management_console();
                    }
                    UserEvent::Help => {
                        self.tx.send(AppEventType::OpenHelp);
                    }
                }
            }
            ViewState::SaveDialog(ref mut state) => {
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
            ViewState::CopyDetailDialog(ref mut state) => {
                handle_user_events! { user_events =>
                    UserEvent::SelectDialogClose => {
                        self.close_copy_detail_dialog();
                    }
                    UserEvent::SelectDialogSelect => {
                        let (name, value) = state.selected_name_and_value();
                        self.tx.send(AppEventType::CopyToClipboard(name, value));
                    }
                    UserEvent::SelectDialogDown => {
                        state.select_next();
                    }
                    UserEvent::SelectDialogUp => {
                        state.select_prev();
                    }
                    UserEvent::Help => {
                        self.tx.send(AppEventType::OpenHelp);
                    }
                }
            }
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::horizontal(Constraint::from_percentages([50, 50])).split(area);

        let offset = self.list_state.offset;
        let selected = self.list_state.selected;

        let list_items = build_list_items_from_object_items(
            &self.object_items,
            offset,
            selected,
            chunks[0],
            &self.ctx.theme,
        );

        let list = ScrollList::new(list_items).theme(&self.ctx.theme);
        f.render_stateful_widget(list, chunks[0], &mut self.list_state);

        let block = Block::bordered().fg(self.ctx.theme.fg);
        f.render_widget(block, chunks[1]);

        let chunks = Layout::vertical([Constraint::Length(2), Constraint::Min(0)])
            .margin(1)
            .split(chunks[1]);

        let tabs = build_tabs(&self.tab, &self.ctx.theme);
        f.render_widget(tabs, chunks[0]);

        match self.tab {
            Tab::Detail(ref mut state) => {
                let detail = DetailTab::new(&self.ctx.theme);
                f.render_stateful_widget(detail, chunks[1], state);
            }
            Tab::Version(ref mut state) => {
                let version = VersionTab::new(&self.ctx.theme);
                f.render_stateful_widget(version, chunks[1], state);
            }
        }

        if let ViewState::SaveDialog(state) = &mut self.view_state {
            let save_dialog = InputDialog::default()
                .title("Save As")
                .max_width(40)
                .theme(&self.ctx.theme);
            f.render_stateful_widget(save_dialog, area, state);

            let (cursor_x, cursor_y) = state.cursor();
            f.set_cursor_position((cursor_x, cursor_y));
        }

        if let ViewState::CopyDetailDialog(state) = &mut self.view_state {
            let copy_detail_dialog = CopyDetailDialog::default().theme(&self.ctx.theme);
            f.render_stateful_widget(copy_detail_dialog, area, state);
        }
    }

    pub fn helps(&self, mapper: &UserEventMapper) -> Vec<Spans> {
        #[rustfmt::skip]
        let helps = match self.view_state {
            ViewState::Default => match self.tab {
                Tab::Detail(_) => {
                    vec![
                        BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailLeft, "Select next tab"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailRight, "Select previous tab"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailBack, "Close detail panel"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailDown, "Scroll forward"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailUp, "Scroll backward"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailCopyDetails, "Open copy dialog"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailDownload, "Download object"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailDownloadAs, "Download object as"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailPreview, "Preview object"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailManagementConsole, "Open management console in browser"),
                    ]
                },
                Tab::Version(_) => {
                    vec![
                        BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailLeft, "Select next tab"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailRight, "Select previous tab"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailDown, "Select next version"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailUp, "Select previous version"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailGoToTop, "Go to top"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailGoToBottom, "Go to bottom"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailBack, "Close detail panel"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailCopyDetails, "Open copy dialog"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailDownload, "Download object"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailDownloadAs, "Download object as"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailPreview, "Preview object"),
                        BuildHelpsItem::new(UserEvent::ObjectDetailManagementConsole, "Open management console in browser"),
                    ]
                },
            }
            ViewState::SaveDialog(_) => {
                vec![
                    BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
                    BuildHelpsItem::new(UserEvent::InputDialogClose, "Close save dialog"),
                    BuildHelpsItem::new(UserEvent::InputDialogApply, "Download object"),
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
        };
        build_help_spans(helps, mapper, self.ctx.theme.help_key_fg)
    }

    pub fn short_helps(&self, mapper: &UserEventMapper) -> Vec<SpansWithPriority> {
        #[rustfmt::skip]
        let helps = match self.view_state {
            ViewState::Default => {
                match self.tab {
                    Tab::Detail(_) => {
                        vec![
                            BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
                            BuildShortHelpsItem::group(vec![UserEvent::ObjectDetailLeft, UserEvent::ObjectDetailRight], "Select tabs", 3),
                            BuildShortHelpsItem::group(vec![UserEvent::ObjectDetailDown, UserEvent::ObjectDetailUp], "Scroll", 5),
                            BuildShortHelpsItem::group(vec![UserEvent::ObjectDetailDownload, UserEvent::ObjectDetailDownloadAs], "Download", 1),
                            BuildShortHelpsItem::single(UserEvent::ObjectDetailPreview, "Preview", 4),
                            BuildShortHelpsItem::single(UserEvent::ObjectDetailBack, "Close", 2),
                            BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
                        ]
                    },
                    Tab::Version(_) => {
                        vec![
                            BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
                            BuildShortHelpsItem::group(vec![UserEvent::ObjectDetailLeft, UserEvent::ObjectDetailRight], "Select tabs", 3),
                            BuildShortHelpsItem::group(vec![UserEvent::ObjectDetailDown, UserEvent::ObjectDetailUp], "Select", 5),
                            BuildShortHelpsItem::group(vec![UserEvent::ObjectDetailDownload, UserEvent::ObjectDetailDownloadAs], "Download", 1),
                            BuildShortHelpsItem::single(UserEvent::ObjectDetailPreview, "Preview", 4),
                            BuildShortHelpsItem::single(UserEvent::ObjectDetailBack, "Close", 2),
                            BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
                        ]
                    },
                }
            },
            ViewState::SaveDialog(_) => {
                vec![
                    BuildShortHelpsItem::single(UserEvent::InputDialogClose, "Close", 2),
                    BuildShortHelpsItem::single(UserEvent::InputDialogApply, "Download", 1),
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

impl ObjectDetailPage {
    fn toggle_tab(&mut self) {
        match self.tab {
            Tab::Detail(_) => {
                if self.file_versions.is_empty() {
                    self.tx.send(AppEventType::OpenObjectVersionsTab);
                } else {
                    self.select_versions_tab();
                }
            }
            Tab::Version(_) => self.select_detail_tab(),
        }
    }

    pub fn select_detail_tab(&mut self) {
        self.tab = Tab::Detail(DetailTabState::new(&self.file_detail, &self.ctx.config.ui));
    }

    pub fn select_versions_tab(&mut self) {
        self.tab = Tab::Version(VersionTabState::new(
            &self.file_versions,
            &self.ctx.config.ui,
        ));
    }

    pub fn set_versions(&mut self, versions: Vec<FileVersion>) {
        self.file_versions = versions;
    }

    fn open_save_dialog(&mut self) {
        self.view_state = ViewState::SaveDialog(InputDialogState::default());
    }

    fn close_save_dialog(&mut self) {
        self.view_state = ViewState::Default;
    }

    fn open_copy_detail_dialog(&mut self) {
        match self.tab {
            Tab::Detail(_) => {
                self.view_state = ViewState::CopyDetailDialog(Box::new(
                    CopyDetailDialogState::object_detail(self.file_detail.clone()),
                ));
            }
            Tab::Version(_) => {
                let version = self.current_selected_version().unwrap().clone();
                self.view_state = ViewState::CopyDetailDialog(Box::new(
                    CopyDetailDialogState::object_version(self.file_detail.clone(), version),
                ));
            }
        }
    }

    fn close_copy_detail_dialog(&mut self) {
        self.view_state = ViewState::Default;
    }

    fn download(&self) {
        let object_key = self.object_key.clone();
        let object_name = self.file_detail.name.clone();
        let size_byte = self.file_detail.size_byte;
        let version_id = self.current_selected_version_id();
        self.tx.send(AppEventType::StartDownloadObject(
            object_key,
            object_name,
            size_byte,
            version_id,
        ));
    }

    fn download_as(&mut self, input: String) {
        let input: String = input.trim().into();
        if input.is_empty() {
            return;
        }

        let object_key = self.object_key.clone();
        let size_byte = self.file_detail.size_byte;
        let version_id = self.current_selected_version_id();
        self.tx.send(AppEventType::StartDownloadObjectAs(
            object_key, size_byte, input, version_id,
        ));

        self.close_save_dialog();
    }

    fn preview(&self) {
        let object_key = self.object_key.clone();
        let file_detail = self.file_detail.clone();
        let version_id = self.current_selected_version_id();
        self.tx.send(AppEventType::OpenPreview(
            object_key,
            file_detail,
            version_id,
        ));
    }

    fn open_management_console(&self) {
        let object_key = self.object_key.clone();
        self.tx
            .send(AppEventType::ObjectDetailOpenManagementConsole(object_key));
    }

    fn current_selected_version(&self) -> Option<&FileVersion> {
        match &self.tab {
            Tab::Detail(_) => None,
            Tab::Version(state) => self.file_versions.get(state.selected),
        }
    }

    fn current_selected_version_id(&self) -> Option<String> {
        self.current_selected_version()
            .map(|v| v.version_id.clone())
    }

    pub fn current_object_key(&self) -> &ObjectKey {
        &self.object_key
    }
}

fn build_list_items_from_object_items<'a>(
    current_items: &'a [ObjectItem],
    offset: usize,
    selected: usize,
    area: Rect,
    theme: &ColorTheme,
) -> Vec<ListItem<'a>> {
    let show_item_count = (area.height as usize) - 2 /* border */;
    current_items
        .iter()
        .skip(offset)
        .take(show_item_count)
        .enumerate()
        .map(|(idx, item)| {
            build_list_item_from_object_item(idx, item, offset, selected, area, theme)
        })
        .collect()
}

fn build_list_item_from_object_item<'a>(
    idx: usize,
    item: &'a ObjectItem,
    offset: usize,
    selected: usize,
    area: Rect,
    theme: &ColorTheme,
) -> ListItem<'a> {
    let content = match item {
        ObjectItem::Dir { name, .. } => {
            let content = format_dir_item(name, area.width);
            let style = Style::default().add_modifier(Modifier::BOLD);
            Span::styled(content, style)
        }
        ObjectItem::File { name, .. } => {
            let content = format_file_item(name, area.width);
            let style = Style::default();
            Span::styled(content, style)
        }
    };
    if idx + offset == selected {
        ListItem::new(content).style(
            Style::default()
                .bg(theme.list_selected_inactive_bg)
                .fg(theme.list_selected_inactive_fg),
        )
    } else {
        ListItem::new(content)
    }
}

fn format_dir_item(name: &str, width: u16) -> String {
    let name_w: usize = (width as usize) - 2 /* spaces */ - 2 /* border */;
    let name = format!("{}/", name);
    format!(" {:<name_w$} ", name, name_w = name_w)
}

fn format_file_item(name: &str, width: u16) -> String {
    let name_w: usize = (width as usize) - 2 /* spaces */ - 4 /* border */;
    format!(" {:<name_w$} ", name, name_w = name_w)
}

fn build_tabs(tab: &Tab, theme: &ColorTheme) -> Tabs<'static> {
    let tabs = vec!["Detail", "Version"];
    Tabs::new(tabs)
        .select(tab.val())
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(theme.detail_selected),
        )
        .block(Block::default().borders(Borders::BOTTOM))
}

fn build_detail_content_lines(detail: &FileDetail, ui_config: &UiConfig) -> Vec<Line<'static>> {
    let details = [
        ("Name:", &detail.name),
        ("Size:", &format_size_byte(detail.size_byte)),
        (
            "Last Modified:",
            &format_datetime(&detail.last_modified, &ui_config.object_detail.date_format),
        ),
        ("ETag:", &detail.e_tag),
        ("Content-Type:", &detail.content_type),
        ("Storage class:", &detail.storage_class),
    ]
    .iter()
    .filter_map(|(label, value)| {
        if value.is_empty() {
            None
        } else {
            let lines = vec![
                Line::from(label.add_modifier(Modifier::BOLD)),
                Line::from(format!(" {}", value)),
            ];
            Some(lines)
        }
    })
    .collect();

    flatten_with_empty_lines(details)
}

#[derive(Debug)]
struct DetailTabState {
    scroll_lines_state: ScrollLinesState,
}

impl DetailTabState {
    fn new(file_detail: &FileDetail, ui_config: &UiConfig) -> Self {
        let scroll_lines = build_detail_content_lines(file_detail, ui_config);
        let scroll_lines_state =
            ScrollLinesState::new(scroll_lines, ScrollLinesOptions::new(false, true));
        Self { scroll_lines_state }
    }
}

#[derive(Debug)]
struct DetailTab<'a> {
    theme: &'a ColorTheme,
}

impl<'a> DetailTab<'a> {
    fn new(theme: &'a ColorTheme) -> Self {
        Self { theme }
    }
}

impl StatefulWidget for DetailTab<'_> {
    type State = DetailTabState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let scroll_lines = ScrollLines::default().theme(self.theme);
        StatefulWidget::render(scroll_lines, area, buf, &mut state.scroll_lines_state);
    }
}

fn build_version_detail_lines(
    versions: &[FileVersion],
    ui_config: &UiConfig,
) -> Vec<Vec<Line<'static>>> {
    versions
        .iter()
        .map(|v| {
            let version_id = format_version(&v.version_id).to_owned();
            let last_modified =
                format_datetime(&v.last_modified, &ui_config.object_detail.date_format);
            let size_byte = format_size_byte(v.size_byte);
            vec![
                Line::from(vec![
                    "   Version ID: ".add_modifier(Modifier::BOLD),
                    Span::raw(version_id),
                ]),
                Line::from(vec![
                    "Last Modified: ".add_modifier(Modifier::BOLD),
                    Span::raw(last_modified),
                ]),
                Line::from(vec![
                    "         Size: ".add_modifier(Modifier::BOLD),
                    Span::raw(size_byte),
                ]),
            ]
        })
        .collect()
}

#[derive(Debug, Default)]
struct VersionTabState {
    lines: Vec<Vec<Line<'static>>>,
    selected: usize,
    offset: usize,
    height: usize,
}

impl VersionTabState {
    fn new(versions: &[FileVersion], ui_config: &UiConfig) -> Self {
        let lines = build_version_detail_lines(versions, ui_config);
        Self {
            lines,
            ..Default::default()
        }
    }

    fn select_next(&mut self) {
        if self.selected >= self.lines.len() - 1 {
            return;
        }

        self.selected += 1;

        let mut total_height = 0;
        for lines in self
            .lines
            .iter()
            .skip(self.offset)
            .take(self.selected - self.offset + 1)
        {
            total_height += lines.len();
            total_height += 1; // divider
        }
        if total_height > self.height {
            self.offset += 1;
        }
    }

    fn select_prev(&mut self) {
        if self.selected == 0 {
            return;
        }

        self.selected -= 1;
        if self.selected < self.offset {
            self.offset -= 1;
        }
    }

    fn select_first(&mut self) {
        self.selected = 0;
        self.offset = 0;
    }

    fn select_last(&mut self) {
        self.selected = self.lines.len() - 1;

        let mut total_height = 0;
        for (i, lines) in self.lines.iter().enumerate().rev() {
            total_height += lines.len();
            total_height += 1; // divider

            if total_height == self.height {
                self.offset = i;
                break;
            } else if total_height > self.height {
                self.offset = i + 1;
                break;
            }
        }
    }
}

#[derive(Debug, Default)]
struct VersionTabColor {
    selected: Color,
    divider: Color,
}

impl VersionTabColor {
    fn new(theme: &ColorTheme) -> Self {
        Self {
            selected: theme.detail_selected,
            divider: theme.divider,
        }
    }
}

#[derive(Debug)]
struct VersionTab {
    color: VersionTabColor,
}

impl VersionTab {
    fn new(theme: &ColorTheme) -> Self {
        Self {
            color: VersionTabColor::new(theme),
        }
    }
}

impl StatefulWidget for VersionTab {
    type State = VersionTabState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // update state
        state.height = area.height as usize;

        let mut area = area;

        for (i, lines) in state.lines.iter().enumerate().skip(state.offset) {
            let lines_count = lines.len() as u16;

            if area.height < lines_count {
                let version_paragraph = Paragraph::new("⋮").alignment(Alignment::Center);
                version_paragraph.render(area, buf);
                break;
            }

            let divider_area_height = if area.height > lines_count { 1 } else { 0 };

            let chunks = Layout::vertical([
                Constraint::Length(lines_count),
                Constraint::Length(divider_area_height),
                Constraint::Min(0),
            ])
            .split(area);
            area = chunks[2];

            let divider = Divider::default().color(self.color.divider);
            divider.render(chunks[1], buf);

            let chunks =
                Layout::horizontal([Constraint::Length(1), Constraint::Min(0)]).split(chunks[0]);

            let version_paragraph = Paragraph::new(lines.clone()).block(
                Block::default()
                    .borders(Borders::NONE)
                    .padding(Padding::left(1)),
            );
            if i == state.selected {
                let bar = Bar::default().color(self.color.selected);
                bar.render(chunks[0], buf);
            }
            version_paragraph.render(chunks[1], buf);
        }
    }
}

fn flatten_with_empty_lines(line_groups: Vec<Vec<Line>>) -> Vec<Line> {
    let n = line_groups.len();
    let mut ret: Vec<Line> = Vec::new();
    for (i, lines) in line_groups.into_iter().enumerate() {
        for line in lines {
            ret.push(line);
        }
        if i != n - 1 {
            ret.push(Line::from(""));
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use crate::set_cells;

    use super::*;
    use chrono::{DateTime, Local, NaiveDateTime};
    use ratatui::{backend::TestBackend, buffer::Buffer, crossterm::event::KeyCode, Terminal};

    #[tokio::test]
    async fn test_render_detail_tab() -> std::io::Result<()> {
        let ctx = Rc::default();
        let tx = sender();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let (items, file_detail, _file_versions, object_key) = fixtures();
            let items_len = items.len();
            let mut page = ObjectDetailPage::new(
                file_detail,
                items,
                object_key,
                ScrollListState::new(items_len),
                ctx,
                tx,
            );
            let area = Rect::new(0, 0, 60, 20);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────── 1 / 3 ┐┌────────────────────────────┐",
            "│  file1                     ││ Detail │ Version           │",
            "│  file2                     ││────────────────────────────│",
            "│  file3                     ││ Name:                      │",
            "│                            ││  file1                     │",
            "│                            ││                            │",
            "│                            ││ Size:                      │",
            "│                            ││  1.01 KiB                  │",
            "│                            ││                            │",
            "│                            ││ Last Modified:             │",
            "│                            ││  2024-01-02 13:01:02       │",
            "│                            ││                            │",
            "│                            ││ ETag:                      │",
            "│                            ││  bef684de-a260-48a4-8178-8 │",
            "│                            ││ a535ecccadb                │",
            "│                            ││                            │",
            "│                            ││ Content-Type:              │",
            "│                            ││  text/plain                │",
            "│                            ││                            │",
            "└────────────────────────────┘└────────────────────────────┘",
        ]);
        set_cells! { expected =>
            // selected item
            (2..28, [1]) => bg: Color::DarkGray, fg: Color::Black,
            // "Detail" is selected
            (32..38, [1]) => fg: Color::Cyan, modifier: Modifier::BOLD,
            // "Name" label
            (32..37, [3]) => modifier: Modifier::BOLD,
            // "Size" label
            (32..37, [6]) => modifier: Modifier::BOLD,
            // "Last Modified" label
            (32..46, [9]) => modifier: Modifier::BOLD,
            // "ETag" label
            (32..37, [12]) => modifier: Modifier::BOLD,
            // "Content-Type" label
            (32..45, [16]) => modifier: Modifier::BOLD,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_render_detail_tab_with_config() -> std::io::Result<()> {
        let tx = sender();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let (items, file_detail, _file_versions, object_key) = fixtures();
            let items_len = items.len();
            let mut ctx = AppContext::default();
            ctx.config.ui.object_detail.date_format = "%Y/%m/%d".to_string();
            let mut page = ObjectDetailPage::new(
                file_detail,
                items,
                object_key,
                ScrollListState::new(items_len),
                Rc::new(ctx),
                tx,
            );
            let area = Rect::new(0, 0, 60, 20);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────── 1 / 3 ┐┌────────────────────────────┐",
            "│  file1                     ││ Detail │ Version           │",
            "│  file2                     ││────────────────────────────│",
            "│  file3                     ││ Name:                      │",
            "│                            ││  file1                     │",
            "│                            ││                            │",
            "│                            ││ Size:                      │",
            "│                            ││  1.01 KiB                  │",
            "│                            ││                            │",
            "│                            ││ Last Modified:             │",
            "│                            ││  2024/01/02                │",
            "│                            ││                            │",
            "│                            ││ ETag:                      │",
            "│                            ││  bef684de-a260-48a4-8178-8 │",
            "│                            ││ a535ecccadb                │",
            "│                            ││                            │",
            "│                            ││ Content-Type:              │",
            "│                            ││  text/plain                │",
            "│                            ││                            │",
            "└────────────────────────────┘└────────────────────────────┘",
        ]);
        set_cells! { expected =>
            // selected item
            (2..28, [1]) => bg: Color::DarkGray, fg: Color::Black,
            // "Detail" is selected
            (32..38, [1]) => fg: Color::Cyan, modifier: Modifier::BOLD,
            // "Name" label
            (32..37, [3]) => modifier: Modifier::BOLD,
            // "Size" label
            (32..37, [6]) => modifier: Modifier::BOLD,
            // "Last Modified" label
            (32..46, [9]) => modifier: Modifier::BOLD,
            // "ETag" label
            (32..37, [12]) => modifier: Modifier::BOLD,
            // "Content-Type" label
            (32..45, [16]) => modifier: Modifier::BOLD,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_render_version_tab() -> std::io::Result<()> {
        let ctx = Rc::default();
        let tx = sender();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let (items, file_detail, file_versions, object_key) = fixtures();
            let items_len = items.len();
            let mut page = ObjectDetailPage::new(
                file_detail,
                items,
                object_key,
                ScrollListState::new(items_len),
                ctx,
                tx,
            );
            page.set_versions(file_versions);
            page.select_versions_tab();
            let area = Rect::new(0, 0, 60, 20);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────── 1 / 3 ┐┌────────────────────────────┐",
            "│  file1                     ││ Detail │ Version           │",
            "│  file2                     ││────────────────────────────│",
            "│  file3                     ││┃    Version ID: 60f36bc2-0f│",
            "│                            ││┃ Last Modified: 2024-01-02 │",
            "│                            ││┃          Size: 1.01 KiB   │",
            "│                            ││────────────────────────────│",
            "│                            ││     Version ID: 1c5d3bcc-2b│",
            "│                            ││  Last Modified: 2024-01-01 │",
            "│                            ││           Size: 1 KiB      │",
            "│                            ││────────────────────────────│",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "└────────────────────────────┘└────────────────────────────┘",
        ]);
        set_cells! { expected =>
            // selected item
            (2..28, [1]) => bg: Color::DarkGray, fg: Color::Black,
            // "Version" is selected
            (41..48, [1]) => fg: Color::Cyan, modifier: Modifier::BOLD,
            // "Version ID" label
            (33..48, [3, 7]) => modifier: Modifier::BOLD,
            // "Last Modified" label
            (33..48, [4, 8]) => modifier: Modifier::BOLD,
            // "Size" label
            (33..48, [5, 9]) => modifier: Modifier::BOLD,
            // selected bar
            ([31], [3, 4, 5]) => fg: Color::Cyan,
            // divider
            (31..59, [6, 10]) => fg: Color::DarkGray,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_render_version_tab_with_config() -> std::io::Result<()> {
        let tx = sender();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let (items, file_detail, file_versions, object_key) = fixtures();
            let items_len = items.len();
            let mut ctx = AppContext::default();
            ctx.config.ui.object_detail.date_format = "%Y/%m/%d".to_string();
            let mut page = ObjectDetailPage::new(
                file_detail,
                items,
                object_key,
                ScrollListState::new(items_len),
                Rc::new(ctx),
                tx,
            );
            page.set_versions(file_versions);
            page.select_versions_tab();
            let area = Rect::new(0, 0, 60, 20);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────── 1 / 3 ┐┌────────────────────────────┐",
            "│  file1                     ││ Detail │ Version           │",
            "│  file2                     ││────────────────────────────│",
            "│  file3                     ││┃    Version ID: 60f36bc2-0f│",
            "│                            ││┃ Last Modified: 2024/01/02 │",
            "│                            ││┃          Size: 1.01 KiB   │",
            "│                            ││────────────────────────────│",
            "│                            ││     Version ID: 1c5d3bcc-2b│",
            "│                            ││  Last Modified: 2024/01/01 │",
            "│                            ││           Size: 1 KiB      │",
            "│                            ││────────────────────────────│",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "└────────────────────────────┘└────────────────────────────┘",
        ]);
        set_cells! { expected =>
            // selected item
            (2..28, [1]) => bg: Color::DarkGray, fg: Color::Black,
            // "Version" is selected
            (41..48, [1]) => fg: Color::Cyan, modifier: Modifier::BOLD,
            // "Version ID" label
            (33..48, [3, 7]) => modifier: Modifier::BOLD,
            // "Last Modified" label
            (33..48, [4, 8]) => modifier: Modifier::BOLD,
            // "Size" label
            (33..48, [5, 9]) => modifier: Modifier::BOLD,
            // selected bar
            ([31], [3, 4, 5]) => fg: Color::Cyan,
            // divider
            (31..59, [6, 10]) => fg: Color::DarkGray,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_render_save_dialog_detail_tab() -> std::io::Result<()> {
        let ctx = Rc::default();
        let tx = sender();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let (items, file_detail, _file_versions, object_key) = fixtures();
            let items_len = items.len();
            let mut page = ObjectDetailPage::new(
                file_detail,
                items,
                object_key,
                ScrollListState::new(items_len),
                ctx,
                tx,
            );
            page.open_save_dialog();
            let area = Rect::new(0, 0, 60, 20);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────── 1 / 3 ┐┌────────────────────────────┐",
            "│  file1                     ││ Detail │ Version           │",
            "│  file2                     ││────────────────────────────│",
            "│  file3                     ││ Name:                      │",
            "│                            ││  file1                     │",
            "│                            ││                            │",
            "│                            ││ Size:                      │",
            "│                            ││  1.01 KiB                  │",
            "│         ╭Save As───────────────────────────────╮         │",
            "│         │                                      │         │",
            "│         ╰──────────────────────────────────────╯ 2       │",
            "│                            ││                            │",
            "│                            ││ ETag:                      │",
            "│                            ││  bef684de-a260-48a4-8178-8 │",
            "│                            ││ a535ecccadb                │",
            "│                            ││                            │",
            "│                            ││ Content-Type:              │",
            "│                            ││  text/plain                │",
            "│                            ││                            │",
            "└────────────────────────────┘└────────────────────────────┘",
        ]);
        set_cells! { expected =>
            // selected item
            (2..28, [1]) => bg: Color::DarkGray, fg: Color::Black,
            // "Detail" is selected
            (32..38, [1]) => fg: Color::Cyan, modifier: Modifier::BOLD,
            // "Name" label
            (32..37, [3]) => modifier: Modifier::BOLD,
            // "Size" label
            (32..37, [6]) => modifier: Modifier::BOLD,
            // "ETag" label
            (32..37, [12]) => modifier: Modifier::BOLD,
            // "Content-Type" label
            (32..45, [16]) => modifier: Modifier::BOLD,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_render_copy_detail_dialog_detail_tab() -> std::io::Result<()> {
        let ctx = Rc::default();
        let tx = sender();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let (items, file_detail, _file_versions, object_key) = fixtures();
            let items_len = items.len();
            let mut page = ObjectDetailPage::new(
                file_detail,
                items,
                object_key,
                ScrollListState::new(items_len),
                ctx,
                tx,
            );
            page.open_copy_detail_dialog();
            let area = Rect::new(0, 0, 60, 20);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────── 1 / 3 ┐┌────────────────────────────┐",
            "│  file1                     ││ Detail │ Version           │",
            "│  file2                     ││────────────────────────────│",
            "│ ╭Copy──────────────────────────────────────────────────╮ │",
            "│ │ Name:                                                │ │",
            "│ │   file1                                              │ │",
            "│ │ Key:                                                 │ │",
            "│ │   file1                                              │ │",
            "│ │ S3 URI:                                              │ │",
            "│ │   s3://bucket-1/file1                                │ │",
            "│ │ ARN:                                                 │ │",
            "│ │   arn:aws:s3:::bucket-1/file1                        │ │",
            "│ │ Object URL:                                          │ │",
            "│ │   https://bucket-1.s3.ap-northeast-1.amazonaws.com/f │ │",
            "│ │ ETag:                                                │ │",
            "│ │   bef684de-a260-48a4-8178-8a535ecccadb               │ │",
            "│ ╰──────────────────────────────────────────────────────╯ │",
            "│                            ││  text/plain                │",
            "│                            ││                            │",
            "└────────────────────────────┘└────────────────────────────┘",
        ]);
        set_cells! { expected =>
            // selected item
            (2..28, [1]) => bg: Color::DarkGray, fg: Color::Black,
            // "Detail" is selected
            (32..38, [1]) => fg: Color::Cyan, modifier: Modifier::BOLD,
            // "Name" label
            (4..9, [4]) => modifier: Modifier::BOLD,
            // "Key" label
            (4..8, [6]) => modifier: Modifier::BOLD,
            // "S3 URI" label
            (4..11, [8]) => modifier: Modifier::BOLD,
            // "ARN" label
            (4..8, [10]) => modifier: Modifier::BOLD,
            // "Object URL" label
            (4..15, [12]) => modifier: Modifier::BOLD,
            // "ETag" label
            (4..9, [14]) => modifier: Modifier::BOLD,
            // "Name" is selected
            (4..56, [4, 5]) => fg: Color::Cyan,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_render_copy_detail_dialog_version_tab() -> std::io::Result<()> {
        let ctx = Rc::default();
        let tx = sender();
        let mut terminal = setup_terminal()?;

        let (items, file_detail, file_versions, object_key) = fixtures();
        let items_len = items.len();
        let mut page = ObjectDetailPage::new(
            file_detail,
            items,
            object_key,
            ScrollListState::new(items_len),
            ctx,
            tx,
        );
        page.set_versions(file_versions);
        page.select_versions_tab();

        let area = Rect::new(0, 0, 60, 20);

        // Call render once to update the StatefulWidget
        terminal.draw(|f| {
            page.render(f, area);
        })?;

        page.handle_key(
            vec![UserEvent::ObjectDetailDown],
            KeyEvent::from(KeyCode::Char('j')),
        );
        page.open_copy_detail_dialog();

        terminal.draw(|f| {
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────── 1 / 3 ┐┌────────────────────────────┐",
            "│  file1                     ││ Detail │ Version           │",
            "│  file2                     ││────────────────────────────│",
            "│ ╭Copy──────────────────────────────────────────────────╮ │",
            "│ │ Name:                                                │ │",
            "│ │   file1                                              │ │",
            "│ │ Key:                                                 │ │",
            "│ │   file1                                              │ │",
            "│ │ S3 URI:                                              │ │",
            "│ │   s3://bucket-1/file1?versionId=1c5d3bcc-2bb3-4cd5-8 │ │",
            "│ │ ARN:                                                 │ │",
            "│ │   arn:aws:s3:::bucket-1/file1                        │ │",
            "│ │ Object URL:                                          │ │",
            "│ │   https://bucket-1.s3.ap-northeast-1.amazonaws.com/f │ │",
            "│ │ ETag:                                                │ │",
            "│ │   6c5db847-d206-4a27-9723-713e3a6cad86               │ │",
            "│ ╰──────────────────────────────────────────────────────╯ │",
            "│                            ││                            │",
            "│                            ││                            │",
            "└────────────────────────────┘└────────────────────────────┘",
        ]);
        set_cells! { expected =>
            // selected item
            (2..28, [1]) => bg: Color::DarkGray, fg: Color::Black,
            // "Version" is selected
            (41..48, [1]) => fg: Color::Cyan, modifier: Modifier::BOLD,
            // "Name" label
            (4..9, [4]) => modifier: Modifier::BOLD,
            // "Key" label
            (4..8, [6]) => modifier: Modifier::BOLD,
            // "S3 URI" label
            (4..11, [8]) => modifier: Modifier::BOLD,
            // "ARN" label
            (4..8, [10]) => modifier: Modifier::BOLD,
            // "Object URL" label
            (4..15, [12]) => modifier: Modifier::BOLD,
            // "ETag" label
            (4..9, [14]) => modifier: Modifier::BOLD,
            // "Name" is selected
            (4..56, [4, 5]) => fg: Color::Cyan,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    fn setup_terminal() -> std::io::Result<Terminal<TestBackend>> {
        let backend = TestBackend::new(60, 20);
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

    fn fixtures() -> (Vec<ObjectItem>, FileDetail, Vec<FileVersion>, ObjectKey) {
        let items = vec![
            object_file_item("file1", 1024 + 10, "2024-01-02 13:01:02"),
            object_file_item("file2", 1024 * 999, "2023-12-31 09:00:00"),
            object_file_item("file3", 1024, "2024-01-03 12:59:59"),
        ];
        let file_detail = FileDetail {
            name: "file1".to_string(),
            size_byte: 1024 + 10,
            last_modified: parse_datetime("2024-01-02 13:01:02"),
            e_tag: "bef684de-a260-48a4-8178-8a535ecccadb".to_string(),
            content_type: "text/plain".to_string(),
            storage_class: "STANDARD".to_string(),
            key: "file1".to_string(),
            s3_uri: "s3://bucket-1/file1".to_string(),
            arn: "arn:aws:s3:::bucket-1/file1".to_string(),
            object_url: "https://bucket-1.s3.ap-northeast-1.amazonaws.com/file1".to_string(),
        };
        let file_versions = vec![
            FileVersion {
                version_id: "60f36bc2-0f38-47b8-9bf0-e24e334b86d5".to_string(),
                size_byte: 1024 + 10,
                last_modified: parse_datetime("2024-01-02 13:01:02"),
                e_tag: "bef684de-a260-48a4-8178-8a535ecccadb".to_string(),
                is_latest: true,
            },
            FileVersion {
                version_id: "1c5d3bcc-2bb3-4cd5-875f-a95a6ae53f65".to_string(),
                size_byte: 1024,
                last_modified: parse_datetime("2024-01-01 23:59:59"),
                e_tag: "6c5db847-d206-4a27-9723-713e3a6cad86".to_string(),
                is_latest: false,
            },
        ];
        let object_key = ObjectKey {
            bucket_name: "test-bucket".to_string(),
            object_path: vec!["path".to_string(), "to".to_string(), "file1".to_string()],
        };
        (items, file_detail, file_versions, object_key)
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
