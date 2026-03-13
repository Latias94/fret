use super::EffectMarker;
use super::render_plan::{
    DebugPostprocess, RenderPlan, RenderPlanDegradation, RenderPlanDegradationKind,
    RenderPlanDegradationReason, RenderPlanPass,
};
use super::render_plan_dump_encode::{
    JsonDumpDebugPostprocess, JsonDumpEffectMarker, JsonDumpPass, encode_debug_postprocess,
    encode_effect_marker, encode_pass,
};
use super::render_plan_dump_summary::{
    JsonDumpCustomEffectSummary, JsonDumpCustomEffectV3DiagnosticsSummary, JsonDumpTargetUsage,
    encode_custom_effect_v3_diagnostics_summary, summarize_custom_effects, summarize_target_usage,
};

#[derive(Debug, serde::Serialize)]
pub(super) struct JsonDumpSegmentFlags {
    has_quad: bool,
    has_viewport: bool,
    has_image: bool,
    has_mask: bool,
    has_text: bool,
    has_path: bool,
}

#[derive(Debug, serde::Serialize)]
pub(super) struct JsonDumpSegment {
    id: usize,
    draw_range: [usize; 2],
    start_uniform_index: Option<u32>,
    start_uniform_fingerprint: String,
    flags: JsonDumpSegmentFlags,
    pass_counts: JsonDumpSegmentPassCounts,
}

#[derive(Debug, serde::Serialize, Clone, Copy)]
pub(super) struct JsonDumpSegmentPassCounts {
    scene_draw_range: usize,
    path_msaa_batch: usize,
}

