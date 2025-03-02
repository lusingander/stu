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
pub enum EncodingType {
    #[default]
    Utf8,
    Ibm866,
    Iso8859_2,
    Iso8859_3,
    Iso8859_4,
    Iso8859_5,
    Iso8859_6,
    Iso8859_7,
    Iso8859_8,
    Iso8859_8I,
    Iso8859_10,
    Iso8859_13,
    Iso8859_14,
    Iso8859_15,
    Iso8859_16,
    Koi8R,
    Koi8U,
    Macintosh,
    Windows874,
    Windows1250,
    Windows1251,
    Windows1252,
    Windows1253,
    Windows1254,
    Windows1255,
    Windows1256,
    Windows1257,
    Windows1258,
    XMacCyrillic,
    Gbk,
    Gb18030,
    Big5,
    EucJp,
    Iso2022Jp,
    ShiftJis,
    EucKr,
    Replacement,
    Utf16Be,
    Utf16Le,
    XUserDefined,
}

impl EncodingType {
    fn str(&self) -> &'static str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::Ibm866 => "IBM866",
            Self::Iso8859_2 => "ISO-8859-2",
            Self::Iso8859_3 => "ISO-8859-3",
            Self::Iso8859_4 => "ISO-8859-4",
            Self::Iso8859_5 => "ISO-8859-5",
            Self::Iso8859_6 => "ISO-8859-6",
            Self::Iso8859_7 => "ISO-8859-7",
            Self::Iso8859_8 => "ISO-8859-8",
            Self::Iso8859_8I => "ISO-8859-8-I",
            Self::Iso8859_10 => "ISO-8859-10",
            Self::Iso8859_13 => "ISO-8859-13",
            Self::Iso8859_14 => "ISO-8859-14",
            Self::Iso8859_15 => "ISO-8859-15",
            Self::Iso8859_16 => "ISO-8859-16",
            Self::Koi8R => "KOI8-R",
            Self::Koi8U => "KOI8-U",
            Self::Macintosh => "macintosh",
            Self::Windows874 => "windows-874",
            Self::Windows1250 => "windows-1250",
            Self::Windows1251 => "windows-1251",
            Self::Windows1252 => "windows-1252",
            Self::Windows1253 => "windows-1253",
            Self::Windows1254 => "windows-1254",
            Self::Windows1255 => "windows-1255",
            Self::Windows1256 => "windows-1256",
            Self::Windows1257 => "windows-1257",
            Self::Windows1258 => "windows-1258",
            Self::XMacCyrillic => "x-mac-cyrillic",
            Self::Gbk => "GBK",
            Self::Gb18030 => "gb18030",
            Self::Big5 => "Big5",
            Self::EucJp => "EUC-JP",
            Self::Iso2022Jp => "ISO-2022-JP",
            Self::ShiftJis => "Shift_JIS",
            Self::EucKr => "EUC-KR",
            Self::Replacement => "replacement",
            Self::Utf16Be => "UTF-16BE",
            Self::Utf16Le => "UTF-16LE",
            Self::XUserDefined => "x-user-defined",
        }
    }
}

impl From<EncodingType> for &encoding_rs::Encoding {
    fn from(value: EncodingType) -> Self {
        match value {
            EncodingType::Utf8 => encoding_rs::UTF_8,
            EncodingType::Ibm866 => encoding_rs::IBM866,
            EncodingType::Iso8859_2 => encoding_rs::ISO_8859_2,
            EncodingType::Iso8859_3 => encoding_rs::ISO_8859_3,
            EncodingType::Iso8859_4 => encoding_rs::ISO_8859_4,
            EncodingType::Iso8859_5 => encoding_rs::ISO_8859_5,
            EncodingType::Iso8859_6 => encoding_rs::ISO_8859_6,
            EncodingType::Iso8859_7 => encoding_rs::ISO_8859_7,
            EncodingType::Iso8859_8 => encoding_rs::ISO_8859_8,
            EncodingType::Iso8859_8I => encoding_rs::ISO_8859_8_I,
            EncodingType::Iso8859_10 => encoding_rs::ISO_8859_10,
            EncodingType::Iso8859_13 => encoding_rs::ISO_8859_13,
            EncodingType::Iso8859_14 => encoding_rs::ISO_8859_14,
            EncodingType::Iso8859_15 => encoding_rs::ISO_8859_15,
            EncodingType::Iso8859_16 => encoding_rs::ISO_8859_16,
            EncodingType::Koi8R => encoding_rs::KOI8_R,
            EncodingType::Koi8U => encoding_rs::KOI8_U,
            EncodingType::Macintosh => encoding_rs::MACINTOSH,
            EncodingType::Windows874 => encoding_rs::WINDOWS_874,
            EncodingType::Windows1250 => encoding_rs::WINDOWS_1250,
            EncodingType::Windows1251 => encoding_rs::WINDOWS_1251,
            EncodingType::Windows1252 => encoding_rs::WINDOWS_1252,
            EncodingType::Windows1253 => encoding_rs::WINDOWS_1253,
            EncodingType::Windows1254 => encoding_rs::WINDOWS_1254,
            EncodingType::Windows1255 => encoding_rs::WINDOWS_1255,
            EncodingType::Windows1256 => encoding_rs::WINDOWS_1256,
            EncodingType::Windows1257 => encoding_rs::WINDOWS_1257,
            EncodingType::Windows1258 => encoding_rs::WINDOWS_1258,
            EncodingType::XMacCyrillic => encoding_rs::X_MAC_CYRILLIC,
            EncodingType::Gbk => encoding_rs::GBK,
            EncodingType::Gb18030 => encoding_rs::GB18030,
            EncodingType::Big5 => encoding_rs::BIG5,
            EncodingType::EucJp => encoding_rs::EUC_JP,
            EncodingType::Iso2022Jp => encoding_rs::ISO_2022_JP,
            EncodingType::ShiftJis => encoding_rs::SHIFT_JIS,
            EncodingType::EucKr => encoding_rs::EUC_KR,
            EncodingType::Replacement => encoding_rs::REPLACEMENT,
            EncodingType::Utf16Be => encoding_rs::UTF_16BE,
            EncodingType::Utf16Le => encoding_rs::UTF_16LE,
            EncodingType::XUserDefined => encoding_rs::X_USER_DEFINED,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EncodingDialogState {
    selected: EncodingType,
}

impl EncodingDialogState {
    pub fn select_next(&mut self) {
        self.selected = self.selected.next();
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.prev();
    }

    pub fn reset(&mut self) {
        self.selected = EncodingType::default();
    }

    pub fn selected(&self) -> EncodingType {
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
        let labels = EncodingType::vars_vec()
            .iter()
            .map(EncodingType::str)
            .collect();
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
    pub encoding: EncodingType,
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
            encoding: EncodingType::Utf8,
        };
        let warn_msg = state.update_lines(file_detail, object, highlight, highlight_theme_name);
        (state, warn_msg)
    }

    pub fn set_encoding(&mut self, encoding: EncodingType) {
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
        let encoding: &encoding_rs::Encoding = self.encoding.into();
        let (s, _, _) = encoding.decode(bytes);
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
