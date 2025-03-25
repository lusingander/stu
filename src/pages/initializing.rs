use std::rc::Rc;

use ratatui::{crossterm::event::KeyEvent, layout::Rect, style::Stylize, widgets::Block, Frame};

use crate::{
    app::AppContext,
    event::Sender,
    help::{build_short_help_spans, BuildShortHelpsItem, Spans, SpansWithPriority},
    keys::{UserEvent, UserEventMapper},
};

#[derive(Debug)]
pub struct InitializingPage {
    ctx: Rc<AppContext>,
    _tx: Sender,
}

impl InitializingPage {
    pub fn new(ctx: Rc<AppContext>, tx: Sender) -> Self {
        Self { ctx, _tx: tx }
    }

    pub fn handle_key(&mut self, _user_events: Vec<UserEvent>, _key_event: KeyEvent) {}

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let content = Block::bordered().fg(self.ctx.theme.fg);
        f.render_widget(content, area);
    }

    pub fn helps(&self, _mapper: &UserEventMapper) -> Vec<Spans> {
        Vec::new()
    }

    pub fn short_helps(&self, mapper: &UserEventMapper) -> Vec<SpansWithPriority> {
        #[rustfmt::skip]
        let helps = vec![
            BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
        ];
        build_short_help_spans(helps, mapper)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

    #[tokio::test]
    async fn test_render() -> std::io::Result<()> {
        let ctx = Rc::default();
        let tx = sender();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let mut page = InitializingPage::new(ctx, tx);
            let area = Rect::new(0, 0, 30, 10);
            page.render(f, area);
        })?;

        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "┌────────────────────────────┐",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "└────────────────────────────┘",
        ]);

        terminal.backend().assert_buffer(&expected);

        Ok(())
    }

    fn setup_terminal() -> std::io::Result<Terminal<TestBackend>> {
        let backend = TestBackend::new(30, 10);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(terminal)
    }

    fn sender() -> Sender {
        let (tx, _) = tokio::sync::mpsc::unbounded_channel();
        Sender::new(tx)
    }
}