#[derive(Debug, serde::Serialize)]
pub(super) struct JsonDumpCounts {
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
    let mut counts = JsonDumpCounts {
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

    for pass in &plan.passes {
        match pass {
            RenderPlanPass::SceneDrawRange(_) => counts.scene += 1,
            RenderPlanPass::PathMsaaBatch(_) => counts.path_msaa += 1,
            RenderPlanPass::PathClipMask(_) => counts.path_clip_mask += 1,
            RenderPlanPass::FullscreenBlit(_) => counts.fullscreen_blit += 1,
            RenderPlanPass::CompositePremul(_) => counts.composite_premul += 1,
            RenderPlanPass::ScaleNearest(_) => counts.scale_nearest += 1,
            RenderPlanPass::Blur(_) => counts.blur += 1,
            RenderPlanPass::BackdropWarp(_) => counts.backdrop_warp += 1,
            RenderPlanPass::ColorAdjust(_) => counts.color_adjust += 1,
            RenderPlanPass::ColorMatrix(_) => counts.color_matrix += 1,
            RenderPlanPass::AlphaThreshold(_) => counts.alpha_threshold += 1,
            RenderPlanPass::Dither(_) => counts.dither += 1,
            RenderPlanPass::Noise(_) => counts.noise += 1,
            RenderPlanPass::DropShadow(_) => counts.drop_shadow += 1,
            RenderPlanPass::CustomEffect(_) => counts.custom_effect_v1 += 1,
            RenderPlanPass::CustomEffectV2(_) => counts.custom_effect_v2 += 1,
            RenderPlanPass::CustomEffectV3(_) => counts.custom_effect_v3 += 1,
            RenderPlanPass::ClipMask(_) => counts.clip_mask += 1,
            RenderPlanPass::ReleaseTarget(_) => counts.release_target += 1,
        }
    }

    counts
}

#[derive(Debug, serde::Serialize)]
pub(super) struct RenderPlanJsonDump<'a> {
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
pub(super) struct JsonDumpDegradation {
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
    pub(super) bytes: Vec<u8>,
}

fn encode_degradation(degradation: RenderPlanDegradation) -> JsonDumpDegradation {
    let kind = match degradation.kind {
        RenderPlanDegradationKind::BackdropEffectNoOp => "BackdropEffectNoOp",
        RenderPlanDegradationKind::FilterContentDisabled => "FilterContentDisabled",
        RenderPlanDegradationKind::ClipPathDisabled => "ClipPathDisabled",
        RenderPlanDegradationKind::CompositeGroupBlendDegradedToOver => {
            "CompositeGroupBlendDegradedToOver"
        }
    };
    let reason = match degradation.reason {
        RenderPlanDegradationReason::BudgetZero => "BudgetZero",
        RenderPlanDegradationReason::BudgetInsufficient => "BudgetInsufficient",
        RenderPlanDegradationReason::TargetExhausted => "TargetExhausted",
    };
    JsonDumpDegradation {
        draw_ix: degradation.draw_ix,
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

pub(super) fn rebuild_render_plan_json_dump_scratch(
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

pub(super) fn assemble_render_plan_json_dump<'a>(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::render_plan::{
        ClipMaskPass, CustomEffectPass, CustomEffectPassCommon, CustomEffectV2Pass,
        CustomEffectV3Pass, LocalScissorRect, PathMsaaBatchPass, PlanTarget,
        RenderPlanCompileStats, RenderPlanDegradation, RenderPlanDegradationKind,
        RenderPlanDegradationReason, RenderPlanPass, RenderPlanSegment, RenderPlanSegmentFlags,
        SceneDrawRangePass, SceneSegmentId,
    };

    fn custom_effect_common(dst: PlanTarget) -> CustomEffectPassCommon {
        CustomEffectPassCommon {
            src: PlanTarget::Intermediate0,
            dst,
            src_size: (64, 64),
            dst_size: (64, 64),
            dst_scissor: None,
            mask_uniform_index: None,
            mask: None,
            effect: fret_core::EffectId::default(),
            params: fret_core::scene::EffectParamsV1::ZERO,
            load: wgpu::LoadOp::Load,
        }
    }

    #[test]
    fn render_plan_dump_assembly_tracks_segment_passes_and_counts() {
        let plan = RenderPlan {
            segments: vec![
                RenderPlanSegment {
                    id: SceneSegmentId(0),
                    draw_range: 0..3,
                    start_uniform_index: Some(7),
                    start_uniform_fingerprint: 0x11,
                    flags: RenderPlanSegmentFlags {
                        has_quad: true,
                        ..Default::default()
                    },
                },
                RenderPlanSegment {
                    id: SceneSegmentId(1),
                    draw_range: 3..6,
                    start_uniform_index: Some(9),
                    start_uniform_fingerprint: 0x22,
                    flags: RenderPlanSegmentFlags {
                        has_path: true,
                        ..Default::default()
                    },
                },
            ],
            passes: vec![
                RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                    segment: SceneSegmentId(0),
                    target: PlanTarget::Intermediate0,
                    target_origin: (0, 0),
                    target_size: (64, 64),
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    draw_range: 0..3,
                }),
                RenderPlanPass::PathMsaaBatch(PathMsaaBatchPass {
                    segment: SceneSegmentId(0),
                    target: PlanTarget::Intermediate0,
                    target_origin: (0, 0),
                    target_size: (64, 64),
                    draw_range: 0..3,
                    union_scissor: crate::renderer::AbsoluteScissorRect(
                        crate::renderer::ScissorRect::full(64, 64),
                    ),
                    batch_uniform_index: 5,
                    load: wgpu::LoadOp::Load,
                }),
                RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                    segment: SceneSegmentId(1),
                    target: PlanTarget::Output,
                    target_origin: (0, 0),
                    target_size: (64, 64),
                    load: wgpu::LoadOp::Load,
                    draw_range: 3..6,
                }),
                RenderPlanPass::CustomEffect(CustomEffectPass {
                    common: custom_effect_common(PlanTarget::Intermediate1),
                }),
                RenderPlanPass::CustomEffectV2(CustomEffectV2Pass {
                    common: custom_effect_common(PlanTarget::Intermediate2),
                    input_image: Some(fret_core::ImageId::default()),
                    input_uv: fret_core::scene::UvRect::FULL,
                    input_sampling: fret_core::scene::ImageSamplingHint::Default,
                }),
                RenderPlanPass::CustomEffectV3(CustomEffectV3Pass {
                    src_raw: PlanTarget::Intermediate0,
                    src_pyramid: PlanTarget::Intermediate1,
                    pyramid_levels: 2,
                    pyramid_build_scissor: Some(LocalScissorRect(
                        crate::renderer::ScissorRect::full(32, 32),
                    )),
                    raw_wanted: true,
                    pyramid_wanted: true,
                    common: custom_effect_common(PlanTarget::Output),
                    user0_image: None,
                    user0_uv: fret_core::scene::UvRect::FULL,
                    user0_sampling: fret_core::scene::ImageSamplingHint::Default,
                    user1_image: None,
                    user1_uv: fret_core::scene::UvRect::FULL,
                    user1_sampling: fret_core::scene::ImageSamplingHint::Default,
                }),
                RenderPlanPass::ClipMask(ClipMaskPass {
                    dst: PlanTarget::Mask0,
                    dst_size: (64, 64),
                    dst_scissor: None,
                    uniform_index: 12,
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                }),
                RenderPlanPass::ReleaseTarget(PlanTarget::Intermediate1),
            ],
            compile_stats: RenderPlanCompileStats::default(),
            degradations: vec![RenderPlanDegradation {
                draw_ix: 5,
                kind: RenderPlanDegradationKind::BackdropEffectNoOp,
                reason: RenderPlanDegradationReason::BudgetZero,
            }],
        };

        let mut scratch = RenderPlanJsonDumpScratch::default();
        rebuild_render_plan_json_dump_scratch(&plan, &[], &mut scratch);
        let dump = assemble_render_plan_json_dump(
            &plan,
            (64, 64),
            wgpu::TextureFormat::Rgba8Unorm,
            9,
            DebugPostprocess::None,
            6,
            &scratch,
        );

        assert_eq!(dump.segments.len(), 2);
        assert_eq!(dump.segments[0].pass_counts.scene_draw_range, 1);
        assert_eq!(dump.segments[0].pass_counts.path_msaa_batch, 1);
        assert_eq!(dump.segments[1].pass_counts.scene_draw_range, 1);
        assert_eq!(dump.segments[1].pass_counts.path_msaa_batch, 0);

        assert_eq!(dump.pass_counts.total, 8);
        assert_eq!(dump.pass_counts.scene, 2);
        assert_eq!(dump.pass_counts.path_msaa, 1);
        assert_eq!(dump.pass_counts.custom_effect_v1, 1);
        assert_eq!(dump.pass_counts.custom_effect_v2, 1);
        assert_eq!(dump.pass_counts.custom_effect_v3, 1);
        assert_eq!(dump.pass_counts.clip_mask, 1);
        assert_eq!(dump.pass_counts.release_target, 1);

        assert_eq!(dump.degradations.len(), 1);
        assert_eq!(dump.degradations[0].draw_ix, 5);
        assert_eq!(dump.degradations[0].kind, "BackdropEffectNoOp");
        assert_eq!(dump.degradations[0].reason, "BudgetZero");
    }
}
