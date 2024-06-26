use std::{
    collections::HashMap,
    fmt::{self, Debug, Formatter},
};

use chrono::{DateTime, Local};

#[derive(Clone, Debug)]
pub struct BucketItem {
    pub name: String,
}

#[derive(Clone, Debug)]
pub enum ObjectItem {
    Dir {
        name: String,
    },
    File {
        name: String,
        size_byte: usize,
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

    pub fn size_byte(&self) -> Option<usize> {
        match self {
            ObjectItem::Dir { .. } => None,
            ObjectItem::File { size_byte, .. } => Some(*size_byte),
        }
    }

    pub fn last_modified(&self) -> Option<DateTime<Local>> {
        match self {
            ObjectItem::Dir { .. } => None,
            ObjectItem::File { last_modified, .. } => Some(*last_modified),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileDetail {
    pub name: String,
    pub size_byte: usize,
    pub last_modified: DateTime<Local>,
    pub e_tag: String,
    pub content_type: String,
    pub storage_class: String,
    pub key: String,
    pub s3_uri: String,
    pub arn: String,
    pub object_url: String,
}

#[derive(Debug, Clone)]
pub struct FileVersion {
    pub version_id: String,
    pub size_byte: usize,
    pub last_modified: DateTime<Local>,
    #[allow(dead_code)]
    pub is_latest: bool,
}

#[derive(Debug, Default)]
pub struct AppObjects {
    bucket_items: Vec<BucketItem>,
    object_items_map: HashMap<ObjectKey, Vec<ObjectItem>>,
    detail_map: HashMap<ObjectKey, FileDetail>,
    versions_map: HashMap<ObjectKey, Vec<FileVersion>>,
}

impl AppObjects {
    pub fn get_bucket_items(&self) -> Vec<BucketItem> {
        self.bucket_items.to_vec()
    }

    pub fn get_object_items(&self, key: &ObjectKey) -> Option<Vec<ObjectItem>> {
        self.object_items_map.get(key).map(|items| items.to_vec())
    }

    pub fn set_bucket_items(&mut self, items: Vec<BucketItem>) {
        self.bucket_items = items;
    }

    pub fn set_object_items(&mut self, key: ObjectKey, items: Vec<ObjectItem>) {
        self.object_items_map.insert(key, items);
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
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct ObjectKey {
    pub bucket_name: String,
    pub object_path: Vec<String>,
}

#[derive(Default, Clone)]
pub struct RawObject {
    pub bytes: Vec<u8>,
}

impl Debug for RawObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "RawObject {{ bytes: [u8; {}] }}", self.bytes.len())
    }
}
