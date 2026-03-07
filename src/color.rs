use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;
use smart_default::SmartDefault;
use umbra::optional;

#[optional(derives = [Deserialize], visibility = pub)]
#[derive(Debug, Clone, SmartDefault)]
pub struct Theme {
    #[default(Color::Reset)]
    pub bg: Color,
    #[default(Color::Reset)]
    pub fg: Color,

    #[default(Color::DarkGray)]
    pub divider: Color,
    #[default(Color::Blue)]
    pub link: Color,

    #[default(Color::Cyan)]
    pub list_selected_bg: Color,
    #[default(Color::Black)]
    pub list_selected_fg: Color,
    #[default(Color::DarkGray)]
    pub list_selected_inactive_bg: Color,
    #[default(Color::Black)]
    pub list_selected_inactive_fg: Color,
    #[default(Color::Red)]
    pub list_filter_match: Color,

    #[default(Color::Cyan)]
    pub detail_selected: Color,

    #[default(Color::Cyan)]
    pub dialog_selected: Color,

    #[default(Color::DarkGray)]
    pub preview_line_number: Color,

    #[default(Color::Yellow)]
    pub help_key_fg: Color,

    #[default(Color::DarkGray)]
    pub status_help: Color,
    #[default(Color::Blue)]
    pub status_info: Color,
    #[default(Color::Green)]
    pub status_success: Color,
    #[default(Color::Yellow)]
    pub status_warn: Color,
    #[default(Color::Red)]
    pub status_error: Color,
    #[default = true]
    pub object_dir_bold: bool,
}

impl Theme {
    pub fn list_item_style(&self, selected: bool, active: bool) -> Style {
        if !selected {
            return Style::default();
        }

        if active {
            self.list_selected_style()
        } else {
            self.list_selected_inactive_style()
        }
    }

    pub fn list_selected_style(&self) -> Style {
        list_style(self.list_selected_bg, self.list_selected_fg)
    }

    pub fn list_selected_inactive_style(&self) -> Style {
        list_style(
            self.list_selected_inactive_bg,
            self.list_selected_inactive_fg,
        )
    }

    pub fn object_dir_style(&self) -> Style {
        let mut style = Style::default();
        if self.object_dir_bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        style
    }
}

fn list_style(bg: Color, fg: Color) -> Style {
    Style::default().bg(bg).fg(fg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_item_style_falls_back_to_default_for_unselected_items() {
        let theme = Theme::default();
        assert_eq!(theme.list_item_style(false, true), Style::default());
        assert_eq!(theme.list_item_style(false, false), Style::default());
    }

    #[test]
    fn object_dir_style_adds_bold_when_enabled() {
        let theme = Theme::default();
        assert_eq!(
            theme.object_dir_style(),
            Style::default().add_modifier(Modifier::BOLD)
        );
    }
}
