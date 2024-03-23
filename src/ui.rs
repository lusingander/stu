use chrono::{DateTime, Local};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Margin,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        block::Title, Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph, Tabs,
        Widget, Wrap,
    },
    Frame,
};

use crate::{
    app::{
        App, CopyDetailViewItemType, CopyDetailViewState, DetailSaveViewState, DetailViewState,
        Notification, PreviewSaveViewState, PreviewViewState, ViewState,
    },
    item::{BucketItem, FileDetail, FileVersion, ObjectItem},
    util::{self, digits},
};

const APP_NAME: &str = "STU";

const SELECTED_COLOR: Color = Color::Cyan;
const SELECTED_DISABLED_COLOR: Color = Color::DarkGray;
const SELECTED_ITEM_TEXT_COLOR: Color = Color::Black;
const DIVIDER_COLOR: Color = Color::DarkGray;
const LINK_TEXT_COLOR: Color = Color::Blue;
const PREVIEW_LINE_NUMBER_COLOR: Color = Color::DarkGray;
const SHORT_HELP_COLOR: Color = Color::DarkGray;
const INFO_STATUS_COLOR: Color = Color::Blue;
const SUCCESS_STATUS_COLOR: Color = Color::Green;
const ERROR_STATUS_COLOR: Color = Color::Red;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const APP_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::new(
        Direction::Vertical,
        [Constraint::Min(0), Constraint::Length(2)],
    )
    .split(f.size());

    render_content(f, chunks[0], app);
    render_footer(f, chunks[1], app);
    render_loading_dialog(f, app);
}

fn render_content(f: &mut Frame, area: Rect, app: &App) {
    match &app.app_view_state.view_state {
        ViewState::Initializing => render_initializing_view(f, area, app),
        ViewState::BucketList => render_bucket_list_view(f, area, app),
        ViewState::ObjectList => render_object_list_view(f, area, app),
        ViewState::Detail(vs) => render_detail_view(f, area, app, vs),
        ViewState::DetailSave(vs) => render_detail_save_view(f, area, app, vs),
        ViewState::CopyDetail(vs) => render_copy_detail_view(f, area, app, vs),
        ViewState::Preview(vs) => render_preview_view(f, area, app, vs),
        ViewState::PreviewSave(vs) => render_preview_save_view(f, area, app, vs),
        ViewState::Help(before) => render_help_view(f, area, app, before),
    }
}

fn render_initializing_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::new(
        Direction::Vertical,
        [Constraint::Length(3), Constraint::Min(0)],
    )
    .split(area);

    let header = build_header(app, chunks[0]);
    f.render_widget(header, chunks[0]);

    let content = Block::bordered();
    f.render_widget(content, chunks[1]);
}

fn render_bucket_list_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::new(
        Direction::Vertical,
        [Constraint::Length(3), Constraint::Min(0)],
    )
    .split(area);

    let header = build_header(app, chunks[0]);
    f.render_widget(header, chunks[0]);

    let current_items = app.bucket_items();
    let list_state = ListViewState {
        current_selected: app.app_view_state.current_list_state().selected,
        current_offset: app.app_view_state.current_list_state().offset,
    };
    let styles = ListItemStyles {
        selected_bg_color: SELECTED_COLOR,
        selected_fg_color: SELECTED_ITEM_TEXT_COLOR,
    };
    let list_items =
        build_list_items_from_bucket_items(&current_items, list_state, chunks[1], styles);
    let list = build_list(list_items, current_items.len(), list_state.current_selected);
    f.render_widget(list, chunks[1]);

    render_list_scroll_bar(f, chunks[1], list_state, current_items.len());
}

