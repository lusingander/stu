mod app;
mod client;
mod ui;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdout, Result, Stdout};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::app::App;
use crate::client::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let mut stdout = stdout();
    let mut terminal = setup(&mut stdout)?;

    let ret = run(&mut terminal).await;

    shutdown(&mut terminal)?;

    ret
}

fn setup(stdout: &mut Stdout) -> std::io::Result<Terminal<CrosstermBackend<&mut Stdout>>> {
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

async fn run<B: Backend>(terminal: &mut Terminal<B>) -> std::io::Result<()> {
    let client = Client::new().await;
    let mut app = App::new(client).await;

    ui::run(&mut app, terminal).await
}

fn shutdown<W: std::io::Write>(
    terminal: &mut Terminal<CrosstermBackend<W>>,
) -> std::io::Result<()> {
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    terminal.show_cursor()?;

    Ok(())
}
