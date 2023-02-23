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

    let header = Paragraph::new(Span::styled(app.current_key_string(), Style::default())).block(
        Block::default()
            .title(Span::styled(APP_NAME, Style::default()))
            .borders(Borders::all()),
    );
    f.render_widget(header, chunks[0]);

    let current_items = app.current_items();
    let list_items: Vec<ListItem> = current_items
        .iter()
        .map(|i| {
            let content = match i {
                Item::Bucket { .. } => {
                    let content = i.display_name();
                    let style = Style::default();
                    Span::styled(content, style)
                }
                Item::Dir { .. } => {
                    let content = i.display_name();
                    let style = Style::default().add_modifier(Modifier::BOLD);
                    Span::styled(content, style)
                }
                Item::File { .. } => {
                    let content = i.display_name();
                    let style = Style::default();
                    Span::styled(content, style)
                }
            };
            ListItem::new(content)
        })
        .collect();
    let list = List::new(list_items)
        .block(Block::default().borders(Borders::all()))
        .highlight_style(Style::default().bg(Color::Green))
        .highlight_symbol(" ");
    f.render_stateful_widget(list, chunks[1], &mut app.current_list_state);
}
