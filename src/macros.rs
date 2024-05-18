#[macro_export]
macro_rules! key_code {
    ( $code:path ) => {
        crossterm::event::KeyEvent { code: $code, .. }
    };
}

#[macro_export]
macro_rules! key_code_char {
    ( $c:ident ) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($c),
            ..
        }
    };
    ( $c:expr ) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($c),
            ..
        }
    };
    ( $c:expr, Ctrl ) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($c),
            modifiers: crossterm::event::KeyModifiers::CONTROL,
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
