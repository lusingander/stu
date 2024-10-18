use std::{env, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

const STU_ROOT_DIR_ENV_VAR: &str = "STU_ROOT_DIR";

const APP_BASE_DIR: &str = ".stu";
const CONFIG_FILE_NAME: &str = "config.toml";
const ERROR_LOG_FILE_NAME: &str = "error.log";
const DEBUG_LOG_FILE_NAME: &str = "debug.log";
const DOWNLOAD_DIR: &str = "download";
const CACHE_FILE_NAME: &str = "cache.txt";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "default_download_dir")]
    pub download_dir: String,
    #[serde(default = "default_default_region")]
    pub default_region: String,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub preview: PreviewConfig,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UiConfig {
    #[serde(default)]
    pub object_list: UiObjectListConfig,
    #[serde(default)]
    pub object_detail: UiObjectDetailConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiObjectListConfig {
    #[serde(default = "default_ui_object_list_date_format")]
    pub date_format: String,
    #[serde(default = "default_ui_object_list_date_width")]
    pub date_width: usize,
}

impl Default for UiObjectListConfig {
    fn default() -> Self {
        Self {
            date_format: default_ui_object_list_date_format(),
            date_width: default_ui_object_list_date_width(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiObjectDetailConfig {
    #[serde(default = "default_ui_object_detail_date_format")]
    pub date_format: String,
}

impl Default for UiObjectDetailConfig {
    fn default() -> Self {
        Self {
            date_format: default_ui_object_detail_date_format(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PreviewConfig {
    #[serde(default)]
    pub highlight: bool,
    #[serde(default = "default_preview_highlight_theme")]
    pub highlight_theme: String,
    #[serde(default)]
    pub image: bool,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            highlight: false,
            highlight_theme: default_preview_highlight_theme(),
            image: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let download_dir = default_download_dir();
        let default_region = default_default_region();
        Self {
            download_dir,
            default_region,
            ui: UiConfig::default(),
            preview: PreviewConfig::default(),
        }
    }
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

fn default_default_region() -> String {
    "us-east-1".to_string()
}

fn default_ui_object_list_date_format() -> String {
    "%Y-%m-%d %H:%M:%S".to_string()
}

fn default_ui_object_list_date_width() -> usize {
    19 // "2021-01-01 12:34:56".len()
}

fn default_ui_object_detail_date_format() -> String {
    "%Y-%m-%d %H:%M:%S".to_string()
}

fn default_preview_highlight_theme() -> String {
    "base16-ocean.dark".to_string()
}

impl Config {
    pub fn load() -> anyhow::Result<Config> {
        let dir = Config::get_app_base_dir()?;
        let path = dir.join(CONFIG_FILE_NAME);
        confy::load_path(path).context("Failed to load config file")
    }

    pub fn download_file_path(&self, name: &str) -> String {
        let dir = PathBuf::from(self.download_dir.clone());
        let path = dir.join(name);
        String::from(path.to_string_lossy())
    }

    pub fn error_log_path(&self) -> anyhow::Result<String> {
        let dir = Config::get_app_base_dir()?;
        let path = dir.join(ERROR_LOG_FILE_NAME);
        Ok(String::from(path.to_string_lossy()))
    }

    pub fn debug_log_path(&self) -> anyhow::Result<String> {
        let dir = Config::get_app_base_dir()?;
        let path = dir.join(DEBUG_LOG_FILE_NAME);
        Ok(String::from(path.to_string_lossy()))
    }

    pub fn cache_file_path() -> anyhow::Result<String> {
        let dir = Config::get_app_base_dir()?;
        let path = dir.join(CACHE_FILE_NAME);
        Ok(String::from(path.to_string_lossy()))
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
