use std::sync::Arc;

mod handles;
mod state;

pub(super) use handles::resize_stack_element;
pub(in crate::imui) use state::{current_resize_snapshot, prepare_resize_state};

#[derive(Debug, Clone)]
pub(super) struct FloatingWindowResizeHandleTestIds {
    pub(super) left: Arc<str>,
    pub(super) right: Arc<str>,
    pub(super) top: Arc<str>,
    pub(super) bottom: Arc<str>,
    pub(super) top_left: Arc<str>,
    pub(super) top_right: Arc<str>,
    pub(super) bottom_left: Arc<str>,
    pub(super) bottom_right: Arc<str>,
}
