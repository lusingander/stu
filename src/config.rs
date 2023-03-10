use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const APP_BASE_DIR: &str = ".stu";
const CONFIG_FILE_NAME: &str = "config.toml";
const DONWLOAD_DIR: &str = "donwload";

#[derive(Serialize, Deserialize)]
pub struct Config {
    download_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        let download_dir = match dirs::home_dir() {
            Some(home) => {
                let mut path = home;
                path.push(APP_BASE_DIR);
                path.push(DONWLOAD_DIR);
                String::from(path.to_string_lossy())
            }
            None => "".to_string(),
        };
        Self { download_dir }
    }
}

impl Config {
    pub fn load() -> Result<Config, String> {
        match dirs::home_dir() {
            Some(home) => {
                let mut path = home;
                path.push(APP_BASE_DIR);
                path.push(CONFIG_FILE_NAME);
                confy::load_path(path).map_err(|_| "Failed to load config file".to_string())
            }
            None => Err("Failed to load home directory".to_string()),
        }
    }

    pub fn download_file_path(&self, name: &String) -> String {
        let dir = PathBuf::from(self.download_dir.clone());
        let path = dir.join(name);
        String::from(path.to_string_lossy())
    }
}
