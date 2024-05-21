use arboard::Clipboard;
use chrono::Local;
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
};

use crate::error::{AppError, Result};

pub fn save_binary(path: &str, bytes: &[u8]) -> Result<()> {
    create_dirs(path)?;

    let f = File::create(path).map_err(|e| AppError::new("Failed to create file", e))?;
    let mut writer = BufWriter::new(f);
    writer
        .write_all(bytes)
        .map_err(|e| AppError::new("Failed to write file", e))?;

    Ok(())
}

pub fn save_error_log(path: &str, e: &AppError) -> Result<()> {
    create_dirs(path)?;

    let mut f =
        open_or_create_append_file(path).map_err(|e| AppError::new("Failed to open file", e))?;

    let now = Local::now();

    match &e.cause {
        Some(cause) => {
            writeln!(f, "{} {}: {:?}", now, e.msg, cause)
        }
        None => {
            writeln!(f, "{} {}", now, e.msg)
        }
    }
    .map_err(|e| AppError::new("Failed to write file", e))
}

pub fn open_or_create_append_file(path: &str) -> std::io::Result<File> {
    OpenOptions::new().create(true).append(true).open(path)
}

fn create_dirs(path: &str) -> Result<()> {
    let path = Path::new(path);
    match path.parent() {
        Some(path) => std::fs::create_dir_all(path)
            .map_err(|e| AppError::new("Failed to create directories", e)),
        None => Ok(()),
    }
}

pub fn copy_to_clipboard(value: String) -> Result<()> {
    Clipboard::new()
        .and_then(|mut c| c.set_text(value))
        .map_err(|e| AppError::new("Failed to copy to clipboard", e))
}
