pub struct AppListState {
    pub selected: usize,
    pub offset: usize,
    pub height: usize,
}

impl AppListState {
    pub fn new(height: usize) -> AppListState {
        AppListState {
            selected: 0,
            offset: 0,
            height: AppListState::calc_list_height(height),
        }
    }

    pub fn select_next(&mut self) {
        if self.selected - self.offset == self.height - 1 {
            self.offset += 1;
        }
        self.selected += 1;
    }

    pub fn select_prev(&mut self) {
        if self.selected - self.offset == 0 {
            self.offset -= 1;
        }
        self.selected -= 1;
    }

    pub fn select_next_page(&mut self, total: usize) {
        if total < self.height {
            self.selected = total - 1;
            self.offset = 0;
        } else if self.selected + self.height < total - 1 {
            self.selected += self.height;
            if self.selected + self.height > total - 1 {
                self.offset = total - self.height;
            } else {
                self.offset = self.selected;
            }
        } else {
            self.selected = total - 1;
            self.offset = total - self.height;
        }
    }

    pub fn select_prev_page(&mut self, total: usize) {
        if total < self.height {
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

    pub fn select_last(&mut self, total: usize) {
        self.selected = total - 1;
        if self.height < total {
            self.offset = total - self.height;
        }
    }

    pub fn calc_list_height(height: usize) -> usize {
        height - 3 /* header */ - 2 /* footer */ - 2 /* list area border */
    }
}
