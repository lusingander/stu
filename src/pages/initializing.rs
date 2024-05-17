use ratatui::{layout::Rect, widgets::Block, Frame};

use crate::event::AppEventType;

#[derive(Debug)]
pub struct InitializingPage {}

impl InitializingPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_event(&mut self, _event: AppEventType) {}

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let content = Block::bordered();
        f.render_widget(content, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

    #[test]
    fn test_render() -> std::io::Result<()> {
        let mut terminal = setup_terminal()?;

        terminal.draw(|f| {
            let mut page = InitializingPage::new();
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
