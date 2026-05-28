use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct DummyOptions {
    pub test_id: Option<Arc<str>>,
}
