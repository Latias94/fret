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
