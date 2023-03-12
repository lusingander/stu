use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use crate::app::AppError;

pub fn save_binary<'a>(path: &String, bytes: &[u8]) -> Result<(), AppError<'a>> {
    create_dirs(path)?;

    let f = File::create(path).map_err(|e| AppError::new("Failed to create file", e))?;
    let mut writer = BufWriter::new(f);
    writer
        .write_all(bytes)
        .map_err(|e| AppError::new("Failed to write file", e))?;

    Ok(())
}

fn create_dirs<'a>(path: &String) -> Result<(), AppError<'a>> {
    let path = Path::new(path);
    match path.parent() {
        Some(path) => std::fs::create_dir_all(path)
            .map_err(|e| AppError::new("Failed to create directories", e)),
        None => Ok(()),
    }
}
