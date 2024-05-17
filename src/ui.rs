use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Stylize},
    text::Line,
    widgets::{Block, BorderType, Padding, Paragraph},
    Frame,
};

use crate::{
    app::{
        App, CopyDetailViewState, DetailSaveViewState, DetailViewState, Notification,
        PreviewSaveViewState, PreviewViewState, ViewState,
    },
    pages::{
        bucket_list::BucketListPage, help::HelpPage, initializing::InitializingPage,
        object_detail::ObjectDetailPage, object_list::ObjectListPage,
        object_preview::ObjectPreviewPage,
    },
    util,
    widget::{Dialog, Header},
};

const SHORT_HELP_COLOR: Color = Color::DarkGray;
const INFO_STATUS_COLOR: Color = Color::Blue;
const SUCCESS_STATUS_COLOR: Color = Color::Green;
const ERROR_STATUS_COLOR: Color = Color::Red;

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(2)]).split(f.size());

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
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

    let header = Header::new(app.breadcrumb_strs());
    f.render_widget(header, chunks[0]);

    let mut page = InitializingPage::new();
    page.render(f, chunks[1]);
}

fn render_bucket_list_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

    let header = Header::new(app.breadcrumb_strs());
    f.render_widget(header, chunks[0]);

    let current_items = app.bucket_items();

    let mut page = BucketListPage::new(current_items, *app.app_view_state.current_list_state());
    page.render(f, chunks[1]);
}

fn render_object_list_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

    let header = Header::new(app.breadcrumb_strs());
    f.render_widget(header, chunks[0]);

    let current_items = app.current_object_items();

    let mut page = ObjectListPage::new(current_items, *app.app_view_state.current_list_state());
    page.render(f, chunks[1]);
}

fn render_detail_view(f: &mut Frame, area: Rect, app: &App, vs: &DetailViewState) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

    let header = Header::new(app.breadcrumb_strs());
    f.render_widget(header, chunks[0]);

    let current_items = app.current_object_items();
    let current_file_detail = app.get_current_file_detail().unwrap();
    let current_file_versions = app.get_current_file_versions().unwrap();

    let mut page = ObjectDetailPage::new(
        current_items,
        current_file_detail.clone(),
        current_file_versions.clone(),
        *vs,
        None,
        None,
        *app.app_view_state.current_list_state(),
    );
    page.render(f, chunks[1]);
}

fn render_detail_save_view(f: &mut Frame, area: Rect, app: &App, vs: &DetailSaveViewState) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

    let header = Header::new(app.breadcrumb_strs());
    f.render_widget(header, chunks[0]);

    let current_items = app.current_object_items();
    let current_file_detail = app.get_current_file_detail().unwrap();
    let current_file_versions = app.get_current_file_versions().unwrap();

    let mut page = ObjectDetailPage::new(
        current_items,
        current_file_detail.clone(),
        current_file_versions.clone(),
        vs.before,
        Some(vs.clone()),
        None,
        *app.app_view_state.current_list_state(),
    );
    page.render(f, chunks[1]);
}

fn render_copy_detail_view(f: &mut Frame, area: Rect, app: &App, vs: &CopyDetailViewState) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

    let header = Header::new(app.breadcrumb_strs());
    f.render_widget(header, chunks[0]);

    let current_items = app.current_object_items();
    let current_file_detail = app.get_current_file_detail().unwrap();
    let current_file_versions = app.get_current_file_versions().unwrap();

    let mut page = ObjectDetailPage::new(
        current_items,
        current_file_detail.clone(),
        current_file_versions.clone(),
        vs.before,
        None,
        Some(*vs),
        *app.app_view_state.current_list_state(),
    );
    page.render(f, chunks[1]);
}

fn render_preview_view(f: &mut Frame, area: Rect, app: &App, vs: &PreviewViewState) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

    let header = Header::new(app.breadcrumb_strs());
    f.render_widget(header, chunks[0]);

    let current_file_detail = app.get_current_file_detail().unwrap();

    let mut page = ObjectPreviewPage::new(
        current_file_detail.clone(),
        vs.preview.clone(),
        vs.preview_max_digits,
        vs.offset,
        None,
    );
    page.render(f, chunks[1]);
}

fn render_preview_save_view(f: &mut Frame, area: Rect, app: &App, vs: &PreviewSaveViewState) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

    let header = Header::new(app.breadcrumb_strs());
    f.render_widget(header, chunks[0]);

    let current_file_detail = app.get_current_file_detail().unwrap();

    let mut page = ObjectPreviewPage::new(
        current_file_detail.clone(),
        vs.before.preview.clone(),
        vs.before.preview_max_digits,
        vs.before.offset,
        Some(vs.clone()),
    );
    page.render(f, chunks[1]);
}

fn render_help_view(f: &mut Frame, area: Rect, app: &App, before: &ViewState) {
    let helps = app.action_manager.helps(before);

    let mut page = HelpPage::new(helps.clone());
    page.render(f, area);
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
        let dialog = Dialog::new(Box::new(loading));
        f.render_widget_ref(dialog, area);
    }
}

fn calc_centered_dialog_rect(r: Rect, dialog_width: u16, dialog_height: u16) -> Rect {
    let vertical_pad = (r.height - dialog_height) / 2;
    let vertical_layout = Layout::vertical(Constraint::from_lengths([
        vertical_pad,
        dialog_height,
        vertical_pad,
    ]))
    .split(r);

    let horizontal_pad = (r.width - dialog_width) / 2;
    Layout::horizontal(Constraint::from_lengths([
        horizontal_pad,
        dialog_width,
        horizontal_pad,
    ]))
    .split(vertical_layout[1])[1]
}
