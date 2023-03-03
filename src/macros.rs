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
