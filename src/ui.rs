use chrono::{DateTime, Local};
use crossterm::event::{self, Event, KeyCode};
use std::io::Result;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::app::{App, Item};

const APP_NAME: &str = "STU";

pub async fn run<B: Backend>(app: &mut App, terminal: &mut Terminal<B>) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => return Ok(()),
                KeyCode::Char('j') => {
                    app.select_next();
                }
                KeyCode::Char('k') => {
                    app.select_prev();
                }
                KeyCode::Char('g') => {
                    app.select_first();
                }
                KeyCode::Char('G') => {
                    app.select_last();
                }
                KeyCode::Char('l') => {
                    app.move_down().await;
                }
                KeyCode::Char('h') => {
                    app.move_up().await;
                }
                _ => {}
            }
        }
    }
}

fn render<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    let current_key = app.current_key_string();
    let header = build_header(&current_key);
    f.render_widget(header, chunks[0]);

    let current_items = app.current_items();
    let list = build_list(&current_items, f.size().width);
    f.render_stateful_widget(list, chunks[1], &mut app.current_list_state);
}

fn build_header(current_key: &String) -> Paragraph {
    Paragraph::new(Span::styled(current_key, Style::default())).block(
        Block::default()
            .title(Span::styled(APP_NAME, Style::default()))
            .borders(Borders::all()),
    )
}

fn build_list(current_items: &[Item], width: u16) -> List {
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

    List::new(list_items)
        .block(Block::default().borders(Borders::all()))
        .highlight_style(Style::default().bg(Color::Green))
        .highlight_symbol("")
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
    let size = humansize::format_size_i(*size_byte, humansize::BINARY);
    let date = last_modified.format("%y/%m/%d %H:%M:%S");
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
