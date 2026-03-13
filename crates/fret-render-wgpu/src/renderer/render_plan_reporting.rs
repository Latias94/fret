use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RenderPlanSegmentReport {
    pub(super) draw_range: (usize, usize),
    pub(super) start_uniform_fingerprint: u64,
    pub(super) flags_mask: u8,
    pub(super) scene_draw_range_passes: u32,
    pub(super) path_msaa_batch_passes: u32,
}

#[derive(Default)]
pub(super) struct RenderPlanReportingState {
    scene_draw_range_passes_scratch: Vec<u32>,
    path_msaa_batch_passes_scratch: Vec<u32>,
    segment_report_scratch: Vec<RenderPlanSegmentReport>,
    dump_scratch: render_plan_dump::RenderPlanJsonDumpScratch,
}

impl RenderPlanReportingState {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn record_render_plan_diagnostics_for_frame(
        &mut self,
        diagnostics_state: &mut DiagnosticsState,
        perf_enabled: bool,
        plan: &RenderPlan,
        viewport_size: (u32, u32),
        format: wgpu::TextureFormat,
        frame_index: u64,
        postprocess: DebugPostprocess,
        ordered_draws_len: usize,
        effect_markers: &[EffectMarker],
        frame_perf: &mut RenderPerfStats,
    ) {
        if perf_enabled {
            use super::render_plan::{
                RenderPlanDegradationKind as DegradationKind,
                RenderPlanDegradationReason as DegradationReason,
            };

            let mut custom_effect_v3_steps_requested: u64 = 0;
            let mut custom_effect_v2_steps_requested: u64 = 0;
            let mut custom_effect_v1_steps_requested: u64 = 0;
            for marker in effect_markers {
                let EffectMarkerKind::Push { chain, .. } = &marker.kind else {
                    continue;
                };
                for step in chain.iter() {
                    match step {
                        fret_core::EffectStep::CustomV1 { .. } => {
                            custom_effect_v1_steps_requested =
                                custom_effect_v1_steps_requested.saturating_add(1);
                        }
                        fret_core::EffectStep::CustomV2 { .. } => {
                            custom_effect_v2_steps_requested =
                                custom_effect_v2_steps_requested.saturating_add(1);
                        }
                        fret_core::EffectStep::CustomV3 { .. } => {
                            custom_effect_v3_steps_requested =
                                custom_effect_v3_steps_requested.saturating_add(1);
                        }
                        _ => {}
                    }
                }
            }
            let custom_effect_v1_passes_emitted = plan
                .passes
                .iter()
                .filter(|p| matches!(p, RenderPlanPass::CustomEffect(_)))
                .count() as u64;
            let custom_effect_v2_passes_emitted = plan
                .passes
                .iter()
                .filter(|p| matches!(p, RenderPlanPass::CustomEffectV2(_)))
                .count() as u64;
            let custom_effect_v3_passes_emitted = plan
                .passes
                .iter()
                .filter(|p| matches!(p, RenderPlanPass::CustomEffectV3(_)))
                .count() as u64;

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
            frame_perf.custom_effect_v1_steps_requested = custom_effect_v1_steps_requested;
            frame_perf.custom_effect_v1_passes_emitted = custom_effect_v1_passes_emitted;
            frame_perf.custom_effect_v2_steps_requested = custom_effect_v2_steps_requested;
            frame_perf.custom_effect_v2_passes_emitted = custom_effect_v2_passes_emitted;
            frame_perf.custom_effect_v3_steps_requested = custom_effect_v3_steps_requested;
            frame_perf.custom_effect_v3_passes_emitted = custom_effect_v3_passes_emitted;
            for d in &plan.degradations {
                match d.reason {
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

                match d.kind {
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
                        frame_perf.render_plan_degradations_composite_group_blend_to_over =
                            frame_perf
                                .render_plan_degradations_composite_group_blend_to_over
                                .saturating_add(1);
                    }
                }
            }

            self.rebuild_segment_report(plan);
            let (segments_changed, segments_passes_increased) = diff_segment_reports(
                diagnostics_state
                    .last_render_plan_segment_report
                    .as_deref()
                    .unwrap_or(&[]),
                &self.segment_report_scratch,
            );
            frame_perf.render_plan_segments_changed = segments_changed;
            frame_perf.render_plan_segments_passes_increased = segments_passes_increased;

            if let Some(prev) = diagnostics_state.last_render_plan_segment_report.as_mut() {
                std::mem::swap(prev, &mut self.segment_report_scratch);
            } else {
                diagnostics_state.last_render_plan_segment_report =
                    Some(std::mem::take(&mut self.segment_report_scratch));
            }
        }

        render_plan_dump::maybe_dump_render_plan_json(
            plan,
            viewport_size,
            format,
            frame_index,
            postprocess,
            ordered_draws_len,
            effect_markers,
            &mut self.dump_scratch,
        );
    }

