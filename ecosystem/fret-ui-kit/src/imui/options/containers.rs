use std::sync::Arc;

use fret_core::Px;

use crate::style::MetricFallback;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChildRegionChrome {
    #[default]
    Framed,
    Bare,
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

#[derive(Debug, Clone)]
pub struct SpacingOptions {
    pub size: Option<fret_core::Size>,
    pub test_id: Option<Arc<str>>,
}

impl Default for SpacingOptions {
    fn default() -> Self {
        Self {
            size: None,
            test_id: None,
        }
    }
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

#[derive(Debug, Clone)]
pub struct ScrollOptions {
    pub layout: crate::LayoutRefinement,
    pub axis: fret_ui::element::ScrollAxis,
    pub show_scrollbar_x: bool,
    pub show_scrollbar_y: bool,
    pub handle: Option<fret_ui::scroll::ScrollHandle>,
    pub test_id: Option<Arc<str>>,
    pub viewport_test_id: Option<Arc<str>>,
}

impl Default for ScrollOptions {
    fn default() -> Self {
        Self {
            layout: crate::LayoutRefinement::default(),
            axis: fret_ui::element::ScrollAxis::Y,
            show_scrollbar_x: false,
            show_scrollbar_y: true,
            handle: None,
            test_id: None,
            viewport_test_id: None,
        }
    }
}

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
