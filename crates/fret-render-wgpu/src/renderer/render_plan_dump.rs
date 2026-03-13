use super::EffectMarker;
use super::render_plan::{
    DebugPostprocess, RenderPlan, RenderPlanDegradation, RenderPlanDegradationKind,
    RenderPlanDegradationReason, RenderPlanPass,
};
#[cfg(not(target_arch = "wasm32"))]
use super::render_plan_dump_emit::{emit_render_plan_dump_json, should_emit_render_plan_dump};
use super::render_plan_dump_encode::{
    JsonDumpDebugPostprocess, JsonDumpEffectMarker, JsonDumpPass, encode_debug_postprocess,
    encode_effect_marker, encode_pass,
};
use super::render_plan_dump_summary::{
    JsonDumpCustomEffectSummary, JsonDumpCustomEffectV3DiagnosticsSummary, JsonDumpTargetUsage,
    encode_custom_effect_v3_diagnostics_summary, summarize_custom_effects, summarize_target_usage,
};

#[derive(Debug, serde::Serialize)]
struct JsonDumpSegmentFlags {
    has_quad: bool,
    has_viewport: bool,
    has_image: bool,
    has_mask: bool,
    has_text: bool,
    has_path: bool,
}

#[derive(Debug, serde::Serialize)]
struct JsonDumpSegment {
    id: usize,
    draw_range: [usize; 2],
    start_uniform_index: Option<u32>,
    start_uniform_fingerprint: String,
    flags: JsonDumpSegmentFlags,
    pass_counts: JsonDumpSegmentPassCounts,
}

#[derive(Debug, serde::Serialize, Clone, Copy)]
struct JsonDumpSegmentPassCounts {
    scene_draw_range: usize,
    path_msaa_batch: usize,
}

#[derive(Debug, serde::Serialize)]
struct JsonDumpCounts {
    total: usize,
    scene: usize,
    path_msaa: usize,
    path_clip_mask: usize,
    fullscreen_blit: usize,
    composite_premul: usize,
    scale_nearest: usize,
    blur: usize,
    backdrop_warp: usize,
    color_adjust: usize,
    color_matrix: usize,
    alpha_threshold: usize,
    dither: usize,
    noise: usize,
    drop_shadow: usize,
    custom_effect_v1: usize,
    custom_effect_v2: usize,
    custom_effect_v3: usize,
    clip_mask: usize,
    release_target: usize,
}

fn pass_counts(plan: &RenderPlan) -> JsonDumpCounts {
    let mut c = JsonDumpCounts {
        total: plan.passes.len(),
        scene: 0,
        path_msaa: 0,
        path_clip_mask: 0,
        fullscreen_blit: 0,
        composite_premul: 0,
        scale_nearest: 0,
        blur: 0,
        backdrop_warp: 0,
        color_adjust: 0,
        color_matrix: 0,
        alpha_threshold: 0,
        dither: 0,
        noise: 0,
        drop_shadow: 0,
        custom_effect_v1: 0,
        custom_effect_v2: 0,
        custom_effect_v3: 0,
        clip_mask: 0,
        release_target: 0,
    };

    for p in &plan.passes {
        match p {
            RenderPlanPass::SceneDrawRange(_) => c.scene += 1,
            RenderPlanPass::PathMsaaBatch(_) => c.path_msaa += 1,
            RenderPlanPass::PathClipMask(_) => c.path_clip_mask += 1,
            RenderPlanPass::FullscreenBlit(_) => c.fullscreen_blit += 1,
            RenderPlanPass::CompositePremul(_) => c.composite_premul += 1,
            RenderPlanPass::ScaleNearest(_) => c.scale_nearest += 1,
            RenderPlanPass::Blur(_) => c.blur += 1,
            RenderPlanPass::BackdropWarp(_) => c.backdrop_warp += 1,
            RenderPlanPass::ColorAdjust(_) => c.color_adjust += 1,
            RenderPlanPass::ColorMatrix(_) => c.color_matrix += 1,
            RenderPlanPass::AlphaThreshold(_) => c.alpha_threshold += 1,
            RenderPlanPass::Dither(_) => c.dither += 1,
            RenderPlanPass::Noise(_) => c.noise += 1,
            RenderPlanPass::DropShadow(_) => c.drop_shadow += 1,
            RenderPlanPass::CustomEffect(_) => c.custom_effect_v1 += 1,
            RenderPlanPass::CustomEffectV2(_) => c.custom_effect_v2 += 1,
            RenderPlanPass::CustomEffectV3(_) => c.custom_effect_v3 += 1,
            RenderPlanPass::ClipMask(_) => c.clip_mask += 1,
            RenderPlanPass::ReleaseTarget(_) => c.release_target += 1,
        }
    }

    c
}

