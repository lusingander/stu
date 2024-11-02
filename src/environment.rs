use crate::config::Config;

#[derive(Debug, Default, Clone)]
pub struct Environment {
    pub image_picker: ImagePicker,
}

impl Environment {
    pub fn new(config: &Config) -> Environment {
        Environment {
            image_picker: build_image_picker(config.preview.image),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub enum ImagePicker {
    #[default]
    Disabled,
    Ok(ratatui_image::picker::Picker),
    Error(String),
}

#[cfg(not(feature = "imggen"))]
fn build_image_picker(image_preview_enabled: bool) -> ImagePicker {
    if image_preview_enabled {
        match ratatui_image::picker::Picker::from_query_stdio() {
            Ok(picker) => {
                if let ratatui_image::picker::ProtocolType::Halfblocks = picker.protocol_type() {
                    ImagePicker::Error("This terminal does not support any protocol".into())
                } else {
                    ImagePicker::Ok(picker)
                }
            }
            Err(e) => ImagePicker::Error(e.to_string()),
        }
    } else {
        ImagePicker::Disabled
    }
}

#[cfg(feature = "imggen")]
fn build_image_picker(_image_preview_enabled: bool) -> ImagePicker {
    // - font size cannot be obtained with xterm.js
    // - want to fix the protocol to iterm2
    // so changed the settings with the imggen feature
    let mut picker = ratatui_image::picker::Picker::from_fontsize((10, 20));
    picker.set_protocol_type(ratatui_image::picker::ProtocolType::Iterm2);
    ImagePicker::Ok(picker)
}
