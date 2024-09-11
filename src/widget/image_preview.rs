use std::{
    fmt::{Debug, Formatter},
    io::Cursor,
};

use image::{DynamicImage, ImageReader};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Padding, StatefulWidget, Widget},
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, StatefulImage};

use crate::ui::common::format_version;

pub struct ImagePreviewState {
    protocol: Option<Box<dyn StatefulProtocol>>,
    // to control image rendering when dialogs are overlapped...
    render: bool,
}

impl Debug for ImagePreviewState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ImagePreviewState")
    }
}

impl ImagePreviewState {
    pub fn new(bytes: &[u8], enabled: bool) -> (Self, Option<String>) {
        match build_image_protocol(bytes, enabled) {
            Ok(protocol) => {
                let state = ImagePreviewState {
                    protocol: Some(protocol),
                    render: true,
                };
                (state, None)
            }
            Err(e) => {
                let state = ImagePreviewState {
                    protocol: None,
                    render: true,
                };
                (state, Some(e))
            }
        }
    }

    pub fn set_render(&mut self, render: bool) {
        self.render = render;
    }
}

fn build_image_protocol(bytes: &[u8], enabled: bool) -> Result<Box<dyn StatefulProtocol>, String> {
    if !enabled {
        return Err("Image preview is disabled".to_string());
    }

    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| format!("Failed to guess image format: {e}"))?;

    let img: DynamicImage = reader
        .decode()
        .map_err(|e| format!("Failed to decode image: {e}"))?;

    let mut picker = build_picker()?;
    Ok(picker.new_resize_protocol(img))
}

#[cfg(not(feature = "imggen"))]
fn build_picker() -> Result<Picker, String> {
    let mut picker = Picker::from_termios().map_err(|e| format!("Failed to create picker: {e}"))?;
    picker.guess_protocol();
    Ok(picker)
}

#[cfg(feature = "imggen")]
fn build_picker() -> Result<Picker, String> {
    // - font size cannot be obtained with xterm.js
    // - want to fix the protocol to iterm2
    // so changed the settings with the imggen feature
    let mut picker = Picker::new((10, 20));
    picker.protocol_type = ratatui_image::picker::ProtocolType::Iterm2;
    Ok(picker)
}

#[derive(Debug)]
pub struct ImagePreview<'a> {
    file_name: &'a str,
    file_version_id: Option<&'a str>,
}

impl<'a> ImagePreview<'a> {
    pub fn new(file_name: &'a str, file_version_id: Option<&'a str>) -> Self {
        Self {
            file_name,
            file_version_id,
        }
    }
}

impl StatefulWidget for ImagePreview<'_> {
    type State = ImagePreviewState;

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
        let block = Block::bordered().padding(Padding::uniform(1)).title(title);
        let image_area = block.inner(area);

        block.render(area, buf);

        if state.render {
            if let Some(protocol) = &mut state.protocol {
                let image = StatefulImage::new(None);
                image.render(image_area, buf, protocol);
            }
        }
    }
}
