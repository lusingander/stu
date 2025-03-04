use itsuki::zero_indexed_enum;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    text::Line,
    widgets::{block::Title, Block, BorderType, Padding, Paragraph, StatefulWidget, WidgetRef},
};

use crate::{
    color::ColorTheme,
    widget::{common::calc_centered_dialog_rect, Dialog, Divider},
};

#[derive(Default)]
#[zero_indexed_enum]
enum ActionType {
    #[default]
    Ok,
    Cancel,
}

#[derive(Debug, Default)]
pub struct ConfirmDialogState {
    selected: ActionType,
}

impl ConfirmDialogState {
    pub fn toggle(&mut self) {
        self.selected = self.selected.next();
    }

    pub fn is_ok(&self) -> bool {
        self.selected == ActionType::Ok
    }
}

#[derive(Debug, Default)]
struct ConfirmDialogColor {
    bg: Color,
    block: Color,
    text: Color,
    selected: Color,
    divider: Color,
}

impl ConfirmDialogColor {
    fn new(theme: &ColorTheme) -> ConfirmDialogColor {
        ConfirmDialogColor {
            bg: theme.bg,
            block: theme.fg,
            text: theme.fg,
            selected: theme.dialog_selected,
            divider: theme.divider,
        }
    }
}

#[derive(Debug, Default)]
pub struct ConfirmDialog<'a> {
    message_lines: Vec<Line<'a>>,

    color: ConfirmDialogColor,
}

impl<'a> ConfirmDialog<'a> {
    pub fn new(message_lines: Vec<Line<'a>>) -> ConfirmDialog<'a> {
        ConfirmDialog {
            message_lines,
            color: ConfirmDialogColor::default(),
        }
    }

    pub fn theme(mut self, theme: &ColorTheme) -> Self {
        self.color = ConfirmDialogColor::new(theme);
        self
    }
}

impl StatefulWidget for ConfirmDialog<'_> {
    type State = ConfirmDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let dialog_width = 70;
        let dialog_height = self.message_lines.len() as u16 + 2 /* divider + select */ + 2 /* border */;
        let dialog_area = calc_centered_dialog_rect(area, dialog_width, dialog_height);

        let divider_lines = build_divider_lines(&self.color, dialog_width);
        let select_lines = build_select_lines(state, &self.color);

        let mut lines = Vec::new();
        lines.extend(self.message_lines);
        lines.extend(divider_lines);
        lines.extend(select_lines);

        let title = Title::from("Confirm");
        let content = Paragraph::new(lines).centered().block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(title)
                .padding(Padding::horizontal(1))
                .bg(self.color.bg)
                .fg(self.color.block),
        );

        let dialog = Dialog::new(Box::new(content), self.color.bg);
        dialog.render_ref(dialog_area, buf);
    }
}

fn build_divider_lines(color: &ConfirmDialogColor, dialog_width: u16) -> Vec<Line<'_>> {
    let line = Divider::default()
        .color(color.divider)
        .to_line(dialog_width - 6);
    vec![line]
}

fn build_select_lines<'a>(
    state: &'a ConfirmDialogState,
    color: &'a ConfirmDialogColor,
) -> Vec<Line<'a>> {
    let line = match state.selected {
        ActionType::Ok => Line::from(vec![
            "OK".fg(color.selected).bold(),
            "    ".into(),
            "Cancel".fg(color.text),
        ]),
        ActionType::Cancel => Line::from(vec![
            "OK".fg(color.text),
            "    ".into(),
            "Cancel".fg(color.selected).bold(),
        ]),
    };
    vec![line]
}
