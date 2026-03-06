use anyhow::{anyhow, Context};
use ratatui::style::{Color, Style};

use crate::config::Config;

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

impl ColorTheme {
    pub fn from_config(config: &Config) -> anyhow::Result<Self> {
        let mut theme = Self::default();
        theme.list_selected_bg = parse_color(&config.ui.theme.list_selected_bg)
            .with_context(|| "Failed to parse ui.theme.list_selected_bg")?;
        theme.list_selected_fg = parse_color(&config.ui.theme.list_selected_fg)
            .with_context(|| "Failed to parse ui.theme.list_selected_fg")?;
        theme.list_selected_inactive_bg = parse_color(&config.ui.theme.list_selected_inactive_bg)
            .with_context(|| "Failed to parse ui.theme.list_selected_inactive_bg")?;
        theme.list_selected_inactive_fg = parse_color(&config.ui.theme.list_selected_inactive_fg)
            .with_context(|| "Failed to parse ui.theme.list_selected_inactive_fg")?;
        Ok(theme)
    }

    pub fn list_selected_style(&self) -> Style {
        Style::default()
            .bg(self.list_selected_bg)
            .fg(self.list_selected_fg)
    }

    pub fn list_selected_inactive_style(&self) -> Style {
        Style::default()
            .bg(self.list_selected_inactive_bg)
            .fg(self.list_selected_inactive_fg)
    }
}

fn parse_color(value: &str) -> anyhow::Result<Color> {
    let value = value.trim();
    let normalized = value.to_ascii_lowercase().replace(['-', ' '], "_");

    match normalized.as_str() {
        "reset" | "default" => Ok(Color::Reset),
        "black" => Ok(Color::Black),
        "red" => Ok(Color::Red),
        "green" => Ok(Color::Green),
        "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue),
        "magenta" => Ok(Color::Magenta),
        "cyan" => Ok(Color::Cyan),
        "gray" | "grey" => Ok(Color::Gray),
        "dark_gray" | "dark_grey" | "bright_black" => Ok(Color::DarkGray),
        "light_red" | "bright_red" => Ok(Color::LightRed),
        "light_green" | "bright_green" => Ok(Color::LightGreen),
        "light_yellow" | "bright_yellow" => Ok(Color::LightYellow),
        "light_blue" | "bright_blue" => Ok(Color::LightBlue),
        "light_magenta" | "bright_magenta" => Ok(Color::LightMagenta),
        "light_cyan" | "bright_cyan" => Ok(Color::LightCyan),
        "white" | "light_white" | "bright_white" => Ok(Color::White),
        _ => parse_hex_color(value).ok_or_else(|| anyhow!("unknown color: {value}")),
    }
}

fn parse_hex_color(value: &str) -> Option<Color> {
    let hex = value.strip_prefix('#')?;

    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(Color::Rgb(r, g, b))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::Rgb(r, g, b))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_named_colors() {
        assert_eq!(parse_color("cyan").unwrap(), Color::Cyan);
        assert_eq!(parse_color("dark-gray").unwrap(), Color::DarkGray);
        assert_eq!(parse_color("bright_white").unwrap(), Color::White);
    }

    #[test]
    fn parse_hex_colors() {
        assert_eq!(parse_color("#123456").unwrap(), Color::Rgb(0x12, 0x34, 0x56));
        assert_eq!(parse_color("#abc").unwrap(), Color::Rgb(0xaa, 0xbb, 0xcc));
    }
}
