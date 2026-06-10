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

impl ListBoxOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn height(mut self, height: Px) -> Self {
        self.layout = self.layout.h_px(height);
        self
    }

    pub fn width(mut self, width: Px) -> Self {
        self.layout = self.layout.w_px(width);
        self
    }

    pub fn size(mut self, width: Px, height: Px) -> Self {
        self.layout = self.layout.w_px(width).h_px(height);
        self
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
