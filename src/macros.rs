#[macro_export]
macro_rules! handle_user_events {
    ($user_events:ident => $($event:pat $(if $cond:expr)? => $body:block)+) => {
        #[allow(unreachable_code)]
        for user_event in &$user_events {
            match user_event {
                $($event $(if $cond)* => $body)+
                _ => {
                    continue;
                }
            }
            break;
        }
    };
}

#[macro_export]
macro_rules! handle_user_events_with_default {
    ($user_events:ident => $($event:pat => $body:block)+ => $default_block:block) => {
        for user_event in &$user_events {
            match user_event {
                $($event => $body)+
                _ => {
                    continue;
                }
            }
            return;
        }
        $default_block
    };
}

#[macro_export]
macro_rules! set_cells {
    ( $buffer:ident => $( ( $xs:expr, $ys:expr ) => $(bg: $bg:expr , )? $(fg: $fg:expr , )? $(modifier: $md:expr , )* )* ) => {{
        $(
            for x in $xs {
                for y in $ys {
                    $( $buffer[(x, y)].set_bg($bg); )?
                    $( $buffer[(x, y)].set_fg($fg); )?
                    $( $buffer[(x, y)].set_style(ratatui::style::Style::default().add_modifier($md)); )*
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
