use super::EffectMarker;
use super::render_plan::{DebugPostprocess, RenderPlan};
pub(super) use super::render_plan_dump_assemble::RenderPlanJsonDumpScratch;
#[cfg(not(target_arch = "wasm32"))]
use super::render_plan_dump_assemble::{
    assemble_render_plan_json_dump, rebuild_render_plan_json_dump_scratch,
};
#[cfg(not(target_arch = "wasm32"))]
use super::render_plan_dump_emit::{emit_render_plan_dump_json, should_emit_render_plan_dump};

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn maybe_dump_render_plan_json(
    plan: &RenderPlan,
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    frame_index: u64,
    postprocess: DebugPostprocess,
    ordered_draws_len: usize,
    effect_markers: &[EffectMarker],
    dump_scratch: &mut RenderPlanJsonDumpScratch,
) {
    if !should_emit_render_plan_dump(frame_index) {
        return;
    }

    rebuild_render_plan_json_dump_scratch(plan, effect_markers, dump_scratch);
    let mut bytes = std::mem::take(&mut dump_scratch.bytes);
    bytes.clear();
    {
        let dump = assemble_render_plan_json_dump(
            plan,
            viewport_size,
            format,
            frame_index,
            postprocess,
            ordered_draws_len,
            dump_scratch,
        );
        if serde_json::to_writer_pretty(&mut bytes, &dump).is_err() {
            dump_scratch.bytes = bytes;
            return;
        }
    }
    emit_render_plan_dump_json(frame_index, &bytes);
    dump_scratch.bytes = bytes;
}

#[cfg(target_arch = "wasm32")]
pub(super) fn maybe_dump_render_plan_json(
    _plan: &RenderPlan,
    _viewport_size: (u32, u32),
    _format: wgpu::TextureFormat,
    _frame_index: u64,
    _postprocess: DebugPostprocess,
    _ordered_draws_len: usize,
    _effect_markers: &[EffectMarker],
    _dump_scratch: &mut RenderPlanJsonDumpScratch,
) {
}
