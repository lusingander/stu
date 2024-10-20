use std::{
    collections::HashMap,
    fmt,
    fs::OpenOptions,
    io::{self, Read, Write},
    path::PathBuf,
    sync::RwLock,
};

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
    pub fn new(file_path: PathBuf) -> SimpleStringCache {
        let cache = HashMap::new();

        let cache = SimpleStringCache {
            cache: RwLock::new(cache),
            file_path: file_path.to_string_lossy().into(),
        };

        cache.load_from_file().unwrap();
        cache
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

    fn load_from_file(&self) -> Result<(), io::Error> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.file_path)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut cache = self.cache.write().unwrap();
        cache.clear();
        for line in contents.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 2 {
                cache.insert(parts[0].to_string(), parts[1].to_string());
            } else {
                panic!("Cache file has invalid format on line: {}", line);
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
