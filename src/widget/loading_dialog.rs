use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Stylize},
    text::Line,
    widgets::{Block, BorderType, Padding, Paragraph, Widget, WidgetRef},
};

use crate::{
    color::ColorTheme,
    widget::{common::calc_centered_dialog_rect, Dialog},
};

#[derive(Debug, Default)]
struct LoadingDialogColor {
    bg: Color,
    block: Color,
    text: Color,
}

impl LoadingDialogColor {
    fn new(theme: &ColorTheme) -> Self {
        LoadingDialogColor {
            bg: theme.bg,
            block: theme.fg,
            text: theme.fg,
        }
    }
}

#[derive(Debug, Default)]
pub struct LoadingDialog {
    color: LoadingDialogColor,
}

impl LoadingDialog {
    pub fn theme(mut self, theme: &ColorTheme) -> Self {
        self.color = LoadingDialogColor::new(theme);
        self
    }
}

impl Widget for LoadingDialog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = calc_centered_dialog_rect(area, 30, 5);

        let text = Line::from(Self::MSG.fg(self.color.text).add_modifier(Modifier::BOLD));
        let paragraph = Paragraph::new(text).alignment(Alignment::Center).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .padding(Padding::vertical(1))
                .fg(self.color.block),
        );

        let dialog = Dialog::new(Box::new(paragraph), self.color.bg);
        dialog.render_ref(area, buf);
    }
}

impl LoadingDialog {
    const MSG: &'static str = "Loading...";
}
