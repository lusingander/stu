use std::{
    collections::HashMap,
    fmt::{self, Debug, Formatter},
};

use chrono::{DateTime, Local};

#[derive(Clone, Debug)]
pub struct BucketItem {
    pub name: String,
    pub s3_uri: String,
    pub arn: String,
    pub object_url: String,
}

#[derive(Clone, Debug)]
pub enum ObjectItem {
    Dir {
        name: String,
        key: String,
        s3_uri: String,
        object_url: String,
    },
    File {
        name: String,
        size_byte: usize,
        last_modified: DateTime<Local>,
        key: String,
        s3_uri: String,
        arn: String,
        object_url: String,
        e_tag: String,
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
    pub e_tag: String,
    #[allow(dead_code)]
    pub is_latest: bool,
}

impl FileVersion {
    pub fn s3_uri(&self, base_file_detail: &FileDetail) -> String {
        format!("{}?versionId={}", base_file_detail.s3_uri, self.version_id)
    }

    pub fn object_url(&self, base_file_detail: &FileDetail) -> String {
        format!(
            "{}?versionId={}",
            base_file_detail.object_url, self.version_id
        )
    }
}

#[derive(Debug, Clone)]
pub struct DownloadObjectInfo {
    pub key: String,
    pub size_byte: usize,
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

    pub fn set_object_detail(&mut self, key: ObjectKey, detail: FileDetail) {
        self.detail_map.insert(key, detail);
    }

    pub fn set_object_versions(&mut self, key: ObjectKey, versions: Vec<FileVersion>) {
        self.versions_map.insert(key, versions);
    }

    pub fn clear_object_items_under(&mut self, key: &ObjectKey) {
        self.object_items_map.retain(|k, _| !k.has_prefix(key));
        self.detail_map.retain(|k, _| !k.has_prefix(key));
        self.versions_map.retain(|k, _| !k.has_prefix(key));
    }

    pub fn clear_all(&mut self) {
        self.bucket_items.clear();
        self.object_items_map.clear();
        self.detail_map.clear();
        self.versions_map.clear();
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct ObjectKey {
    pub bucket_name: String,
    pub object_path: Vec<String>,
}

impl ObjectKey {
    pub fn bucket(name: impl Into<String>) -> Self {
        ObjectKey {
            bucket_name: name.into(),
            object_path: vec![],
        }
    }

    pub fn paths(&self) -> Vec<String> {
        let mut paths = vec![];
        paths.push(self.bucket_name.clone());
        paths.extend(self.object_path.clone());
        paths
    }

    pub fn joined_object_path(&self, contains_file_name: bool) -> String {
        let mut joined = self.object_path.join("/");
        if !contains_file_name && !self.object_path.is_empty() {
            joined.push('/');
        }
        joined
    }

    fn has_prefix(&self, prefix: &ObjectKey) -> bool {
        if self.bucket_name != prefix.bucket_name {
            return false;
        }
        if self.object_path.len() < prefix.object_path.len() {
            return false;
        }
        for (a, b) in self.object_path.iter().zip(prefix.object_path.iter()) {
            if a != b {
                return false;
            }
        }
        true
    }
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("foo", &["a", "b"], true)]
    #[case("foo", &["a", "b", "c"], true)]
    #[case("foo", &[], true)]
    #[case("foo", &["a", "c"], false)]
    #[case("bar", &["a", "b"], false)]
    #[case("foo", &["a", "b", "c", "d"], false)]
    fn test_object_key_has_prefix(
        #[case] prefix_bucket_name: &str,
        #[case] prefix_object_path: &[&str],
        #[case] expected: bool,
    ) {
        let key = object_key("foo", &["a", "b", "c"]);
        let prefix = object_key(prefix_bucket_name, prefix_object_path);
        assert_eq!(key.has_prefix(&prefix), expected);
    }

    #[test]
    fn test_clear_object_items_under() {
        let mut app_objects = AppObjects::default();
        app_objects.set_object_items(object_key("foo", &[]), Vec::new());
        app_objects.set_object_items(object_key("foo", &["a"]), Vec::new());
        app_objects.set_object_items(object_key("foo", &["a", "b"]), Vec::new());
        app_objects.set_object_items(object_key("foo", &["a", "b", "c"]), Vec::new());
        app_objects.set_object_items(object_key("foo", &["a", "b", "c", "d"]), Vec::new());
        app_objects.set_object_items(object_key("foo", &["a", "b", "e"]), Vec::new());
        app_objects.set_object_items(object_key("foo", &["a", "f"]), Vec::new());
        app_objects.set_object_items(object_key("bar", &[]), Vec::new());
        app_objects.set_object_items(object_key("bar", &["a"]), Vec::new());
        app_objects.set_object_items(object_key("bar", &["a", "b"]), Vec::new());
        app_objects.set_object_items(object_key("bar", &["a", "b", "c"]), Vec::new());

        app_objects.clear_object_items_under(&object_key("foo", &["a", "b"]));

        let expected = vec![
            object_key("foo", &[]),
            object_key("foo", &["a"]),
            object_key("foo", &["a", "f"]),
            object_key("bar", &[]),
            object_key("bar", &["a"]),
            object_key("bar", &["a", "b"]),
            object_key("bar", &["a", "b", "c"]),
        ]
        .into_iter()
        .collect::<HashSet<_>>();

        let actual = app_objects
            .object_items_map
            .keys()
            .cloned()
            .collect::<HashSet<_>>();

        assert_eq!(actual, expected);
    }

    fn object_key(bucket_name: &str, object_path: &[&str]) -> ObjectKey {
        ObjectKey {
            bucket_name: bucket_name.to_string(),
            object_path: object_path.iter().map(|s| s.to_string()).collect(),
        }
    }
}
