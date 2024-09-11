use ansi_to_tui::IntoText;
use once_cell::sync::Lazy;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Line,
    widgets::{Block, StatefulWidget},
};
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};

use crate::{
    object::{FileDetail, RawObject},
    ui::common::format_version,
    util::extension_from_file_name,
    widget::{ScrollLines, ScrollLinesOptions, ScrollLinesState},
};

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

#[derive(Debug)]
pub struct TextPreviewState {
    pub scroll_lines_state: ScrollLinesState,
}

impl TextPreviewState {
    pub fn new(
        file_detail: &FileDetail,
        object: &RawObject,
        highlight: bool,
    ) -> (Self, Option<String>) {
        let mut warn_msg = None;

        let s = to_preview_string(&object.bytes);

        let lines: Vec<Line<'static>> =
            match build_highlighted_lines(&s, &file_detail.name, highlight) {
                Ok(lines) => lines,
                Err(msg) => {
                    // If there is an error, display the original text
                    if let Some(msg) = msg {
                        warn_msg = Some(msg);
                    }
                    s.split('\n').map(|s| Line::raw(s.to_string())).collect()
                }
            };

        let scroll_lines_state = ScrollLinesState::new(lines, ScrollLinesOptions::default());

        let state = Self { scroll_lines_state };
        (state, warn_msg)
    }
}

fn to_preview_string(bytes: &[u8]) -> String {
    let s: String = String::from_utf8_lossy(bytes).into();
    // tab is not rendered correctly, so replace it
    let s = s.replace('\t', "    ");
    if s.ends_with('\n') {
        s.trim_end().into()
    } else {
        s
    }
}

fn build_highlighted_lines(
    s: &str,
    file_name: &str,
    highlight: bool,
) -> Result<Vec<Line<'static>>, Option<String>> {
    if highlight {
        let extension = extension_from_file_name(file_name);
        if let Some(syntax) = SYNTAX_SET.find_syntax_by_extension(&extension) {
            let mut h = HighlightLines::new(syntax, &THEME_SET.themes["base16-ocean.dark"]);
            let s = LinesWithEndings::from(s)
                .map(|line| {
                    let ranges: Vec<(syntect::highlighting::Style, &str)> =
                        h.highlight_line(line, &SYNTAX_SET).unwrap();
                    as_24_bit_terminal_escaped(&ranges[..], false)
                })
                .collect::<Vec<String>>()
                .join("");
            Ok(s.into_text().unwrap().into_iter().collect())
        } else {
            let msg = format!("No syntax definition found for `.{}`", extension);
            Err(Some(msg))
        }
    } else {
        Err(None)
    }
}

#[derive(Debug)]
pub struct TextPreview<'a> {
    file_name: &'a str,
    file_version_id: Option<&'a str>,
}

impl<'a> TextPreview<'a> {
    pub fn new(file_name: &'a str, file_version_id: Option<&'a str>) -> Self {
        Self {
            file_name,
            file_version_id,
        }
    }
}

impl StatefulWidget for TextPreview<'_> {
    type State = TextPreviewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let title = if let Some(version_id) = self.file_version_id {
            format!(
                "Preview [{} (Version ID: {})]",
                self.file_name,
                format_version(version_id)
            )
        } else {
            format!("Preview [{}]", self.file_name)
        };
        ScrollLines::default()
            .block(Block::bordered().title(title))
            .render(area, buf, &mut state.scroll_lines_state);
    }
}
