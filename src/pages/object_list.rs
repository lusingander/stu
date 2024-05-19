use chrono::{DateTime, Local};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::ListItem,
    Frame,
};

use crate::{
    event::AppEventType,
    object::ObjectItem,
    ui::common::{format_datetime, format_size_byte},
    widget::{ScrollList, ScrollListState},
};

const SELECTED_COLOR: Color = Color::Cyan;
const SELECTED_ITEM_TEXT_COLOR: Color = Color::Black;

#[derive(Debug)]
pub struct ObjectListPage {
    object_items: Vec<ObjectItem>,

    list_state: ScrollListState,
}

impl ObjectListPage {
    pub fn new(object_items: Vec<ObjectItem>) -> Self {
        let items_len = object_items.len();
        Self {
            object_items,
            list_state: ScrollListState::new(items_len),
        }
    }

    pub fn handle_event(&mut self, _event: AppEventType) {}

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let offset = self.list_state.offset;
        let selected = self.list_state.selected;

        let list_items =
            build_list_items_from_object_items(&self.object_items, offset, selected, area);

        let list = ScrollList::new(list_items);
        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    pub fn select_next(&mut self) {
        self.list_state.select_next();
    }

    pub fn select_prev(&mut self) {
        self.list_state.select_prev();
    }

    pub fn select_first(&mut self) {
        self.list_state.select_first();
    }

    pub fn select_last(&mut self) {
        self.list_state.select_last();
    }

    pub fn select_next_page(&mut self) {
        self.list_state.select_next_page();
    }

    pub fn select_prev_page(&mut self) {
        self.list_state.select_prev_page();
    }

    pub fn current_selected_item(&self) -> &ObjectItem {
        self.object_items
            .get(self.list_state.selected)
            .unwrap_or_else(|| {
                panic!(
                    "selected index {} is out of range {}",
                    self.list_state.selected,
                    self.object_items.len()
                )
            })
    }

    pub fn object_list(&self) -> &Vec<ObjectItem> {
        &self.object_items
    }

    pub fn list_state(&self) -> ScrollListState {
        self.list_state
    }
}

fn build_list_items_from_object_items(
    current_items: &[ObjectItem],
    offset: usize,
    selected: usize,
    area: Rect,
) -> Vec<ListItem> {
    let show_item_count = (area.height as usize) - 2 /* border */;
    current_items
        .iter()
        .skip(offset)
        .take(show_item_count)
        .enumerate()
        .map(|(idx, item)| build_list_item_from_object_item(idx, item, offset, selected, area))
        .collect()
}

fn build_list_item_from_object_item(
    idx: usize,
    item: &ObjectItem,
    offset: usize,
    selected: usize,
    area: Rect,
) -> ListItem {
    let content = match item {
        ObjectItem::Dir { name, .. } => {
            let content = format_dir_item(name, area.width);
            let style = Style::default().add_modifier(Modifier::BOLD);
            Span::styled(content, style)
        }
        ObjectItem::File {
            name,
            size_byte,
            last_modified,
            ..
        } => {
            let content = format_file_item(name, *size_byte, last_modified, area.width);
            let style = Style::default();
            Span::styled(content, style)
        }
    };
    if idx + offset == selected {
        ListItem::new(content).style(
            Style::default()
                .bg(SELECTED_COLOR)
                .fg(SELECTED_ITEM_TEXT_COLOR),
        )
    } else {
        ListItem::new(content)
    }
}

fn format_dir_item(name: &str, width: u16) -> String {
    let name_w: usize = (width as usize) - 2 /* spaces */ - 2 /* border */;
    let name = format!("{}/", name);
    format!(" {:<name_w$} ", name, name_w = name_w)
}

fn format_file_item(
    name: &str,
    size_byte: usize,
    last_modified: &DateTime<Local>,
    width: u16,
) -> String {
    let size = format_size_byte(size_byte);
    let date = format_datetime(last_modified);
    let date_w: usize = 19;
    let size_w: usize = 10;
    let name_w: usize = (width as usize) - date_w - size_w - 10 /* spaces */ - 4 /* border + space */;
    format!(
        " {:<name_w$}    {:<date_w$}    {:>size_w$} ",
        name,
        date,
        size,
        name_w = name_w,
        date_w = date_w,
        size_w = size_w
    )
}

#[cfg(test)]
mod tests {
    use crate::set_cells;

    use super::*;
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

    #[test]
    fn test_render_without_scroll() -> std::io::Result<()> {
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let items = vec![
                ObjectItem::Dir {
                    name: "dir1".to_string(),
                    paths: vec![],
                },
                ObjectItem::Dir {
                    name: "dir2".to_string(),
                    paths: vec![],
                },
                ObjectItem::File {
                    name: "file1".to_string(),
                    size_byte: 1024 + 10,
                    last_modified: parse_datetime("2024-01-02T13:01:02+09:00"),
                    paths: vec![],
                },
                ObjectItem::File {
                    name: "file2".to_string(),
                    size_byte: 1024 * 999,
                    last_modified: parse_datetime("2023-12-31T09:00:00+09:00"),
                    paths: vec![],
                },
            ];
            let mut page = ObjectListPage::new(items);
            let area = Rect::new(0, 0, 60, 10);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌─────────────────────────────────────────────────── 1 / 4 ┐",
            "│  dir1/                                                   │",
            "│  dir2/                                                   │",
            "│  file1                2024-01-02 13:01:02      1.01 KiB  │",
            "│  file2                2023-12-31 09:00:00       999 KiB  │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "└──────────────────────────────────────────────────────────┘",
        ]);
        set_cells! { expected =>
            // dir items
            (2..58, [1, 2]) => modifier: Modifier::BOLD,
            // selected item
            (2..58, [1]) => bg: Color::Cyan, fg: Color::Black,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn test_render_with_scroll() -> std::io::Result<()> {
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let items = (0..32)
                .map(|i| ObjectItem::File {
                    name: format!("file{}", i + 1),
                    size_byte: 1024,
                    last_modified: parse_datetime("2024-01-02T13:01:02+09:00"),
                    paths: vec![],
                })
                .collect();
            let mut page = ObjectListPage::new(items);
            let area = Rect::new(0, 0, 60, 10);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
                "┌─────────────────────────────────────────────────  1 / 32 ┐",
                "│  file1                2024-01-02 13:01:02         1 KiB ││",
                "│  file2                2024-01-02 13:01:02         1 KiB ││",
                "│  file3                2024-01-02 13:01:02         1 KiB  │",
                "│  file4                2024-01-02 13:01:02         1 KiB  │",
                "│  file5                2024-01-02 13:01:02         1 KiB  │",
                "│  file6                2024-01-02 13:01:02         1 KiB  │",
                "│  file7                2024-01-02 13:01:02         1 KiB  │",
                "│  file8                2024-01-02 13:01:02         1 KiB  │",
                "└──────────────────────────────────────────────────────────┘",
        ]);
        set_cells! { expected =>
            // selected item
            (2..58, [1]) => bg: Color::Cyan, fg: Color::Black,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    fn setup_terminal() -> std::io::Result<Terminal<TestBackend>> {
        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(terminal)
    }

    fn parse_datetime(s: &str) -> DateTime<Local> {
        DateTime::parse_from_rfc3339(s)
            .unwrap()
            .with_timezone(&Local)
    }
}
