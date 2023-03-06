use std::{
    fs::File,
    io::{BufWriter, Write},
};

pub fn save_binary(path: &String, bytes: &[u8]) -> Result<(), String> {
    let f = File::create(path).map_err(|_| "Failed to create file".to_string())?;
    let mut writer = BufWriter::new(f);
    writer
        .write_all(bytes)
        .map_err(|_| "Failed to write file".to_string())?;
    Ok(())
}