fn render_object_list_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::new(
        Direction::Vertical,
        [Constraint::Length(3), Constraint::Min(0)],
    )
    .split(area);

    let header = build_header(app, chunks[0]);
    f.render_widget(header, chunks[0]);

    let current_items = app.current_object_items();
    let list_state = ListViewState {
        current_selected: app.app_view_state.current_list_state().selected,
        current_offset: app.app_view_state.current_list_state().offset,
    };
    let styles = ListItemStyles {
        selected_bg_color: SELECTED_COLOR,
        selected_fg_color: SELECTED_ITEM_TEXT_COLOR,
    };
    let list_items =
        build_list_items_from_object_items(&current_items, list_state, chunks[1], styles, true);
    let list = build_list(list_items, current_items.len(), list_state.current_selected);
    f.render_widget(list, chunks[1]);

    render_list_scroll_bar(f, chunks[1], list_state, current_items.len());
}

fn render_list_scroll_bar(
    f: &mut Frame,
    area: Rect,
    list_state: ListViewState,
    current_items_len: usize,
) {
    // implemented independently to calculate based on offset position
    let area = area.inner(&Margin::new(2, 1));
    let scrollbar_area = Rect::new(area.right(), area.top(), 1, area.height);
    let scrollbar_area_h = scrollbar_area.height;
    let items_len = current_items_len as u16;
    let offset = list_state.current_offset as u16;

    if items_len > scrollbar_area_h {
        let scrollbar_h = calc_scrollbar_height(scrollbar_area_h, items_len);
        let scrollbar_t = calc_scrollbar_top(scrollbar_area_h, scrollbar_h, offset, items_len);

        let buf = f.buffer_mut();
        let x = scrollbar_area.x;
        for h in 0..scrollbar_h {
            let y = scrollbar_area.y + scrollbar_t + h;
            buf.get_mut(x, y).set_char('│'); // use '┃' or '║' instead...?
        }
    }
}

fn calc_scrollbar_height(scrollbar_area_h: u16, items_len: u16) -> u16 {
    let sah = scrollbar_area_h as f64;
    let il = items_len as f64;
    let h = sah * (sah / il);
    (h as u16).max(1)
}

fn calc_scrollbar_top(scrollbar_area_h: u16, scrollbar_h: u16, offset: u16, items_len: u16) -> u16 {
    let sah = scrollbar_area_h as f64;
    let sh = scrollbar_h as f64;
    let o = offset as f64;
    let il = items_len as f64;
    let t = ((sah - sh) * o) / (il - sah);
    t as u16
}

fn render_detail_view(f: &mut Frame, area: Rect, app: &App, vs: &DetailViewState) {
    let chunks = Layout::new(
        Direction::Vertical,
        [Constraint::Length(3), Constraint::Min(0)],
    )
    .split(area);

    let header = build_header(app, chunks[0]);
    f.render_widget(header, chunks[0]);

    let chunks = Layout::new(
        Direction::Horizontal,
        Constraint::from_percentages([50, 50]),
    )
    .split(chunks[1]);

    let current_items = app.current_object_items();
    let list_state = ListViewState {
        current_selected: app.app_view_state.current_list_state().selected,
        current_offset: app.app_view_state.current_list_state().offset,
    };
    let styles = ListItemStyles {
        selected_bg_color: SELECTED_DISABLED_COLOR,
        selected_fg_color: SELECTED_ITEM_TEXT_COLOR,
    };
    let list_items =
        build_list_items_from_object_items(&current_items, list_state, chunks[0], styles, false);
    let list = build_list(list_items, current_items.len(), list_state.current_selected);
    f.render_widget(list, chunks[0]);

    let block = build_file_detail_block();
    f.render_widget(block, chunks[1]);

    let chunks = Layout::new(
        Direction::Vertical,
        [Constraint::Length(2), Constraint::Min(0)],
    )
    .margin(1)
    .split(chunks[1]);

    let tabs = build_file_detail_tabs(vs);
    f.render_widget(tabs, chunks[0]);

    match vs {
        DetailViewState::Detail => {
            let current_file_detail = app.get_current_file_detail().unwrap();
            let detail = build_file_detail(current_file_detail);
            f.render_widget(detail, chunks[1]);
        }
        DetailViewState::Version => {
            let current_file_versions = app.get_current_file_versions().unwrap();
            let versions = build_file_versions(current_file_versions, chunks[1].width);
            f.render_widget(versions, chunks[1]);
        }
    }
}

