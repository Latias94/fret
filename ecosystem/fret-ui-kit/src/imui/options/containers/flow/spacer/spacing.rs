use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct SpacingOptions {
    pub size: Option<fret_core::Size>,
    pub test_id: Option<Arc<str>>,
}
