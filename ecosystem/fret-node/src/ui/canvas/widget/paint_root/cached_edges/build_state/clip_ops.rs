use super::*;

fn append_temp_ops_before_trailing_pop_clip(ops: &mut Vec<SceneOp>, temp_ops: &[SceneOp]) {
    if temp_ops.is_empty() {
        return;
    }

    match ops.pop() {
        Some(SceneOp::PopClip) => {
            ops.extend_from_slice(temp_ops);
            ops.push(SceneOp::PopClip);
        }
        Some(other) => {
            ops.push(other);
            ops.extend_from_slice(temp_ops);
        }
        None => {
            ops.extend_from_slice(temp_ops);
        }
    }

    if !matches!(ops.last(), Some(SceneOp::PopClip)) {
        ops.push(SceneOp::PopClip);
    }
}

pub(super) fn paint_root_cached_edge_build_state_initial_clip_ops(clip_rect: Rect) -> Vec<SceneOp> {
    vec![SceneOp::PushClipRect { rect: clip_rect }, SceneOp::PopClip]
}

pub(super) fn paint_root_cached_edge_build_state_merge_temp_ops(
    ops: &mut Vec<SceneOp>,
    temp_ops: &[SceneOp],
) {
    append_temp_ops_before_trailing_pop_clip(ops, temp_ops);
}
