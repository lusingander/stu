use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use crate::error::{AppError, Result};

const APP_BASE_DIR: &str = ".stu";
const CONFIG_FILE_NAME: &str = "config.toml";
const ERROR_LOG_FILE_NAME: &str = "error.log";
const DONWLOAD_DIR: &str = "donwload";

#[derive(Serialize, Deserialize)]
pub struct Config {
    download_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        let download_dir = match Config::get_app_base_dir() {
            Ok(dir) => {
                let path = dir.join(DONWLOAD_DIR);
                String::from(path.to_string_lossy())
            }
            Err(_) => "".to_string(),
        };
        Self { download_dir }
    }
}

impl Config {
    pub fn load() -> Result<Config> {
        let dir = Config::get_app_base_dir()?;
        let path = dir.join(CONFIG_FILE_NAME);
        confy::load_path(path).map_err(|e| AppError::new("Failed to load config file", e))
    }

    pub fn download_file_path(&self, name: &String) -> String {
        let dir = PathBuf::from(self.download_dir.clone());
        let path = dir.join(name);
        String::from(path.to_string_lossy())
    }

    pub fn error_log_path(&self) -> Result<String> {
        let dir = Config::get_app_base_dir()?;
        let path = dir.join(ERROR_LOG_FILE_NAME);
        Ok(String::from(path.to_string_lossy()))
    }

    fn get_app_base_dir() -> Result<PathBuf> {
        dirs::home_dir()
            .map(|home| home.join(APP_BASE_DIR))
            .ok_or_else(|| AppError::msg("Failed to load home directory"))
    }
}
