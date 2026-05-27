use std::sync::Arc;

use fret_core::Px;

use super::ScrollOptions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChildRegionChrome {
    #[default]
    Framed,
    Bare,
}

#[derive(Debug, Clone, Default)]
pub struct ChildRegionOptions {
    pub chrome: ChildRegionChrome,
    pub layout: crate::LayoutRefinement,
    pub scroll: ScrollOptions,
    pub resize_x: Option<ChildRegionResizeXOptions>,
    pub resize_y: Option<ChildRegionResizeYOptions>,
    pub test_id: Option<Arc<str>>,
    pub content_test_id: Option<Arc<str>>,
}

#[derive(Debug, Clone)]
pub struct ChildRegionResizeXOptions {
    pub min_width: Option<Px>,
    pub max_width: Option<Px>,
    pub handle_test_id: Option<Arc<str>>,
}

impl Default for ChildRegionResizeXOptions {
    fn default() -> Self {
        Self {
            min_width: Some(Px(32.0)),
            max_width: None,
            handle_test_id: None,
        }
    }
}

impl ChildRegionResizeXOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn min_width(mut self, min_width: Px) -> Self {
        self.min_width = Some(min_width);
        self
    }

    pub fn max_width(mut self, max_width: Px) -> Self {
        self.max_width = Some(max_width);
        self
    }

    pub fn handle_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.handle_test_id = Some(test_id.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct ChildRegionResizeYOptions {
    pub min_height: Option<Px>,
    pub max_height: Option<Px>,
    pub handle_test_id: Option<Arc<str>>,
}

impl Default for ChildRegionResizeYOptions {
    fn default() -> Self {
        Self {
            min_height: Some(Px(32.0)),
            max_height: None,
            handle_test_id: None,
        }
    }
}

impl ChildRegionResizeYOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn min_height(mut self, min_height: Px) -> Self {
        self.min_height = Some(min_height);
        self
    }

    pub fn max_height(mut self, max_height: Px) -> Self {
        self.max_height = Some(max_height);
        self
    }

    pub fn handle_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.handle_test_id = Some(test_id.into());
        self
    }
}
