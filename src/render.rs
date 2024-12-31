use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::Block,
    Frame,
};

use crate::{
    app::{App, Notification},
    pages::page::Page,
    widget::{Header, LoadingDialog, Status, StatusType},
};

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::vertical([
        Constraint::Length(header_height(app)),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(f.area());

    render_background(f, f.area(), app);
    render_header(f, chunks[0], app);
    render_content(f, chunks[1], app);
    render_footer(f, chunks[2], app);
    render_loading_dialog(f, app);
}

fn header_height(app: &App) -> u16 {
    match app.page_stack.current_page() {
        Page::Help(_) => 0, // Hide header
        _ => 3,
    }
}

fn render_background(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().bg(app.theme().bg);
    f.render_widget(block, area);
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    if !area.is_empty() {
        let header = Header::new(app.breadcrumb()).theme(app.theme());
        f.render_widget(header, area);
    }
}

fn render_content(f: &mut Frame, area: Rect, app: &mut App) {
    app.page_stack.current_page_mut().render(f, area);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let status_type = match app.current_notification() {
        Notification::Info(msg) => StatusType::Info(msg.into()),
        Notification::Success(msg) => StatusType::Success(msg.into()),
        Notification::Warn(msg) => StatusType::Warn(msg.into()),
        Notification::Error(msg) => StatusType::Error(msg.into()),
        Notification::None => StatusType::Help(app.page_stack.current_page().short_helps()),
    };
    let status = Status::new(status_type).theme(app.theme());
    f.render_widget(status, area);
}

fn render_loading_dialog(f: &mut Frame, app: &App) {
    if app.loading() {
        let dialog = LoadingDialog::default().theme(app.theme());
        f.render_widget(dialog, f.area());
    }
}
