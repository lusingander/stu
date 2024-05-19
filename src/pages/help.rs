use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, Padding, Paragraph},
    Frame,
};

use crate::{
    constant::{APP_DESCRIPTION, APP_HOMEPAGE, APP_NAME, APP_VERSION},
    event::AppEventType,
    util::group_strings_to_fit_width,
};

const DIVIDER_COLOR: Color = Color::DarkGray;
const LINK_TEXT_COLOR: Color = Color::Blue;

#[derive(Debug)]
pub struct HelpPage {
    helps: Vec<String>,
}

impl HelpPage {
    pub fn new(helps: Vec<String>) -> Self {
        Self { helps }
    }

    pub fn handle_event(&mut self, _event: AppEventType) {}

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let content_area = area.inner(&Margin::new(1, 1)); // border
        let w: usize = content_area.width as usize;

        let app_details = vec![
            Line::from(format!(" {} - {}", APP_NAME, APP_DESCRIPTION)),
            Line::from(format!(" Version: {}", APP_VERSION)),
            Line::from(format!(" {}", APP_HOMEPAGE).fg(LINK_TEXT_COLOR)),
            Line::from("-".repeat(w).fg(DIVIDER_COLOR)),
        ];
        let app_detail = with_empty_lines(app_details).into_iter();

        let max_help_width: usize = 80;
        let max_width = max_help_width.min(w) - 2;
        let help = build_help_lines(&self.helps, max_width);

        let content: Vec<Line> = app_detail.chain(help).collect();
        let paragraph = Paragraph::new(content).block(
            Block::bordered()
                .title(APP_NAME)
                .padding(Padding::uniform(1)),
        );

        f.render_widget(paragraph, area);
    }
}

fn with_empty_lines(lines: Vec<Line>) -> Vec<Line> {
    let line_groups = lines.into_iter().map(|l| vec![l]).collect();
    flatten_with_empty_lines(line_groups, true)
}

fn flatten_with_empty_lines(line_groups: Vec<Vec<Line>>, add_to_end: bool) -> Vec<Line> {
    let n = line_groups.len();
    let mut ret: Vec<Line> = Vec::new();
    for (i, lines) in line_groups.into_iter().enumerate() {
        for line in lines {
            ret.push(line);
        }
        if add_to_end || i != n - 1 {
            ret.push(Line::from(""));
        }
    }
    ret
}

fn build_help_lines(helps: &[String], max_width: usize) -> Vec<Line> {
    let delimiter = ",  ";
    let word_groups = group_strings_to_fit_width(helps, max_width, delimiter);
    let lines: Vec<Line> = word_groups
        .iter()
        .map(|ws| Line::from(format!(" {} ", ws.join(delimiter))))
        .collect();
    with_empty_lines(lines)
}

#[cfg(test)]
mod tests {
    use crate::set_cells;

    use super::*;
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

    #[test]
    fn test_render() -> std::io::Result<()> {
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let helps = [
                "<key1>: action1",
                "<key2>: action2",
                "<key3>: action3",
                "<key4>: action4",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect();
            let mut page = HelpPage::new(helps);
            let area = Rect::new(0, 0, 70, 20);
            page.render(f, area);
        })?;

        // fixme: should not depend on environment variables...
        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌STU─────────────────────────────────────────────────────────────────┐",
            "│                                                                    │",
            "│  STU - TUI application for AWS S3 written in Rust using ratatui    │",
            "│                                                                    │",
            "│  Version: 0.4.1                                                    │",
            "│                                                                    │",
            "│  https://github.com/lusingander/stu                                │",
            "│                                                                    │",
            "│ ------------------------------------------------------------------ │",
            "│                                                                    │",
            "│  <key1>: action1,  <key2>: action2,  <key3>: action3               │",
            "│                                                                    │",
            "│  <key4>: action4                                                   │",
            "│                                                                    │",
            "│                                                                    │",
            "│                                                                    │",
            "│                                                                    │",
            "│                                                                    │",
            "│                                                                    │",
            "└────────────────────────────────────────────────────────────────────┘",
        ]);
        set_cells! { expected =>
            // link
            (2..37, [6]) => fg: Color::Blue,
            // divider
            (2..68, [8]) => fg: Color::DarkGray,
        }

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    fn setup_terminal() -> std::io::Result<Terminal<TestBackend>> {
        let backend = TestBackend::new(70, 20);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(terminal)
    }
}