#[derive(Debug, serde::Serialize)]
struct RenderPlanJsonDump<'a> {
    schema_version: u32,
    frame_index: u64,
    viewport_size: [u32; 2],
    format: String,
    postprocess: JsonDumpDebugPostprocess,
    ordered_draws_len: usize,
    segments: &'a [JsonDumpSegment],
    effect_markers: &'a [JsonDumpEffectMarker],
    pass_counts: JsonDumpCounts,
    custom_effects: &'a [JsonDumpCustomEffectSummary],
    target_usage: &'a [JsonDumpTargetUsage],
    estimated_peak_intermediate_bytes: u64,
    custom_effect_v3_diagnostics: JsonDumpCustomEffectV3DiagnosticsSummary,
    degradations: &'a [JsonDumpDegradation],
    passes: &'a [JsonDumpPass],
}

#[derive(Debug, serde::Serialize)]
struct JsonDumpDegradation {
    draw_ix: usize,
    kind: &'static str,
    reason: &'static str,
}

#[derive(Debug, Default)]
pub(super) struct RenderPlanJsonDumpScratch {
    segment_pass_counts: Vec<JsonDumpSegmentPassCounts>,
    segments: Vec<JsonDumpSegment>,
    effect_markers: Vec<JsonDumpEffectMarker>,
    custom_effects: Vec<JsonDumpCustomEffectSummary>,
    target_usage: Vec<JsonDumpTargetUsage>,
    degradations: Vec<JsonDumpDegradation>,
    passes: Vec<JsonDumpPass>,
    bytes: Vec<u8>,
}

fn encode_degradation(d: RenderPlanDegradation) -> JsonDumpDegradation {
    let kind = match d.kind {
        RenderPlanDegradationKind::BackdropEffectNoOp => "BackdropEffectNoOp",
        RenderPlanDegradationKind::FilterContentDisabled => "FilterContentDisabled",
        RenderPlanDegradationKind::ClipPathDisabled => "ClipPathDisabled",
        RenderPlanDegradationKind::CompositeGroupBlendDegradedToOver => {
            "CompositeGroupBlendDegradedToOver"
        }
    };
    let reason = match d.reason {
        RenderPlanDegradationReason::BudgetZero => "BudgetZero",
        RenderPlanDegradationReason::BudgetInsufficient => "BudgetInsufficient",
        RenderPlanDegradationReason::TargetExhausted => "TargetExhausted",
    };
    JsonDumpDegradation {
        draw_ix: d.draw_ix,
        kind,
        reason,
    }
}

fn rebuild_segment_pass_counts(plan: &RenderPlan, dump_scratch: &mut RenderPlanJsonDumpScratch) {
    dump_scratch.segment_pass_counts.resize(
        plan.segments.len(),
        JsonDumpSegmentPassCounts {
            scene_draw_range: 0,
            path_msaa_batch: 0,
        },
    );
    dump_scratch
        .segment_pass_counts
        .fill(JsonDumpSegmentPassCounts {
            scene_draw_range: 0,
            path_msaa_batch: 0,
        });
    for pass in &plan.passes {
        match pass {
            RenderPlanPass::SceneDrawRange(pass) => {
                if let Some(count) = dump_scratch.segment_pass_counts.get_mut(pass.segment.0) {
                    count.scene_draw_range += 1;
                }
            }
            RenderPlanPass::PathMsaaBatch(pass) => {
                if let Some(count) = dump_scratch.segment_pass_counts.get_mut(pass.segment.0) {
                    count.path_msaa_batch += 1;
                }
            }
            _ => {}
        }
    }
}

