mod app;
mod cache;
mod client;
mod config;
mod constant;
mod error;
mod event;
mod file;
mod macros;
mod object;
mod pages;
mod run;
mod ui;
mod util;
mod widget;

use clap::{arg, Parser};
use event::AppEventType;
use file::open_or_create_append_file;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};
use std::{
    io::{stdout, Stdout},
    panic,
    sync::Mutex,
};
use tokio::spawn;
use tracing_subscriber::fmt::time::ChronoLocal;

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

    /// Output debug logs
    #[arg(long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config::load()?;

    initialize_debug_log(&args, &config)?;
    initialize_panic_handler();

    let mut terminal = setup()?;
    let ret = run(&mut terminal, args, config).await;

    shutdown()?;

    ret
}

fn setup() -> anyhow::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

async fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    args: Args,
    config: Config,
) -> anyhow::Result<()> {
    let (tx, rx) = event::new();
    let (width, height) = get_frame_size(terminal);
    let mut app = App::new(config, tx.clone(), width, height);

    spawn(async move {
        let client = Client::new(args.region, args.endpoint_url, args.profile).await;
        tx.send(AppEventType::Initialize(client, args.bucket));
    });

    run::run(&mut app, terminal, rx).await?;

    Ok(())
}

fn get_frame_size<B: Backend>(terminal: &mut Terminal<B>) -> (usize, usize) {
    let size = terminal.get_frame().size();
    (size.width as usize, size.height as usize)
}

fn shutdown() -> anyhow::Result<()> {
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

fn initialize_debug_log(args: &Args, config: &Config) -> anyhow::Result<()> {
    if args.debug {
        let path = config.debug_log_path()?;
        let file = open_or_create_append_file(&path)?;
        tracing_subscriber::fmt()
            .with_ansi(false)
            .with_timer(ChronoLocal::rfc_3339())
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(Mutex::new(file))
            .init();
    }
    Ok(())
}
