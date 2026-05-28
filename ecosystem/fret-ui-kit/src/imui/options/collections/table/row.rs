use std::sync::Arc;

use fret_core::Color;

#[derive(Debug, Clone, Default)]
pub struct TableRowOptions {
    pub test_id: Option<Arc<str>>,
    pub background: Option<Color>,
}
