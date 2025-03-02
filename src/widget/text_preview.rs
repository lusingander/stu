use ansi_to_tui::IntoText;
use itsuki::zero_indexed_enum;
use once_cell::sync::Lazy;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    text::Line,
    widgets::{
        block::Title, Block, BorderType, List, ListItem, Padding, StatefulWidget, Widget, WidgetRef,
    },
};
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};

use crate::{
    color::ColorTheme,
    config::Config,
    format::format_version,
    object::{FileDetail, RawObject},
    util::extension_from_file_name,
    widget::{
        common::calc_centered_dialog_rect, Dialog, ScrollLines, ScrollLinesOptions,
        ScrollLinesState,
    },
};

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
    if let Ok(path) = Config::preview_syntax_dir_path() {
        if path.exists() {
            // SyntaxSetBuilder::build is terribly slow in debug build...
            // To avoid unnecessary processing, we won't use the builder if the syntax directory doesn't exist...
            let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
            builder.add_from_folder(path, true).unwrap();
            builder.build()
        } else {
            SyntaxSet::load_defaults_newlines()
        }
    } else {
        SyntaxSet::load_defaults_newlines()
    }
});
static DEFAULT_THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);
static USER_THEME_SET: Lazy<ThemeSet> = Lazy::new(|| {
    Config::preview_theme_dir_path()
        .and_then(|path| ThemeSet::load_from_folder(path).map_err(Into::into))
        .unwrap_or_default()
});

#[derive(Default)]
#[zero_indexed_enum]
pub enum Encoding {
    #[default]
    Utf8,
    Utf16Le,
    Utf16Be,
    ShiftJis,
}

impl Encoding {
    fn str(&self) -> &'static str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::Utf16Le => "UTF-16 (LE)",
            Self::Utf16Be => "UTF-16 (BE)",
            Self::ShiftJis => "Shift_JIS",
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EncodingDialogState {
    selected: Encoding,
}

impl EncodingDialogState {
    pub fn select_next(&mut self) {
        self.selected = self.selected.next();
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.prev();
    }

    pub fn reset(&mut self) {
        self.selected = Encoding::default();
    }

    pub fn selected(&self) -> Encoding {
        self.selected
    }
}

#[derive(Debug, Default)]
struct EncodingDialogColor {
    bg: Color,
    block: Color,
    text: Color,
    selected: Color,
}

impl EncodingDialogColor {
    fn new(theme: &ColorTheme) -> Self {
        Self {
            bg: theme.bg,
            block: theme.fg,
            text: theme.fg,
            selected: theme.dialog_selected,
        }
    }
}

pub struct EncodingDialog {
    state: EncodingDialogState,
    labels: Vec<&'static str>,
    color: EncodingDialogColor,
}

impl EncodingDialog {
    pub fn new(state: EncodingDialogState) -> Self {
        let labels = Encoding::vars_vec().iter().map(Encoding::str).collect();
        Self {
            state,
            labels,
            color: EncodingDialogColor::default(),
        }
    }

    pub fn theme(mut self, theme: &ColorTheme) -> Self {
        self.color = EncodingDialogColor::new(theme);
        self
    }
}

impl Widget for EncodingDialog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let list_items: Vec<ListItem> = self
            .labels
            .iter()
            .enumerate()
            .map(|(i, label)| {
                let item = ListItem::new(Line::raw(*label));
                if i == self.state.selected.val() {
                    item.fg(self.color.selected)
                } else {
                    item.fg(self.color.text)
                }
            })
            .collect();

        let dialog_width = (area.width - 4).min(20);
        let dialog_height = self.labels.len() as u16 + 2 /* border */;
        let area = calc_centered_dialog_rect(area, dialog_width, dialog_height);

        let title = Title::from("Encoding");
        let list = List::new(list_items).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(title)
                .padding(Padding::horizontal(1))
                .bg(self.color.bg)
                .fg(self.color.block),
        );
        let dialog = Dialog::new(Box::new(list), self.color.bg);
        dialog.render_ref(area, buf);
    }
}

