use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{block::Title, Block, BorderType, Padding, Paragraph, StatefulWidget, WidgetRef},
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{ui::common::calc_centered_dialog_rect, widget::Dialog};

#[derive(Debug, Default)]
pub struct SaveDialogState {
    input: Input,
    cursor: (u16, u16),
}

impl SaveDialogState {
    pub fn input(&self) -> &str {
        self.input.value()
    }

    pub fn cursor(&self) -> (u16, u16) {
        self.cursor
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        let event = &crossterm::event::Event::Key(key);
        self.input.handle_event(event);
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
        let input_max_width = (dialog_width - 4) as usize;
        let input_start_index = state.input.visual_cursor().saturating_sub(input_max_width);
        let input_view: &str = &state.input.value()[input_start_index..];

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
        let cursor_x = dialog_area.x + state.input.visual_cursor().min(input_max_width) as u16 + 2;
        let cursor_y = dialog_area.y + 1;
        state.cursor = (cursor_x, cursor_y);
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyCode;

    use super::*;

    #[test]
    fn test_render_save_dialog() {
        let mut state = SaveDialogState::default();
        let save_dialog = SaveDialog::default();

        for c in "file.txt".chars() {
            state.handle_key_event(KeyEvent::from(KeyCode::Char(c)));
        }

        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 20));
        save_dialog.render(buf.area, &mut buf, &mut state);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "                                        ",
            "                                        ",
            "                                        ",
            "                                        ",
            "                                        ",
            "                                        ",
            "                                        ",
            "                                        ",
            "  ╭Save As───────────────────────────╮  ",
            "  │ file.txt                         │  ",
            "  ╰──────────────────────────────────╯  ",
            "                                        ",
            "                                        ",
            "                                        ",
            "                                        ",
            "                                        ",
            "                                        ",
            "                                        ",
            "                                        ",
            "                                        ",
        ]);

        assert_eq!(buf, expected);
        assert_eq!(state.cursor(), (12, 9));
    }
}
