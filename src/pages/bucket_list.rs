use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::ListItem,
    Frame,
};

use crate::{
    event::{AppEventType, AppKeyAction, Sender},
    key_code, key_code_char,
    object::BucketItem,
    pages::util::{build_helps, build_short_helps},
    widget::{ScrollList, ScrollListState},
};

const SELECTED_COLOR: Color = Color::Cyan;
const SELECTED_ITEM_TEXT_COLOR: Color = Color::Black;

#[derive(Debug)]
pub struct BucketListPage {
    bucket_items: Vec<BucketItem>,

    list_state: ScrollListState,
    tx: Sender,
}

impl BucketListPage {
    pub fn new(bucket_items: Vec<BucketItem>, tx: Sender) -> Self {
        let items_len = bucket_items.len();
        Self {
            bucket_items,
            list_state: ScrollListState::new(items_len),
            tx,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key {
            key_code!(KeyCode::Esc) => {
                self.tx.send(AppEventType::Quit);
            }
            key_code!(KeyCode::Enter) => {
                self.tx
                    .send(AppEventType::KeyAction(AppKeyAction::BucketListMoveDown));
            }
            key_code_char!('j') => {
                self.select_next();
            }
            key_code_char!('k') => {
                self.select_prev();
            }
            key_code_char!('g') => {
                self.select_first();
            }
            key_code_char!('G') => {
                self.select_last();
            }
            key_code_char!('f') => {
                self.select_next_page();
            }
            key_code_char!('b') => {
                self.select_prev_page();
            }
            key_code_char!('x') => {
                self.tx.send(AppEventType::KeyAction(
                    AppKeyAction::BucketListOpenManagementConsole,
                ));
            }
            key_code_char!('?') => {
                self.tx
                    .send(AppEventType::KeyAction(AppKeyAction::ToggleHelp));
            }
            _ => {}
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let offset = self.list_state.offset;
        let selected = self.list_state.selected;

        let list_items = build_list_items(&self.bucket_items, offset, selected, area);

        let list = ScrollList::new(list_items);
        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    pub fn helps(&self) -> Vec<String> {
        let helps: &[(&[&str], &str)] = &[
            (&["Esc", "Ctrl-c"], "Quit app"),
            (&["j/k"], "Select item"),
            (&["g/G"], "Go to top/bottom"),
            (&["f"], "Scroll page forward"),
            (&["b"], "Scroll page backward"),
            (&["Enter"], "Open bucket"),
            (&["x"], "Open management console in browser"),
        ];
        build_helps(helps)
    }

    pub fn short_helps(&self) -> Vec<(String, usize)> {
        let helps: &[(&[&str], &str, usize)] = &[
            (&["Esc"], "Quit", 0),
            (&["j/k"], "Select", 1),
            (&["g/G"], "Top/Bottom", 3),
            (&["Enter"], "Open", 2),
            (&["?"], "Help", 0),
        ];
        build_short_helps(helps)
    }
}

impl BucketListPage {
    fn select_next(&mut self) {
        self.list_state.select_next();
    }

    fn select_prev(&mut self) {
        self.list_state.select_prev();
    }

    fn select_first(&mut self) {
        self.list_state.select_first();
    }

    fn select_last(&mut self) {
        self.list_state.select_last();
    }

    fn select_next_page(&mut self) {
        self.list_state.select_next_page();
    }

    fn select_prev_page(&mut self) {
        self.list_state.select_prev_page();
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

fn build_list_items(
    current_items: &[BucketItem],
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
        .map(|(idx, item)| {
            let content = format_bucket_item(&item.name, area.width);
            let style = Style::default();
            let span = Span::styled(content, style);
            if idx + offset == selected {
                ListItem::new(span).style(
                    Style::default()
                        .bg(SELECTED_COLOR)
                        .fg(SELECTED_ITEM_TEXT_COLOR),
                )
            } else {
                ListItem::new(span)
            }
        })
        .collect()
}

fn format_bucket_item(name: &str, width: u16) -> String {
    let name_w: usize = (width as usize) - 2 /* spaces */ - 2 /* border */;
    format!(" {:<name_w$} ", name, name_w = name_w)
}

#[cfg(test)]
mod tests {
    use crate::{event, set_cells};

    use super::*;
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

    #[test]
    fn test_render_without_scroll() -> std::io::Result<()> {
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let items = ["bucket1", "bucket2", "bucket3"]
                .iter()
                .map(|name| BucketItem {
                    name: name.to_string(),
                })
                .collect();
            let mut page = BucketListPage::new(items, tx);
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
        set_cells! { expected =>
            (2..28, [1]) => bg: Color::Cyan, fg: Color::Black,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn test_render_with_scroll() -> std::io::Result<()> {
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let items = (0..16)
                .map(|i| BucketItem {
                    name: format!("bucket{}", i + 1),
                })
                .collect();
            let mut page = BucketListPage::new(items, tx);
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
        set_cells! { expected =>
            // selected item
            (2..28, [1]) => bg: Color::Cyan, fg: Color::Black,
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
