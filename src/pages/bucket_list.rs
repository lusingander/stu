use ratatui::{
    layout::{Alignment, Margin, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, List, ListItem, Padding},
    Frame,
};

use crate::{
    component::AppListState, event::AppEventType, object::BucketItem, util::digits,
    widget::ScrollBar,
};

const SELECTED_COLOR: Color = Color::Cyan;
const SELECTED_ITEM_TEXT_COLOR: Color = Color::Black;

#[derive(Debug)]
pub struct BucketListPage {
    bucket_items: Vec<BucketItem>,

    list_state: AppListState,
}

impl BucketListPage {
    pub fn new(bucket_items: Vec<BucketItem>) -> Self {
        Self {
            bucket_items,
            list_state: AppListState::default(),
        }
    }

    pub fn handle_event(&mut self, _event: AppEventType) {}

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        // todo: reconsider list state management
        self.list_state.height = area.height as usize - 2 /* border */;

        let list_state = ListViewState {
            current_selected: self.list_state.selected,
            current_offset: self.list_state.offset,
        };
        let styles = ListItemStyles {
            selected_bg_color: SELECTED_COLOR,
            selected_fg_color: SELECTED_ITEM_TEXT_COLOR,
        };
        let list_items =
            build_list_items_from_bucket_items(&self.bucket_items, list_state, area, styles);
        let list = build_list(
            list_items,
            self.bucket_items.len(),
            list_state.current_selected,
        );
        f.render_widget(list, area);

        render_list_scroll_bar(f, area, list_state, self.bucket_items.len());
    }

    pub fn select_next(&mut self) {
        if self.list_state.selected >= self.bucket_items.len() - 1 {
            self.list_state.select_first();
        } else {
            self.list_state.select_next();
        };
    }

    pub fn select_prev(&mut self) {
        if self.list_state.selected == 0 {
            self.list_state.select_last(self.bucket_items.len());
        } else {
            self.list_state.select_prev();
        };
    }

    pub fn select_first(&mut self) {
        self.list_state.select_first();
    }

    pub fn select_last(&mut self) {
        self.list_state.select_last(self.bucket_items.len());
    }

    pub fn select_next_page(&mut self) {
        self.list_state.select_next_page(self.bucket_items.len());
    }

    pub fn select_prev_page(&mut self) {
        self.list_state.select_prev_page(self.bucket_items.len());
    }

    pub fn current_selected_item(&self) -> &BucketItem {
        self.bucket_items
            .get(self.list_state.selected)
            .unwrap_or_else(|| {
                panic!(
                    "selected index {} is out of range {}",
                    self.list_state.selected,
                    self.bucket_items.len()
                )
            })
    }
}

fn build_list_items_from_bucket_items(
    current_items: &[BucketItem],
    list_state: ListViewState,
    area: Rect,
    styles: ListItemStyles,
) -> Vec<ListItem> {
    let show_item_count = (area.height as usize) - 2 /* border */;
    current_items
        .iter()
        .skip(list_state.current_offset)
        .take(show_item_count)
        .enumerate()
        .map(|(idx, item)| build_list_item_from_bucket_item(idx, item, list_state, area, styles))
        .collect()
}

fn build_list_item_from_bucket_item(
    idx: usize,
    item: &BucketItem,
    list_state: ListViewState,
    area: Rect,
    styles: ListItemStyles,
) -> ListItem {
    let content = format_bucket_item(&item.name, area.width);
    let style = Style::default();
    let span = Span::styled(content, style);
    if idx + list_state.current_offset == list_state.current_selected {
        ListItem::new(span).style(
            Style::default()
                .bg(styles.selected_bg_color)
                .fg(styles.selected_fg_color),
        )
    } else {
        ListItem::new(span)
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

fn format_bucket_item(name: &str, width: u16) -> String {
    let name_w: usize = (width as usize) - 2 /* spaces */ - 2 /* border */;
    format!(" {:<name_w$} ", name, name_w = name_w)
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
            let items = ["bucket1", "bucket2", "bucket3"]
                .iter()
                .map(|name| BucketItem {
                    name: name.to_string(),
                })
                .collect();
            let mut page = BucketListPage::new(items);
            let area = Rect::new(0, 0, 30, 10);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────── 1 / 3 ┐",
            "│  bucket1                   │",
            "│  bucket2                   │",
            "│  bucket3                   │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "└────────────────────────────┘",
        ]);
        for x in 2..28 {
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
            let items = (0..16)
                .map(|i| BucketItem {
                    name: format!("bucket{}", i + 1),
                })
                .collect();
            let mut page = BucketListPage::new(items);
            let area = Rect::new(0, 0, 30, 10);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────  1 / 16 ┐",
            "│  bucket1                  ││",
            "│  bucket2                  ││",
            "│  bucket3                  ││",
            "│  bucket4                  ││",
            "│  bucket5                   │",
            "│  bucket6                   │",
            "│  bucket7                   │",
            "│  bucket8                   │",
            "└────────────────────────────┘",
        ]);
        for x in 2..28 {
            // selected item
            expected.get_mut(x, 1).set_bg(Color::Cyan);
            expected.get_mut(x, 1).set_fg(Color::Black);
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    fn setup_terminal() -> std::io::Result<Terminal<TestBackend>> {
        let backend = TestBackend::new(30, 10);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(terminal)
    }
}
