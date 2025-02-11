use itsuki::zero_indexed_enum;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Stylize},
    text::Line,
    widgets::{
        block::Title, Block, BorderType, List, ListItem, Padding, StatefulWidget, WidgetRef,
    },
};

use crate::{
    color::ColorTheme,
    object::{BucketItem, FileDetail, FileVersion, ObjectItem},
    widget::{common::calc_centered_dialog_rect, Dialog},
};

#[derive(Default)]
#[zero_indexed_enum]
enum BucketListItemType {
    #[default]
    Name,
    S3Uri,
    Arn,
    ObjectUrl,
}

impl BucketListItemType {
    fn name_and_value(&self, bucket_item: &BucketItem) -> (String, String) {
        let (name, value) = match self {
            Self::Name => ("Name", &bucket_item.name),
            Self::S3Uri => ("S3 URI", &bucket_item.s3_uri),
            Self::Arn => ("ARN", &bucket_item.arn),
            Self::ObjectUrl => ("Object URL", &bucket_item.object_url),
        };
        (name.into(), value.into())
    }
}

#[derive(Default)]
#[zero_indexed_enum]
enum ObjectListFileItemType {
    #[default]
    Name,
    Key,
    S3Uri,
    Arn,
    ObjectUrl,
    Etag,
}

impl ObjectListFileItemType {
    fn name_and_value(&self, object_item: &ObjectItem) -> (String, String) {
        let (name, value) = match object_item {
            ObjectItem::Dir { .. } => unreachable!(),
            ObjectItem::File {
                name,
                key,
                s3_uri,
                arn,
                object_url,
                e_tag,
                ..
            } => match self {
                Self::Name => ("Name", name),
                Self::Key => ("Key", key),
                Self::S3Uri => ("S3 URI", s3_uri),
                Self::Arn => ("ARN", arn),
                Self::ObjectUrl => ("Object URL", object_url),
                Self::Etag => ("ETag", e_tag),
            },
        };
        (name.into(), value.into())
    }
}

#[derive(Default)]
#[zero_indexed_enum]
enum ObjectListDirItemType {
    #[default]
    Name,
    Key,
    S3Uri,
    ObjectUrl,
}

impl ObjectListDirItemType {
    fn name_and_value(&self, object_item: &ObjectItem) -> (String, String) {
        let (name, value) = match object_item {
            ObjectItem::Dir {
                name,
                key,
                s3_uri,
                object_url,
                ..
            } => match self {
                Self::Name => ("Name", name),
                Self::Key => ("Key", key),
                Self::S3Uri => ("S3 URI", s3_uri),
                Self::ObjectUrl => ("Object URL", object_url),
            },
            ObjectItem::File { .. } => unreachable!(),
        };
        (name.into(), value.into())
    }
}

#[derive(Default)]
#[zero_indexed_enum]
enum ObjectDetailItemType {
    #[default]
    Name,
    Key,
    S3Uri,
    Arn,
    ObjectUrl,
    Etag,
}

impl ObjectDetailItemType {
    fn name_and_value(&self, file_detail: &FileDetail) -> (String, String) {
        let (name, value) = match self {
            Self::Name => ("Name", &file_detail.name),
            Self::Key => ("Key", &file_detail.key),
            Self::S3Uri => ("S3 URI", &file_detail.s3_uri),
            Self::Arn => ("ARN", &file_detail.arn),
            Self::ObjectUrl => ("Object URL", &file_detail.object_url),
            Self::Etag => ("ETag", &file_detail.e_tag),
        };
        (name.into(), value.into())
    }
}

#[derive(Default)]
#[zero_indexed_enum]
enum ObjectVersionItemType {
    #[default]
    Name,
    Key,
    S3Uri,
    Arn,
    ObjectUrl,
    Etag,
}

impl ObjectVersionItemType {
    fn name_and_value(
        &self,
        file_detail: &FileDetail,
        file_version: &FileVersion,
    ) -> (String, String) {
        let (name, value) = match self {
            Self::Name => ("Name", &file_detail.name),
            Self::Key => ("Key", &file_detail.key),
            Self::S3Uri => ("S3 URI", &file_version.s3_uri(file_detail)),
            Self::Arn => ("ARN", &file_detail.arn),
            Self::ObjectUrl => ("Object URL", &file_version.object_url(file_detail)),
            Self::Etag => ("ETag", &file_version.e_tag),
        };
        (name.into(), value.into())
    }
}

