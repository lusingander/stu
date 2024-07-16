use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::{self, Write};
use std::sync::RwLock;

pub struct SimpleStringCache {
    pub cache: RwLock<HashMap<String, String>>,
    pub file_path: String,
}

impl fmt::Debug for SimpleStringCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cache = self.cache.read().unwrap();
        f.debug_struct("SimpleCache")
            .field("file_path", &self.file_path)
            .field("cache", &*cache)
            .finish()
    }
}

impl SimpleStringCache {
    pub fn new(file_path: String) -> io::Result<Self> {
        let mut cache = HashMap::new();

        Ok(SimpleStringCache {
            cache: RwLock::new(cache),
            file_path,
        })
    }

    pub fn put(&self, key: String, value: String) -> io::Result<()> {
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(key.clone(), value);
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let cache = self.cache.read().unwrap();
        cache.get(key).cloned()
    }

    pub fn load_from_file(&self) -> io::Result<()> {
        let mut file = File::open(&self.file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut cache = self.cache.write().unwrap();
        cache.clear();
        for line in contents.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 2 {
                cache.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
        Ok(())
    }

    pub fn write_cache(&self) -> io::Result<()> {
        let temp_file_path = format!("{}.tmp", self.file_path);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_file_path)?;

        let cache = self.cache.read().unwrap();
        for (key, value) in &*cache {
            writeln!(file, "{},{}", key, value)?;
        }

        std::fs::rename(temp_file_path, &self.file_path)?;
        Ok(())
    }
}
