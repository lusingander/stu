use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, widgets::Block, Frame};

use crate::{
    event::{AppEventType, Sender},
    key_code,
    pages::util::build_short_helps,
};

#[derive(Debug)]
pub struct InitializingPage {
    tx: Sender,
}

impl InitializingPage {
    pub fn new(tx: Sender) -> Self {
        Self { tx }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if let key_code!(KeyCode::Esc) = key {
            self.tx.send(AppEventType::Quit);
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let content = Block::bordered();
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
        let (tx, _) = event::new();
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let mut page = InitializingPage::new(tx);
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
