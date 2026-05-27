use std::sync::Arc;

use fret_core::Px;

use crate::style::MetricFallback;

pub const IMUI_ITEM_SPACING_X_TOKEN: &str = "component.imui.item_spacing_x_px";
pub const IMUI_ITEM_SPACING_Y_TOKEN: &str = "component.imui.item_spacing_y_px";
pub const IMUI_INDENT_SPACING_TOKEN: &str = "component.imui.indent_spacing_px";

pub(crate) fn imui_item_spacing_x() -> crate::MetricRef {
    crate::MetricRef::Token {
        key: IMUI_ITEM_SPACING_X_TOKEN,
        fallback: MetricFallback::Px(Px(8.0)),
    }
}

pub(crate) fn imui_item_spacing_y() -> crate::MetricRef {
    crate::MetricRef::Token {
        key: IMUI_ITEM_SPACING_Y_TOKEN,
        fallback: MetricFallback::Px(Px(4.0)),
    }
}

pub(crate) fn imui_indent_spacing() -> crate::MetricRef {
    crate::MetricRef::Token {
        key: IMUI_INDENT_SPACING_TOKEN,
        fallback: MetricFallback::Px(Px(21.0)),
    }
}

#[derive(Debug, Clone)]
pub struct HorizontalOptions {
    pub layout: crate::LayoutRefinement,
    pub gap: crate::MetricRef,
    pub justify: crate::Justify,
    pub items: crate::Items,
    pub wrap: bool,
    pub test_id: Option<Arc<str>>,
}

impl Default for HorizontalOptions {
    fn default() -> Self {
        Self {
            layout: crate::LayoutRefinement::default(),
            gap: crate::MetricRef::space(crate::Space::N0),
            justify: crate::Justify::Start,
            items: crate::Items::Center,
            wrap: false,
            test_id: None,
        }
    }
}

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

#[derive(Debug, Clone, Default)]
pub struct DummyOptions {
    pub test_id: Option<Arc<str>>,
}

#[derive(Debug, Clone, Default)]
pub struct SpacingOptions {
    pub size: Option<fret_core::Size>,
    pub test_id: Option<Arc<str>>,
}

#[derive(Debug, Clone)]
pub struct IndentOptions {
    pub width: crate::MetricRef,
    pub test_id: Option<Arc<str>>,
    pub content_test_id: Option<Arc<str>>,
}

impl Default for IndentOptions {
    fn default() -> Self {
        Self {
            width: imui_indent_spacing(),
            test_id: None,
            content_test_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VerticalOptions {
    pub layout: crate::LayoutRefinement,
    pub gap: crate::MetricRef,
    pub justify: crate::Justify,
    pub items: crate::Items,
    pub wrap: bool,
    pub test_id: Option<Arc<str>>,
}

impl Default for VerticalOptions {
    fn default() -> Self {
        Self {
            layout: crate::LayoutRefinement::default(),
            gap: crate::MetricRef::space(crate::Space::N0),
            justify: crate::Justify::Start,
            items: crate::Items::Stretch,
            wrap: false,
            test_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridOptions {
    pub layout: crate::LayoutRefinement,
    pub columns: usize,
    pub column_gap: crate::MetricRef,
    pub row_gap: crate::MetricRef,
    pub row_justify: crate::Justify,
    pub row_items: crate::Items,
    pub test_id: Option<Arc<str>>,
}

impl Default for GridOptions {
    fn default() -> Self {
        Self {
            layout: crate::LayoutRefinement::default(),
            columns: 1,
            column_gap: crate::MetricRef::space(crate::Space::N0),
            row_gap: crate::MetricRef::space(crate::Space::N0),
            row_justify: crate::Justify::Start,
            row_items: crate::Items::Center,
            test_id: None,
        }
    }
}
