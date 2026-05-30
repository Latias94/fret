use std::sync::Arc;

use fret_ui::element::{LayoutStyle, Length, SizeStyle};

#[derive(Debug, Clone)]
pub struct EditorThemePresetPickerOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub focusable: bool,
    pub label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub item_test_id_prefix: Option<Arc<str>>,
}

impl Default for EditorThemePresetPickerOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: true,
            focusable: true,
            label: Some(Arc::from("Editor theme preset")),
            test_id: None,
            item_test_id_prefix: None,
        }
    }
}