fn render_detail_save_view(f: &mut Frame, area: Rect, app: &App, vs: &DetailSaveViewState) {
    render_detail_view(f, area, app, &vs.before);
    render_save_object_as_dialog(f, area, &vs.input, vs.cursor);
}

fn render_copy_detail_view(f: &mut Frame, area: Rect, app: &App, vs: &CopyDetailViewState) {
    render_detail_view(f, area, app, &vs.before);

    let current_file_detail = app.get_current_file_detail().unwrap();
    render_copy_details_dialog(f, area, vs, current_file_detail);
}

fn render_copy_details_dialog(
    f: &mut Frame,
    area: Rect,
    vs: &CopyDetailViewState,
    detail: &FileDetail,
) {
    let selected = vs.selected as usize;
    let list_items: Vec<ListItem> = [
        (CopyDetailViewItemType::Key, &detail.key),
        (CopyDetailViewItemType::S3Uri, &detail.s3_uri),
        (CopyDetailViewItemType::Arn, &detail.arn),
        (CopyDetailViewItemType::ObjectUrl, &detail.object_url),
        (CopyDetailViewItemType::Etag, &detail.e_tag),
    ]
    .iter()
    .enumerate()
    .map(|(i, (tp, value))| {
        let item = ListItem::new(vec![
            Line::from(format!("{}:", tp.name()).add_modifier(Modifier::BOLD)),
            Line::from(format!("  {}", value)),
        ]);
        if i == selected {
            item.fg(SELECTED_COLOR)
        } else {
            item
        }
    })
    .collect();

    let dialog_width = (area.width - 4).min(80);
    let dialog_height = 2 * 5 /* list */ + 2 /* border */;
    let area = calc_centered_dialog_rect(area, dialog_width, dialog_height);

    let title = Title::from("Copy");
    let list = List::new(list_items).block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .title(title)
            .padding(Padding::horizontal(1)),
    );
    render_dialog(f, list, area);
}

fn render_preview_view(f: &mut Frame, area: Rect, app: &App, vs: &PreviewViewState) {
    let chunks = Layout::new(
        Direction::Vertical,
        [Constraint::Length(3), Constraint::Min(0)],
    )
    .split(area);

    let header = build_header(app, chunks[0]);
    f.render_widget(header, chunks[0]);

    let content = build_preview(app, vs, chunks[1]);
    f.render_widget(content, chunks[1]);
}

fn render_preview_save_view(f: &mut Frame, area: Rect, app: &App, vs: &PreviewSaveViewState) {
    render_preview_view(f, area, app, &vs.before);
    render_save_object_as_dialog(f, area, &vs.input, vs.cursor);
}

fn render_help_view(f: &mut Frame, area: Rect, app: &App, before: &ViewState) {
    let content = build_help(before, area, app);
    f.render_widget(content, area);
}

fn render_save_object_as_dialog(f: &mut Frame, area: Rect, input: &str, cursor: u16) {
    let dialog_width = (area.width - 4).min(40);
    let dialog_height = 1 + 2 /* border */;
    let area = calc_centered_dialog_rect(area, dialog_width, dialog_height);

    let max_width = dialog_width - 2 /* border */- 2/* pad */;
    let input_width = input.len().saturating_sub(max_width as usize);
    let input_view: &str = &input[input_width..];

    let title = Title::from("Save As");
    let dialog = Paragraph::new(input_view).block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .title(title)
            .padding(Padding::horizontal(1)),
    );
    render_dialog(f, dialog, area);

    let cursor_x = area.x + cursor.min(max_width) + 1 /* border */ + 1/* pad */;
    let cursor_y = area.y + 1;
    f.set_cursor(cursor_x, cursor_y);
}

