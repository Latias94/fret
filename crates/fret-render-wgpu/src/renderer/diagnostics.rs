use super::*;
use fret_render_core::RenderTargetIngestStrategy;

pub(super) struct DiagnosticsState {
    perf_enabled: bool,
    perf_pending_render_target_updates_requested_by_ingest:
        [u64; RenderTargetIngestStrategy::COUNT],
    perf_pending_render_target_updates_by_ingest: [u64; RenderTargetIngestStrategy::COUNT],
    perf_pending_render_target_updates_ingest_fallbacks: u64,
    perf_pending_render_target_metadata_degradations_color_encoding_dropped: u64,
    pub(super) perf: RenderPerfStats,
    pub(super) last_frame_perf: Option<RenderPerfSnapshot>,
    pub(super) last_render_plan_segment_report: Option<Vec<RenderPlanSegmentReport>>,
    render_scene_frame_index: u64,
}

impl Default for DiagnosticsState {
    fn default() -> Self {
        Self {
            perf_enabled: false,
            perf_pending_render_target_updates_requested_by_ingest: [0;
                RenderTargetIngestStrategy::COUNT],
            perf_pending_render_target_updates_by_ingest: [0; RenderTargetIngestStrategy::COUNT],
            perf_pending_render_target_updates_ingest_fallbacks: 0,
            perf_pending_render_target_metadata_degradations_color_encoding_dropped: 0,
            perf: RenderPerfStats::default(),
            last_frame_perf: None,
            last_render_plan_segment_report: None,
            render_scene_frame_index: 0,
        }
    }
}

impl DiagnosticsState {
    pub(super) fn perf_enabled(&self) -> bool {
        self.perf_enabled
    }

    pub(super) fn set_perf_enabled(&mut self, enabled: bool) {
        self.perf_enabled = enabled;
        self.perf = RenderPerfStats::default();
    }

    pub(super) fn next_render_scene_frame_index(&mut self) -> u64 {
        self.render_scene_frame_index = self.render_scene_frame_index.saturating_add(1);
        self.render_scene_frame_index
    }

    pub(super) fn note_render_target_metadata_degradation_color_encoding_dropped(&mut self) {
        if self.perf_enabled {
            self.perf_pending_render_target_metadata_degradations_color_encoding_dropped = self
                .perf_pending_render_target_metadata_degradations_color_encoding_dropped
                .saturating_add(1);
        }
    }

    pub(super) fn note_render_target_update(
        &mut self,
        requested: RenderTargetIngestStrategy,
        effective: RenderTargetIngestStrategy,
    ) {
        if !self.perf_enabled {
            return;
        }

        let effective_ix = render_target_ingest_strategy_perf_index(effective);
        self.perf_pending_render_target_updates_by_ingest[effective_ix] =
            self.perf_pending_render_target_updates_by_ingest[effective_ix].saturating_add(1);

        let requested_ix = render_target_ingest_strategy_perf_index(requested);
        self.perf_pending_render_target_updates_requested_by_ingest[requested_ix] = self
            .perf_pending_render_target_updates_requested_by_ingest[requested_ix]
            .saturating_add(1);

        if requested != effective {
            self.perf_pending_render_target_updates_ingest_fallbacks = self
                .perf_pending_render_target_updates_ingest_fallbacks
                .saturating_add(1);
        }
    }

    pub(super) fn drain_pending_render_target_update_counters(
        &mut self,
        frame_perf: &mut RenderPerfStats,
    ) {
        let pending_effective = self.perf_pending_render_target_updates_by_ingest;
        frame_perf.render_target_updates_ingest_unknown = pending_effective[0];
        frame_perf.render_target_updates_ingest_owned = pending_effective[1];
        frame_perf.render_target_updates_ingest_external_zero_copy = pending_effective[2];
        frame_perf.render_target_updates_ingest_gpu_copy = pending_effective[3];
        frame_perf.render_target_updates_ingest_cpu_upload = pending_effective[4];
        self.perf_pending_render_target_updates_by_ingest = [0; RenderTargetIngestStrategy::COUNT];

        let pending_requested = self.perf_pending_render_target_updates_requested_by_ingest;
        frame_perf.render_target_updates_requested_ingest_unknown = pending_requested[0];
        frame_perf.render_target_updates_requested_ingest_owned = pending_requested[1];
        frame_perf.render_target_updates_requested_ingest_external_zero_copy = pending_requested[2];
        frame_perf.render_target_updates_requested_ingest_gpu_copy = pending_requested[3];
        frame_perf.render_target_updates_requested_ingest_cpu_upload = pending_requested[4];
        self.perf_pending_render_target_updates_requested_by_ingest =
            [0; RenderTargetIngestStrategy::COUNT];

        frame_perf.render_target_updates_ingest_fallbacks =
            self.perf_pending_render_target_updates_ingest_fallbacks;
        self.perf_pending_render_target_updates_ingest_fallbacks = 0;

        frame_perf.render_target_metadata_degradations_color_encoding_dropped =
            self.perf_pending_render_target_metadata_degradations_color_encoding_dropped;
        self.perf_pending_render_target_metadata_degradations_color_encoding_dropped = 0;
    }

    pub(super) fn take_last_frame_perf_snapshot(&mut self) -> Option<RenderPerfSnapshot> {
        if !self.perf_enabled {
            return None;
        }

        self.last_frame_perf.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drains_pending_render_target_perf_counters_into_frame_perf() {
        let mut state = DiagnosticsState::default();
        state.set_perf_enabled(true);
        state.note_render_target_update(
            RenderTargetIngestStrategy::Owned,
            RenderTargetIngestStrategy::GpuCopy,
        );
        state.note_render_target_update(
            RenderTargetIngestStrategy::GpuCopy,
            RenderTargetIngestStrategy::GpuCopy,
        );
        state.note_render_target_metadata_degradation_color_encoding_dropped();

        let mut frame_perf = RenderPerfStats::default();
        state.drain_pending_render_target_update_counters(&mut frame_perf);

        assert_eq!(frame_perf.render_target_updates_ingest_owned, 0);
        assert_eq!(frame_perf.render_target_updates_ingest_gpu_copy, 2);
        assert_eq!(frame_perf.render_target_updates_requested_ingest_owned, 1);
        assert_eq!(
            frame_perf.render_target_updates_requested_ingest_gpu_copy,
            1
        );
        assert_eq!(frame_perf.render_target_updates_ingest_fallbacks, 1);
        assert_eq!(
            frame_perf.render_target_metadata_degradations_color_encoding_dropped,
            1
        );

        let mut next_frame_perf = RenderPerfStats::default();
        state.drain_pending_render_target_update_counters(&mut next_frame_perf);

        assert_eq!(next_frame_perf.render_target_updates_ingest_gpu_copy, 0);
        assert_eq!(
            next_frame_perf.render_target_updates_requested_ingest_owned,
            0
        );
        assert_eq!(next_frame_perf.render_target_updates_ingest_fallbacks, 0);
        assert_eq!(
            next_frame_perf.render_target_metadata_degradations_color_encoding_dropped,
            0
        );
    }
}
