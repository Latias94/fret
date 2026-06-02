use std::sync::Arc;

use fret_ui::element::{LayoutStyle, Length, SizeStyle};

#[derive(Debug, Clone)]
pub struct CheckboxOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for CheckboxOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Auto,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
        }
    }
}
