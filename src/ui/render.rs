use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Stylize},
    text::Line,
    widgets::{Block, BorderType, Padding, Paragraph},
    Frame,
};

use crate::{
    app::{App, Notification},
    pages::page::Page,
    ui::common::calc_centered_dialog_rect,
    util,
    widget::{Dialog, Header},
};

const SHORT_HELP_COLOR: Color = Color::DarkGray;
const INFO_STATUS_COLOR: Color = Color::Blue;
const SUCCESS_STATUS_COLOR: Color = Color::Green;
const ERROR_STATUS_COLOR: Color = Color::Red;

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::vertical([
        Constraint::Length(header_height(app)),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(f.size());

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

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    if area.is_empty() {
        return;
    }
    let header = build_header(app);
    f.render_widget(header, area);
}

fn render_content(f: &mut Frame, area: Rect, app: &mut App) {
    match app.page_stack.current_page_mut() {
        Page::Initializing(page) => page.render(f, area),
        Page::BucketList(page) => page.render(f, area),
        Page::ObjectList(page) => page.render(f, area),
        Page::ObjectDetail(page) => page.render(f, area),
        Page::ObjectPreview(page) => page.render(f, area),
        Page::Help(page) => page.render(f, area),
    }
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

fn render_loading_dialog(f: &mut Frame, app: &App) {
    if app.app_view_state.is_loading {
        let loading = build_loading_dialog("Loading...");
        let area = calc_centered_dialog_rect(f.size(), 30, 5);
        let dialog = Dialog::new(Box::new(loading));
        f.render_widget_ref(dialog, area);
    }
}

fn build_header(app: &App) -> Header {
    let mut breadcrumb: Vec<String> = app
        .page_stack
        .iter()
        .filter_map(|page| match page {
            Page::BucketList(page) => Some(page.current_selected_item().name.clone()),
            Page::ObjectList(page) => Some(page.current_selected_item().name().into()),
            _ => None,
        })
        .collect();
    breadcrumb.pop(); // Remove the last item
    Header::new(breadcrumb)
}

fn build_short_help(app: &App, width: u16) -> Paragraph {
    let helps = app.action_manager.short_helps(app.view_state_tag());
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

fn build_loading_dialog(msg: &str) -> Paragraph {
    let text = Line::from(msg.add_modifier(Modifier::BOLD));
    Paragraph::new(text).alignment(Alignment::Center).block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::vertical(1)),
    )
}
