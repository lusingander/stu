use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

pub fn save_binary(path: &String, bytes: &[u8]) -> Result<(), String> {
    create_dirs(path)?;

    let f = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
    let mut writer = BufWriter::new(f);
    writer
        .write_all(bytes)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

fn create_dirs(path: &String) -> Result<(), String> {
    let path = Path::new(path);
    match path.parent() {
        Some(path) => std::fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create directories: {}", e)),
        None => Ok(()),
    }
}
