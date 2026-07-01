use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RenderPlanSegmentReport {
    pub(super) draw_range: (usize, usize),
    pub(super) start_uniform_fingerprint: u64,
    pub(super) flags_mask: u8,
    pub(super) scene_chunk_candidate_eligible: bool,
    pub(super) scene_chunk_candidate_draw_count: u32,
    pub(super) scene_chunk_candidate_fingerprint: u64,
    pub(super) scene_chunk_candidate_upload_bytes_estimate: u64,
    pub(super) stream_ranges: RenderPlanSegmentStreamRanges,
    pub(super) scene_draw_range_passes: u32,
    pub(super) path_msaa_batch_passes: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RenderPlanSegmentReportDiff {
    segments_changed: u64,
    segments_passes_increased: u64,
    scene_chunk_candidates: u64,
    scene_chunk_candidate_draws: u64,
    scene_chunk_candidates_stable: u64,
    scene_chunk_candidates_changed: u64,
    scene_chunk_candidate_upload_bytes_estimate: u64,
    scene_chunk_candidate_stream_ranges_changed: u64,
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
            render_plan_reporting_perf::record_render_plan_frame_perf(
                frame_perf,
                plan,
                viewport_size,
                format,
                effect_markers,
            );

            self.rebuild_segment_report(plan);
            let diff = diff_segment_reports(
                diagnostics_state
                    .last_render_plan_segment_report
                    .as_deref()
                    .unwrap_or(&[]),
                &self.segment_report_scratch,
            );
            frame_perf.render_plan_segments_changed = diff.segments_changed;
            frame_perf.render_plan_segments_passes_increased = diff.segments_passes_increased;
            frame_perf.render_plan_scene_chunk_candidates = diff.scene_chunk_candidates;
            frame_perf.render_plan_scene_chunk_candidate_draws = diff.scene_chunk_candidate_draws;
            frame_perf.render_plan_scene_chunk_candidates_stable =
                diff.scene_chunk_candidates_stable;
            frame_perf.render_plan_scene_chunk_candidates_changed =
                diff.scene_chunk_candidates_changed;
            frame_perf.render_plan_scene_chunk_candidate_upload_bytes_estimate =
                diff.scene_chunk_candidate_upload_bytes_estimate;
            frame_perf.render_plan_scene_chunk_candidate_stream_ranges_changed =
                diff.scene_chunk_candidate_stream_ranges_changed;

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
            self.segment_report_scratch.push(RenderPlanSegmentReport {
                draw_range: (seg.draw_range.start, seg.draw_range.end),
                start_uniform_fingerprint: seg.start_uniform_fingerprint,
                flags_mask: seg.flags.diagnostics_mask(),
                scene_chunk_candidate_eligible: seg.scene_chunk_candidate.eligible,
                scene_chunk_candidate_draw_count: seg.scene_chunk_candidate.draw_count,
                scene_chunk_candidate_fingerprint: seg.scene_chunk_candidate.fingerprint,
                scene_chunk_candidate_upload_bytes_estimate: seg
                    .stream_ranges
                    .estimated_upload_bytes(),
                stream_ranges: seg.stream_ranges,
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
) -> RenderPlanSegmentReportDiff {
    let mut diff = RenderPlanSegmentReportDiff::default();
    diff.scene_chunk_candidates = current
        .iter()
        .filter(|report| report.scene_chunk_candidate_eligible)
        .count() as u64;
    diff.scene_chunk_candidate_draws = current
        .iter()
        .filter(|report| report.scene_chunk_candidate_eligible)
        .map(|report| u64::from(report.scene_chunk_candidate_draw_count))
        .sum();
    diff.scene_chunk_candidate_upload_bytes_estimate = current
        .iter()
        .filter(|report| report.scene_chunk_candidate_eligible)
        .map(|report| report.scene_chunk_candidate_upload_bytes_estimate)
        .sum();

    if previous.len() != current.len() {
        diff.segments_changed = current.len() as u64;
        diff.scene_chunk_candidates_changed = diff.scene_chunk_candidates;
        diff.scene_chunk_candidate_stream_ranges_changed = diff.scene_chunk_candidates;
        return diff;
    }

    for (prev, cur) in previous.iter().zip(current.iter()) {
        if prev.draw_range != cur.draw_range
            || prev.start_uniform_fingerprint != cur.start_uniform_fingerprint
            || prev.flags_mask != cur.flags_mask
        {
            diff.segments_changed = diff.segments_changed.saturating_add(1);
        }

        let prev_passes = prev
            .scene_draw_range_passes
            .saturating_add(prev.path_msaa_batch_passes);
        let cur_passes = cur
            .scene_draw_range_passes
            .saturating_add(cur.path_msaa_batch_passes);
        if cur_passes > prev_passes {
            diff.segments_passes_increased = diff.segments_passes_increased.saturating_add(1);
        }

        if cur.scene_chunk_candidate_eligible {
            if prev.scene_chunk_candidate_eligible
                && prev.scene_chunk_candidate_draw_count == cur.scene_chunk_candidate_draw_count
                && prev.scene_chunk_candidate_fingerprint == cur.scene_chunk_candidate_fingerprint
            {
                diff.scene_chunk_candidates_stable =
                    diff.scene_chunk_candidates_stable.saturating_add(1);
            } else {
                diff.scene_chunk_candidates_changed =
                    diff.scene_chunk_candidates_changed.saturating_add(1);
            }

            if !prev.scene_chunk_candidate_eligible || prev.stream_ranges != cur.stream_ranges {
                diff.scene_chunk_candidate_stream_ranges_changed = diff
                    .scene_chunk_candidate_stream_ranges_changed
                    .saturating_add(1);
            }
        }
    }

    diff
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(
        draw_range: (usize, usize),
        start_uniform_fingerprint: u64,
        flags_mask: u8,
        scene_chunk_candidate_eligible: bool,
        scene_chunk_candidate_draw_count: u32,
        scene_chunk_candidate_fingerprint: u64,
        stream_ranges: RenderPlanSegmentStreamRanges,
        scene_draw_range_passes: u32,
        path_msaa_batch_passes: u32,
    ) -> RenderPlanSegmentReport {
        RenderPlanSegmentReport {
            draw_range,
            start_uniform_fingerprint,
            flags_mask,
            scene_chunk_candidate_eligible,
            scene_chunk_candidate_draw_count,
            scene_chunk_candidate_fingerprint,
            scene_chunk_candidate_upload_bytes_estimate: stream_ranges.estimated_upload_bytes(),
            stream_ranges,
            scene_draw_range_passes,
            path_msaa_batch_passes,
        }
    }

    fn quad_ranges(start: u32, end: u32) -> RenderPlanSegmentStreamRanges {
        RenderPlanSegmentStreamRanges {
            quad_instances: RenderPlanStreamRange::new(start, end),
            ..Default::default()
        }
    }

    #[test]
    fn diff_segment_reports_tracks_shape_changes_and_pass_growth() {
        let first = quad_ranges(0, 4);
        let second = quad_ranges(4, 8);
        let moved_second = quad_ranges(5, 9);
        let previous = [
            report((0, 4), 11, 0b000001, true, 4, 0xA11C, first, 1, 0),
            report((4, 8), 22, 0b000010, true, 4, 0xB22D, second, 1, 1),
        ];
        let current = [
            report((0, 4), 11, 0b000001, true, 4, 0xA11C, first, 2, 0),
            report((5, 9), 22, 0b000010, true, 4, 0xC33E, moved_second, 1, 1),
        ];

        let diff = diff_segment_reports(&previous, &current);

        assert_eq!(diff.segments_changed, 1);
        assert_eq!(diff.segments_passes_increased, 1);
        assert_eq!(diff.scene_chunk_candidates, 2);
        assert_eq!(diff.scene_chunk_candidate_draws, 8);
        assert_eq!(diff.scene_chunk_candidates_stable, 1);
        assert_eq!(diff.scene_chunk_candidates_changed, 1);
        assert_eq!(
            diff.scene_chunk_candidate_upload_bytes_estimate,
            first
                .estimated_upload_bytes()
                .saturating_add(moved_second.estimated_upload_bytes())
        );
        assert_eq!(diff.scene_chunk_candidate_stream_ranges_changed, 1);
    }

    #[test]
    fn diff_segment_reports_treats_new_shape_candidates_as_changed() {
        let current = [
            report(
                (0, 2),
                11,
                0b000001,
                true,
                2,
                0xA11C,
                quad_ranges(0, 2),
                1,
                0,
            ),
            report(
                (2, 2),
                0,
                0,
                false,
                0,
                0,
                RenderPlanSegmentStreamRanges::default(),
                1,
                0,
            ),
        ];

        let diff = diff_segment_reports(&[], &current);

        assert_eq!(diff.segments_changed, 2);
        assert_eq!(diff.segments_passes_increased, 0);
        assert_eq!(diff.scene_chunk_candidates, 1);
        assert_eq!(diff.scene_chunk_candidate_draws, 2);
        assert_eq!(diff.scene_chunk_candidates_stable, 0);
        assert_eq!(diff.scene_chunk_candidates_changed, 1);
        assert_eq!(diff.scene_chunk_candidate_stream_ranges_changed, 1);
    }
}
