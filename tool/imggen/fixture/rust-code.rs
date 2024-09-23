use chrono::{DateTime, Local};

pub const APP_NAME: &str = "STU";

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
    pub fn last_modified(&self) -> Option<DateTime<Local>> {
        match self {
            ObjectItem::Dir { .. } => None,
            ObjectItem::File { last_modified, .. } => Some(*last_modified),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_item() {
        let obj = ObjectItem::File {
            name: "file.txt".to_string(),
            size_byte: 100,
            last_modified: Local::now(),
        };
        assert_eq!(obj.last_modified().is_some(), true);
    }
}