fn rebuild_segment_dump_scratch(plan: &RenderPlan, dump_scratch: &mut RenderPlanJsonDumpScratch) {
    dump_scratch.segments.clear();
    dump_scratch.segments.reserve(plan.segments.len());
    for (ix, segment) in plan.segments.iter().enumerate() {
        dump_scratch.segments.push(JsonDumpSegment {
            id: segment.id.0,
            draw_range: [segment.draw_range.start, segment.draw_range.end],
            start_uniform_index: segment.start_uniform_index,
            start_uniform_fingerprint: format!("0x{:016x}", segment.start_uniform_fingerprint),
            flags: JsonDumpSegmentFlags {
                has_quad: segment.flags.has_quad,
                has_viewport: segment.flags.has_viewport,
                has_image: segment.flags.has_image,
                has_mask: segment.flags.has_mask,
                has_text: segment.flags.has_text,
                has_path: segment.flags.has_path,
            },
            pass_counts: dump_scratch.segment_pass_counts.get(ix).copied().unwrap_or(
                JsonDumpSegmentPassCounts {
                    scene_draw_range: 0,
                    path_msaa_batch: 0,
                },
            ),
        });
    }
}

fn rebuild_effect_marker_dump_scratch(
    effect_markers: &[EffectMarker],
    dump_scratch: &mut RenderPlanJsonDumpScratch,
) {
    dump_scratch.effect_markers.clear();
    dump_scratch.effect_markers.reserve(effect_markers.len());
    for marker in effect_markers.iter().copied() {
        dump_scratch
            .effect_markers
            .push(encode_effect_marker(marker));
    }
}

fn rebuild_degradation_dump_scratch(
    plan: &RenderPlan,
    dump_scratch: &mut RenderPlanJsonDumpScratch,
) {
    dump_scratch.degradations.clear();
    dump_scratch.degradations.reserve(plan.degradations.len());
    for degradation in plan.degradations.iter().copied() {
        dump_scratch
            .degradations
            .push(encode_degradation(degradation));
    }
}

fn rebuild_pass_dump_scratch(plan: &RenderPlan, dump_scratch: &mut RenderPlanJsonDumpScratch) {
    dump_scratch.passes.clear();
    dump_scratch.passes.reserve(plan.passes.len());
    for pass in &plan.passes {
        dump_scratch.passes.push(encode_pass(pass));
    }
}

fn rebuild_render_plan_json_dump_scratch(
    plan: &RenderPlan,
    effect_markers: &[EffectMarker],
    dump_scratch: &mut RenderPlanJsonDumpScratch,
) {
    rebuild_segment_pass_counts(plan, dump_scratch);
    rebuild_segment_dump_scratch(plan, dump_scratch);
    rebuild_effect_marker_dump_scratch(effect_markers, dump_scratch);
    rebuild_degradation_dump_scratch(plan, dump_scratch);
    rebuild_pass_dump_scratch(plan, dump_scratch);
    dump_scratch.custom_effects = summarize_custom_effects(&plan.passes);
    dump_scratch.target_usage = summarize_target_usage(&plan.passes);
}

fn assemble_render_plan_json_dump<'a>(
    plan: &'a RenderPlan,
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    frame_index: u64,
    postprocess: DebugPostprocess,
    ordered_draws_len: usize,
    dump_scratch: &'a RenderPlanJsonDumpScratch,
) -> RenderPlanJsonDump<'a> {
    RenderPlanJsonDump {
        schema_version: 7,
        frame_index,
        viewport_size: [viewport_size.0, viewport_size.1],
        format: format!("{format:?}"),
        postprocess: encode_debug_postprocess(postprocess),
        ordered_draws_len,
        segments: &dump_scratch.segments,
        effect_markers: &dump_scratch.effect_markers,
        pass_counts: pass_counts(plan),
        custom_effects: &dump_scratch.custom_effects,
        target_usage: &dump_scratch.target_usage,
        estimated_peak_intermediate_bytes: plan.compile_stats.estimated_peak_intermediate_bytes,
        custom_effect_v3_diagnostics: encode_custom_effect_v3_diagnostics_summary(
            plan.compile_stats.effect_degradations,
        ),
        degradations: &dump_scratch.degradations,
        passes: &dump_scratch.passes,
    }
}

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
