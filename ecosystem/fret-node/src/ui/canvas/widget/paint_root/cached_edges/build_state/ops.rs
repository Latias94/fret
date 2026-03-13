use super::*;

fn extend_clip_stack_ops(ops: &mut Vec<SceneOp>, tmp: &[SceneOp]) {
    if tmp.is_empty() {
        return;
    }

    match ops.pop() {
        Some(SceneOp::PopClip) => {
            ops.extend_from_slice(tmp);
            ops.push(SceneOp::PopClip);
        }
        Some(other) => {
            ops.push(other);
            ops.extend_from_slice(tmp);
        }
        None => {
            ops.extend_from_slice(tmp);
        }
    }

    if !matches!(ops.last(), Some(SceneOp::PopClip)) {
        ops.push(SceneOp::PopClip);
    }
}

pub(super) fn initial_clip_ops(clip_rect: Rect) -> Vec<SceneOp> {
    vec![SceneOp::PushClipRect { rect: clip_rect }, SceneOp::PopClip]
}

pub(super) fn finish_build_state_step(
    ops: &mut Vec<SceneOp>,
    edge_count: usize,
    next_edge_slot: &mut usize,
    tmp: &fret_core::Scene,
    next_edge: usize,
    skipped: bool,
) -> bool {
    *next_edge_slot = next_edge;
    extend_clip_stack_ops(ops, tmp.ops());
    skipped || *next_edge_slot < edge_count
}
