use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TabBarOptions {
    pub selected: Option<fret_runtime::Model<Option<Arc<str>>>>,
    pub gap: crate::MetricRef,
    pub test_id: Option<Arc<str>>,
}

impl Default for TabBarOptions {
    fn default() -> Self {
        Self {
            selected: None,
            gap: crate::MetricRef::space(crate::Space::N1),
            test_id: None,
        }
    }
}
