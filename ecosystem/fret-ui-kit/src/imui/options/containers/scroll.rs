use std::sync::Arc;

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
