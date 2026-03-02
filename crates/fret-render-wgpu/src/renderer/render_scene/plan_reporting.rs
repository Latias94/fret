use super::super::*;

impl Renderer {
    pub(super) fn record_render_plan_diagnostics_for_frame(
        &mut self,
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
            use super::super::render_plan::{
                RenderPlanDegradationKind as DegradationKind,
                RenderPlanDegradationReason as DegradationReason,
            };

            let mut custom_effect_v3_steps_requested: u64 = 0;
            for marker in effect_markers {
                let EffectMarkerKind::Push { chain, .. } = &marker.kind else {
                    continue;
                };
                for step in chain.iter() {
                    if matches!(step, fret_core::EffectStep::CustomV3 { .. }) {
                        custom_effect_v3_steps_requested =
                            custom_effect_v3_steps_requested.saturating_add(1);
                    }
                }
            }
            let custom_effect_v3_passes_emitted = plan
                .passes
                .iter()
                .filter(|p| matches!(p, RenderPlanPass::CustomEffectV3(_)))
                .count() as u64;

            frame_perf.render_plan_estimated_peak_intermediate_bytes =
                plan.compile_stats.estimated_peak_intermediate_bytes;
            frame_perf.render_plan_segments = plan.segments.len() as u64;
            frame_perf.render_plan_degradations = plan.degradations.len() as u64;
            frame_perf.effect_degradations = plan.compile_stats.effect_degradations;
            frame_perf.effect_blur_quality = plan.compile_stats.effect_blur_quality;
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

            let segments_len = plan.segments.len();
            self.render_plan_scene_draw_range_passes_scratch.clear();
            self.render_plan_scene_draw_range_passes_scratch
                .resize(segments_len, 0);
            self.render_plan_path_msaa_batch_passes_scratch.clear();
            self.render_plan_path_msaa_batch_passes_scratch
                .resize(segments_len, 0);
            for p in &plan.passes {
                match p {
                    RenderPlanPass::SceneDrawRange(p) => {
                        if let Some(c) = self
                            .render_plan_scene_draw_range_passes_scratch
                            .get_mut(p.segment.0)
                        {
                            *c = c.saturating_add(1);
                        }
                    }
                    RenderPlanPass::PathMsaaBatch(p) => {
                        if let Some(c) = self
                            .render_plan_path_msaa_batch_passes_scratch
                            .get_mut(p.segment.0)
                        {
                            *c = c.saturating_add(1);
                        }
                    }
                    _ => {}
                }
            }

            self.render_plan_segment_report_scratch.clear();
            self.render_plan_segment_report_scratch
                .reserve(plan.segments.len());
            for (ix, seg) in plan.segments.iter().enumerate() {
                let flags_mask = u8::from(seg.flags.has_quad)
                    | (u8::from(seg.flags.has_viewport) << 1)
                    | (u8::from(seg.flags.has_image) << 2)
                    | (u8::from(seg.flags.has_mask) << 3)
                    | (u8::from(seg.flags.has_text) << 4)
                    | (u8::from(seg.flags.has_path) << 5);
                self.render_plan_segment_report_scratch
                    .push(RenderPlanSegmentReport {
                        draw_range: (seg.draw_range.start, seg.draw_range.end),
                        start_uniform_fingerprint: seg.start_uniform_fingerprint,
                        flags_mask,
                        scene_draw_range_passes: *self
                            .render_plan_scene_draw_range_passes_scratch
                            .get(ix)
                            .unwrap_or(&0),
                        path_msaa_batch_passes: *self
                            .render_plan_path_msaa_batch_passes_scratch
                            .get(ix)
                            .unwrap_or(&0),
                    });
            }

            let mut segments_changed: u64 = 0;
            let mut segments_passes_increased: u64 = 0;
            if let Some(prev) = &self.last_render_plan_segment_report {
                if prev.len() != self.render_plan_segment_report_scratch.len() {
                    segments_changed = self.render_plan_segment_report_scratch.len() as u64;
                } else {
                    for (p, c) in prev
                        .iter()
                        .zip(self.render_plan_segment_report_scratch.iter())
                    {
                        if p.draw_range != c.draw_range
                            || p.start_uniform_fingerprint != c.start_uniform_fingerprint
                            || p.flags_mask != c.flags_mask
                        {
                            segments_changed = segments_changed.saturating_add(1);
                        }

                        let prev_passes = p
                            .scene_draw_range_passes
                            .saturating_add(p.path_msaa_batch_passes);
                        let cur_passes = c
                            .scene_draw_range_passes
                            .saturating_add(c.path_msaa_batch_passes);
                        if cur_passes > prev_passes {
                            segments_passes_increased = segments_passes_increased.saturating_add(1);
                        }
                    }
                }
            }
            frame_perf.render_plan_segments_changed = segments_changed;
            frame_perf.render_plan_segments_passes_increased = segments_passes_increased;
            if let Some(prev) = self.last_render_plan_segment_report.as_mut() {
                std::mem::swap(prev, &mut self.render_plan_segment_report_scratch);
            } else {
                self.last_render_plan_segment_report =
                    Some(std::mem::take(&mut self.render_plan_segment_report_scratch));
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
            &mut self.render_plan_dump_scratch,
        );
    }
}
