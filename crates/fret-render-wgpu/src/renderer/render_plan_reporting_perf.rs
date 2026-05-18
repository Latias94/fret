use super::render_plan::{
    RenderPlanDegradation, RenderPlanDegradationKind as DegradationKind,
    RenderPlanDegradationReason as DegradationReason, RenderPlanPass,
};
use super::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct RequestedCustomEffectStepCounts {
    v1: u64,
    v2: u64,
    v3: u64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EmittedCustomEffectPassCounts {
    v1: u64,
    v2: u64,
    v3: u64,
}

pub(super) fn record_render_plan_frame_perf(
    frame_perf: &mut RenderPerfStats,
    plan: &RenderPlan,
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    effect_markers: &[EffectMarker],
) {
    let requested_steps = count_requested_custom_effect_steps(effect_markers);
    let emitted_passes = count_emitted_custom_effect_passes(&plan.passes);

    frame_perf.render_plan_estimated_peak_intermediate_bytes =
        plan.compile_stats.estimated_peak_intermediate_bytes;
    frame_perf.render_plan_segments = plan.segments.len() as u64;
    frame_perf.render_plan_degradations = plan.degradations.len() as u64;
    frame_perf.render_plan_effect_chain_budget_samples =
        plan.compile_stats.effect_chain_budget_samples;
    frame_perf.render_plan_effect_chain_effective_budget_min_bytes =
        plan.compile_stats.effect_chain_effective_budget_min_bytes;
    frame_perf.render_plan_effect_chain_effective_budget_max_bytes =
        plan.compile_stats.effect_chain_effective_budget_max_bytes;
    frame_perf.render_plan_effect_chain_other_live_max_bytes =
        plan.compile_stats.effect_chain_other_live_max_bytes;
    frame_perf.render_plan_custom_effect_chain_budget_samples =
        plan.compile_stats.custom_effect_chain_budget_samples;
    frame_perf.render_plan_custom_effect_chain_effective_budget_min_bytes = plan
        .compile_stats
        .custom_effect_chain_effective_budget_min_bytes;
    frame_perf.render_plan_custom_effect_chain_effective_budget_max_bytes = plan
        .compile_stats
        .custom_effect_chain_effective_budget_max_bytes;
    frame_perf.render_plan_custom_effect_chain_other_live_max_bytes =
        plan.compile_stats.custom_effect_chain_other_live_max_bytes;
    frame_perf.render_plan_custom_effect_chain_base_required_max_bytes = plan
        .compile_stats
        .custom_effect_chain_base_required_max_bytes;
    frame_perf.render_plan_custom_effect_chain_optional_required_max_bytes = plan
        .compile_stats
        .custom_effect_chain_optional_required_max_bytes;
    frame_perf.render_plan_custom_effect_chain_base_required_full_targets_max = plan
        .compile_stats
        .custom_effect_chain_base_required_full_targets_max;
    frame_perf.render_plan_custom_effect_chain_optional_mask_max_bytes = plan
        .compile_stats
        .custom_effect_chain_optional_mask_max_bytes;
    frame_perf.render_plan_custom_effect_chain_optional_pyramid_max_bytes = plan
        .compile_stats
        .custom_effect_chain_optional_pyramid_max_bytes;
    frame_perf.effect_degradations = plan.compile_stats.effect_degradations;
    frame_perf.effect_blur_quality = plan.compile_stats.effect_blur_quality;
    frame_perf.intermediate_full_target_bytes =
        crate::renderer::estimate_texture_bytes(viewport_size, format, 1);
    frame_perf.custom_effect_v1_steps_requested = requested_steps.v1;
    frame_perf.custom_effect_v1_passes_emitted = emitted_passes.v1;
    frame_perf.custom_effect_v2_steps_requested = requested_steps.v2;
    frame_perf.custom_effect_v2_passes_emitted = emitted_passes.v2;
    frame_perf.custom_effect_v3_steps_requested = requested_steps.v3;
    frame_perf.custom_effect_v3_passes_emitted = emitted_passes.v3;

    accumulate_render_plan_degradation_counters(frame_perf, &plan.degradations);
}

fn count_requested_custom_effect_steps(
    effect_markers: &[EffectMarker],
) -> RequestedCustomEffectStepCounts {
    let mut counts = RequestedCustomEffectStepCounts::default();
    for marker in effect_markers {
        let EffectMarkerKind::Push { chain, .. } = &marker.kind else {
            continue;
        };
        for step in chain.iter() {
            match step {
                fret_core::EffectStep::CustomV1 { .. } => {
                    counts.v1 = counts.v1.saturating_add(1);
                }
                fret_core::EffectStep::CustomV2 { .. } => {
                    counts.v2 = counts.v2.saturating_add(1);
                }
                fret_core::EffectStep::CustomV3 { .. } => {
                    counts.v3 = counts.v3.saturating_add(1);
                }
                _ => {}
            }
        }
    }
    counts
}

fn count_emitted_custom_effect_passes(passes: &[RenderPlanPass]) -> EmittedCustomEffectPassCounts {
    let mut counts = EmittedCustomEffectPassCounts::default();
    for pass in passes {
        match pass {
            RenderPlanPass::CustomEffect(_) => {
                counts.v1 = counts.v1.saturating_add(1);
            }
            RenderPlanPass::CustomEffectV2(_) => {
                counts.v2 = counts.v2.saturating_add(1);
            }
            RenderPlanPass::CustomEffectV3(_) => {
                counts.v3 = counts.v3.saturating_add(1);
            }
            _ => {}
        }
    }
    counts
}

fn accumulate_render_plan_degradation_counters(
    frame_perf: &mut RenderPerfStats,
    degradations: &[RenderPlanDegradation],
) {
    for degradation in degradations {
        match degradation.reason {
            DegradationReason::BudgetZero => {
                frame_perf.render_plan_degradations_budget_zero = frame_perf
                    .render_plan_degradations_budget_zero
                    .saturating_add(1);
            }
            DegradationReason::BudgetInsufficient => {
                frame_perf.render_plan_degradations_budget_insufficient = frame_perf
                    .render_plan_degradations_budget_insufficient
                    .saturating_add(1);
            }
            DegradationReason::TargetExhausted => {
                frame_perf.render_plan_degradations_target_exhausted = frame_perf
                    .render_plan_degradations_target_exhausted
                    .saturating_add(1);
            }
        }

        match degradation.kind {
            DegradationKind::BackdropEffectNoOp => {
                frame_perf.render_plan_degradations_backdrop_noop = frame_perf
                    .render_plan_degradations_backdrop_noop
                    .saturating_add(1);
            }
            DegradationKind::FilterContentDisabled => {
                frame_perf.render_plan_degradations_filter_content_disabled = frame_perf
                    .render_plan_degradations_filter_content_disabled
                    .saturating_add(1);
            }
            DegradationKind::ClipPathDisabled => {
                frame_perf.render_plan_degradations_clip_path_disabled = frame_perf
                    .render_plan_degradations_clip_path_disabled
                    .saturating_add(1);
            }
            DegradationKind::CompositeGroupBlendDegradedToOver => {
                frame_perf.render_plan_degradations_composite_group_blend_to_over = frame_perf
                    .render_plan_degradations_composite_group_blend_to_over
                    .saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn requested_and_emitted_custom_effect_counters_track_all_versions() {
        let effect_markers = [
            EffectMarker {
                draw_ix: 0,
                kind: EffectMarkerKind::Push {
                    scissor: ScissorRect::full(64, 64),
                    uniform_index: 0,
                    mode: fret_core::EffectMode::FilterContent,
                    chain: fret_core::EffectChain::from_steps(&[
                        fret_core::EffectStep::CustomV1 {
                            id: fret_core::EffectId::default(),
                            params: fret_core::scene::EffectParamsV1::ZERO,
                            max_sample_offset_px: fret_core::Px(0.0),
                        },
                        fret_core::EffectStep::CustomV2 {
                            id: fret_core::EffectId::default(),
                            params: fret_core::scene::EffectParamsV1::ZERO,
                            max_sample_offset_px: fret_core::Px(0.0),
                            input_image: None,
                        },
                        fret_core::EffectStep::CustomV3 {
                            id: fret_core::EffectId::default(),
                            params: fret_core::scene::EffectParamsV1::ZERO,
                            max_sample_offset_px: fret_core::Px(0.0),
                            user0: None,
                            user1: None,
                            sources: fret_core::scene::CustomEffectSourcesV3 {
                                want_raw: false,
                                pyramid: None,
                            },
                        },
                        fret_core::EffectStep::CustomV3 {
                            id: fret_core::EffectId::default(),
                            params: fret_core::scene::EffectParamsV1::ZERO,
                            max_sample_offset_px: fret_core::Px(0.0),
                            user0: None,
                            user1: None,
                            sources: fret_core::scene::CustomEffectSourcesV3 {
                                want_raw: true,
                                pyramid: Some(fret_core::scene::CustomEffectPyramidRequestV1 {
                                    max_levels: 4,
                                    max_radius_px: fret_core::Px(12.0),
                                }),
                            },
                        },
                    ]),
                    quality: fret_core::EffectQuality::Auto,
                },
            },
            EffectMarker {
                draw_ix: 1,
                kind: EffectMarkerKind::Pop,
            },
        ];

        let requested = count_requested_custom_effect_steps(&effect_markers);
        assert_eq!(
            requested,
            RequestedCustomEffectStepCounts {
                v1: 1,
                v2: 1,
                v3: 2,
            }
        );

        let passes = [
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
                pyramid_levels: 3,
                pyramid_build_scissor: None,
                raw_wanted: true,
                pyramid_wanted: true,
                common: custom_effect_common(PlanTarget::Output),
                user0_image: Some(fret_core::ImageId::default()),
                user0_uv: fret_core::scene::UvRect::FULL,
                user0_sampling: fret_core::scene::ImageSamplingHint::Default,
                user1_image: None,
                user1_uv: fret_core::scene::UvRect::FULL,
                user1_sampling: fret_core::scene::ImageSamplingHint::Default,
            }),
            RenderPlanPass::CustomEffectV3(CustomEffectV3Pass {
                src_raw: PlanTarget::Intermediate0,
                src_pyramid: PlanTarget::Intermediate2,
                pyramid_levels: 1,
                pyramid_build_scissor: None,
                raw_wanted: false,
                pyramid_wanted: false,
                common: custom_effect_common(PlanTarget::Intermediate3),
                user0_image: None,
                user0_uv: fret_core::scene::UvRect::FULL,
                user0_sampling: fret_core::scene::ImageSamplingHint::Default,
                user1_image: None,
                user1_uv: fret_core::scene::UvRect::FULL,
                user1_sampling: fret_core::scene::ImageSamplingHint::Default,
            }),
            RenderPlanPass::ReleaseTarget(PlanTarget::Intermediate1),
        ];

        let emitted = count_emitted_custom_effect_passes(&passes);
        assert_eq!(
            emitted,
            EmittedCustomEffectPassCounts {
                v1: 1,
                v2: 1,
                v3: 2
            }
        );
    }

    #[test]
    fn degradation_counters_track_reason_and_kind_totals() {
        let degradations = [
            RenderPlanDegradation {
                draw_ix: 1,
                kind: DegradationKind::BackdropEffectNoOp,
                reason: DegradationReason::BudgetZero,
            },
            RenderPlanDegradation {
                draw_ix: 2,
                kind: DegradationKind::FilterContentDisabled,
                reason: DegradationReason::BudgetInsufficient,
            },
            RenderPlanDegradation {
                draw_ix: 3,
                kind: DegradationKind::ClipPathDisabled,
                reason: DegradationReason::TargetExhausted,
            },
            RenderPlanDegradation {
                draw_ix: 4,
                kind: DegradationKind::CompositeGroupBlendDegradedToOver,
                reason: DegradationReason::BudgetZero,
            },
        ];

        let mut frame_perf = RenderPerfStats::default();
        accumulate_render_plan_degradation_counters(&mut frame_perf, &degradations);

        assert_eq!(frame_perf.render_plan_degradations_budget_zero, 2);
        assert_eq!(frame_perf.render_plan_degradations_budget_insufficient, 1);
        assert_eq!(frame_perf.render_plan_degradations_target_exhausted, 1);
        assert_eq!(frame_perf.render_plan_degradations_backdrop_noop, 1);
        assert_eq!(
            frame_perf.render_plan_degradations_filter_content_disabled,
            1
        );
        assert_eq!(frame_perf.render_plan_degradations_clip_path_disabled, 1);
        assert_eq!(
            frame_perf.render_plan_degradations_composite_group_blend_to_over,
            1
        );
    }
}
