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

impl EditorThemePresetPickerOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn layout(mut self, layout: LayoutStyle) -> Self {
        self.layout = layout;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn without_label(mut self) -> Self {
        self.label = None;
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn item_test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.item_test_id_prefix = Some(prefix.into());
        self
    }
}
