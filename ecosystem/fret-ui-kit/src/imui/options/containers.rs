use std::sync::Arc;

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

#[derive(Debug, Clone, Default)]
pub struct ChildRegionOptions {
    pub layout: crate::LayoutRefinement,
    pub scroll: ScrollOptions,
    pub test_id: Option<Arc<str>>,
    pub content_test_id: Option<Arc<str>>,
}
