use std::collections::HashMap;

use chrono::{DateTime, Local};

#[derive(Clone, Debug)]
pub struct BucketItem {
    pub name: String,
}

#[derive(Clone, Debug)]
pub enum ObjectItem {
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

impl ObjectItem {
    pub fn name(&self) -> &str {
        match self {
            ObjectItem::Dir { name, .. } => name,
            ObjectItem::File { name, .. } => name,
        }
    }
}

pub struct FileDetail {
    pub name: String,
    pub size_byte: i64,
    pub last_modified: DateTime<Local>,
    pub e_tag: String,
    pub content_type: String,
    pub storage_class: String,
    pub key: String,
    pub s3_uri: String,
    pub arn: String,
    pub object_url: String,
}

pub struct FileVersion {
    pub version_id: String,
    pub size_byte: i64,
    pub last_modified: DateTime<Local>,
    pub is_latest: bool,
}

// fixme: data structure
pub struct AppObjects {
    bucket_items: Vec<BucketItem>,
    object_items_map: HashMap<ObjectKey, Vec<ObjectItem>>,
    detail_map: HashMap<ObjectKey, FileDetail>,
    versions_map: HashMap<ObjectKey, Vec<FileVersion>>,
}

impl AppObjects {
    pub fn new() -> AppObjects {
        AppObjects {
            bucket_items: Vec::new(),
            object_items_map: HashMap::new(),
            detail_map: HashMap::new(),
            versions_map: HashMap::new(),
        }
    }

    pub fn get_bucket_items(&self) -> Vec<BucketItem> {
        self.bucket_items.to_vec()
    }

    pub fn get_object_items(&self, key: &ObjectKey) -> Vec<ObjectItem> {
        self.object_items_map
            .get(key)
            .unwrap_or(&Vec::new())
            .to_vec()
    }

    pub fn get_bucket_item(&self, idx: usize) -> Option<&BucketItem> {
        self.bucket_items.get(idx)
    }

    pub fn get_object_item(&self, key: &ObjectKey, idx: usize) -> Option<&ObjectItem> {
        self.object_items_map
            .get(key)
            .and_then(|items| items.get(idx))
    }

    pub fn set_bucket_items(&mut self, items: Vec<BucketItem>) {
        self.bucket_items = items;
    }

    pub fn set_object_items(&mut self, key: ObjectKey, items: Vec<ObjectItem>) {
        self.object_items_map.insert(key, items);
    }

    pub fn exists_object_item(&self, key: &ObjectKey) -> bool {
        self.object_items_map.contains_key(key)
    }

    pub fn get_object_detail(&self, key: &ObjectKey) -> Option<&FileDetail> {
        self.detail_map.get(key)
    }

    pub fn get_object_versions(&self, key: &ObjectKey) -> Option<&Vec<FileVersion>> {
        self.versions_map.get(key)
    }

    pub fn set_object_details(
        &mut self,
        key: ObjectKey,
        detail: FileDetail,
        versions: Vec<FileVersion>,
    ) {
        self.detail_map.insert(key.to_owned(), detail);
        self.versions_map.insert(key.to_owned(), versions);
    }

    pub fn exists_object_details(&self, key: &ObjectKey) -> bool {
        self.detail_map.contains_key(key) && self.versions_map.contains_key(key)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ObjectKey {
    pub bucket_name: String,
    pub object_path: Vec<String>,
}

#[derive(Clone)]
pub struct Object {
    pub content_type: String,
    pub bytes: Vec<u8>,
}