fn build_header(app: &App, area: Rect) -> Paragraph {
    let area = area.inner(&Margin::new(1, 1));
    let pad = Padding::horizontal(1);

    let max_width = (area.width - pad.left - pad.right) as usize;
    let delimiter = " / ";
    let ellipsis = "...";
    let breadcrumbs = app.breadcrumb_strs();

    let current_key = if breadcrumbs.is_empty() {
        "".to_string()
    } else {
        let current_key = breadcrumbs.join(delimiter);
        if current_key.len() <= max_width {
            current_key
        } else {
            //   string: <bucket> / ... / s1 / s2 / s3 / s4 / s5
            // priority:        1 /   0 /  4 /  3 /  2 /  1 /  0
            let bl = breadcrumbs.len();
            let mut bs: Vec<(String, usize)> = breadcrumbs
                .into_iter()
                .enumerate()
                .map(|(i, p)| (p, bl - i - 1))
                .collect();
            bs.insert(1, (ellipsis.to_string(), 0));
            bs.first_mut().unwrap().1 = 1;
            bs.last_mut().unwrap().1 = 0;

            let keys = util::prune_strings_to_fit_width(&bs, max_width, delimiter);
            keys.join(delimiter)
        }
    };

    Paragraph::new(Span::styled(current_key, Style::default())).block(
        Block::bordered()
            .title(Span::styled(APP_NAME, Style::default()))
            .padding(pad),
    )
}

fn build_list(list_items: Vec<ListItem>, total_count: usize, current_selected: usize) -> List {
    let title = format_list_count(total_count, current_selected);
    List::new(list_items).block(
        Block::bordered()
            .title(title)
            .title_alignment(Alignment::Right)
            .padding(Padding::horizontal(1)),
    )
}

#[derive(Clone, Copy, Debug)]
struct ListViewState {
    current_selected: usize,
    current_offset: usize,
}

#[derive(Clone, Copy, Debug)]
struct ListItemStyles {
    selected_bg_color: Color,
    selected_fg_color: Color,
}

fn build_list_items_from_bucket_items(
    current_items: &[BucketItem],
    list_state: ListViewState,
    area: Rect,
    styles: ListItemStyles,
) -> Vec<ListItem> {
    let show_item_count = (area.height as usize) - 2 /* border */;
    current_items
        .iter()
        .skip(list_state.current_offset)
        .take(show_item_count)
        .enumerate()
        .map(|(idx, item)| build_list_item_from_bucket_item(idx, item, list_state, area, styles))
        .collect()
}

fn build_list_item_from_bucket_item(
    idx: usize,
    item: &BucketItem,
    list_state: ListViewState,
    area: Rect,
    styles: ListItemStyles,
) -> ListItem {
    let content = format_bucket_item(&item.name, area.width);
    let style = Style::default();
    let span = Span::styled(content, style);
    if idx + list_state.current_offset == list_state.current_selected {
        ListItem::new(span).style(
            Style::default()
                .bg(styles.selected_bg_color)
                .fg(styles.selected_fg_color),
        )
    } else {
        ListItem::new(span)
    }
}

fn build_list_items_from_object_items(
    current_items: &[ObjectItem],
    list_state: ListViewState,
    area: Rect,
    styles: ListItemStyles,
    show_file_detail: bool,
) -> Vec<ListItem> {
    let show_item_count = (area.height as usize) - 2 /* border */;
    current_items
        .iter()
        .skip(list_state.current_offset)
        .take(show_item_count)
        .enumerate()
        .map(|(idx, item)| {
            build_list_item_from_object_item(idx, item, list_state, area, styles, show_file_detail)
        })
        .collect()
}

