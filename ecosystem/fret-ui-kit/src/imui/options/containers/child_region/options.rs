use std::sync::Arc;

use super::super::ScrollOptions;
use super::chrome::ChildRegionChrome;
use super::resize::{ChildRegionResizeXOptions, ChildRegionResizeYOptions};

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
