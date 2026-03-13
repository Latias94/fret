use super::super::*;

impl Renderer {
    pub(super) fn begin_frame_perf_collection(&mut self, frame_perf: &mut RenderPerfStats) {
        frame_perf.frames = 1;
        self.svg_raster_state.reset_frame_perf_counters();

        let counters = crate::upload_counters::take_upload_counters();
        frame_perf.svg_uploads = frame_perf.svg_uploads.saturating_add(counters.svg_uploads);
        frame_perf.svg_upload_bytes = frame_perf
            .svg_upload_bytes
            .saturating_add(counters.svg_upload_bytes);
        frame_perf.image_uploads = frame_perf
            .image_uploads
            .saturating_add(counters.image_uploads);
        frame_perf.image_upload_bytes = frame_perf
            .image_upload_bytes
            .saturating_add(counters.image_upload_bytes);

        let pending_effective = self.perf_pending_render_target_updates_by_ingest;
        frame_perf.render_target_updates_ingest_unknown = pending_effective[0];
        frame_perf.render_target_updates_ingest_owned = pending_effective[1];
        frame_perf.render_target_updates_ingest_external_zero_copy = pending_effective[2];
        frame_perf.render_target_updates_ingest_gpu_copy = pending_effective[3];
        frame_perf.render_target_updates_ingest_cpu_upload = pending_effective[4];
        self.perf_pending_render_target_updates_by_ingest =
            [0; fret_render_core::RenderTargetIngestStrategy::COUNT];

        let pending_requested = self.perf_pending_render_target_updates_requested_by_ingest;
        frame_perf.render_target_updates_requested_ingest_unknown = pending_requested[0];
        frame_perf.render_target_updates_requested_ingest_owned = pending_requested[1];
        frame_perf.render_target_updates_requested_ingest_external_zero_copy = pending_requested[2];
        frame_perf.render_target_updates_requested_ingest_gpu_copy = pending_requested[3];
        frame_perf.render_target_updates_requested_ingest_cpu_upload = pending_requested[4];
        self.perf_pending_render_target_updates_requested_by_ingest =
            [0; fret_render_core::RenderTargetIngestStrategy::COUNT];

        frame_perf.render_target_updates_ingest_fallbacks =
            self.perf_pending_render_target_updates_ingest_fallbacks;
        self.perf_pending_render_target_updates_ingest_fallbacks = 0;

        frame_perf.render_target_metadata_degradations_color_encoding_dropped =
            self.perf_pending_render_target_metadata_degradations_color_encoding_dropped;
        self.perf_pending_render_target_metadata_degradations_color_encoding_dropped = 0;
    }
}
