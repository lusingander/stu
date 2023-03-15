use chrono::Local;
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
};

use crate::error::{AppError, Result};

pub fn save_binary(path: &String, bytes: &[u8]) -> Result<()> {
    create_dirs(path)?;

    let f = File::create(path).map_err(|e| AppError::new("Failed to create file", e))?;
    let mut writer = BufWriter::new(f);
    writer
        .write_all(bytes)
        .map_err(|e| AppError::new("Failed to write file", e))?;

    Ok(())
}

pub fn save_error_log(path: &String, e: &AppError) -> Result<()> {
    create_dirs(path)?;

    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| AppError::new("Failed to open file", e))?;

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

fn create_dirs(path: &String) -> Result<()> {
    let path = Path::new(path);
    match path.parent() {
        Some(path) => std::fs::create_dir_all(path)
            .map_err(|e| AppError::new("Failed to create directories", e)),
        None => Ok(()),
    }
}
