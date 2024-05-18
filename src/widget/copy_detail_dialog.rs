use itsuki::zero_indexed_enum;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Stylize},
    text::Line,
    widgets::{block::Title, Block, BorderType, List, ListItem, Padding, Widget, WidgetRef},
};

use crate::{object::FileDetail, ui::common::calc_centered_dialog_rect, widget::Dialog};

const SELECTED_COLOR: Color = Color::Cyan;

#[derive(Default)]
#[zero_indexed_enum]
enum ItemType {
    #[default]
    Key,
    S3Uri,
    Arn,
    ObjectUrl,
    Etag,
}

impl ItemType {
    pub fn name_and_value(&self, file_detail: &FileDetail) -> (String, String) {
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

#[derive(Debug, Default, Clone, Copy)]
pub struct CopyDetailDialogState {
    selected: ItemType,
}

impl CopyDetailDialogState {
    pub fn select_next(&mut self) {
        self.selected = self.selected.next();
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.prev();
    }

    pub fn selected_name_and_value(&self, file_detail: &FileDetail) -> (String, String) {
        self.selected.name_and_value(file_detail)
    }
}

pub struct CopyDetailDialog<'a> {
    state: CopyDetailDialogState,
    file_detail: &'a FileDetail,
}

impl<'a> CopyDetailDialog<'a> {
    pub fn new(state: CopyDetailDialogState, file_detail: &'a FileDetail) -> Self {
        Self { state, file_detail }
    }
}

impl Widget for CopyDetailDialog<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let selected = self.state.selected.val();
        let list_items: Vec<ListItem> = ItemType::vars_vec()
            .iter()
            .enumerate()
            .map(|(i, item_type)| build_list_item(i, selected, *item_type, self.file_detail))
            .collect();

        let dialog_width = (area.width - 4).min(80);
        let dialog_height = 2 * 5 /* list */ + 2 /* border */;
        let area = calc_centered_dialog_rect(area, dialog_width, dialog_height);

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

fn build_list_item(
    i: usize,
    selected: usize,
    item_type: ItemType,
    file_detail: &FileDetail,
) -> ListItem {
    let (name, value) = item_type.name_and_value(file_detail);
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