fn build_list_item_from_object_item(
    idx: usize,
    item: &ObjectItem,
    list_state: ListViewState,
    area: Rect,
    styles: ListItemStyles,
    show_file_detail: bool,
) -> ListItem {
    let content = match item {
        ObjectItem::Dir { name, .. } => {
            let content = format_dir_item(name, area.width);
            let style = Style::default().add_modifier(Modifier::BOLD);
            Span::styled(content, style)
        }
        ObjectItem::File {
            name,
            size_byte,
            last_modified,
            ..
        } => {
            let content =
                format_file_item(name, size_byte, last_modified, area.width, show_file_detail);
            let style = Style::default();
            Span::styled(content, style)
        }
    };
    if idx + list_state.current_offset == list_state.current_selected {
        ListItem::new(content).style(
            Style::default()
                .bg(styles.selected_bg_color)
                .fg(styles.selected_fg_color),
        )
    } else {
        ListItem::new(content)
    }
}

fn format_bucket_item(name: &str, width: u16) -> String {
    let name_w: usize = (width as usize) - 2 /* spaces */ - 2 /* border */;
    format!(" {:<name_w$} ", name, name_w = name_w)
}

fn format_dir_item(name: &str, width: u16) -> String {
    let name_w: usize = (width as usize) - 2 /* spaces */ - 2 /* border */;
    let name = format!("{}/", name);
    format!(" {:<name_w$} ", name, name_w = name_w)
}

fn format_file_item(
    name: &str,
    size_byte: &i64,
    last_modified: &DateTime<Local>,
    width: u16,
    show_file_detail: bool,
) -> String {
    if show_file_detail {
        let size = format_size_byte(*size_byte);
        let date = format_datetime(last_modified);
        let date_w: usize = 19;
        let size_w: usize = 10;
        let name_w: usize = (width as usize) - date_w - size_w - 10 /* spaces */ - 4 /* border + space */;
        format!(
            " {:<name_w$}    {:<date_w$}    {:>size_w$} ",
            name,
            date,
            size,
            name_w = name_w,
            date_w = date_w,
            size_w = size_w
        )
    } else {
        let name_w: usize = (width as usize) - 2 /* spaces */ - 4 /* border */;
        format!(" {:<name_w$} ", name, name_w = name_w)
    }
}

fn format_list_count(total_count: usize, current_selected: usize) -> String {
    if total_count == 0 {
        String::new()
    } else {
        format_count(current_selected + 1, total_count)
    }
}

fn format_count(selected: usize, total: usize) -> String {
    let digits = digits(total);
    format!(" {:>digits$} / {} ", selected, total)
}

fn build_file_detail_block() -> Block<'static> {
    Block::bordered()
}

fn build_file_detail_tabs(selected: &DetailViewState) -> Tabs {
    let tabs = vec![Line::from("Detail"), Line::from("Version")];
    Tabs::new(tabs)
        .select(*selected as usize)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(SELECTED_COLOR),
        )
        .block(Block::default().borders(Borders::BOTTOM))
}

