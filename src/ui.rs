use chrono::{DateTime, Local};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::io::Result;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};

use crate::app::{App, FileDetail, FileDetailViewState, FileVersion, Item, ViewState};

const APP_NAME: &str = "STU";

const SELECTED_COLOR: Color = Color::Cyan;

pub async fn run<B: Backend>(app: &mut App, terminal: &mut Terminal<B>) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;
        if let Event::Key(key) = event::read()? {
            match key {
                KeyEvent {
                    code: KeyCode::Esc, ..
                }
                | KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => return Ok(()),
                KeyEvent {
                    code: KeyCode::Char('j'),
                    ..
                } => {
                    app.select_next();
                }
                KeyEvent {
                    code: KeyCode::Char('k'),
                    ..
                } => {
                    app.select_prev();
                }
                KeyEvent {
                    code: KeyCode::Char('g'),
                    ..
                } => {
                    app.select_first();
                }
                KeyEvent {
                    code: KeyCode::Char('G'),
                    ..
                } => {
                    app.select_last();
                }
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Char('m'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => {
                    app.move_down().await;
                }
                KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Char('h'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => {
                    app.move_up().await;
                }
                KeyEvent {
                    code: KeyCode::Char('h'),
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Char('l'),
                    ..
                } => {
                    app.select_tabs();
                }
                _ => {}
            }
        }
    }
}

fn render<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    match app.view_state {
        ViewState::Default => render_default_view(f, app),
        ViewState::ObjectDetail => render_object_detail_view(f, app),
    }
}

fn render_default_view<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    let current_key = app.current_key_string();
    let header = build_header(&current_key);
    f.render_widget(header, chunks[0]);

    let current_items = app.current_items();
    let current_selected = app.current_list_state.selected();
    let list = build_list(
        &current_items,
        current_selected,
        f.size().width,
        SELECTED_COLOR,
    );
    f.render_stateful_widget(list, chunks[1], &mut app.current_list_state);
}

fn render_object_detail_view<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    let current_key = app.current_key_string();
    let header = build_header(&current_key);
    f.render_widget(header, chunks[0]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    let current_items = app.current_items();
    let current_selected = app.current_list_state.selected();
    let list = build_list(
        &current_items,
        current_selected,
        f.size().width,
        Color::DarkGray,
    );
    f.render_stateful_widget(list, chunks[0], &mut app.current_list_state);

    let block = build_file_detail_block("");
    f.render_widget(block, chunks[1]);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)].as_ref())
        .margin(1)
        .split(chunks[1]);

    let selected = app.file_detail_view_state as usize;
    let tabs = build_file_detail_tabs(&selected);
    f.render_widget(tabs, chunks[0]);

    match app.file_detail_view_state {
        FileDetailViewState::Detail => {
            let current_file_detail = app.get_current_file_detail().unwrap();
            let detail = build_file_detail(current_file_detail);
            f.render_widget(detail, chunks[1]);
        }
        FileDetailViewState::Version => {
            let current_file_versions = app.get_current_file_versions().unwrap();
            let versions = build_file_versions(current_file_versions, chunks[1].width);
            f.render_widget(versions, chunks[1]);
        }
    }
}

fn build_header(current_key: &String) -> Paragraph {
    Paragraph::new(Span::styled(current_key, Style::default())).block(
        Block::default()
            .title(Span::styled(APP_NAME, Style::default()))
            .borders(Borders::all()),
    )
}

fn build_list(
    current_items: &[Item],
    current_selected: Option<usize>,
    width: u16,
    color: Color,
) -> List {
    let list_items: Vec<ListItem> = current_items
        .iter()
        .map(|i| {
            let content = match i {
                Item::Bucket { name, .. } => {
                    let content = format_bucket_item(name, width);
                    let style = Style::default();
                    Span::styled(content, style)
                }
                Item::Dir { name, .. } => {
                    let content = format_dir_item(name, width);
                    let style = Style::default().add_modifier(Modifier::BOLD);
                    Span::styled(content, style)
                }
                Item::File {
                    name,
                    size_byte,
                    last_modified,
                    ..
                } => {
                    let content = format_file_item(name, size_byte, last_modified, width);
                    let style = Style::default();
                    Span::styled(content, style)
                }
            };
            ListItem::new(content)
        })
        .collect();

    let title = format_list_count(current_items, current_selected);
    List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::all())
                .title(title)
                .title_alignment(Alignment::Right),
        )
        .highlight_style(Style::default().bg(color))
}