    fn rebuild_segment_report(&mut self, plan: &RenderPlan) {
        let segments_len = plan.segments.len();
        self.scene_draw_range_passes_scratch.clear();
        self.scene_draw_range_passes_scratch.resize(segments_len, 0);
        self.path_msaa_batch_passes_scratch.clear();
        self.path_msaa_batch_passes_scratch.resize(segments_len, 0);
        for pass in &plan.passes {
            match pass {
                RenderPlanPass::SceneDrawRange(pass) => {
                    if let Some(count) =
                        self.scene_draw_range_passes_scratch.get_mut(pass.segment.0)
                    {
                        *count = count.saturating_add(1);
                    }
                }
                RenderPlanPass::PathMsaaBatch(pass) => {
                    if let Some(count) = self.path_msaa_batch_passes_scratch.get_mut(pass.segment.0)
                    {
                        *count = count.saturating_add(1);
                    }
                }
                _ => {}
            }
        }

        self.segment_report_scratch.clear();
        self.segment_report_scratch.reserve(plan.segments.len());
        for (ix, seg) in plan.segments.iter().enumerate() {
            let flags_mask = u8::from(seg.flags.has_quad)
                | (u8::from(seg.flags.has_viewport) << 1)
                | (u8::from(seg.flags.has_image) << 2)
                | (u8::from(seg.flags.has_mask) << 3)
                | (u8::from(seg.flags.has_text) << 4)
                | (u8::from(seg.flags.has_path) << 5);
            self.segment_report_scratch.push(RenderPlanSegmentReport {
                draw_range: (seg.draw_range.start, seg.draw_range.end),
                start_uniform_fingerprint: seg.start_uniform_fingerprint,
                flags_mask,
                scene_draw_range_passes: *self
                    .scene_draw_range_passes_scratch
                    .get(ix)
                    .unwrap_or(&0),
                path_msaa_batch_passes: *self.path_msaa_batch_passes_scratch.get(ix).unwrap_or(&0),
            });
        }
    }
}

fn diff_segment_reports(
    previous: &[RenderPlanSegmentReport],
    current: &[RenderPlanSegmentReport],
) -> (u64, u64) {
    if previous.len() != current.len() {
        return (current.len() as u64, 0);
    }

    let mut segments_changed = 0u64;
    let mut segments_passes_increased = 0u64;
    for (prev, cur) in previous.iter().zip(current.iter()) {
        if prev.draw_range != cur.draw_range
            || prev.start_uniform_fingerprint != cur.start_uniform_fingerprint
            || prev.flags_mask != cur.flags_mask
        {
            segments_changed = segments_changed.saturating_add(1);
        }

        let prev_passes = prev
            .scene_draw_range_passes
            .saturating_add(prev.path_msaa_batch_passes);
        let cur_passes = cur
            .scene_draw_range_passes
            .saturating_add(cur.path_msaa_batch_passes);
        if cur_passes > prev_passes {
            segments_passes_increased = segments_passes_increased.saturating_add(1);
        }
    }

    (segments_changed, segments_passes_increased)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(
        draw_range: (usize, usize),
        start_uniform_fingerprint: u64,
        flags_mask: u8,
        scene_draw_range_passes: u32,
        path_msaa_batch_passes: u32,
    ) -> RenderPlanSegmentReport {
        RenderPlanSegmentReport {
            draw_range,
            start_uniform_fingerprint,
            flags_mask,
            scene_draw_range_passes,
            path_msaa_batch_passes,
        }
    }

    #[test]
    fn diff_segment_reports_tracks_shape_changes_and_pass_growth() {
        let previous = [
            report((0, 4), 11, 0b000001, 1, 0),
            report((4, 8), 22, 0b000010, 1, 1),
        ];
        let current = [
            report((0, 4), 11, 0b000001, 2, 0),
            report((5, 9), 22, 0b000010, 1, 1),
        ];

        let (segments_changed, segments_passes_increased) =
            diff_segment_reports(&previous, &current);

        assert_eq!(segments_changed, 1);
        assert_eq!(segments_passes_increased, 1);
    }
}
