use std::sync::Arc;

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