#[derive(Debug)]
pub enum CopyDetailDialogState {
    BucketList(BucketListItemType, BucketItem),
    ObjectDetail(ObjectDetailItemType, FileDetail),
    ObjectVersion(ObjectVersionItemType, FileDetail, FileVersion),
    ObjectListFile(ObjectListFileItemType, ObjectItem),
    ObjectListDir(ObjectListDirItemType, ObjectItem),
}

impl CopyDetailDialogState {
    pub fn bucket_list(bucket_item: BucketItem) -> Self {
        Self::BucketList(BucketListItemType::default(), bucket_item)
    }

    pub fn object_list_file(object_item: ObjectItem) -> Self {
        Self::ObjectListFile(ObjectListFileItemType::default(), object_item)
    }

    pub fn object_list_dir(object_item: ObjectItem) -> Self {
        Self::ObjectListDir(ObjectListDirItemType::default(), object_item)
    }

    pub fn object_detail(file_detail: FileDetail) -> Self {
        Self::ObjectDetail(ObjectDetailItemType::default(), file_detail)
    }

    pub fn object_version(file_detail: FileDetail, file_version: FileVersion) -> Self {
        Self::ObjectVersion(ObjectVersionItemType::default(), file_detail, file_version)
    }
}

impl CopyDetailDialogState {
    pub fn select_next(&mut self) {
        match self {
            Self::BucketList(selected, _) => *selected = selected.next(),
            Self::ObjectDetail(selected, _) => *selected = selected.next(),
            Self::ObjectVersion(selected, _, _) => *selected = selected.next(),
            Self::ObjectListFile(selected, _) => *selected = selected.next(),
            Self::ObjectListDir(selected, _) => *selected = selected.next(),
        }
    }

    pub fn select_prev(&mut self) {
        match self {
            Self::BucketList(selected, _) => *selected = selected.prev(),
            Self::ObjectDetail(selected, _) => *selected = selected.prev(),
            Self::ObjectVersion(selected, _, _) => *selected = selected.prev(),
            Self::ObjectListFile(selected, _) => *selected = selected.prev(),
            Self::ObjectListDir(selected, _) => *selected = selected.prev(),
        }
    }

    fn selected_value(&self) -> usize {
        match self {
            Self::BucketList(selected, _) => selected.val(),
            Self::ObjectDetail(selected, _) => selected.val(),
            Self::ObjectVersion(selected, _, _) => selected.val(),
            Self::ObjectListFile(selected, _) => selected.val(),
            Self::ObjectListDir(selected, _) => selected.val(),
        }
    }

    pub fn selected_name_and_value(&self) -> (String, String) {
        match self {
            Self::BucketList(selected, bucket_item) => selected.name_and_value(bucket_item),
            Self::ObjectDetail(selected, file_detail) => selected.name_and_value(file_detail),
            Self::ObjectVersion(selected, file_detail, file_version) => {
                selected.name_and_value(file_detail, file_version)
            }
            Self::ObjectListFile(selected, object_item) => selected.name_and_value(object_item),
            Self::ObjectListDir(selected, object_item) => selected.name_and_value(object_item),
        }
    }

    fn name_and_value_vec(&self) -> Vec<(String, String)> {
        match self {
            Self::BucketList(_, bucket_item) => BucketListItemType::vars_array()
                .into_iter()
                .map(|t| t.name_and_value(bucket_item))
                .collect(),
            Self::ObjectDetail(_, file_detail) => ObjectDetailItemType::vars_array()
                .into_iter()
                .map(|t| t.name_and_value(file_detail))
                .collect(),
            Self::ObjectVersion(_, file_detail, file_version) => {
                ObjectVersionItemType::vars_array()
                    .into_iter()
                    .map(|t| t.name_and_value(file_detail, file_version))
                    .collect()
            }
            Self::ObjectListFile(_, object_item) => ObjectListFileItemType::vars_array()
                .into_iter()
                .map(|t| t.name_and_value(object_item))
                .collect(),
            Self::ObjectListDir(_, object_item) => ObjectListDirItemType::vars_array()
                .into_iter()
                .map(|t| t.name_and_value(object_item))
                .collect(),
        }
    }

    fn item_type_len(&self) -> usize {
        match self {
            Self::BucketList(_, _) => BucketListItemType::len(),
            Self::ObjectDetail(_, _) => ObjectDetailItemType::len(),
            Self::ObjectVersion(_, _, _) => ObjectVersionItemType::len(),
            Self::ObjectListFile(_, _) => ObjectListFileItemType::len(),
            Self::ObjectListDir(_, _) => ObjectListDirItemType::len(),
        }
    }
}

#[derive(Debug, Default)]
struct CopyDetailDialogColor {
    bg: Color,
    block: Color,
    text: Color,
    selected: Color,
}