fn build_file_detail(detail: &FileDetail) -> Paragraph {
    let details = [
        ("Name:", &detail.name),
        ("Size:", &format_size_byte(detail.size_byte)),
        ("Last Modified:", &format_datetime(&detail.last_modified)),
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

    let content = flatten_with_empty_lines(details, false);
    Paragraph::new(content)
        .block(Block::default().padding(Padding::horizontal(1)))
        .wrap(Wrap { trim: false })
}

fn format_size_byte(size_byte: i64) -> String {
    humansize::format_size_i(size_byte, humansize::BINARY)
}

#[cfg(not(feature = "imggen"))]
fn format_datetime(datetime: &DateTime<Local>) -> String {
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(feature = "imggen")]
fn format_datetime(_datetime: &DateTime<Local>) -> String {
    String::from("2024-01-02 13:04:05")
}

#[cfg(not(feature = "imggen"))]
fn format_version(version: &str) -> &str {
    version
}

#[cfg(feature = "imggen")]
fn format_version(_version: &str) -> &str {
    "GeJeVLwoQlknMCcSa"
}

fn build_file_versions(versions: &[FileVersion], width: u16) -> List {
    let list_items: Vec<ListItem> = versions
        .iter()
        .map(|v| {
            let content = vec![
                Line::from(vec![
                    "    Version ID: ".add_modifier(Modifier::BOLD),
                    Span::raw(format_version(&v.version_id)),
                ]),
                Line::from(vec![
                    " Last Modified: ".add_modifier(Modifier::BOLD),
                    Span::raw(format_datetime(&v.last_modified)),
                ]),
                Line::from(vec![
                    "          Size: ".add_modifier(Modifier::BOLD),
                    Span::raw(format_size_byte(v.size_byte)),
                ]),
                Line::from("-".repeat(width as usize).fg(DIVIDER_COLOR)),
            ];
            ListItem::new(content)
        })
        .collect();
    List::new(list_items)
        .block(Block::default())
        .highlight_style(Style::default().bg(SELECTED_COLOR))
}

fn build_preview<'a>(app: &'a App, vs: &'a PreviewViewState, area: Rect) -> Paragraph<'a> {
    let area = area.inner(&Margin::new(1, 1)); // border

    let preview_max_digits = vs.preview_max_digits;
    let show_lines_count = area.height as usize;
    let content_max_width = (area.width as usize) - preview_max_digits - 3 /* pad */;

    let content: Vec<Line> = ((vs.offset + 1)..)
        .zip(vs.preview.iter().skip(vs.offset))
        .flat_map(|(n, s)| {
            let ss = textwrap::wrap(s, content_max_width);
            ss.into_iter().enumerate().map(move |(i, s)| {
                let line_number = if i == 0 {
                    format!("{:>preview_max_digits$}", n)
                } else {
                    " ".repeat(preview_max_digits)
                };
                Line::from(vec![
                    line_number.fg(PREVIEW_LINE_NUMBER_COLOR),
                    " ".into(),
                    s.into(),
                ])
            })
        })
        .take(show_lines_count)
        .collect();

    let current_file_detail = app.get_current_file_detail().unwrap();
    let title = format!("Preview [{}]", &current_file_detail.name);

    Paragraph::new(content).block(
        Block::bordered()
            .title(title)
            .padding(Padding::horizontal(1)),
    )
}

fn build_help<'a>(before: &'a ViewState, area: Rect, app: &'a App) -> Paragraph<'a> {
    let area = area.inner(&Margin::new(1, 1)); // border
    let w: usize = area.width as usize;

    let app_details = vec![
        Line::from(format!(" {} - {}", APP_NAME, APP_DESCRIPTION)),
        Line::from(format!(" Version: {}", APP_VERSION)),
        Line::from(format!(" {}", APP_HOMEPAGE).fg(LINK_TEXT_COLOR)),
        Line::from("-".repeat(w).fg(DIVIDER_COLOR)),
    ];
    let app_detail = with_empty_lines(app_details).into_iter();

    let helps = app.action_manager.helps(before);
    let max_help_width: usize = 80;
    let max_width = max_help_width.min(w) - 2;
    let help = build_help_lines(helps, max_width);

    let content: Vec<Line> = app_detail.chain(help).collect();
    Paragraph::new(content).block(
        Block::bordered()
            .title(APP_NAME)
            .padding(Padding::uniform(1)),
    )
}

fn build_help_lines(helps: &[String], max_width: usize) -> Vec<Line> {
    let delimiter = ",  ";
    let word_groups = util::group_strings_to_fit_width(helps, max_width, delimiter);
    let lines: Vec<Line> = word_groups
        .iter()
        .map(|ws| Line::from(format!(" {} ", ws.join(delimiter))))
        .collect();
    with_empty_lines(lines)
}

fn build_short_help(app: &App, width: u16) -> Paragraph {
    let helps = app
        .action_manager
        .short_helps(&app.app_view_state.view_state);
    let pad = Padding::horizontal(2);
    let max_width = (width - pad.left - pad.right) as usize;
    let help = build_short_help_string(helps, max_width);
    Paragraph::new(help.fg(SHORT_HELP_COLOR)).block(Block::default().padding(pad))
}

fn build_short_help_string(helps: &[(String, usize)], max_width: usize) -> String {
    let delimiter = ", ";
    let ss = util::prune_strings_to_fit_width(helps, max_width, delimiter);
    ss.join(delimiter)
}

fn build_info_status(msg: &str) -> Paragraph {
    Paragraph::new(msg.fg(INFO_STATUS_COLOR))
        .block(Block::default().padding(Padding::horizontal(2)))
}

fn build_success_status(msg: &str) -> Paragraph {
    Paragraph::new(msg.add_modifier(Modifier::BOLD).fg(SUCCESS_STATUS_COLOR))
        .block(Block::default().padding(Padding::horizontal(2)))
}

fn build_error_status(err: &str) -> Paragraph {
    let err = format!("ERROR: {}", err);
    Paragraph::new(err.add_modifier(Modifier::BOLD).fg(ERROR_STATUS_COLOR))
        .block(Block::default().padding(Padding::horizontal(2)))
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    match &app.app_view_state.notification {
        Notification::Info(msg) => {
            let msg = build_info_status(msg);
            f.render_widget(msg, area);
        }
        Notification::Success(msg) => {
            let msg = build_success_status(msg);
            f.render_widget(msg, area);
        }
        Notification::Error(msg) => {
            let msg = build_error_status(msg);
            f.render_widget(msg, area);
        }
        Notification::None => {
            let help = build_short_help(app, area.width);
            f.render_widget(help, area);
        }
    }
}

fn build_loading_dialog(msg: &str) -> Paragraph {
    let text = Line::from(msg.add_modifier(Modifier::BOLD));
    Paragraph::new(text).alignment(Alignment::Center).block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::vertical(1)),
    )
}

