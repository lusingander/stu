use std::collections::HashMap;

use chrono::{DateTime, Local};

#[derive(Clone, Debug)]
pub enum Item {
    Bucket {
        name: String,
    },
    Dir {
        name: String,
        paths: Vec<String>,
    },
    File {
        name: String,
        paths: Vec<String>,
        size_byte: i64,
        last_modified: DateTime<Local>,
    },
}

impl Item {
    pub fn name(&self) -> &String {
        match self {
            Item::Bucket { name } => name,
            Item::Dir { name, .. } => name,
            Item::File { name, .. } => name,
        }
    }
}

pub struct FileDetail {
    pub name: String,
    pub size_byte: i64,
    pub last_modified: DateTime<Local>,
    pub e_tag: String,
    pub content_type: String,
}

pub struct FileVersion {
    pub version_id: String,
    pub size_byte: i64,
    pub last_modified: DateTime<Local>,
    pub is_latest: bool,
}

pub struct AppObjects {
    items_map: HashMap<Vec<String>, Vec<Item>>,
    detail_map: HashMap<String, FileDetail>,
    versions_map: HashMap<String, Vec<FileVersion>>,
}

impl AppObjects {
    pub fn new() -> AppObjects {
        AppObjects {
            items_map: HashMap::new(),
            detail_map: HashMap::new(),
            versions_map: HashMap::new(),
        }
    }

    pub fn get_items(&self, keys: &[String]) -> Vec<Item> {
        self.items_map.get(keys).unwrap_or(&Vec::new()).to_vec()
    }

    pub fn get_items_len(&self, keys: &[String]) -> usize {
        self.items_map.get(keys).unwrap_or(&Vec::new()).len()
    }

    pub fn get_item(&self, keys: &[String], idx: usize) -> Option<&Item> {
        self.items_map.get(keys).and_then(|items| items.get(idx))
    }

    pub fn set_items(&mut self, keys: Vec<String>, items: Vec<Item>) {
        self.items_map.insert(keys, items);
    }

    pub fn exists_item(&self, keys: &[String]) -> bool {
        self.items_map.contains_key(keys)
    }

    pub fn get_object_detail(&self, key: &str) -> Option<&FileDetail> {
        self.detail_map.get(key)
    }

    pub fn get_object_versions(&self, key: &str) -> Option<&Vec<FileVersion>> {
        self.versions_map.get(key)
    }

    pub fn set_object_details(
        &mut self,
        key: &str,
        detail: FileDetail,
        versions: Vec<FileVersion>,
    ) {
        self.detail_map.insert(key.to_string(), detail);
        self.versions_map.insert(key.to_string(), versions);
    }

    pub fn exists_object_details(&self, key: &str) -> bool {
        self.detail_map.contains_key(key) && self.versions_map.contains_key(key)
    }
}
