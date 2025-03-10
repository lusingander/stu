use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::Deserialize;
use smart_default::SmartDefault;
use umbra::optional;

const STU_ROOT_DIR_ENV_VAR: &str = "STU_ROOT_DIR";

const APP_BASE_DIR: &str = ".stu";
const CONFIG_FILE_NAME: &str = "config.toml";
const KEYBINDINGS_FILE_NAME: &str = "keybindings.toml";
const ERROR_LOG_FILE_NAME: &str = "error.log";
const DEBUG_LOG_FILE_NAME: &str = "debug.log";
const DOWNLOAD_DIR: &str = "download";
const PREVIEW_THEME_DIR: &str = "preview_theme";
const PREVIEW_SYNTAX_DIR: &str = "preview_syntax";

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, SmartDefault)]
pub struct Config {
    #[default(_code = "default_download_dir()")]
    pub download_dir: String,
    #[default = 5]
    pub max_concurrent_requests: usize,
    #[default = "us-east-1"]
    pub default_region: String,
    #[nested]
    pub ui: UiConfig,
    #[nested]
    pub preview: PreviewConfig,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, SmartDefault)]
pub struct UiConfig {
    #[nested]
    pub object_list: UiObjectListConfig,
    #[nested]
    pub object_detail: UiObjectDetailConfig,
    #[nested]
    pub help: UiHelpConfig,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, SmartDefault)]
pub struct UiObjectListConfig {
    #[default = "%Y-%m-%d %H:%M:%S"]
    pub date_format: String,
    #[default = 19] // // "2021-01-01 12:34:56".len()
    pub date_width: usize,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, SmartDefault)]
pub struct UiObjectDetailConfig {
    #[default = "%Y-%m-%d %H:%M:%S"]
    pub date_format: String,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, SmartDefault)]
pub struct UiHelpConfig {
    #[default = 100]
    pub max_help_width: usize,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, SmartDefault)]
pub struct PreviewConfig {
    #[default = false]
    pub highlight: bool,
    #[default = "base16-ocean.dark"]
    pub highlight_theme: String,
    #[default = false]
    pub image: bool,
    #[default(vec![
        "utf-8".into(),
        "utf-16be".into(),
        "utf-16le".into(),
    ])]
    pub encodings: Vec<String>,
}

fn default_download_dir() -> String {
    match Config::get_app_base_dir() {
        Ok(dir) => {
            let path = dir.join(DOWNLOAD_DIR);
            String::from(path.to_string_lossy())
        }
        Err(_) => "".to_string(),
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Config> {
        let dir = Config::get_app_base_dir()?;
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        let path = dir.join(CONFIG_FILE_NAME);
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let config: OptionalConfig = toml::from_str(&content)?;
            Ok(config.into())
        } else {
            Ok(Config::default())
        }
    }

    pub fn download_file_path<P: AsRef<Path>>(&self, name: P) -> PathBuf {
        let dir = PathBuf::from(self.download_dir.clone());
        dir.join(name)
    }

    pub fn keybindings_file_path(&self) -> anyhow::Result<PathBuf> {
        let dir = Config::get_app_base_dir()?;
        Ok(dir.join(KEYBINDINGS_FILE_NAME))
    }

    pub fn error_log_path(&self) -> anyhow::Result<PathBuf> {
        let dir = Config::get_app_base_dir()?;
        Ok(dir.join(ERROR_LOG_FILE_NAME))
    }

    pub fn debug_log_path(&self) -> anyhow::Result<PathBuf> {
        let dir = Config::get_app_base_dir()?;
        Ok(dir.join(DEBUG_LOG_FILE_NAME))
    }

    pub fn preview_theme_dir_path() -> anyhow::Result<PathBuf> {
        let dir = Config::get_app_base_dir()?;
        Ok(dir.join(PREVIEW_THEME_DIR))
    }

    pub fn preview_syntax_dir_path() -> anyhow::Result<PathBuf> {
        let dir = Config::get_app_base_dir()?;
        Ok(dir.join(PREVIEW_SYNTAX_DIR))
    }

    fn get_app_base_dir() -> anyhow::Result<PathBuf> {
        match env::var(STU_ROOT_DIR_ENV_VAR) {
            Ok(dir) => Ok(PathBuf::from(dir)),
            Err(_) => {
                // default
                dirs::home_dir()
                    .map(|home| home.join(APP_BASE_DIR))
                    .context("Failed to load home directory")
            }
        }
    }
}
