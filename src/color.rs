use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct ColorTheme {
    pub bg: Color,
    pub text: Color,
    pub selected: Color,
    pub selected_text: Color,
    pub disabled: Color,
    pub match_text: Color,
    pub link: Color,
    pub short_help: Color,
    pub info_status: Color,
    pub success_status: Color,
    pub warn_status: Color,
    pub error_status: Color,
    pub line_number: Color,
    pub divider: Color,
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self {
            bg: Color::Reset,
            text: Color::Reset,
            selected: Color::Cyan,
            selected_text: Color::Black,
            disabled: Color::DarkGray,
            match_text: Color::Red,
            link: Color::Blue,
            short_help: Color::DarkGray,
            info_status: Color::Blue,
            success_status: Color::Green,
            warn_status: Color::Yellow,
            error_status: Color::Red,
            line_number: Color::DarkGray,
            divider: Color::DarkGray,
        }
    }
}
