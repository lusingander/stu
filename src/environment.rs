#[derive(Debug, Default, Clone)]
pub struct Environment {
    pub image_picker: ImagePicker,
    pub fix_dynamic_values: bool,
}

impl Environment {
    pub fn new(image_preview_enabled: bool, fix_dynamic_values: bool) -> Environment {
        Environment {
            image_picker: build_image_picker(image_preview_enabled, fix_dynamic_values),
            fix_dynamic_values,
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

fn build_image_picker(image_preview_enabled: bool, fix_dynamic_values: bool) -> ImagePicker {
    if fix_dynamic_values {
        // - font size cannot be obtained with xterm.js
        // - want to fix the protocol to iterm2
        // so changed the settings if fix_dynamic_values is true
        let mut picker = ratatui_image::picker::Picker::from_fontsize((10, 20));
        picker.set_protocol_type(ratatui_image::picker::ProtocolType::Iterm2);
        return ImagePicker::Ok(picker);
    }

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
