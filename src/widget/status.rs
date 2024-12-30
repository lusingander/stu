use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    text::Span,
    widgets::{Block, Padding, Paragraph, Widget},
};

use crate::{color::ColorTheme, util};

#[derive(Debug)]
pub enum StatusType {
    Help(Vec<(String, usize)>),
    Info(String),
    Success(String),
    Warn(String),
    Error(String),
}

#[derive(Debug, Default)]
struct StatusColor {
    help: Color,
    info: Color,
    success: Color,
    warn: Color,
    error: Color,
}

impl StatusColor {
    fn new(theme: &ColorTheme) -> Self {
        StatusColor {
            help: theme.status_help,
            info: theme.status_info,
            success: theme.status_success,
            warn: theme.status_warn,
            error: theme.status_error,
        }
    }
}

#[derive(Debug)]
pub struct Status {
    status_type: StatusType,
    color: StatusColor,
}

impl Status {
    pub fn new(status_type: StatusType) -> Self {
        Status {
            status_type,
            color: StatusColor::default(),
        }
    }

    pub fn theme(mut self, theme: &ColorTheme) -> Self {
        self.color = StatusColor::new(theme);
        self
    }
}

impl Widget for Status {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let pad = Padding::horizontal(2);
        let msg = self.build_msg(area, pad);
        let paragraph = Paragraph::new(msg).block(Block::default().padding(pad));
        paragraph.render(area, buf);
    }
}

impl Status {
    fn build_msg(self, area: Rect, pad: Padding) -> Span<'static> {
        match self.status_type {
            StatusType::Help(helps) => {
                let max_width = (area.width - pad.left - pad.right) as usize;
                let delimiter = ", ";
                let ss = util::prune_strings_to_fit_width(&helps, max_width, delimiter);
                let msg = ss.join(delimiter);
                msg.fg(self.color.help)
            }
            StatusType::Info(msg) => msg.fg(self.color.info),
            StatusType::Success(msg) => msg.fg(self.color.success).bold(),
            StatusType::Warn(msg) => msg.fg(self.color.warn).bold(),
            StatusType::Error(msg) => {
                let msg = format!("ERROR: {}", msg);
                msg.fg(self.color.error).bold()
            }
        }
    }
}
