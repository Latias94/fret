use crate::ops::GraphOp;
use crate::ui::canvas::state::GroupResize;

pub(in super::super) fn build_group_resize_ops(resize: &GroupResize) -> Vec<GraphOp> {
    if resize.current_rect == resize.start_rect {
        return Vec::new();
    }
    vec![GraphOp::SetGroupRect {
        id: resize.group,
        from: resize.start_rect,
        to: resize.current_rect,
    }]
}

#[cfg(test)]
mod tests;
