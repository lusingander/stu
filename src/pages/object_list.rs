use chrono::{DateTime, Local};
use ratatui::{
    layout::{Alignment, Margin, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, List, ListItem, Padding},
    Frame,
};

use crate::{component::AppListState, object::ObjectItem, util::digits, widget::ScrollBar};

const SELECTED_COLOR: Color = Color::Cyan;
const SELECTED_ITEM_TEXT_COLOR: Color = Color::Black;

pub struct ObjectListPage {
    object_items: Vec<ObjectItem>,

    list_state: AppListState,
}

impl ObjectListPage {
    pub fn new(object_items: Vec<ObjectItem>, list_state: AppListState) -> Self {
        Self {
            object_items,
            list_state,
        }
    }
}

impl ObjectListPage {
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let list_state = ListViewState {
            current_selected: self.list_state.selected,
            current_offset: self.list_state.offset,
        };
        let styles = ListItemStyles {
            selected_bg_color: SELECTED_COLOR,
            selected_fg_color: SELECTED_ITEM_TEXT_COLOR,
        };
        let list_items =
            build_list_items_from_object_items(&self.object_items, list_state, area, styles, true);
        let list = build_list(
            list_items,
            self.object_items.len(),
            list_state.current_selected,
        );
        f.render_widget(list, area);

        render_list_scroll_bar(f, area, list_state, self.object_items.len());
    }
}

fn build_list_items_from_object_items(
    current_items: &[ObjectItem],
    list_state: ListViewState,
    area: Rect,
    styles: ListItemStyles,
    show_file_detail: bool,
) -> Vec<ListItem> {
    let show_item_count = (area.height as usize) - 2 /* border */;
    current_items
        .iter()
        .skip(list_state.current_offset)
        .take(show_item_count)
        .enumerate()
        .map(|(idx, item)| {
            build_list_item_from_object_item(idx, item, list_state, area, styles, show_file_detail)
        })
        .collect()
}

fn build_list_item_from_object_item(
    idx: usize,
    item: &ObjectItem,
    list_state: ListViewState,
    area: Rect,
    styles: ListItemStyles,
    show_file_detail: bool,
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
            let content = format_file_item(
                name,
                *size_byte,
                last_modified,
                area.width,
                show_file_detail,
            );
            let style = Style::default();
            Span::styled(content, style)
        }
    };
    if idx + list_state.current_offset == list_state.current_selected {
        ListItem::new(content).style(
            Style::default()
                .bg(styles.selected_bg_color)
                .fg(styles.selected_fg_color),
        )
    } else {
        ListItem::new(content)
    }
}

fn build_list(list_items: Vec<ListItem>, total_count: usize, current_selected: usize) -> List {
    let title = format_list_count(total_count, current_selected);
    List::new(list_items).block(
        Block::bordered()
            .title(title)
            .title_alignment(Alignment::Right)
            .padding(Padding::horizontal(1)),
    )
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
    show_file_detail: bool,
) -> String {
    if show_file_detail {
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
    } else {
        let name_w: usize = (width as usize) - 2 /* spaces */ - 4 /* border */;
        format!(" {:<name_w$} ", name, name_w = name_w)
    }
}

fn format_list_count(total_count: usize, current_selected: usize) -> String {
    if total_count == 0 {
        String::new()
    } else {
        format_count(current_selected + 1, total_count)
    }
}

fn format_count(selected: usize, total: usize) -> String {
    let digits = digits(total);
    format!(" {:>digits$} / {} ", selected, total)
}

fn format_size_byte(size_byte: usize) -> String {
    humansize::format_size_i(size_byte, humansize::BINARY)
}

#[cfg(not(feature = "imggen"))]
fn format_datetime(datetime: &DateTime<Local>) -> String {
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(feature = "imggen")]
fn format_datetime(_datetime: &DateTime<Local>) -> String {
    String::from("2024-01-02 13:04:05")
}

fn render_list_scroll_bar(
    f: &mut Frame,
    area: Rect,
    list_state: ListViewState,
    current_items_len: usize,
) {
    let area = area.inner(&Margin::new(2, 1));
    let scrollbar_area = Rect::new(area.right(), area.top(), 1, area.height);

    if current_items_len > (scrollbar_area.height as usize) {
        let scroll_bar = ScrollBar::new(current_items_len, list_state.current_offset);
        f.render_widget(scroll_bar, scrollbar_area);
    }
}

#[derive(Clone, Copy, Debug)]
struct ListViewState {
    current_selected: usize,
    current_offset: usize,
}

#[derive(Clone, Copy, Debug)]
struct ListItemStyles {
    selected_bg_color: Color,
    selected_fg_color: Color,
}

#[cfg(test)]
mod tests {
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
            let mut page = ObjectListPage::new(items, AppListState::new(10));
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
        for x in 2..58 {
            // dir items
            expected
                .get_mut(x, 1)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
            expected
                .get_mut(x, 2)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 2..58 {
            // selected item
            expected.get_mut(x, 1).set_bg(Color::Cyan);
            expected.get_mut(x, 1).set_fg(Color::Black);
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
            let mut page = ObjectListPage::new(items, AppListState::new(10));
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
        for x in 2..58 {
            expected.get_mut(x, 1).set_bg(Color::Cyan);
            expected.get_mut(x, 1).set_fg(Color::Black);
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