#[derive(Debug)]
pub struct TextPreviewState {
    pub scroll_lines_state: ScrollLinesState,
    pub encoding: Encoding,
}

impl TextPreviewState {
    pub fn new(
        file_detail: &FileDetail,
        object: &RawObject,
        highlight: bool,
        highlight_theme_name: &str,
    ) -> (Self, Option<String>) {
        let mut state = Self {
            scroll_lines_state: ScrollLinesState::new(vec![], ScrollLinesOptions::default()),
            encoding: Encoding::Utf8,
        };
        let warn_msg = state.update_lines(file_detail, object, highlight, highlight_theme_name);
        (state, warn_msg)
    }

    pub fn set_encoding(&mut self, encoding: Encoding) {
        self.encoding = encoding;
    }

    pub fn update_lines(
        &mut self,
        file_detail: &FileDetail,
        object: &RawObject,
        highlight: bool,
        highlight_theme_name: &str,
    ) -> Option<String> {
        let mut warn_msg = None;
        let s = self.to_preview_string(&object.bytes);

        let lines: Vec<Line<'static>> =
            match build_highlighted_lines(&s, &file_detail.name, highlight, highlight_theme_name) {
                Ok(lines) => lines,
                Err(msg) => {
                    // If there is an error, display the original text
                    if let Some(msg) = msg {
                        warn_msg = Some(msg);
                    }
                    s.lines().map(drop_control_chars).map(Line::raw).collect()
                }
            };

        let options = self.scroll_lines_state.current_options();
        self.scroll_lines_state = ScrollLinesState::new(lines, options);

        warn_msg
    }

    fn to_preview_string(&self, bytes: &[u8]) -> String {
        let enc = match self.encoding {
            Encoding::Utf8 => encoding_rs::UTF_8,
            Encoding::Utf16Le => encoding_rs::UTF_16LE,
            Encoding::Utf16Be => encoding_rs::UTF_16BE,
            Encoding::ShiftJis => encoding_rs::SHIFT_JIS,
        };
        let (s, _, _) = enc.decode(bytes);
        // tab is not rendered correctly, so replace it
        let s = s.replace('\t', "    ");
        if s.ends_with('\n') {
            s.trim_end().into()
        } else {
            s
        }
    }
}

fn drop_control_chars(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).collect()
}

fn build_highlighted_lines(
    s: &str,
    file_name: &str,
    highlight: bool,
    highlight_theme_name: &str,
) -> Result<Vec<Line<'static>>, Option<String>> {
    if !highlight {
        return Err(None);
    }

    let extension = extension_from_file_name(file_name);
    let syntax = SYNTAX_SET
        .find_syntax_by_extension(&extension)
        .ok_or_else(|| {
            let msg = format!("No syntax definition found for `.{}`", extension);
            Some(msg)
        })?;
    let theme = &DEFAULT_THEME_SET
        .themes
        .get(highlight_theme_name)
        .or_else(|| USER_THEME_SET.themes.get(highlight_theme_name))
        .ok_or_else(|| {
            let msg = format!("Theme `{}` not found", highlight_theme_name);
            Some(msg)
        })?;
    let mut h = HighlightLines::new(syntax, theme);
    let s = LinesWithEndings::from(s)
        .map(|line| {
            let ranges: Vec<(syntect::highlighting::Style, &str)> =
                h.highlight_line(line, &SYNTAX_SET).unwrap();
            as_24_bit_terminal_escaped(&ranges[..], false)
        })
        .collect::<Vec<String>>()
        .join("");
    Ok(s.into_text().unwrap().into_iter().collect())
}

#[derive(Debug)]
pub struct TextPreview<'a> {
    file_name: &'a str,
    file_version_id: Option<&'a str>,

    theme: &'a ColorTheme,
}

impl<'a> TextPreview<'a> {
    pub fn new(
        file_name: &'a str,
        file_version_id: Option<&'a str>,
        theme: &'a ColorTheme,
    ) -> Self {
        Self {
            file_name,
            file_version_id,
            theme,
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
            .theme(self.theme)
            .render(area, buf, &mut state.scroll_lines_state);
    }
}
