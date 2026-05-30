use std::sync::Arc;

use super::super::spacing::imui_indent_spacing;

#[derive(Debug, Clone)]
pub struct IndentOptions {
    pub width: crate::MetricRef,
    pub test_id: Option<Arc<str>>,
    pub content_test_id: Option<Arc<str>>,
}

impl Default for IndentOptions {
    fn default() -> Self {
        Self {
            width: imui_indent_spacing(),
            test_id: None,
            content_test_id: None,
        }
    }
}
