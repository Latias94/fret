use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MenuBarOptions {
    pub gap: crate::MetricRef,
    pub test_id: Option<Arc<str>>,
}

impl Default for MenuBarOptions {
    fn default() -> Self {
        Self {
            gap: crate::MetricRef::space(crate::Space::N1),
            test_id: None,
        }
    }
}
