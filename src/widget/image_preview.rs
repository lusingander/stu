use std::{
    fmt::{Debug, Formatter},
    io::Cursor,
};

use image::{DynamicImage, GenericImageView, ImageReader};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Padding, StatefulWidget, Widget},
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, StatefulImage};

use crate::{environment::Environment, format::format_version};

pub struct ImagePreviewState {
    protocol: Option<StatefulProtocol>,
    base_image: Option<DynamicImage>,
    // to control image rendering when dialogs are overlapped...
    render: bool,
}

impl Debug for ImagePreviewState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ImagePreviewState")
    }
}

pub enum ImagePicker {
    Disabled,
    Ok(Picker),
    Error(String),
}

impl ImagePreviewState {
    pub fn new(bytes: &[u8], image_picker: ImagePicker) -> (Self, Option<String>) {
        match build_image_protocol(bytes, image_picker) {
            Ok((protocol, img)) => {
                let state = ImagePreviewState {
                    protocol: Some(protocol),
                    base_image: Some(img),
                    render: true,
                };
                (state, None)
            }
            Err(e) => {
                let state = ImagePreviewState {
                    protocol: None,
                    base_image: None,
                    render: true,
                };
                (state, Some(e))
            }
        }
    }

    pub fn set_render(&mut self, render: bool) {
        self.render = render;
    }

    pub fn base_image_data(&self) -> Option<(usize, usize, Vec<u8>)> {
        self.base_image.as_ref().map(|img| {
            let (w, h) = img.dimensions();
            let bytes = img.to_rgba8().into_raw();
            (w as usize, h as usize, bytes)
        })
    }
}

fn build_image_protocol(
    bytes: &[u8],
    image_picker: ImagePicker,
) -> Result<(StatefulProtocol, DynamicImage), String> {
    match image_picker {
        ImagePicker::Ok(picker) => {
            let reader = ImageReader::new(Cursor::new(bytes))
                .with_guessed_format()
                .map_err(|e| format!("Failed to guess image format: {e}"))?;
            let img: DynamicImage = reader
                .decode()
                .map_err(|e| format!("Failed to decode image: {e}"))?;
            let protocol = picker.new_resize_protocol(img.clone());
            Ok((protocol, img))
        }
        ImagePicker::Error(e) => Err(format!("Failed to create picker: {e}")),
        ImagePicker::Disabled => Err("Image preview is disabled".into()),
    }
}

#[derive(Debug)]
pub struct ImagePreview<'a> {
    file_name: &'a str,
    file_version_id: Option<&'a str>,

    env: &'a Environment,
}

impl<'a> ImagePreview<'a> {
    pub fn new(file_name: &'a str, file_version_id: Option<&'a str>, env: &'a Environment) -> Self {
        Self {
            file_name,
            file_version_id,
            env,
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
                format_version(Some(version_id), self.env.fix_dynamic_values)
            )
        } else {
            format!("Preview [{}]", self.file_name)
        };
        let block = Block::bordered().padding(Padding::uniform(1)).title(title);
        let image_area = block.inner(area);

        block.render(area, buf);

        if state.render {
            if let Some(protocol) = &mut state.protocol {
                let image = StatefulImage::default();
                image.render(image_area, buf, protocol);
            }
        }
    }
}
