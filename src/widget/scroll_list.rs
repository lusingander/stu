use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Margin, Rect},
    style::{Color, Stylize},
    widgets::{Block, List, ListItem, Padding, StatefulWidget, Widget},
};

use crate::{color::ColorTheme, util::digits};

use crate::widget::ScrollBar;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScrollListState {
    pub selected: usize,
    pub offset: usize,
    total: usize,
    height: usize,
}

impl ScrollListState {
    pub fn new(total: usize) -> ScrollListState {
        ScrollListState {
            total,
            ..Default::default()
        }
    }

    pub fn select_next(&mut self) {
        if self.total == 0 {
            return;
        }
        if self.selected >= self.total - 1 {
            self.select_first();
        } else {
            if self.selected - self.offset == self.height - 1 {
                self.offset += 1;
            }
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.total == 0 {
            return;
        }
        if self.selected == 0 {
            self.select_last();
        } else {
            if self.selected - self.offset == 0 {
                self.offset -= 1;
            }
            self.selected -= 1;
        }
    }

    pub fn select_next_page(&mut self) {
        if self.total == 0 {
            return;
        }
        if self.total < self.height {
            self.selected = self.total - 1;
            self.offset = 0;
        } else if self.selected + self.height < self.total - 1 {
            self.selected += self.height;
            if self.selected + self.height > self.total - 1 {
                self.offset = self.total - self.height;
            } else {
                self.offset = self.selected;
            }
        } else {
            self.selected = self.total - 1;
            self.offset = self.total - self.height;
        }
    }

    pub fn select_prev_page(&mut self) {
        if self.total == 0 {
            return;
        }
        if self.total < self.height {
            self.selected = 0;
            self.offset = 0;
        } else if self.selected > self.height {
            self.selected -= self.height;
            if self.selected < self.height {
                self.offset = 0;
            } else {
                self.offset = self.selected - self.height + 1;
            }
        } else {
            self.selected = 0;
            self.offset = 0;
        }
    }

    pub fn select_first(&mut self) {
        if self.total == 0 {
            return;
        }
        self.selected = 0;
        self.offset = 0;
    }

    pub fn select_last(&mut self) {
        if self.total == 0 {
            return;
        }
        self.selected = self.total - 1;
        if self.height < self.total {
            self.offset = self.total - self.height;
        }
    }
}

#[derive(Debug, Default)]
struct ScrollListColor {
    block: Color,
    bar: Color,
}

impl ScrollListColor {
    fn new(theme: &ColorTheme) -> ScrollListColor {
        ScrollListColor {
            block: theme.fg,
            bar: theme.fg,
        }
    }
}

#[derive(Debug)]
pub struct ScrollList<'a> {
    items: Vec<ListItem<'a>>,
    color: ScrollListColor,
}

impl ScrollList<'_> {
    pub fn new(items: Vec<ListItem>) -> ScrollList {
        ScrollList {
            items,
            color: Default::default(),
        }
    }

    pub fn theme(mut self, theme: &ColorTheme) -> Self {
        self.color = ScrollListColor::new(theme);
        self
    }
}

impl StatefulWidget for ScrollList<'_> {
    type State = ScrollListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.height = area.height as usize - 2 /* border */;

        let title = format_list_count(state.total, state.selected);
        let list = List::new(self.items).block(
            Block::bordered()
                .title(title)
                .title_alignment(Alignment::Right)
                .padding(Padding::horizontal(1))
                .fg(self.color.block),
        );
        Widget::render(list, area, buf);

        let area = area.inner(Margin::new(2, 1));
        let scrollbar_area = Rect::new(area.right(), area.top(), 1, area.height);

        if state.total > (scrollbar_area.height as usize) {
            let scroll_bar = ScrollBar::new(state.total, state.offset).color(self.color.bar);
            Widget::render(scroll_bar, scrollbar_area, buf);
        }
    }
}

fn format_list_count(total_count: usize, selected: usize) -> String {
    if total_count == 0 {
        String::new()
    } else {
        let digits = digits(total_count);
        format!(" {:>digits$} / {} ", selected + 1, total_count)
    }
}

#[cfg(test)]
mod tests {
    use ratatui::text::Line;

    use super::*;

    #[test]
    fn test_render_scroll_list_without_scroll() {
        let theme = ColorTheme::default();
        let mut state = ScrollListState::new(5);
        let items: Vec<ListItem> = (1..=5)
            .map(|i| ListItem::new(vec![Line::from(format!("Item {}", i))]))
            .collect();
        let scroll_list = ScrollList::new(items).theme(&theme);

        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 12));
        scroll_list.render(buf.area, &mut buf, &mut state);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "┌─────────── 1 / 5 ┐",
            "│ Item 1           │",
            "│ Item 2           │",
            "│ Item 3           │",
            "│ Item 4           │",
            "│ Item 5           │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "└──────────────────┘",
        ]);

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_render_scroll_list_with_scroll() {
        let mut state = ScrollListState::new(20);

        let buf = render_scroll_list(&mut state);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "┌─────────  1 / 20 ┐",
            "│ Item 1          ││",
            "│ Item 2          ││",
            "│ Item 3          ││",
            "│ Item 4          ││",
            "│ Item 5          ││",
            "│ Item 6           │",
            "│ Item 7           │",
            "│ Item 8           │",
            "│ Item 9           │",
            "│ Item 10          │",
            "└──────────────────┘",
        ]);

        assert_eq!(buf, expected);

        for _ in 0..9 {
            state.select_next();
        }
        let buf = render_scroll_list(&mut state);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "┌───────── 10 / 20 ┐",
            "│ Item 1          ││",
            "│ Item 2          ││",
            "│ Item 3          ││",
            "│ Item 4          ││",
            "│ Item 5          ││",
            "│ Item 6           │",
            "│ Item 7           │",
            "│ Item 8           │",
            "│ Item 9           │",
            "│ Item 10          │",
            "└──────────────────┘",
        ]);

        assert_eq!(buf, expected);

        for _ in 0..4 {
            state.select_next();
        }
        let buf = render_scroll_list(&mut state);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "┌───────── 14 / 20 ┐",
            "│ Item 5           │",
            "│ Item 6           │",
            "│ Item 7          ││",
            "│ Item 8          ││",
            "│ Item 9          ││",
            "│ Item 10         ││",
            "│ Item 11         ││",
            "│ Item 12          │",
            "│ Item 13          │",
            "│ Item 14          │",
            "└──────────────────┘",
        ]);

        assert_eq!(buf, expected);
    }

    fn render_scroll_list(state: &mut ScrollListState) -> Buffer {
        let show_item_count = 10_u16;
        let items: Vec<ListItem> = (1..=20)
            .map(|i| ListItem::new(vec![Line::from(format!("Item {}", i))]))
            .skip(state.offset)
            .take(show_item_count as usize)
            .collect();
        let theme = ColorTheme::default();
        let scroll_list = ScrollList::new(items).theme(&theme);
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, show_item_count + 2));
        scroll_list.render(buf.area, &mut buf, state);
        buf
    }
}
