use super::super::*;

impl Renderer {
    pub(super) fn ensure_frame_pipelines_and_path_samples(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        _viewport_size: (u32, u32),
        perf_enabled: bool,
        trace_enabled: bool,
        frame_perf: &mut RenderPerfStats,
    ) -> u32 {
        let (path_samples, ensure_elapsed) = fret_perf::measure_span_with(
            perf_enabled,
            trace_enabled,
            || {
                tracing::trace_span!(
                    "fret.renderer.ensure_pipelines",
                    format = ?format,
                    path_samples = tracing::field::Empty,
                )
            },
            |span| {
                self.ensure_material_catalog_uploaded(queue);
                self.ensure_mask_image_identity_uploaded(queue);
                self.ensure_custom_effect_input_fallback_uploaded(queue);

                self.ensure_viewport_pipeline(device, format);
                self.ensure_quad_pipelines(format);
                self.ensure_text_pipeline(device, format);
                self.ensure_text_color_pipeline(device, format);
                self.ensure_text_subpixel_pipeline(device, format);
                self.ensure_mask_pipeline(device, format);
                self.ensure_path_pipeline(device, format);
                self.ensure_path_clip_mask_pipeline(device);
                let path_samples = self
                    .render_scene_config_state
                    .effective_path_msaa_samples(&self.adapter, format);
                span.record("path_samples", path_samples);
                if path_samples > 1 {
                    self.ensure_composite_pipeline(device, format);
                    self.ensure_path_msaa_pipeline(device, format, path_samples);
                }
                path_samples
            },
        );
        if let Some(ensure_elapsed) = ensure_elapsed {
            frame_perf.ensure_pipelines += ensure_elapsed;
        }

        if perf_enabled {
            frame_perf.path_msaa_samples_requested =
                self.render_scene_config_state.path_msaa_samples();
            frame_perf.path_msaa_samples_effective = path_samples;
            if self.adapter.get_info().backend == wgpu::Backend::Vulkan
                && self.render_scene_config_state.path_msaa_samples() > 1
                && path_samples == 1
                && std::env::var_os("FRET_DISABLE_VULKAN_PATH_MSAA").is_some()
            {
                frame_perf.path_msaa_vulkan_safety_valve_degradations = frame_perf
                    .path_msaa_vulkan_safety_valve_degradations
                    .saturating_add(1);
            }
        }
        path_samples
    }
}
