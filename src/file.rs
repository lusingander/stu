use chrono::Local;
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
};

use crate::error::{AppError, Result};

pub fn save_binary<'a>(path: &String, bytes: &[u8]) -> Result<'a, ()> {
    create_dirs(path)?;

    let f = File::create(path).map_err(|e| AppError::new("Failed to create file", e))?;
    let mut writer = BufWriter::new(f);
    writer
        .write_all(bytes)
        .map_err(|e| AppError::new("Failed to write file", e))?;

    Ok(())
}

pub fn save_error_log<'a>(path: &String, msg: &String, e: &String) -> Result<'a, ()> {
    create_dirs(path)?;

    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| AppError::new("Failed to open file", e))?;

    let now = Local::now();
    writeln!(f, "{} {}: {}", now, msg, e).map_err(|e| AppError::new("Failed to write file", e))?;

    Ok(())
}

fn create_dirs<'a>(path: &String) -> Result<'a, ()> {
    let path = Path::new(path);
    match path.parent() {
        Some(path) => std::fs::create_dir_all(path)
            .map_err(|e| AppError::new("Failed to create directories", e)),
        None => Ok(()),
    }
}