impl CopyDetailDialogColor {
    fn new(theme: &ColorTheme) -> Self {
        Self {
            bg: theme.bg,
            block: theme.fg,
            text: theme.fg,
            selected: theme.dialog_selected,
        }
    }
}

#[derive(Debug, Default)]
pub struct CopyDetailDialog {
    color: CopyDetailDialogColor,
}

impl CopyDetailDialog {
    pub fn theme(mut self, theme: &ColorTheme) -> Self {
        self.color = CopyDetailDialogColor::new(theme);
        self
    }
}

impl StatefulWidget for CopyDetailDialog {
    type State = CopyDetailDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let selected = state.selected_value();
        let list_items: Vec<ListItem> = state
            .name_and_value_vec()
            .into_iter()
            .enumerate()
            .map(|(i, (name, value))| self.build_list_item(i, selected, (name, value)))
            .collect();

        let dialog_width = (area.width - 4).min(80);
        let dialog_height = state.item_type_len() * 2 + 2 /* border */;
        let area = calc_centered_dialog_rect(area, dialog_width, dialog_height as u16);

        let title = Title::from("Copy");
        let list = List::new(list_items).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(title)
                .bg(self.color.bg)
                .fg(self.color.block)
                .padding(Padding::horizontal(1)),
        );
        let dialog = Dialog::new(Box::new(list), self.color.bg);
        dialog.render_ref(area, buf);
    }
}

impl CopyDetailDialog {
    fn build_list_item<'a>(
        &self,
        i: usize,
        selected: usize,
        (name, value): (String, String),
    ) -> ListItem<'a> {
        let item = ListItem::new(vec![
            Line::from(format!("{}:", name).add_modifier(Modifier::BOLD)),
            Line::from(format!("  {}", value)),
        ]);
        if i == selected {
            item.fg(self.color.selected)
        } else {
            item.fg(self.color.text)
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Local, NaiveDateTime};

    use crate::set_cells;

    use super::*;

    #[test]
    fn test_render_copy_detail_dialog() {
        let file_detail = file_detail();
        let theme = ColorTheme::default();
        let mut state = CopyDetailDialogState::object_detail(file_detail);
        let copy_detail_dialog = CopyDetailDialog::default().theme(&theme);

        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 20));
        copy_detail_dialog.render(buf.area, &mut buf, &mut state);

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "                                        ",
            "                                        ",
            "                                        ",
            "  ╭Copy──────────────────────────────╮  ",
            "  │ Name:                            │  ",
            "  │   file.txt                       │  ",
            "  │ Key:                             │  ",
            "  │   file.txt                       │  ",
            "  │ S3 URI:                          │  ",
            "  │   s3://bucket-1/file.txt         │  ",
            "  │ ARN:                             │  ",
            "  │   arn:aws:s3:::bucket-1/file.txt │  ",
            "  │ Object URL:                      │  ",
            "  │   https://bucket-1.s3.ap-northea │  ",
            "  │ ETag:                            │  ",
            "  │   bef684de-a260-48a4-8178-8a535e │  ",
            "  ╰──────────────────────────────────╯  ",
            "                                        ",
            "                                        ",
            "                                        ",
        ]);
        set_cells! { expected =>
            // "Name" is bold
            (4..9, [4]) => modifier: Modifier::BOLD,
            // "Key" is bold
            (4..8, [6]) => modifier: Modifier::BOLD,
            // "S3 URI" is bold
            (4..11, [8]) => modifier: Modifier::BOLD,
            // "ARN" is bold
            (4..8, [10]) => modifier: Modifier::BOLD,
            // "Object URL" is bold
            (4..15, [12]) => modifier: Modifier::BOLD,
            // "ETag" is bold
            (4..9, [14]) => modifier: Modifier::BOLD,
            // selected item
            (4..36, [4, 5]) => fg: Color::Cyan,
        }

        assert_eq!(buf, expected);
    }

    fn file_detail() -> FileDetail {
        FileDetail {
            name: "file.txt".to_string(),
            size_byte: 1024 + 10,
            last_modified: parse_datetime("2024-01-02 13:01:02"),
            e_tag: "bef684de-a260-48a4-8178-8a535ecccadb".to_string(),
            content_type: "text/plain".to_string(),
            storage_class: "STANDARD".to_string(),
            key: "file.txt".to_string(),
            s3_uri: "s3://bucket-1/file.txt".to_string(),
            arn: "arn:aws:s3:::bucket-1/file.txt".to_string(),
            object_url: "https://bucket-1.s3.ap-northeast-1.amazonaws.com/file.txt".to_string(),
        }
    }

    fn parse_datetime(s: &str) -> DateTime<Local> {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
    }
}
