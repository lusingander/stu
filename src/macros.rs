#[macro_export]
macro_rules! key_code {
    ( $code:pat ) => {
        ratatui::crossterm::event::KeyEvent { code: $code, .. }
    };
}

#[macro_export]
macro_rules! key_code_char {
    ( $c:ident ) => {
        ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Char($c),
            ..
        }
    };
    ( $c:expr ) => {
        ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Char($c),
            ..
        }
    };
    ( $c:expr, Ctrl ) => {
        ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Char($c),
            modifiers: ratatui::crossterm::event::KeyModifiers::CONTROL,
            ..
        }
    };
}

#[macro_export]
macro_rules! lines {
    ( $($s:expr),* $(,)? ) => {
        vec![
        $(
            ratatui::widgets::Line::from($s),
        )*
        ]
    }
}

#[macro_export]
macro_rules! lines_with_empty_line {
    ( $s:expr, $($ss:expr),* $(,)? ) => {
        vec![
            lines_with_empty_line!($s),
        $(
            ratatui::widgets::Line::from(""),
            lines_with_empty_line!($ss),
        )*
        ]
    };

    ( $s:expr ) => {
        ratatui::widgets::Line::from($s)
    };
}

#[macro_export]
macro_rules! if_match {
    ( $i:ident : $p:pat => $ret:expr ) => {
        match $i {
            $p => Some($ret),
            _ => None,
        }
    };
}

#[macro_export]
macro_rules! set_cells {
    ( $buffer:ident => $( ( $xs:expr, $ys:expr ) => $(bg: $bg:expr , )? $(fg: $fg:expr , )? $(modifier: $md:expr , )* )* ) => {{
        $(
            for x in $xs {
                for y in $ys {
                    $( $buffer.get_mut(x, y).set_bg($bg); )?
                    $( $buffer.get_mut(x, y).set_fg($fg); )?
                    $( $buffer.get_mut(x, y).set_style(ratatui::style::Style::default().add_modifier($md)); )*
                }
            }
        )*
    }};
}

#[cfg(test)]
mod tests {
    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        style::{Color, Modifier},
    };

    #[test]
    fn test_set_cells() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 10, 10));
        set_cells! { buffer =>
            (0..10, 0..10) => bg: Color::White, fg: Color::Black, modifier: Modifier::BOLD,
            (2..5, 4..7) => bg: Color::White,
            ([1, 3, 5], [2, 4]) => fg: Color::Black,
            ([4], [8]) => modifier: Modifier::BOLD,
            (3..=6, [0]) => bg: Color::Black, modifier: Modifier::BOLD, modifier: Modifier::ITALIC,
        }
    }
}