fn render_loading_dialog(f: &mut Frame, app: &App) {
    if app.app_view_state.is_loading {
        let loading = build_loading_dialog("Loading...");
        let area = calc_centered_dialog_rect(f.size(), 30, 5);
        render_dialog(f, loading, area);
    }
}

fn calc_centered_dialog_rect(r: Rect, dialog_width: u16, dialog_height: u16) -> Rect {
    let vertical_pad = (r.height - dialog_height) / 2;
    let vertical_layout = Layout::new(
        Direction::Vertical,
        Constraint::from_lengths([vertical_pad, dialog_height, vertical_pad]),
    )
    .split(r);

    let horizontal_pad = (r.width - dialog_width) / 2;
    Layout::new(
        Direction::Horizontal,
        Constraint::from_lengths([horizontal_pad, dialog_width, horizontal_pad]),
    )
    .split(vertical_layout[1])[1]
}

fn render_dialog<W: Widget>(f: &mut Frame, dialog: W, area: Rect) {
    f.render_widget(Clear, outer_rect(area, &Margin::new(1, 0)));
    f.render_widget(dialog, area);
}

fn outer_rect(r: Rect, margin: &Margin) -> Rect {
    let doubled_margin_horizontal = margin.horizontal.saturating_mul(2);
    let doubled_margin_vertical = margin.vertical.saturating_mul(2);
    Rect {
        x: r.x.saturating_sub(margin.horizontal),
        y: r.y.saturating_sub(margin.vertical),
        width: r.width.saturating_add(doubled_margin_horizontal),
        height: r.height.saturating_add(doubled_margin_vertical),
    }
}

fn with_empty_lines(lines: Vec<Line>) -> Vec<Line> {
    let line_groups = lines.into_iter().map(|l| vec![l]).collect();
    flatten_with_empty_lines(line_groups, true)
}

fn flatten_with_empty_lines(line_groups: Vec<Vec<Line>>, add_to_end: bool) -> Vec<Line> {
    let n = line_groups.len();
    let mut ret: Vec<Line> = Vec::new();
    for (i, lines) in line_groups.into_iter().enumerate() {
        for line in lines {
            ret.push(line);
        }
        if add_to_end || i != n - 1 {
            ret.push(Line::from(""));
        }
    }
    ret
}
