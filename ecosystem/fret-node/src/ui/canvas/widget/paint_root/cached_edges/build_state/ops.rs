use super::*;

pub(super) fn initial_clip_ops(clip_rect: Rect) -> Vec<SceneOp> {
    super::clip_ops::paint_root_cached_edge_build_state_initial_clip_ops(clip_rect)
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
    super::clip_ops::paint_root_cached_edge_build_state_merge_temp_ops(ops, tmp.ops());
    skipped || *next_edge_slot < edge_count
}
