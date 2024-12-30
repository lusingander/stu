use itsuki::zero_indexed_enum;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    text::Line,
    widgets::{block::Title, Block, BorderType, List, ListItem, Padding, Widget, WidgetRef},
};

use crate::{
    color::ColorTheme,
    widget::{common::calc_centered_dialog_rect, Dialog},
};

#[derive(Default)]
#[zero_indexed_enum]
pub enum BucketListSortType {
    #[default]
    Default,
    NameAsc,
    NameDesc,
}

impl BucketListSortType {
    pub fn str(&self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::NameAsc => "Name (Asc)",
            Self::NameDesc => "Name (Desc)",
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BucketListSortDialogState {
    selected: BucketListSortType,
}

impl BucketListSortDialogState {
    pub fn select_next(&mut self) {
        self.selected = self.selected.next();
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.prev();
    }

    pub fn reset(&mut self) {
        self.selected = BucketListSortType::Default;
    }

    pub fn selected(&self) -> BucketListSortType {
        self.selected
    }
}

pub struct BucketListSortDialog {
    state: BucketListSortDialogState,
    labels: Vec<&'static str>,
    color: ListSortDialogColor,
}

impl BucketListSortDialog {
    pub fn new(state: BucketListSortDialogState) -> Self {
        let labels = BucketListSortType::vars_vec()
            .iter()
            .map(|sort_type| sort_type.str())
            .collect();
        Self {
            state,
            labels,
            color: ListSortDialogColor::default(),
        }
    }

    pub fn theme(mut self, theme: &ColorTheme) -> Self {
        self.color = ListSortDialogColor::new(theme);
        self
    }
}

impl Widget for BucketListSortDialog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let dialog = ListSortDialog::new(self.state.selected.val(), self.labels, self.color);
        dialog.render(area, buf);
    }
}

#[derive(Default)]
#[zero_indexed_enum]
pub enum ObjectListSortType {
    #[default]
    Default,
    NameAsc,
    NameDesc,
    LastModifiedAsc,
    LastModifiedDesc,
    SizeAsc,
    SizeDesc,
}

impl ObjectListSortType {
    pub fn str(&self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::NameAsc => "Name (Asc)",
            Self::NameDesc => "Name (Desc)",
            Self::LastModifiedAsc => "Last Modified (Asc)",
            Self::LastModifiedDesc => "Last Modified (Desc)",
            Self::SizeAsc => "Size (Asc)",
            Self::SizeDesc => "Size (Desc)",
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ObjectListSortDialogState {
    selected: ObjectListSortType,
}

impl ObjectListSortDialogState {
    pub fn select_next(&mut self) {
        self.selected = self.selected.next();
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.prev();
    }

    pub fn reset(&mut self) {
        self.selected = ObjectListSortType::Default;
    }

    pub fn selected(&self) -> ObjectListSortType {
        self.selected
    }
}

pub struct ObjectListSortDialog {
    state: ObjectListSortDialogState,
    labels: Vec<&'static str>,
    color: ListSortDialogColor,
}

impl ObjectListSortDialog {
    pub fn new(state: ObjectListSortDialogState) -> Self {
        let labels = ObjectListSortType::vars_vec()
            .iter()
            .map(|sort_type| sort_type.str())
            .collect();
        Self {
            state,
            labels,
            color: ListSortDialogColor::default(),
        }
    }

    pub fn theme(mut self, theme: &ColorTheme) -> Self {
        self.color = ListSortDialogColor::new(theme);
        self
    }
}

impl Widget for ObjectListSortDialog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let dialog = ListSortDialog::new(self.state.selected.val(), self.labels, self.color);
        dialog.render(area, buf);
    }
}

#[derive(Debug, Default)]
struct ListSortDialogColor {
    bg: Color,
    block: Color,
    text: Color,
    selected: Color,
}

impl ListSortDialogColor {
    fn new(theme: &ColorTheme) -> ListSortDialogColor {
        ListSortDialogColor {
            bg: theme.bg,
            block: theme.fg,
            text: theme.fg,
            selected: theme.dialog_selected,
        }
    }
}

struct ListSortDialog {
    selected: usize,
    labels: Vec<&'static str>,
    color: ListSortDialogColor,
}

impl ListSortDialog {
    fn new(selected: usize, labels: Vec<&'static str>, color: ListSortDialogColor) -> Self {
        Self {
            selected,
            labels,
            color,
        }
    }
}

impl Widget for ListSortDialog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let list_items: Vec<ListItem> = self
            .labels
            .iter()
            .enumerate()
            .map(|(i, label)| {
                let item = ListItem::new(Line::raw(*label));
                if i == self.selected {
                    item.fg(self.color.selected)
                } else {
                    item.fg(self.color.text)
                }
            })
            .collect();

        let dialog_width = (area.width - 4).min(30);
        let dialog_height = self.labels.len() as u16 + 2 /* border */;
        let area = calc_centered_dialog_rect(area, dialog_width, dialog_height);

        let title = Title::from("Sort");
        let list = List::new(list_items).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(title)
                .padding(Padding::horizontal(1))
                .bg(self.color.bg)
                .fg(self.color.block),
        );
        let dialog = Dialog::new(Box::new(list), self.color.bg);
        dialog.render_ref(area, buf);
    }
}
