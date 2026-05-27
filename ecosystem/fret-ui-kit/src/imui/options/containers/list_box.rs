use std::sync::Arc;

use fret_core::Px;

use super::ScrollOptions;

#[derive(Debug, Clone)]
pub struct ListBoxOptions {
    pub layout: crate::LayoutRefinement,
    pub scroll: ScrollOptions,
    pub label: Option<Arc<str>>,
    pub multiselectable: bool,
    pub test_id: Option<Arc<str>>,
    pub content_test_id: Option<Arc<str>>,
}

impl Default for ListBoxOptions {
    fn default() -> Self {
        Self {
            layout: crate::LayoutRefinement::default().h_px(Px(160.0)),
            scroll: ScrollOptions::default(),
            label: None,
            multiselectable: false,
            test_id: None,
            content_test_id: None,
        }
    }
}
