use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Stylize},
    text::Line,
    widgets::{Block, BorderType, Padding, Paragraph},
    Frame,
};

use crate::{
    app::{App, Notification},
    color::ColorTheme,
    pages::page::Page,
    ui::common::calc_centered_dialog_rect,
    util,
    widget::{Dialog, Header},
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
    let block = Block::default().bg(app.theme.bg);
    f.render_widget(block, area);
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    if area.is_empty() {
        return;
    }
    let header = build_header(app);
    f.render_widget(header, area);
}

fn render_content(f: &mut Frame, area: Rect, app: &mut App) {
    app.page_stack.current_page_mut().render(f, area);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    match &app.app_view_state.notification {
        Notification::Info(msg) => {
            let msg = build_info_status(msg, &app.theme);
            f.render_widget(msg, area);
        }
        Notification::Success(msg) => {
            let msg = build_success_status(msg, &app.theme);
            f.render_widget(msg, area);
        }
        Notification::Warn(msg) => {
            let msg = build_warn_status(msg, &app.theme);
            f.render_widget(msg, area);
        }
        Notification::Error(msg) => {
            let msg = build_error_status(msg, &app.theme);
            f.render_widget(msg, area);
        }
        Notification::None => {
            let help = build_short_help(app, area.width);
            f.render_widget(help, area);
        }
    }
}

fn render_loading_dialog(f: &mut Frame, app: &App) {
    if app.app_view_state.is_loading {
        let loading = build_loading_dialog("Loading...", &app.theme);
        let area = calc_centered_dialog_rect(f.area(), 30, 5);
        let dialog = Dialog::new(Box::new(loading), app.theme.bg);
        f.render_widget_ref(dialog, area);
    }
}

fn build_header(app: &App) -> Header {
    let mut target_pages: Vec<&Page> = app
        .page_stack
        .iter()
        .filter(|page| matches!(page, Page::BucketList(_) | Page::ObjectList(_)))
        .collect();
    target_pages.pop(); // Remove the last item (current page)

    let breadcrumb: Vec<String> = target_pages
        .iter()
        .map(|page| match page {
            Page::BucketList(page) => page.current_selected_item().name.clone(),
            Page::ObjectList(page) => page.current_selected_item().name().into(),
            _ => unreachable!(),
        })
        .collect();
    Header::new(breadcrumb).theme(&app.theme)
}

fn build_short_help(app: &App, width: u16) -> Paragraph {
    let helps = app.page_stack.current_page().short_helps();
    let pad = Padding::horizontal(2);
    let max_width = (width - pad.left - pad.right) as usize;
    let help = build_short_help_string(&helps, max_width);
    Paragraph::new(help.fg(app.theme.status_help)).block(Block::default().padding(pad))
}

fn build_short_help_string(helps: &[(String, usize)], max_width: usize) -> String {
    let delimiter = ", ";
    let ss = util::prune_strings_to_fit_width(helps, max_width, delimiter);
    ss.join(delimiter)
}

fn build_info_status<'a>(msg: &'a str, theme: &'a ColorTheme) -> Paragraph<'a> {
    Paragraph::new(msg.fg(theme.status_info))
        .block(Block::default().padding(Padding::horizontal(2)))
}

fn build_success_status<'a>(msg: &'a str, theme: &'a ColorTheme) -> Paragraph<'a> {
    Paragraph::new(msg.add_modifier(Modifier::BOLD).fg(theme.status_success))
        .block(Block::default().padding(Padding::horizontal(2)))
}

fn build_warn_status<'a>(msg: &'a str, theme: &'a ColorTheme) -> Paragraph<'a> {
    Paragraph::new(msg.add_modifier(Modifier::BOLD).fg(theme.status_warn))
        .block(Block::default().padding(Padding::horizontal(2)))
}

fn build_error_status<'a>(err: &'a str, theme: &'a ColorTheme) -> Paragraph<'a> {
    let err = format!("ERROR: {}", err);
    Paragraph::new(err.add_modifier(Modifier::BOLD).fg(theme.status_error))
        .block(Block::default().padding(Padding::horizontal(2)))
}

fn build_loading_dialog<'a>(msg: &'a str, theme: &'a ColorTheme) -> Paragraph<'a> {
    let text = Line::from(msg.add_modifier(Modifier::BOLD));
    Paragraph::new(text).alignment(Alignment::Center).block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::vertical(1))
            .fg(theme.fg),
    )
}
