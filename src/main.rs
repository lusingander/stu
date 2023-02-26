mod app;
mod client;
mod ui;

use clap::{arg, Parser};
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

/// STU - S3 Terminal UI
#[derive(Parser)]
#[command(version)]
struct Args {
    /// AWS region
    #[arg(short, long)]
    region: Option<String>,

    /// AWS endpoint url
    #[arg(short, long, value_name = "URL")]
    endpoint_url: Option<String>,

    /// AWS profile name
    #[arg(short, long, value_name = "NAME")]
    profile: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut stdout = stdout();
    let mut terminal = setup(&mut stdout)?;

    let ret = run(&mut terminal, args).await;

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

async fn run<B: Backend>(terminal: &mut Terminal<B>, args: Args) -> std::io::Result<()> {
    let Args {
        region,
        endpoint_url,
        profile,
        ..
    } = args;
    let client = Client::new(region, endpoint_url, profile).await;
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
