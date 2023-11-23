#[macro_export]
macro_rules! key_code {
    ( $code:path ) => {
        KeyEvent { code: $code, .. }
    };
}

#[macro_export]
macro_rules! key_code_char {
    ( $c:expr ) => {
        KeyEvent {
            code: KeyCode::Char($c),
            ..
        }
    };
    ( $c:expr, Ctrl ) => {
        KeyEvent {
            code: KeyCode::Char($c),
            modifiers: KeyModifiers::CONTROL,
            ..
        }
    };
}

#[macro_export]
macro_rules! lines {
    ( $($s:expr),* $(,)? ) => {
        vec![
        $(
            Line::from($s),
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
            Line::from(""),
            lines_with_empty_line!($ss),
        )*
        ]
    };

    ( $s:expr ) => {
        Line::from($s)
    };
}
