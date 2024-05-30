use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{block::Title, Block, BorderType, Padding, Paragraph, StatefulWidget, WidgetRef},
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{ui::common::calc_centered_dialog_rect, widget::Dialog};

#[derive(Debug, Default)]
pub struct InputDialogState {
    input: Input,
    cursor: (u16, u16),
}

impl InputDialogState {
    pub fn input(&self) -> &str {
        self.input.value()
    }

    pub fn clear_input(&mut self) {
        self.input.reset();
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
pub struct InputDialog {
    title: &'static str,
    max_width: Option<u16>,
}

impl InputDialog {
    pub fn title(mut self, title: &'static str) -> Self {
        self.title = title;
        self
    }

    pub fn max_width(mut self, max_width: u16) -> Self {
        self.max_width = Some(max_width);
        self
    }
}

impl StatefulWidget for InputDialog {
    type State = InputDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut dialog_width = area.width - 4;
        if let Some(max_width) = self.max_width {
            dialog_width = dialog_width.min(max_width);
        }
        let dialog_height = 3;
        let dialog_area = calc_centered_dialog_rect(area, dialog_width, dialog_height);

        // show the last `input_max_width` characters of the input
        let input_max_width = (dialog_width - 4) as usize;
        let input_start_index = state.input.visual_cursor().saturating_sub(input_max_width);
        let input_view: &str = &state.input.value()[input_start_index..];

        let title = Title::from(self.title);
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
    fn test_render_input_dialog() {
        let mut state = InputDialogState::default();
        let save_dialog = InputDialog::default();

        for c in "abc".chars() {
            state.handle_key_event(KeyEvent::from(KeyCode::Char(c)));
        }

        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 10));
        save_dialog.render(buf.area, &mut buf, &mut state);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "                                        ",
            "                                        ",
            "                                        ",
            "  ╭──────────────────────────────────╮  ",
            "  │ abc                              │  ",
            "  ╰──────────────────────────────────╯  ",
            "                                        ",
            "                                        ",
            "                                        ",
            "                                        ",
        ]);

        assert_eq!(buf, expected);
        assert_eq!(state.cursor(), (7, 4));
    }

    #[test]
    fn test_render_input_dialog_with_params() {
        let mut state = InputDialogState::default();
        let save_dialog = InputDialog::default().title("xyz").max_width(20);

        for c in "abc".chars() {
            state.handle_key_event(KeyEvent::from(KeyCode::Char(c)));
        }

        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 9));
        save_dialog.render(buf.area, &mut buf, &mut state);

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "                                        ",
            "                                        ",
            "                                        ",
            "          ╭xyz───────────────╮          ",
            "          │ abc              │          ",
            "          ╰──────────────────╯          ",
            "                                        ",
            "                                        ",
            "                                        ",
        ]);

        assert_eq!(buf, expected);
        assert_eq!(state.cursor(), (15, 4));
    }
}
