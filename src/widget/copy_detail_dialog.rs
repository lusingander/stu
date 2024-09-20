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

use crate::{object::FileDetail, ui::common::calc_centered_dialog_rect, widget::Dialog};

const SELECTED_COLOR: Color = Color::Cyan;

#[derive(Default)]
#[zero_indexed_enum]
enum ObjectDetailItemType {
    #[default]
    Key,
    S3Uri,
    Arn,
    ObjectUrl,
    Etag,
}

impl ObjectDetailItemType {
    fn name_and_value(&self, file_detail: &FileDetail) -> (String, String) {
        let (name, value) = match self {
            Self::Key => ("Key", &file_detail.key),
            Self::S3Uri => ("S3 URI", &file_detail.s3_uri),
            Self::Arn => ("ARN", &file_detail.arn),
            Self::ObjectUrl => ("Object URL", &file_detail.object_url),
            Self::Etag => ("ETag", &file_detail.e_tag),
        };
        (name.into(), value.into())
    }
}

#[derive(Debug)]
pub enum CopyDetailDialogState {
    ObjectDetail(ObjectDetailItemType, FileDetail),
}

impl CopyDetailDialogState {
    pub fn object_detail(file_detail: FileDetail) -> Self {
        Self::ObjectDetail(ObjectDetailItemType::default(), file_detail)
    }

    pub fn select_next(&mut self) {
        match self {
            Self::ObjectDetail(selected, _) => *selected = selected.next(),
        }
    }

    pub fn select_prev(&mut self) {
        match self {
            Self::ObjectDetail(selected, _) => *selected = selected.prev(),
        }
    }

    fn selected_value(&self) -> usize {
        match self {
            Self::ObjectDetail(selected, _) => selected.val(),
        }
    }

    pub fn selected_name_and_value(&self) -> (String, String) {
        match self {
            Self::ObjectDetail(selected, file_detail) => selected.name_and_value(file_detail),
        }
    }

    fn name_and_value_iter(&self) -> impl Iterator<Item = (String, String)> + '_ {
        match self {
            Self::ObjectDetail(_, file_detail) => ObjectDetailItemType::vars_array()
                .into_iter()
                .map(|t| t.name_and_value(file_detail)),
        }
    }

    fn item_type_len(&self) -> usize {
        match self {
            Self::ObjectDetail(_, _) => ObjectDetailItemType::len(),
        }
    }
}

#[derive(Debug, Default)]
pub struct CopyDetailDialog {}

impl StatefulWidget for CopyDetailDialog {
    type State = CopyDetailDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let selected = state.selected_value();
        let list_items: Vec<ListItem> = state
            .name_and_value_iter()
            .enumerate()
            .map(|(i, (name, value))| build_list_item(i, selected, (name, value)))
            .collect();

        let dialog_width = (area.width - 4).min(80);
        let dialog_height = state.item_type_len() * 2 + 2 /* border */;
        let area = calc_centered_dialog_rect(area, dialog_width, dialog_height as u16);

        let title = Title::from("Copy");
        let list = List::new(list_items).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(title)
                .padding(Padding::horizontal(1)),
        );
        let dialog = Dialog::new(Box::new(list));
        dialog.render_ref(area, buf);
    }
}

fn build_list_item<'a>(i: usize, selected: usize, (name, value): (String, String)) -> ListItem<'a> {
    let item = ListItem::new(vec![
        Line::from(format!("{}:", name).add_modifier(Modifier::BOLD)),
        Line::from(format!("  {}", value)),
    ]);
    if i == selected {
        item.fg(SELECTED_COLOR)
    } else {
        item
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
        let mut state = CopyDetailDialogState::object_detail(file_detail);
        let copy_detail_dialog = CopyDetailDialog::default();

        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 20));
        copy_detail_dialog.render(buf.area, &mut buf, &mut state);

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "                                        ",
            "                                        ",
            "                                        ",
            "                                        ",
            "  ╭Copy──────────────────────────────╮  ",
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
            "                                        ",
        ]);
        set_cells! { expected =>
            // "Key" is bold
            (4..8, [5]) => modifier: Modifier::BOLD,
            // "S3 URI" is bold
            (4..11, [7]) => modifier: Modifier::BOLD,
            // "ARN" is bold
            (4..8, [9]) => modifier: Modifier::BOLD,
            // "Object URL" is bold
            (4..15, [11]) => modifier: Modifier::BOLD,
            // "ETag" is bold
            (4..9, [13]) => modifier: Modifier::BOLD,
            // selected item
            (4..36, [5, 6]) => fg: Color::Cyan,
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
