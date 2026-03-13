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

        self.diagnostics_state
            .drain_pending_render_target_update_counters(frame_perf);
    }
}
