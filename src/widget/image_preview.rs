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

pub struct ImagePreviewState {
    protocol: Option<Box<dyn StatefulProtocol>>,
}

impl Debug for ImagePreviewState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ImagePreviewState")
    }
}

impl ImagePreviewState {
    pub fn new(bytes: &[u8]) -> (Self, Option<String>) {
        match build_image_protocol(bytes) {
            Ok(protocol) => {
                let protocol = Some(protocol);
                let state = ImagePreviewState { protocol };
                (state, None)
            }
            Err(e) => {
                let protocol = None;
                let state = ImagePreviewState { protocol };
                (state, Some(e))
            }
        }
    }
}

fn build_image_protocol(bytes: &[u8]) -> Result<Box<dyn StatefulProtocol>, String> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| format!("Failed to guess image format: {e}"))?;

    let img: DynamicImage = reader
        .decode()
        .map_err(|e| format!("Failed to decode image: {e}"))?;

    let mut picker = Picker::from_termios().map_err(|e| format!("Failed to create picker: {e}"))?;
    picker.guess_protocol();

    Ok(picker.new_resize_protocol(img))
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
            format!("Preview [{} (Version ID: {})]", self.file_name, version_id)
        } else {
            format!("Preview [{}]", self.file_name)
        };
        let block = Block::bordered().padding(Padding::uniform(1)).title(title);
        let image_area = block.inner(area);

        block.render(area, buf);

        if let Some(protocol) = &mut state.protocol {
            let image = StatefulImage::new(None);
            image.render(image_area, buf, protocol);
        }
    }
}
