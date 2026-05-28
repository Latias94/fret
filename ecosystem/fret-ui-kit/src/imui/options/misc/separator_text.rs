use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct SeparatorTextOptions {
    pub test_id: Option<Arc<str>>,
}
