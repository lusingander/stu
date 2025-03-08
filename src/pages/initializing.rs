use std::rc::Rc;

use ratatui::{crossterm::event::KeyEvent, layout::Rect, style::Stylize, widgets::Block, Frame};

use crate::{app::AppContext, event::Sender, keys::UserEvent, pages::util::build_short_helps};

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

    pub fn helps(&self) -> Vec<String> {
        Vec::new()
    }

    pub fn short_helps(&self) -> Vec<(String, usize)> {
        let helps: &[(&[&str], &str, usize)] = &[(&["Esc"], "Quit", 0)];
        build_short_helps(helps)
    }
}

#[cfg(test)]
mod tests {
    use crate::event;

    use super::*;
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

    #[test]
    fn test_render() -> std::io::Result<()> {
        let ctx = Rc::default();
        let (tx, _) = event::new();
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
}
