use std::sync::Arc;

use fret_core::Px;
use fret_runtime::Model;
use fret_ui::element::{LayoutStyle, Length, SizeStyle};

#[derive(Debug, Clone)]
pub struct PropertyGroupOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub collapsible: bool,
    pub default_collapsed: bool,
    pub collapsed: Option<Model<bool>>,
    pub header_height: Option<Px>,
    pub gap: Option<Px>,
    pub test_id: Option<Arc<str>>,
    pub header_test_id: Option<Arc<str>>,
    pub content_test_id: Option<Arc<str>>,
}

impl Default for PropertyGroupOptions {
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
            collapsible: true,
            default_collapsed: false,
            collapsed: None,
            header_height: None,
            gap: None,
            test_id: None,
            header_test_id: None,
            content_test_id: None,
        }
    }
}
