use super::DrawScope;
use super::context::RenderPlanCompilerCtx;
use crate::renderer::{OrderedDraw, PathMsaaBatchPass, RenderPlanPass, SceneEncoding};

pub(super) fn try_compile_path_msaa_batch(
    plan: &mut RenderPlanCompilerCtx,
    draw_scopes: &mut Vec<DrawScope>,
    draws: &[OrderedDraw],
    encoding: &SceneEncoding,
    cursor: usize,
    next_marker_at: usize,
    path_samples: u32,
    scene_range_start: &mut usize,
) -> Option<usize> {
    if path_samples <= 1 {
        return None;
    }

    let OrderedDraw::Path(first) = &draws[cursor] else {
        return None;
    };

    plan.flush_scene_range(cursor, draw_scopes, draws, encoding, scene_range_start);

    let batch_uniform_index = first.uniform_index;
    let mut union = first.scissor;
    let mut end = cursor + 1;
    while end < draws.len() && end < next_marker_at {
        match &draws[end] {
            OrderedDraw::Path(draw) if draw.uniform_index == batch_uniform_index => {
                union = super::super::union_scissor(union, draw.scissor);
                end += 1;
            }
            _ => break,
        }
    }

    let scope = draw_scopes.last().expect("draw scope");
    let target = scope.target;
    let segment = plan.alloc_segment(cursor..end, draws, encoding);
    plan.push_pass(RenderPlanPass::PathMsaaBatch(PathMsaaBatchPass {
        segment,
        target,
        target_origin: scope.origin,
        target_size: scope.size,
        draw_range: cursor..end,
        union_scissor: super::super::AbsoluteScissorRect(union),
        batch_uniform_index,
        load: wgpu::LoadOp::Load,
    }));

    *scene_range_start = end;
    Some(end)
}
