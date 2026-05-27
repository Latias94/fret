use std::sync::Arc;

use super::spacing::{imui_item_spacing_x, imui_item_spacing_y};

#[derive(Debug, Clone)]
pub struct ItemFlowOptions {
    pub layout: crate::LayoutRefinement,
    pub gap: crate::MetricRef,
    pub justify: crate::Justify,
    pub items: crate::Items,
    pub wrap: bool,
    pub test_id: Option<Arc<str>>,
}

impl Default for ItemFlowOptions {
    fn default() -> Self {
        Self {
            layout: crate::LayoutRefinement::default(),
            gap: imui_item_spacing_y(),
            justify: crate::Justify::Start,
            items: crate::Items::Stretch,
            wrap: false,
            test_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SameLineOptions {
    pub layout: crate::LayoutRefinement,
    pub gap: crate::MetricRef,
    pub justify: crate::Justify,
    pub items: crate::Items,
    pub wrap: bool,
    pub test_id: Option<Arc<str>>,
}

impl Default for SameLineOptions {
    fn default() -> Self {
        Self {
            layout: crate::LayoutRefinement::default(),
            gap: imui_item_spacing_x(),
            justify: crate::Justify::Start,
            items: crate::Items::Center,
            wrap: false,
            test_id: None,
        }
    }
}
