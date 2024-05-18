use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Margin, Rect},
    widgets::{Block, List, ListItem, Padding, StatefulWidget, Widget},
};

use crate::util::digits;

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
        self.selected = 0;
        self.offset = 0;
    }

    pub fn select_last(&mut self) {
        self.selected = self.total - 1;
        if self.height < self.total {
            self.offset = self.total - self.height;
        }
    }
}

#[derive(Debug)]
pub struct ScrollList<'a> {
    items: Vec<ListItem<'a>>,
}

impl ScrollList<'_> {
    pub fn new(items: Vec<ListItem>) -> ScrollList {
        ScrollList { items }
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
                .padding(Padding::horizontal(1)),
        );
        Widget::render(list, area, buf);

        let area = area.inner(&Margin::new(2, 1));
        let scrollbar_area = Rect::new(area.right(), area.top(), 1, area.height);

        if state.total > (scrollbar_area.height as usize) {
            let scroll_bar = ScrollBar::new(state.total, state.offset);
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
