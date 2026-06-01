use std::sync::Arc;

use fret_core::{Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{LayoutStyle, Length, SizeStyle};
use fret_ui_kit::headless::text_assist::{InputOwnedTextAssistKeyOptions, TextAssistItem};

#[derive(Debug, Clone)]
pub struct InspectorPanelSearchAssistOptions {
    pub dismissed_query_model: Model<String>,
    pub active_item_id_model: Model<Option<Arc<str>>>,
    pub items: Arc<[TextAssistItem]>,
    pub list_label: Arc<str>,
    pub empty_label: Arc<str>,
    pub key_options: InputOwnedTextAssistKeyOptions,
    pub list_test_id: Option<Arc<str>>,
    pub item_test_id_prefix: Option<Arc<str>>,
    pub empty_test_id: Option<Arc<str>>,
    pub max_list_height: Option<Px>,
}

#[derive(Debug, Clone)]
pub struct InspectorPanelOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub title: Option<Arc<str>>,
    pub padding: Option<Edges>,
    pub gap: Option<Px>,
    pub header_gap: Option<Px>,
    pub test_id: Option<Arc<str>>,
    pub header_test_id: Option<Arc<str>>,
    pub toolbar_test_id: Option<Arc<str>>,
    pub search_test_id: Option<Arc<str>>,
    pub search_clear_test_id: Option<Arc<str>>,
    pub search_assist: Option<InspectorPanelSearchAssistOptions>,
    pub content_test_id: Option<Arc<str>>,
}

impl Default for InspectorPanelOptions {
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
            title: None,
            padding: None,
            gap: None,
            header_gap: None,
            test_id: None,
            header_test_id: None,
            toolbar_test_id: None,
            search_test_id: None,
            search_clear_test_id: None,
            search_assist: None,
            content_test_id: None,
        }
    }
}
