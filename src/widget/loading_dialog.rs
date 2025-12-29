use laurier::layout::calc_centered_area;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Margin, Rect},
    style::{Color, Modifier, Stylize},
    text::Line,
    widgets::{Block, BorderType, Padding, Paragraph, Widget},
};

use crate::{color::ColorTheme, widget::Dialog};

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
        let area = calc_centered_area(area, 30, 5);

        let text = Line::from(Self::MSG.fg(self.color.text).add_modifier(Modifier::BOLD));
        let paragraph = Paragraph::new(text).alignment(Alignment::Center).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .padding(Padding::vertical(1))
                .fg(self.color.block),
        );

        let dialog = Dialog::new(paragraph)
            .margin(Margin::new(1, 0))
            .bg(self.color.bg);
        dialog.render(area, buf);
    }
}

impl LoadingDialog {
    const MSG: &'static str = "Loading...";
}
