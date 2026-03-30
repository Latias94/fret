use super::super::*;
use fret_core::time::Instant;

impl Renderer {
    pub(super) fn prepare_text_for_frame(
        &mut self,
        queue: &wgpu::Queue,
        scene: &Scene,
        frame_index: u64,
        perf_enabled: bool,
        trace_enabled: bool,
        frame_perf: &mut RenderPerfStats,
    ) -> u64 {
        let text_prepare_start = perf_enabled.then(Instant::now);
        {
            let text_prepare_span = if trace_enabled {
                tracing::trace_span!("fret.renderer.text.prepare", frame_index)
            } else {
                tracing::Span::none()
            };
            let _guard = text_prepare_span.enter();
            self.text_system.prepare_for_scene(scene, frame_index);
            self.text_system.flush_uploads(queue);
        }
        let text_atlas_revision = self.text_system.atlas_revision();
        if perf_enabled {
            let atlas_perf = self.text_system.take_atlas_perf_snapshot();
            frame_perf.text_atlas_revision = text_atlas_revision;
            frame_perf.text_atlas_uploads = atlas_perf.uploads;
            frame_perf.text_atlas_upload_bytes = atlas_perf.upload_bytes;
            frame_perf.text_atlas_evicted_glyphs = atlas_perf.evicted_glyphs;
            frame_perf.text_atlas_evicted_pages = atlas_perf.evicted_pages;
            frame_perf.text_atlas_evicted_page_glyphs = atlas_perf.evicted_page_glyphs;
            frame_perf.text_atlas_resets = atlas_perf.resets;
            frame_perf.intermediate_budget_bytes = self.intermediate_state.budget_bytes;
        }
        if let Some(text_prepare_start) = text_prepare_start {
            frame_perf.prepare_text += text_prepare_start.elapsed();
        }
        text_atlas_revision
    }

    pub(super) fn prepare_svg_for_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &Scene,
        scale_factor: f32,
        frame_index: u64,
        perf_enabled: bool,
        trace_enabled: bool,
        frame_perf: &mut RenderPerfStats,
    ) {
        if self.svg_raster_state.perf_enabled {
            self.svg_raster_state.perf.frames = self.svg_raster_state.perf.frames.saturating_add(1);
        }
        self.intermediate_state.record_frame();
        self.bump_svg_raster_epoch();
        self.svg_raster_state.begin_text_bridge_diagnostics_frame();

        let svg_prepare_start = self.svg_raster_state.perf_enabled.then(Instant::now);
        let perf_svg_prepare_start = perf_enabled.then(Instant::now);
        {
            let svg_prepare_span = if trace_enabled {
                tracing::trace_span!("fret.renderer.svg.prepare_ops", frame_index)
            } else {
                tracing::Span::none()
            };
            let _guard = svg_prepare_span.enter();
            self.prepare_svg_ops(device, queue, scene, scale_factor);
        }
        self.svg_raster_state.commit_text_bridge_diagnostics_frame();
        if perf_enabled {
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
        }
        if let Some(svg_prepare_start) = svg_prepare_start {
            self.svg_raster_state.perf.prepare_svg_ops += svg_prepare_start.elapsed();
        }
        if let Some(perf_svg_prepare_start) = perf_svg_prepare_start {
            frame_perf.prepare_svg += perf_svg_prepare_start.elapsed();
        }
    }
}
