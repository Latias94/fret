use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct BulletTextOptions {
    pub test_id: Option<Arc<str>>,
}