fn format_bucket_item(name: &String, width: u16) -> String {
    let name_w: usize = (width as usize) - 2 /* spaces */ - 2 /* border */;
    format!(" {:<name_w$} ", name, name_w = name_w)
}

fn format_dir_item(name: &String, width: u16) -> String {
    let name_w: usize = (width as usize) - 2 /* spaces */ - 2 /* border */;
    let name = format!("{}/", name);
    format!(" {:<name_w$} ", name, name_w = name_w)
}

fn format_file_item(
    name: &String,
    size_byte: &i64,
    last_modified: &DateTime<Local>,
    width: u16,
) -> String {
    let size = format_size_byte(*size_byte);
    let date = format_datetime(last_modified);
    let date_w: usize = 17;
    let size_w: usize = 10;
    let name_w: usize = (width as usize) - date_w - size_w - 12 /* spaces */ - 2 /* border */;
    format!(
        " {:<name_w$}    {:<date_w$}    {:<size_w$} ",
        name,
        date,
        size,
        name_w = name_w,
        date_w = date_w,
        size_w = size_w
    )
}

fn format_list_count(current_items: &[Item], current_selected: Option<usize>) -> String {
    current_selected
        .and_then(|n| {
            let total = current_items.len();
            if total == 0 {
                None
            } else {
                Some(format_count(n + 1, total))
            }
        })
        .unwrap_or_default()
}

fn format_count(selected: usize, total: usize) -> String {
    let digits = digits(total);
    format!(" {:>digits$} / {} ", selected, total)
}

fn digits(n: usize) -> usize {
    n.to_string().len()
}

fn build_file_detail_block(title: &str) -> Block {
    Block::default().title(title).borders(Borders::all())
}

fn build_file_detail_tabs(selected: &usize) -> Tabs {
    let tabs = vec![
        Spans::from(Span::styled("Detail", Style::default())),
        Spans::from(Span::styled("Version", Style::default())),
    ];
    Tabs::new(tabs)
        .select(*selected)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(SELECTED_COLOR),
        )
        .block(Block::default().borders(Borders::BOTTOM))
}

fn build_file_detail(detail: &FileDetail) -> Paragraph {
    let text = vec![
        Spans::from(Span::styled(
            " Name:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Spans::from(Span::styled(
            format!("  {}", &detail.name),
            Style::default(),
        )),
        Spans::from(""),
        Spans::from(Span::styled(
            " Size:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Spans::from(Span::styled(
            format!("  {}", format_size_byte(detail.size_byte)),
            Style::default(),
        )),
        Spans::from(""),
        Spans::from(Span::styled(
            " Last Modified:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Spans::from(Span::styled(
            format!("  {}", format_datetime(&detail.last_modified)),
            Style::default(),
        )),
        Spans::from(""),
        Spans::from(Span::styled(
            " ETag:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Spans::from(Span::styled(
            format!("  {}", &detail.e_tag),
            Style::default(),
        )),
        Spans::from(""),
        Spans::from(Span::styled(
            " Content-Type:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Spans::from(Span::styled(
            format!("  {}", &detail.content_type),
            Style::default(),
        )),
    ];
    Paragraph::new(text).block(Block::default())
}

fn format_size_byte(size_byte: i64) -> String {
    humansize::format_size_i(size_byte, humansize::BINARY)
}

fn format_datetime(datetime: &DateTime<Local>) -> String {
    datetime.format("%y/%m/%d %H:%M:%S").to_string()
}

fn build_file_versions(versions: &[FileVersion], width: u16) -> List {
    let list_items: Vec<ListItem> = versions
        .iter()
        .map(|v| {
            let content = vec![
                Spans::from(vec![
                    Span::styled(
                        "    Version ID: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(&v.version_id, Style::default()),
                ]),
                Spans::from(vec![
                    Span::styled(
                        " Last Modified: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format_datetime(&v.last_modified), Style::default()),
                ]),
                Spans::from(vec![
                    Span::styled(
                        "          Size: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format_size_byte(v.size_byte), Style::default()),
                ]),
                Spans::from(Span::styled(
                    "-".repeat(width as usize),
                    Style::default().fg(Color::DarkGray),
                )),
            ];
            ListItem::new(content)
        })
        .collect();
    List::new(list_items)
        .block(Block::default())
        .highlight_style(Style::default().bg(SELECTED_COLOR))
}
