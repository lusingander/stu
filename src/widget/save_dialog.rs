use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{block::Title, Block, BorderType, Padding, Paragraph, StatefulWidget, WidgetRef},
};

use crate::{ui::common::calc_centered_dialog_rect, widget::Dialog};

#[derive(Debug, Default)]
pub struct SaveDialogState {
    input: String,
    cursor: (u16, u16),
}

impl SaveDialogState {
    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn cursor(&self) -> (u16, u16) {
        self.cursor
    }

    pub fn add_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn delete_char(&mut self) {
        self.input.pop();
    }
}

#[derive(Debug, Default)]
pub struct SaveDialog {}

impl StatefulWidget for SaveDialog {
    type State = SaveDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let dialog_width = (area.width - 4).min(40);
        let dialog_height = 3;
        let dialog_area = calc_centered_dialog_rect(area, dialog_width, dialog_height);

        // show the last `input_max_width` characters of the input
        let input_max_width = dialog_width - 4;
        let input_start_index = state.input.len().saturating_sub(input_max_width as usize);
        let input_view: &str = &state.input[input_start_index..];

        let title = Title::from("Save As");
        let dialog_content = Paragraph::new(input_view).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(title)
                .padding(Padding::horizontal(1)),
        );
        let dialog = Dialog::new(Box::new(dialog_content));
        dialog.render_ref(dialog_area, buf);

        // update cursor position
        let cursor_x = dialog_area.x + (state.input.len() as u16).min(input_max_width) + 2;
        let cursor_y = dialog_area.y + 1;
        state.cursor = (cursor_x, cursor_y);
    }
}
