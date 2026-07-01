use super::super::*;

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
        let (text_atlas_revision, prepare_elapsed) = fret_perf::measure_span(
            perf_enabled,
            trace_enabled,
            || tracing::trace_span!("fret.renderer.text.prepare", frame_index),
            || {
                let mut text_prepare_perf =
                    self.text_system
                        .prepare_for_scene_with_perf(scene, frame_index, perf_enabled);
                let flush_start = perf_enabled.then(std::time::Instant::now);
                self.text_system.flush_uploads(queue);
                if let Some(start) = flush_start {
                    text_prepare_perf.flush_uploads += start.elapsed();
                }
                let text_atlas_revision = self.text_system.atlas_revision();
                if perf_enabled {
                    frame_perf.record_text_prepare_scene_perf(text_prepare_perf);
                    let scene_resource_snapshot =
                        self.text_system.scene_text_resource_snapshot(scene);
                    let scene_resource_observation = self
                        .text_scene_resource_key_state
                        .observe(text_atlas_revision, scene_resource_snapshot.fingerprint);
                    frame_perf.record_text_scene_resource_snapshot(
                        scene_resource_snapshot,
                        scene_resource_observation,
                    );
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
                text_atlas_revision
            },
        );
        if let Some(prepare_elapsed) = prepare_elapsed {
            frame_perf.prepare_text += prepare_elapsed;
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

        let svg_perf_enabled = self.svg_raster_state.perf_enabled;
        let (_, prepare_elapsed) = fret_perf::measure_span(
            perf_enabled || svg_perf_enabled,
            trace_enabled,
            || tracing::trace_span!("fret.renderer.svg.prepare_ops", frame_index),
            || {
                self.prepare_svg_ops(device, queue, scene, scale_factor);
                self.svg_raster_state.commit_text_bridge_diagnostics_frame();
                if perf_enabled {
                    let counters = crate::upload_counters::take_upload_counters();
                    frame_perf.svg_uploads =
                        frame_perf.svg_uploads.saturating_add(counters.svg_uploads);
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
            },
        );
        if let Some(prepare_elapsed) = prepare_elapsed {
            if svg_perf_enabled {
                self.svg_raster_state.perf.prepare_svg_ops += prepare_elapsed;
            }
            if perf_enabled {
                frame_perf.prepare_svg += prepare_elapsed;
            }
        }
    }
}
