use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct ColorTheme {
    pub bg: Color,
    pub fg: Color,

    pub divider: Color,
    pub link: Color,

    pub list_selected_bg: Color,
    pub list_selected_fg: Color,
    pub list_selected_inactive_bg: Color,
    pub list_selected_inactive_fg: Color,
    pub list_filter_match: Color,

    pub detail_selected: Color,

    pub dialog_selected: Color,

    pub preview_line_number: Color,

    pub help_key_fg: Color,

    pub status_help: Color,
    pub status_info: Color,
    pub status_success: Color,
    pub status_warn: Color,
    pub status_error: Color,
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self {
            bg: Color::Reset,
            fg: Color::Reset,

            divider: Color::DarkGray,
            link: Color::Blue,

            list_selected_bg: Color::Cyan,
            list_selected_fg: Color::Black,
            list_selected_inactive_bg: Color::DarkGray,
            list_selected_inactive_fg: Color::Black,
            list_filter_match: Color::Red,

            detail_selected: Color::Cyan,

            dialog_selected: Color::Cyan,

            preview_line_number: Color::DarkGray,

            help_key_fg: Color::Yellow,

            status_help: Color::DarkGray,
            status_info: Color::Blue,
            status_success: Color::Green,
            status_warn: Color::Yellow,
            status_error: Color::Red,
        }
    }
}
