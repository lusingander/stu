use arboard::Clipboard;
use chrono::Local;
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
};

use crate::error::{AppError, Result};

pub fn create_binary_file<P: AsRef<Path>>(path: P) -> Result<BufWriter<File>> {
    create_dirs(&path)?;
    let f = File::create(&path).map_err(|e| AppError::new("Failed to create file", e))?;
    Ok(BufWriter::new(f))
}

pub fn save_error_log<P: AsRef<Path>>(path: P, e: &AppError) -> Result<()> {
    create_dirs(&path)?;

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

pub fn open_or_create_append_file<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
    OpenOptions::new().create(true).append(true).open(path)
}

fn create_dirs<P: AsRef<Path>>(path: P) -> Result<()> {
    match path.as_ref().parent() {
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
