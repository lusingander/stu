use itsuki::zero_indexed_enum;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::{
    event::{AppEventType, AppKeyInput},
    object::{FileDetail, FileVersion, ObjectItem},
    ui::common::{format_datetime, format_size_byte, format_version},
    widget::{
        CopyDetailDialog, CopyDetailDialogState, SaveDialog, SaveDialogState, ScrollList,
        ScrollListState,
    },
};

const SELECTED_COLOR: Color = Color::Cyan;
const SELECTED_ITEM_TEXT_COLOR: Color = Color::Black;
const SELECTED_DISABLED_COLOR: Color = Color::DarkGray;
const DIVIDER_COLOR: Color = Color::DarkGray;

#[derive(Debug)]
pub struct ObjectDetailPage {
    file_detail: FileDetail,
    file_versions: Vec<FileVersion>,

    tab: Tab,
    save_dialog_state: Option<SaveDialogState>,
    copy_detail_dialog_state: Option<CopyDetailDialogState>,

    object_items: Vec<ObjectItem>,
    list_state: ScrollListState,
}

#[derive(Default)]
#[zero_indexed_enum]
enum Tab {
    #[default]
    Detail,
    Version,
}

impl ObjectDetailPage {
    pub fn new(
        file_detail: FileDetail,
        file_versions: Vec<FileVersion>,
        object_items: Vec<ObjectItem>,
        list_state: ScrollListState,
    ) -> Self {
        Self {
            file_detail,
            file_versions,
            tab: Tab::Detail,
            save_dialog_state: None,
            copy_detail_dialog_state: None,
            object_items,
            list_state,
        }
    }

