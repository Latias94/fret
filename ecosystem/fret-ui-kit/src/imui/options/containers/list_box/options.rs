use std::sync::Arc;

use crate::imui::ScrollOptions;

#[derive(Debug, Clone)]
pub struct ListBoxOptions {
    pub layout: crate::LayoutRefinement,
    pub scroll: ScrollOptions,
    pub label: Option<Arc<str>>,
    pub multiselectable: bool,
    pub test_id: Option<Arc<str>>,
    pub content_test_id: Option<Arc<str>>,
}

impl ListBoxOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn multiselectable(mut self) -> Self {
        self.multiselectable = true;
        self
    }

    pub fn with_multiselectable(mut self, multiselectable: bool) -> Self {
        self.multiselectable = multiselectable;
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn content_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.content_test_id = Some(test_id.into());
        self
    }

    pub fn scroll(mut self, scroll: ScrollOptions) -> Self {
        self.scroll = scroll;
        self
    }

    pub fn scroll_handle(mut self, handle: fret_ui::scroll::ScrollHandle) -> Self {
        self.scroll.handle = Some(handle);
        self
    }

    pub fn viewport_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.scroll.viewport_test_id = Some(test_id.into());
        self
    }
}
