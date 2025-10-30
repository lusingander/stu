mod app;
mod client;
mod color;
mod config;
mod constant;
mod environment;
mod error;
mod event;
mod file;
mod format;
mod help;
mod keys;
mod macros;
mod object;
mod pages;
mod run;
mod util;
mod widget;

use clap::{arg, Parser, ValueEnum};
use event::AppEventType;
use file::open_or_create_append_file;
use std::sync::Mutex;
use tracing_subscriber::fmt::time::ChronoLocal;

use crate::{
    app::{App, AppContext},
    color::ColorTheme,
    config::Config,
    environment::Environment,
    keys::UserEventMapper,
};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum PathStyle {
    Auto,
    Always,
    Never,
}

impl From<PathStyle> for client::AddressingStyle {
    fn from(style: PathStyle) -> Self {
        match style {
            PathStyle::Auto => client::AddressingStyle::Auto,
            PathStyle::Always => client::AddressingStyle::Path,
            PathStyle::Never => client::AddressingStyle::VirtualHosted,
        }
    }
}

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

    /// Prefix for object keys
    #[arg(short = 'P', long, value_name = "PREFIX", requires = "bucket")]
    prefix: Option<String>,

    /// Path style type for object paths
    #[arg(long, value_name = "TYPE", default_value = "auto")]
    path_style: PathStyle,

    /// Disable request signing
    #[arg(long)]
    no_sign_request: bool,

    /// Enable debug logs
    #[arg(long)]
    debug: bool,

    // Fix dynamic values (e.g., datetime, version) for tests
    // This option is hidden and intended for internal testing only
    #[arg(long, hide = true)]
    fix_dynamic_values_for_test: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config::load()?;
    let mapper = UserEventMapper::load()?;
    let env = Environment::new(config.preview.image, args.fix_dynamic_values_for_test);
    let theme = ColorTheme::default();
    let ctx = AppContext::new(config, env, theme);

    initialize_debug_log(&args)?;

    let client = client::new(
        args.region,
        args.endpoint_url,
        args.profile,
        ctx.config.default_region.clone(),
        args.path_style.into(),
    )
    .await;

    let (tx, rx) = event::new();
    let mut app = App::new(mapper, client, ctx, tx.clone());
    tx.send(AppEventType::Initialize(args.bucket, args.prefix));

    let mut terminal = ratatui::try_init()?;
    let ret = run::run(&mut app, &mut terminal, rx).await;
    ratatui::try_restore()?;

    ret
}

fn initialize_debug_log(args: &Args) -> anyhow::Result<()> {
    if args.debug {
        let path = Config::debug_log_path()?;
        let file = open_or_create_append_file(path)?;
        tracing_subscriber::fmt()
            .with_ansi(false)
            .with_timer(ChronoLocal::rfc_3339())
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(Mutex::new(file))
            .init();
    }
    Ok(())
}
