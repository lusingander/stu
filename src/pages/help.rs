use std::rc::Rc;

use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyEvent,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, Borders, Padding, Paragraph, Widget},
    Frame,
};

use crate::{
    app::AppContext,
    constant::{APP_DESCRIPTION, APP_HOMEPAGE, APP_NAME, APP_VERSION},
    event::{AppEventType, Sender},
    handle_user_events,
    help::{
        build_short_help_spans, group_spans_to_fit_width, BuildShortHelpsItem, Spans,
        SpansWithPriority,
    },
    keys::{UserEvent, UserEventMapper},
    widget::Divider,
};

#[derive(Debug)]
pub struct HelpPage {
    helps: Vec<Spans>,

    ctx: Rc<AppContext>,
    tx: Sender,
}

impl HelpPage {
    pub fn new(helps: Vec<Spans>, ctx: Rc<AppContext>, tx: Sender) -> Self {
        Self { helps, ctx, tx }
    }

    pub fn handle_key(&mut self, user_events: Vec<UserEvent>, _key_event: KeyEvent) {
        handle_user_events! { user_events =>
            UserEvent::HelpClose => {
                self.tx.send(AppEventType::CloseCurrentPage);
            }
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .padding(Padding::horizontal(1))
            .title(APP_NAME)
            .fg(self.ctx.theme.fg);

        let content_area = block.inner(area);

        let chunks = Layout::vertical([
            Constraint::Length(7),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(content_area);

        let about = About::new(
            APP_NAME,
            APP_DESCRIPTION,
            APP_VERSION,
            APP_HOMEPAGE,
            self.ctx.theme.link,
        );
        let divider = Divider::default().color(self.ctx.theme.divider);
        let help = Help::new(&self.helps, self.ctx.config.ui.help.max_help_width);

        f.render_widget(block, area);
        f.render_widget(about, chunks[0]);
        f.render_widget(divider, chunks[1]);
        f.render_widget(help, chunks[2]);
    }

    pub fn helps(&self, _mapper: &UserEventMapper) -> Vec<Spans> {
        Vec::new()
    }

    pub fn short_helps(&self, mapper: &UserEventMapper) -> Vec<SpansWithPriority> {
        #[rustfmt::skip]
        let helps = vec![
            BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
            BuildShortHelpsItem::single(UserEvent::HelpClose, "Close help", 0),
        ];
        build_short_help_spans(helps, mapper)
    }
}

#[derive(Debug)]
struct About<'a> {
    name: &'a str,
    description: &'a str,
    version: &'a str,
    homepage: &'a str,

    link_color: Color,
}

impl<'a> About<'a> {
    fn new(
        name: &'a str,
        description: &'a str,
        version: &'a str,
        homepage: &'a str,
        link_color: Color,
    ) -> Self {
        Self {
            name,
            description,
            version,
            homepage,
            link_color,
        }
    }
}

impl Widget for About<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines = vec![
            Line::from(format!("{} - {}", self.name, self.description)),
            Line::from(format!("Version: {}", self.version)),
            Line::from(self.homepage.fg(self.link_color)),
        ];
        let content = with_empty_lines(lines);
        let paragraph = Paragraph::new(content).block(
            Block::default()
                .borders(Borders::NONE)
                .padding(Padding::uniform(1)),
        );
        paragraph.render(area, buf);
    }
}

#[derive(Debug)]
struct Help<'a> {
    helps: &'a [Spans],
    max_width: usize,
}

impl<'a> Help<'a> {
    fn new(helps: &'a [Spans], max_width: usize) -> Self {
        Self { helps, max_width }
    }
}

impl Widget for Help<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let max_width = self.max_width.min(area.width as usize) - 2;

        let help = build_help_lines(self.helps, max_width);

        let paragraph = Paragraph::new(help).block(
            Block::default()
                .borders(Borders::NONE)
                .padding(Padding::uniform(1)),
        );
        paragraph.render(area, buf);
    }
}

fn build_help_lines(helps: &[Spans], max_width: usize) -> Vec<Line> {
    let word_groups = group_spans_to_fit_width(helps, max_width, "  ");
    let lines: Vec<Line> = word_groups.into_iter().map(Line::from).collect();
    with_empty_lines(lines)
}

fn with_empty_lines(lines: Vec<Line>) -> Vec<Line> {
    let n = lines.len();
    let mut ret = Vec::new();
    for (i, line) in lines.into_iter().enumerate() {
        ret.push(line);
        if i != n - 1 {
            ret.push(Line::raw(""));
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use crate::set_cells;

    use super::*;
    use ratatui::{backend::TestBackend, buffer::Buffer, text::Span, Terminal};

    #[tokio::test]
    async fn test_render() -> std::io::Result<()> {
        let ctx = Rc::default();
        let tx = sender();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let helps = [
                "<key1>: action1",
                "<key2>: action2",
                "<key3>: action3",
                "<key4>: action4",
            ]
            .iter()
            .map(|s| Spans::new(vec![Span::raw(s.to_string())]))
            .collect();
            let mut page = HelpPage::new(helps, ctx, tx);
            let area = Rect::new(0, 0, 70, 20);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let mut expected = Buffer::with_lines([
            "┌STU─────────────────────────────────────────────────────────────────┐",
            "│                                                                    │",
            "│  STU - S3 Terminal UI                                              │",
            "│                                                                    │",
            "│  Version: 1.2.3                                                    │",
            "│                                                                    │",
            "│  http://example.com/stu                                            │",
            "│                                                                    │",
            "│ ────────────────────────────────────────────────────────────────── │",
            "│                                                                    │",
            "│  <key1>: action1  <key2>: action2  <key3>: action3                 │",
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
            (3..25, [6]) => fg: Color::Blue,
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

    fn sender() -> Sender {
        let (tx, _) = tokio::sync::mpsc::unbounded_channel();
        Sender::new(tx)
    }
}
