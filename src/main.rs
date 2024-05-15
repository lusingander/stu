mod app;
mod client;
mod component;
mod config;
mod error;
mod event;
mod file;
mod keys;
mod macros;
mod object;
mod pages;
mod run;
mod ui;
mod util;
mod widget;

use clap::{arg, Parser};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use event::{AppEventType, Sender};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{
    io::{stdout, Result, Stdout},
    panic,
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

    /// Target bucket name
    #[arg(short, long, value_name = "NAME")]
    bucket: Option<String>,
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
        bucket,
    } = args;

    let (tx, rx) = event::new();
    let (width, height) = get_frame_size(terminal);
    let mut app = App::new(tx.clone(), width, height);

    spawn(async move {
        initialize(tx, region, endpoint_url, profile, bucket).await;
    });

    run::run(&mut app, terminal, rx).await
}

fn get_frame_size<B: Backend>(terminal: &mut Terminal<B>) -> (usize, usize) {
    let size = terminal.get_frame().size();
    (size.width as usize, size.height as usize)
}

async fn initialize(
    tx: Sender,
    region: Option<String>,
    endpoint_url: Option<String>,
    profile: Option<String>,
    bucket: Option<String>,
) {
    match Config::load() {
        Ok(config) => {
            let client = Client::new(region, endpoint_url, profile).await;
            tx.send(AppEventType::Initialize(config, client, bucket));
        }
        Err(e) => {
            tx.send(AppEventType::NotifyError(e));
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
