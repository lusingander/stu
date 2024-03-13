mod app;
mod client;
mod component;
mod config;
mod error;
mod event;
mod file;
mod item;
mod keys;
mod macros;
mod run;
mod ui;
mod util;

use clap::{arg, Parser};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use event::AppEventType;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{
    io::{stdout, Result, Stdout},
    panic,
    sync::mpsc::Sender,
};
use tokio::spawn;

use crate::app::App;
use crate::client::Client;
use crate::config::Config;

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

    initialize_panic_handler();

    let mut terminal = setup()?;
    let ret = run(&mut terminal, args).await;

    shutdown()?;

    ret
}

fn setup() -> std::io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
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

    let (tx, rx) = event::new();
    let (_, height) = get_frame_size(terminal);
    let mut app = App::new(tx.clone(), height);

    spawn(async move {
        load_config(tx, region, endpoint_url, profile).await;
    });

    run::run(&mut app, terminal, rx).await
}

fn get_frame_size<B: Backend>(terminal: &mut Terminal<B>) -> (usize, usize) {
    let size = terminal.get_frame().size();
    (size.width as usize, size.height as usize)
}

async fn load_config(
    tx: Sender<AppEventType>,
    region: Option<String>,
    endpoint_url: Option<String>,
    profile: Option<String>,
) {
    match Config::load() {
        Ok(config) => {
            let client = Client::new(region, endpoint_url, profile).await;
            tx.send(AppEventType::Initialize(config, client)).unwrap();
        }
        Err(e) => {
            tx.send(AppEventType::Error(e)).unwrap();
        }
    };
}

fn shutdown() -> std::io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn initialize_panic_handler() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        shutdown().unwrap();
        original_hook(panic_info);
    }));
}
