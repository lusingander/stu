use std::{env, path::PathBuf};

use anyhow::Context;
use serde_derive::{Deserialize, Serialize};

const STU_ROOT_DIR_ENV_VAR: &str = "STU_ROOT_DIR";

const APP_BASE_DIR: &str = ".stu";
const CONFIG_FILE_NAME: &str = "config.toml";
const ERROR_LOG_FILE_NAME: &str = "error.log";
const DEBUG_LOG_FILE_NAME: &str = "debug.log";
const DOWNLOAD_DIR: &str = "download";

#[derive(Serialize, Deserialize)]
pub struct Config {
    download_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        let download_dir = match Config::get_app_base_dir() {
            Ok(dir) => {
                let path = dir.join(DOWNLOAD_DIR);
                String::from(path.to_string_lossy())
            }
            Err(_) => "".to_string(),
        };
        Self { download_dir }
    }
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
