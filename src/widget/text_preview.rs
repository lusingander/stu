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

    #[rustfmt::skip]
    fn from(value: &str) -> Option<EncodingType> {
        match value.to_lowercase().as_str() {
            "unicode-1-1-utf-8" | "unicode11utf8" | "unicode20utf8" | "utf-8" | "utf8" | "x-unicode20utf8" => Some(EncodingType::Utf8),
            "866" | "cp866" | "csibm866" | "ibm866" => Some(EncodingType::Ibm866),
            "csisolatin2" | "iso-8859-2" | "iso-ir-101" | "iso8859-2" | "iso88592" | "iso_8859-2" | "iso_8859-2:1987" | "l2" | "latin2" => Some(EncodingType::Iso8859_2),
            "csisolatin3" | "iso-8859-3" | "iso-ir-109" | "iso8859-3" | "iso88593" | "iso_8859-3" | "iso_8859-3:1988" | "l3" | "latin3" => Some(EncodingType::Iso8859_3),
            "csisolatin4" | "iso-8859-4" | "iso-ir-110" | "iso8859-4" | "iso88594" | "iso_8859-4" | "iso_8859-4:1988" | "l4" | "latin4" => Some(EncodingType::Iso8859_4),
            "csisolatincyrillic" | "cyrillic" | "iso-8859-5" | "iso-ir-144" | "iso8859-5" | "iso88595" | "iso_8859-5" | "iso_8859-5:1988" => Some(EncodingType::Iso8859_5),
            "arabic" | "asmo-708" | "csiso88596e" | "csiso88596i" | "csisolatinarabic" | "ecma-114" | "iso-8859-6" | "iso-8859-6-e" | "iso-8859-6-i" | "iso-ir-127" | "iso8859-6" | "iso88596" | "iso_8859-6" | "iso_8859-6:1987" => Some(EncodingType::Iso8859_6),
            "csisolatingreek" | "ecma-118" | "elot_928" | "greek" | "greek8" | "iso-8859-7" | "iso-ir-126" | "iso8859-7" | "iso88597" | "iso_8859-7" | "iso_8859-7:1987" | "sun_eu_greek" => Some(EncodingType::Iso8859_7),
            "csiso88598e" | "csisolatinhebrew" | "hebrew" | "iso-8859-8" | "iso-8859-8-e" | "iso-ir-138" | "iso8859-8" | "iso88598" | "iso_8859-8" | "iso_8859-8:1988" | "visual" => Some(EncodingType::Iso8859_8),
            "csiso88598i" | "iso-8859-8-i" | "logical" => Some(EncodingType::Iso8859_8I),
            "csisolatin6" | "iso-8859-10" | "iso-ir-157" | "iso8859-10" | "iso885910" | "l6" | "latin6" => Some(EncodingType::Iso8859_10),
            "iso-8859-13" | "iso8859-13" | "iso885913" => Some(EncodingType::Iso8859_13),
            "iso-8859-14" | "iso8859-14" | "iso885914" => Some(EncodingType::Iso8859_14),
            "csisolatin9" | "iso-8859-15" | "iso8859-15" | "iso885915" | "iso_8859-15" | "l9" => Some(EncodingType::Iso8859_15),
            "iso-8859-16" => Some(EncodingType::Iso8859_16),
            "cskoi8r" | "koi" | "koi8" | "koi8-r" | "koi8_r" => Some(EncodingType::Koi8R),
            "koi8-ru" | "koi8-u" => Some(EncodingType::Koi8U),
            "csmacintosh" | "mac" | "macintosh" | "x-mac-roman" => Some(EncodingType::Macintosh),
            "dos-874" | "iso-8859-11" | "iso8859-11" | "iso885911" | "tis-620" | "windows-874" => Some(EncodingType::Windows874),
            "cp1250" | "windows-1250" | "x-cp1250" => Some(EncodingType::Windows1250),
            "cp1251" | "windows-1251" | "x-cp1251" => Some(EncodingType::Windows1251),
            "ansi_x3.4-1968" | "ascii" | "cp1252" | "cp819" | "csisolatin1" | "ibm819" | "iso-8859-1" | "iso-ir-100" | "iso8859-1" | "iso88591" | "iso_8859-1" | "iso_8859-1:1987" | "l1" | "latin1" | "us-ascii" | "windows-1252" | "x-cp1252" => Some(EncodingType::Windows1252),
            "cp1253" | "windows-1253" | "x-cp1253" => Some(EncodingType::Windows1253),
            "cp1254" | "csisolatin5" | "iso-8859-9" | "iso-ir-148" | "iso8859-9" | "iso88599" | "iso_8859-9" | "iso_8859-9:1989" | "l5" | "latin5" | "windows-1254" | "x-cp1254" => Some(EncodingType::Windows1254),
            "cp1255" | "windows-1255" | "x-cp1255" => Some(EncodingType::Windows1255),
            "cp1256" | "windows-1256" | "x-cp1256" => Some(EncodingType::Windows1256),
            "cp1257" | "windows-1257" | "x-cp1257" => Some(EncodingType::Windows1257),
            "cp1258" | "windows-1258" | "x-cp1258" => Some(EncodingType::Windows1258),
            "x-mac-cyrillic" | "x-mac-ukrainian" => Some(EncodingType::XMacCyrillic),
            "chinese" | "csgb2312" | "csiso58gb231280" | "gb2312" | "gb_2312" | "gb_2312-80" | "gbk" | "iso-ir-58" | "x-gbk" => Some(EncodingType::Gbk),
            "gb18030" => Some(EncodingType::Gb18030),
            "big5" | "big5-hkscs" | "cn-big5" | "csbig5" | "x-x-big5" => Some(EncodingType::Big5),
            "cseucpkdfmtjapanese" | "euc-jp" | "x-euc-jp" => Some(EncodingType::EucJp),
            "csiso2022jp" | "iso-2022-jp" => Some(EncodingType::Iso2022Jp),
            "csshiftjis" | "ms932" | "ms_kanji" | "shift-jis" | "shift_jis" | "sjis" | "windows-31j" | "x-sjis" => Some(EncodingType::ShiftJis),
            "cseuckr" | "csksc56011987" | "euc-kr" | "iso-ir-149" | "korean" | "ks_c_5601-1987" | "ks_c_5601-1989" | "ksc5601" | "ksc_5601" | "windows-949" => Some(EncodingType::EucKr),
            "csiso2022kr" | "hz-gb-2312" | "iso-2022-cn" | "iso-2022-cn-ext" | "iso-2022-kr" | "replacement" => Some(EncodingType::Replacement),
            "unicodefffe" | "utf-16be" => Some(EncodingType::Utf16Be),
            "csunicode" | "iso-10646-ucs-2" | "ucs-2" | "unicode" | "unicodefeff" | "utf-16" | "utf-16le" => Some(EncodingType::Utf16Le),
            "x-user-defined" => Some(EncodingType::XUserDefined),
            _ => None,
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

#[derive(Debug, Clone)]
pub struct EncodingDialogState {
    selected: usize,
    encodings: Vec<EncodingType>,
}

impl EncodingDialogState {
    pub fn new(encoding_labels: &[String]) -> Self {
        let mut encodings: Vec<_> = encoding_labels
            .iter()
            .filter_map(|l| EncodingType::from(l))
            .collect();
        if encodings.is_empty() {
            encodings.push(EncodingType::Utf8);
        }
        Self {
            selected: 0,
            encodings,
        }
    }

    pub fn select_next(&mut self) {
        if self.selected == self.encodings.len() - 1 {
            self.selected = 0;
        } else {
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected == 0 {
            self.selected = self.encodings.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    pub fn reset(&mut self) {
        self.selected = 0;
    }

    pub fn selected(&self) -> EncodingType {
        self.encodings[self.selected]
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

pub struct EncodingDialog<'a> {
    state: &'a EncodingDialogState,
    labels: Vec<&'static str>,
    color: EncodingDialogColor,
}

impl<'a> EncodingDialog<'a> {
    pub fn new(state: &'a EncodingDialogState) -> Self {
        let labels = state.encodings.iter().map(EncodingType::str).collect();
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

impl Widget for EncodingDialog<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let list_items: Vec<ListItem> = self
            .labels
            .iter()
            .enumerate()
            .map(|(i, label)| {
                let item = ListItem::new(Line::raw(*label));
                if i == self.state.selected {
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
        default_encoding: EncodingType,
    ) -> (Self, Option<String>) {
        let mut state = Self {
            scroll_lines_state: ScrollLinesState::new(vec![], ScrollLinesOptions::default()),
            encoding: default_encoding,
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