    pub fn handle_event(&mut self, event: AppEventType) {
        if let AppEventType::KeyInput(input) = event {
            self.handle_key_input(input);
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::horizontal(Constraint::from_percentages([50, 50])).split(area);

        let offset = self.list_state.offset;
        let selected = self.list_state.selected;

        let list_items =
            build_list_items_from_object_items(&self.object_items, offset, selected, chunks[0]);

        let list = ScrollList::new(list_items);
        f.render_stateful_widget(list, chunks[0], &mut self.list_state);

        let block = build_file_detail_block();
        f.render_widget(block, chunks[1]);

        let chunks = Layout::vertical([Constraint::Length(2), Constraint::Min(0)])
            .margin(1)
            .split(chunks[1]);

        let tabs = build_file_detail_tabs(self.tab);
        f.render_widget(tabs, chunks[0]);

        match self.tab {
            Tab::Detail => {
                let detail = build_file_detail(&self.file_detail);
                f.render_widget(detail, chunks[1]);
            }
            Tab::Version => {
                let versions = build_file_versions(&self.file_versions, chunks[1].width);
                f.render_widget(versions, chunks[1]);
            }
        }

        if let Some(state) = &mut self.save_dialog_state {
            let save_dialog = SaveDialog::default();
            f.render_stateful_widget(save_dialog, area, state);

            let (cursor_x, cursor_y) = state.cursor();
            f.set_cursor(cursor_x, cursor_y);
        }

        if let Some(state) = &self.copy_detail_dialog_state {
            let copy_detail_dialog = CopyDetailDialog::new(*state, &self.file_detail);
            f.render_widget(copy_detail_dialog, area);
        }
    }

    pub fn toggle_tab(&mut self) {
        match self.tab {
            Tab::Detail => {
                self.tab = Tab::Version;
            }
            Tab::Version => {
                self.tab = Tab::Detail;
            }
        }
    }

    pub fn open_save_dialog(&mut self) {
        self.save_dialog_state = Some(SaveDialogState::default());
    }

    pub fn close_save_dialog(&mut self) {
        self.save_dialog_state = None;
    }

    pub fn open_copy_detail_dialog(&mut self) {
        self.copy_detail_dialog_state = Some(CopyDetailDialogState::default());
    }

    pub fn close_copy_detail_dialog(&mut self) {
        self.copy_detail_dialog_state = None;
    }

    pub fn select_next_copy_detail_item(&mut self) {
        if let Some(ref mut state) = self.copy_detail_dialog_state {
            state.select_next();
        }
    }

    pub fn select_prev_copy_detail_item(&mut self) {
        if let Some(ref mut state) = self.copy_detail_dialog_state {
            state.select_prev();
        }
    }

    fn handle_key_input(&mut self, input: AppKeyInput) {
        if let Some(ref mut state) = self.save_dialog_state {
            match input {
                AppKeyInput::Char(c) => {
                    if c == '?' {
                        return;
                    }
                    state.add_char(c);
                }
                AppKeyInput::Backspace => {
                    state.delete_char();
                }
            }
        }
    }

    pub fn file_detail(&self) -> &FileDetail {
        &self.file_detail
    }

    pub fn save_dialog_key_input(&self) -> Option<String> {
        self.save_dialog_state.as_ref().map(|s| s.input().into())
    }

    pub fn copy_detail_dialog_selected(&self) -> Option<(String, String)> {
        self.copy_detail_dialog_state
            .as_ref()
            .map(|s| s.selected_name_and_value(&self.file_detail))
    }

    pub fn status(&self) -> (bool, bool) {
        (
            self.save_dialog_state.is_some(),
            self.copy_detail_dialog_state.is_some(),
        )
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
        ObjectItem::File { name, .. } => {
            let content = format_file_item(name, area.width);
            let style = Style::default();
            Span::styled(content, style)
        }
    };
    if idx + offset == selected {
        ListItem::new(content).style(
            Style::default()
                .bg(SELECTED_DISABLED_COLOR)
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

fn format_file_item(name: &str, width: u16) -> String {
    let name_w: usize = (width as usize) - 2 /* spaces */ - 4 /* border */;
    format!(" {:<name_w$} ", name, name_w = name_w)
}

fn build_file_detail_block() -> Block<'static> {
    Block::bordered()
}

fn build_file_detail_tabs(tab: Tab) -> Tabs<'static> {
    let tabs = vec![Line::from("Detail"), Line::from("Version")];
    Tabs::new(tabs)
        .select(tab.val())
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(SELECTED_COLOR),
        )
        .block(Block::default().borders(Borders::BOTTOM))
}

fn build_file_detail(detail: &FileDetail) -> Paragraph {
    let details = [
        ("Name:", &detail.name),
        ("Size:", &format_size_byte(detail.size_byte)),
        ("Last Modified:", &format_datetime(&detail.last_modified)),
        ("ETag:", &detail.e_tag),
        ("Content-Type:", &detail.content_type),
        ("Storage class:", &detail.storage_class),
    ]
    .iter()
    .filter_map(|(label, value)| {
        if value.is_empty() {
            None
        } else {
            let lines = vec![
                Line::from(label.add_modifier(Modifier::BOLD)),
                Line::from(format!(" {}", value)),
            ];
            Some(lines)
        }
    })
    .collect();

    let content = flatten_with_empty_lines(details, false);
    Paragraph::new(content)
        .block(Block::default().padding(Padding::horizontal(1)))
        .wrap(Wrap { trim: false })
}

fn build_file_versions(versions: &[FileVersion], width: u16) -> List {
    let list_items: Vec<ListItem> = versions
        .iter()
        .map(|v| {
            let content = vec![
                Line::from(vec![
                    "    Version ID: ".add_modifier(Modifier::BOLD),
                    Span::raw(format_version(&v.version_id)),
                ]),
                Line::from(vec![
                    " Last Modified: ".add_modifier(Modifier::BOLD),
                    Span::raw(format_datetime(&v.last_modified)),
                ]),
                Line::from(vec![
                    "          Size: ".add_modifier(Modifier::BOLD),
                    Span::raw(format_size_byte(v.size_byte)),
                ]),
                Line::from("-".repeat(width as usize).fg(DIVIDER_COLOR)),
            ];
            ListItem::new(content)
        })
        .collect();
    List::new(list_items)
        .block(Block::default())
        .highlight_style(Style::default().bg(SELECTED_COLOR))
}

fn flatten_with_empty_lines(line_groups: Vec<Vec<Line>>, add_to_end: bool) -> Vec<Line> {
    let n = line_groups.len();
    let mut ret: Vec<Line> = Vec::new();
    for (i, lines) in line_groups.into_iter().enumerate() {
        for line in lines {
            ret.push(line);
        }
        if add_to_end || i != n - 1 {
            ret.push(Line::from(""));
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Local};
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

    #[test]
    fn test_render_detail_tab() -> std::io::Result<()> {
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let (items, file_detail, file_versions) = fixtures();
            let items_len = items.len();
            let mut page = ObjectDetailPage::new(
                file_detail,
                file_versions,
                items,
                ScrollListState::new(items_len),
            );
            let area = Rect::new(0, 0, 60, 20);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────── 1 / 3 ┐┌────────────────────────────┐",
            "│  file1                     ││ Detail │ Version           │",
            "│  file2                     ││────────────────────────────│",
            "│  file3                     ││ Name:                      │",
            "│                            ││  file1                     │",
            "│                            ││                            │",
            "│                            ││ Size:                      │",
            "│                            ││  1.01 KiB                  │",
            "│                            ││                            │",
            "│                            ││ Last Modified:             │",
            "│                            ││  2024-01-02 13:01:02       │",
            "│                            ││                            │",
            "│                            ││ ETag:                      │",
            "│                            ││  bef684de-a260-48a4-8178-8 │",
            "│                            ││ a535ecccadb                │",
            "│                            ││                            │",
            "│                            ││ Content-Type:              │",
            "│                            ││  text/plain                │",
            "│                            ││                            │",
            "└────────────────────────────┘└────────────────────────────┘",
        ]);
        for x in 2..28 {
            // selected item
            expected.get_mut(x, 1).set_bg(Color::DarkGray);
            expected.get_mut(x, 1).set_fg(Color::Black);
        }
        for x in 32..38 {
            // "Detail" is selected
            expected
                .get_mut(x, 1)
                .set_fg(Color::Cyan)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 32..37 {
            // "Name" label
            expected
                .get_mut(x, 3)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 32..37 {
            // "Size" label
            expected
                .get_mut(x, 6)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 32..46 {
            // "Last Modified" label
            expected
                .get_mut(x, 9)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 32..37 {
            // "ETag" label
            expected
                .get_mut(x, 12)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 32..45 {
            // "Content-Type" label
            expected
                .get_mut(x, 16)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn test_render_version_tab() -> std::io::Result<()> {
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let (items, file_detail, file_versions) = fixtures();
            let items_len = items.len();
            let mut page = ObjectDetailPage::new(
                file_detail,
                file_versions,
                items,
                ScrollListState::new(items_len),
            );
            page.toggle_tab();
            let area = Rect::new(0, 0, 60, 20);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────── 1 / 3 ┐┌────────────────────────────┐",
            "│  file1                     ││ Detail │ Version           │",
            "│  file2                     ││────────────────────────────│",
            "│  file3                     ││    Version ID: 60f36bc2-0f3│",
            "│                            ││ Last Modified: 2024-01-02 1│",
            "│                            ││          Size: 1.01 KiB    │",
            "│                            ││----------------------------│",
            "│                            ││    Version ID: 1c5d3bcc-2bb│",
            "│                            ││ Last Modified: 2024-01-01 2│",
            "│                            ││          Size: 1 KiB       │",
            "│                            ││----------------------------│",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "│                            ││                            │",
            "└────────────────────────────┘└────────────────────────────┘",
        ]);
        for x in 2..28 {
            // selected item
            expected.get_mut(x, 1).set_bg(Color::DarkGray);
            expected.get_mut(x, 1).set_fg(Color::Black);
        }
        for x in 41..48 {
            // "Version" is selected
            expected
                .get_mut(x, 1)
                .set_fg(Color::Cyan)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 31..47 {
            for y in [3, 7] {
                // "Version ID" label
                expected
                    .get_mut(x, y)
                    .set_style(Style::default().add_modifier(Modifier::BOLD));
            }
        }
        for x in 31..47 {
            for y in [4, 8] {
                // "Last Modified" label
                expected
                    .get_mut(x, y)
                    .set_style(Style::default().add_modifier(Modifier::BOLD));
            }
        }
        for x in 31..47 {
            for y in [5, 9] {
                // "Size" label
                expected
                    .get_mut(x, y)
                    .set_style(Style::default().add_modifier(Modifier::BOLD));
            }
        }
        for x in 31..59 {
            for y in [6, 10] {
                // divider
                expected.get_mut(x, y).set_fg(Color::DarkGray);
            }
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn test_render_save_dialog_detail_tab() -> std::io::Result<()> {
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let (items, file_detail, file_versions) = fixtures();
            let items_len = items.len();
            let mut page = ObjectDetailPage::new(
                file_detail,
                file_versions,
                items,
                ScrollListState::new(items_len),
            );
            page.open_save_dialog();
            let area = Rect::new(0, 0, 60, 20);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────── 1 / 3 ┐┌────────────────────────────┐",
            "│  file1                     ││ Detail │ Version           │",
            "│  file2                     ││────────────────────────────│",
            "│  file3                     ││ Name:                      │",
            "│                            ││  file1                     │",
            "│                            ││                            │",
            "│                            ││ Size:                      │",
            "│                            ││  1.01 KiB                  │",
            "│         ╭Save As───────────────────────────────╮         │",
            "│         │                                      │         │",
            "│         ╰──────────────────────────────────────╯ 2       │",
            "│                            ││                            │",
            "│                            ││ ETag:                      │",
            "│                            ││  bef684de-a260-48a4-8178-8 │",
            "│                            ││ a535ecccadb                │",
            "│                            ││                            │",
            "│                            ││ Content-Type:              │",
            "│                            ││  text/plain                │",
            "│                            ││                            │",
            "└────────────────────────────┘└────────────────────────────┘",
        ]);
        for x in 2..28 {
            // selected item
            expected.get_mut(x, 1).set_bg(Color::DarkGray);
            expected.get_mut(x, 1).set_fg(Color::Black);
        }
        for x in 32..38 {
            // "Detail" is selected
            expected
                .get_mut(x, 1)
                .set_fg(Color::Cyan)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 32..37 {
            // "Name" label
            expected
                .get_mut(x, 3)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 32..37 {
            // "Size" label
            expected
                .get_mut(x, 6)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 32..37 {
            // "ETag" label
            expected
                .get_mut(x, 12)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 32..45 {
            // "Content-Type" label
            expected
                .get_mut(x, 16)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    #[test]
    fn test_render_copy_detail_dialog_detail_tab() -> std::io::Result<()> {
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let (items, file_detail, file_versions) = fixtures();
            let items_len = items.len();
            let mut page = ObjectDetailPage::new(
                file_detail,
                file_versions,
                items,
                ScrollListState::new(items_len),
            );
            page.open_copy_detail_dialog();
            let area = Rect::new(0, 0, 60, 20);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌───────────────────── 1 / 3 ┐┌────────────────────────────┐",
            "│  file1                     ││ Detail │ Version           │",
            "│  file2                     ││────────────────────────────│",
            "│  file3                     ││ Name:                      │",
            "│ ╭Copy──────────────────────────────────────────────────╮ │",
            "│ │ Key:                                                 │ │",
            "│ │   file1                                              │ │",
            "│ │ S3 URI:                                              │ │",
            "│ │   s3://bucket-1/file1                                │ │",
            "│ │ ARN:                                                 │ │",
            "│ │   arn:aws:s3:::bucket-1/file1                        │ │",
            "│ │ Object URL:                                          │ │",
            "│ │   https://bucket-1.s3.ap-northeast-1.amazonaws.com/f │ │",
            "│ │ ETag:                                                │ │",
            "│ │   bef684de-a260-48a4-8178-8a535ecccadb               │ │",
            "│ ╰──────────────────────────────────────────────────────╯ │",
            "│                            ││ Content-Type:              │",
            "│                            ││  text/plain                │",
            "│                            ││                            │",
            "└────────────────────────────┘└────────────────────────────┘",
        ]);
        for x in 2..28 {
            // selected item
            expected.get_mut(x, 1).set_bg(Color::DarkGray);
            expected.get_mut(x, 1).set_fg(Color::Black);
        }
        for x in 32..38 {
            // "Detail" is selected
            expected
                .get_mut(x, 1)
                .set_fg(Color::Cyan)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 32..37 {
            // "Name" label
            expected
                .get_mut(x, 3)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 32..45 {
            // "Content-Type" label
            expected
                .get_mut(x, 16)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 4..8 {
            // "Key" label
            expected
                .get_mut(x, 5)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 4..11 {
            // "S3 URI" label
            expected
                .get_mut(x, 7)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 4..8 {
            // "ARN" label
            expected
                .get_mut(x, 9)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 4..15 {
            // "Object URL" label
            expected
                .get_mut(x, 11)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 4..9 {
            // "ETag" label
            expected
                .get_mut(x, 13)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for x in 4..56 {
            for y in [5, 6] {
                // "Key" is selected
                expected.get_mut(x, y).set_fg(Color::Cyan);
            }
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    fn setup_terminal() -> std::io::Result<Terminal<TestBackend>> {
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(terminal)
    }

    fn parse_datetime(s: &str) -> DateTime<Local> {
        DateTime::parse_from_rfc3339(s)
            .unwrap()
            .with_timezone(&Local)
    }

    fn fixtures() -> (Vec<ObjectItem>, FileDetail, Vec<FileVersion>) {
        let items = vec![
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
            ObjectItem::File {
                name: "file3".to_string(),
                size_byte: 1024,
                last_modified: parse_datetime("2024-01-03T12:59:59+09:00"),
                paths: vec![],
            },
        ];
        let file_detail = FileDetail {
            name: "file1".to_string(),
            size_byte: 1024 + 10,
            last_modified: parse_datetime("2024-01-02T13:01:02+09:00"),
            e_tag: "bef684de-a260-48a4-8178-8a535ecccadb".to_string(),
            content_type: "text/plain".to_string(),
            storage_class: "STANDARD".to_string(),
            key: "file1".to_string(),
            s3_uri: "s3://bucket-1/file1".to_string(),
            arn: "arn:aws:s3:::bucket-1/file1".to_string(),
            object_url: "https://bucket-1.s3.ap-northeast-1.amazonaws.com/file1".to_string(),
        };
        let file_versions = vec![
            FileVersion {
                version_id: "60f36bc2-0f38-47b8-9bf0-e24e334b86d5".to_string(),
                size_byte: 1024 + 10,
                last_modified: parse_datetime("2024-01-02T13:01:02+09:00"),
                is_latest: true,
            },
            FileVersion {
                version_id: "1c5d3bcc-2bb3-4cd5-875f-a95a6ae53f65".to_string(),
                size_byte: 1024,
                last_modified: parse_datetime("2024-01-01T23:59:59+09:00"),
                is_latest: false,
            },
        ];
        (items, file_detail, file_versions)
    }
}
