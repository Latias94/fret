use super::*;
use crate::text::{TextFontFamilyConfig, TextQualitySettings};
use crate::{SystemFontRescanResult, SystemFontRescanSeed};
use std::sync::OnceLock;

impl Renderer {
    pub fn begin_text_diagnostics_frame(&mut self) {
        self.text_system.begin_frame_diagnostics();
    }

    pub fn text_font_trace_snapshot(
        &self,
        frame_id: fret_core::FrameId,
    ) -> fret_core::RendererTextFontTraceSnapshot {
        self.text_system.font_trace_snapshot(frame_id)
    }

    pub fn text_fallback_policy_snapshot(
        &self,
        frame_id: fret_core::FrameId,
    ) -> fret_core::RendererTextFallbackPolicySnapshot {
        self.text_system.fallback_policy_snapshot(frame_id)
    }

    pub fn text_diagnostics_snapshot(
        &self,
        frame_id: fret_core::FrameId,
    ) -> fret_core::RendererTextPerfSnapshot {
        self.text_system.diagnostics_snapshot(frame_id)
    }

    pub fn set_perf_enabled(&mut self, enabled: bool) {
        self.diagnostics_state.set_perf_enabled(enabled);
    }

    pub fn take_perf_snapshot(&mut self) -> Option<RenderPerfSnapshot> {
        if !self.diagnostics_state.perf_enabled() {
            return None;
        }

        let registry_est = self.gpu_resources.diagnostics_estimated_bytes();
        let path_intermediate_msaa_bytes_estimate = self
            .path_intermediate
            .as_ref()
            .map_or(0, PathIntermediate::estimated_msaa_bytes);
        let path_intermediate_resolved_bytes_estimate = self
            .path_intermediate
            .as_ref()
            .map_or(0, PathIntermediate::estimated_resolved_bytes);
        let path_intermediate_bytes_estimate = self
            .path_intermediate
            .as_ref()
            .map_or(0, PathIntermediate::estimated_bytes);
        let custom_effect_v3_pyramid_scratch_bytes_estimate =
            self.custom_effect_v3_pyramid.scratch_bytes_estimate();
        let perf = &self.diagnostics_state.perf;

        let snap = RenderPerfSnapshot {
            frames: perf.frames,
            encode_scene_us: perf.encode_scene.as_micros() as u64,
            ensure_pipelines_us: perf.ensure_pipelines.as_micros() as u64,
            plan_compile_us: perf.plan_compile.as_micros() as u64,
            upload_us: perf.upload.as_micros() as u64,
            record_passes_us: perf.record_passes.as_micros() as u64,
            encoder_finish_us: perf.encoder_finish.as_micros() as u64,
            prepare_svg_us: perf.prepare_svg.as_micros() as u64,
            prepare_text_us: perf.prepare_text.as_micros() as u64,
            svg_uploads: perf.svg_uploads,
            svg_upload_bytes: perf.svg_upload_bytes,
            image_uploads: perf.image_uploads,
            image_upload_bytes: perf.image_upload_bytes,
            render_target_updates_ingest_unknown: perf.render_target_updates_ingest_unknown,
            render_target_updates_ingest_owned: perf.render_target_updates_ingest_owned,
            render_target_updates_ingest_external_zero_copy: perf
                .render_target_updates_ingest_external_zero_copy,
            render_target_updates_ingest_gpu_copy: perf.render_target_updates_ingest_gpu_copy,
            render_target_updates_ingest_cpu_upload: perf.render_target_updates_ingest_cpu_upload,
            render_target_updates_requested_ingest_unknown: perf
                .render_target_updates_requested_ingest_unknown,
            render_target_updates_requested_ingest_owned: perf
                .render_target_updates_requested_ingest_owned,
            render_target_updates_requested_ingest_external_zero_copy: perf
                .render_target_updates_requested_ingest_external_zero_copy,
            render_target_updates_requested_ingest_gpu_copy: perf
                .render_target_updates_requested_ingest_gpu_copy,
            render_target_updates_requested_ingest_cpu_upload: perf
                .render_target_updates_requested_ingest_cpu_upload,
            render_target_updates_ingest_fallbacks: perf.render_target_updates_ingest_fallbacks,
            render_target_metadata_degradations_color_encoding_dropped: perf
                .render_target_metadata_degradations_color_encoding_dropped,
            svg_raster_budget_bytes: perf.svg_raster_budget_bytes,
            svg_rasters_live: perf.svg_rasters_live,
            svg_standalone_bytes_live: perf.svg_standalone_bytes_live,
            svg_mask_atlas_pages_live: perf.svg_mask_atlas_pages_live,
            svg_mask_atlas_bytes_live: perf.svg_mask_atlas_bytes_live,
            svg_mask_atlas_used_px: perf.svg_mask_atlas_used_px,
            svg_mask_atlas_capacity_px: perf.svg_mask_atlas_capacity_px,
            svg_raster_cache_hits: perf.svg_raster_cache_hits,
            svg_raster_cache_misses: perf.svg_raster_cache_misses,
            svg_raster_budget_evictions: perf.svg_raster_budget_evictions,
            svg_mask_atlas_page_evictions: perf.svg_mask_atlas_page_evictions,
            svg_mask_atlas_entries_evicted: perf.svg_mask_atlas_entries_evicted,
            text_atlas_revision: perf.text_atlas_revision,
            text_atlas_uploads: perf.text_atlas_uploads,
            text_atlas_upload_bytes: perf.text_atlas_upload_bytes,
            text_atlas_evicted_glyphs: perf.text_atlas_evicted_glyphs,
            text_atlas_evicted_pages: perf.text_atlas_evicted_pages,
            text_atlas_evicted_page_glyphs: perf.text_atlas_evicted_page_glyphs,
            text_atlas_resets: perf.text_atlas_resets,
            intermediate_budget_bytes: perf.intermediate_budget_bytes,
            intermediate_full_target_bytes: perf.intermediate_full_target_bytes,
            intermediate_in_use_bytes: perf.intermediate_in_use_bytes,
            intermediate_peak_in_use_bytes: perf.intermediate_peak_in_use_bytes,
            intermediate_release_targets: perf.intermediate_release_targets,
            intermediate_pool_allocations: perf.intermediate_pool_allocations,
            intermediate_pool_reuses: perf.intermediate_pool_reuses,
            intermediate_pool_releases: perf.intermediate_pool_releases,
            intermediate_pool_evictions: perf.intermediate_pool_evictions,
            intermediate_pool_free_bytes: perf.intermediate_pool_free_bytes,
            intermediate_pool_free_textures: perf.intermediate_pool_free_textures,
            path_intermediate_bytes_estimate,
            path_intermediate_msaa_bytes_estimate,
            path_intermediate_resolved_bytes_estimate,
            custom_effect_v3_pyramid_scratch_bytes_estimate,
            gpu_images_live: registry_est.images_live,
            gpu_images_bytes_estimate: registry_est.images_bytes_estimate,
            gpu_images_max_bytes_estimate: registry_est.images_max_bytes_estimate,
            gpu_render_targets_live: registry_est.render_targets_live,
            gpu_render_targets_bytes_estimate: registry_est.render_targets_bytes_estimate,
            gpu_render_targets_max_bytes_estimate: registry_est.render_targets_max_bytes_estimate,
            render_plan_estimated_peak_intermediate_bytes: perf
                .render_plan_estimated_peak_intermediate_bytes,
            render_plan_segments: perf.render_plan_segments,
            render_plan_segments_changed: perf.render_plan_segments_changed,
            render_plan_segments_passes_increased: perf.render_plan_segments_passes_increased,
            render_plan_degradations: perf.render_plan_degradations,
            render_plan_effect_chain_budget_samples: perf.render_plan_effect_chain_budget_samples,
            render_plan_effect_chain_effective_budget_min_bytes: perf
                .render_plan_effect_chain_effective_budget_min_bytes,
            render_plan_effect_chain_effective_budget_max_bytes: perf
                .render_plan_effect_chain_effective_budget_max_bytes,
            render_plan_effect_chain_other_live_max_bytes: perf
                .render_plan_effect_chain_other_live_max_bytes,
            render_plan_custom_effect_chain_budget_samples: perf
                .render_plan_custom_effect_chain_budget_samples,
            render_plan_custom_effect_chain_effective_budget_min_bytes: perf
                .render_plan_custom_effect_chain_effective_budget_min_bytes,
            render_plan_custom_effect_chain_effective_budget_max_bytes: perf
                .render_plan_custom_effect_chain_effective_budget_max_bytes,
            render_plan_custom_effect_chain_other_live_max_bytes: perf
                .render_plan_custom_effect_chain_other_live_max_bytes,
            render_plan_custom_effect_chain_base_required_max_bytes: perf
                .render_plan_custom_effect_chain_base_required_max_bytes,
            render_plan_custom_effect_chain_optional_required_max_bytes: perf
                .render_plan_custom_effect_chain_optional_required_max_bytes,
            render_plan_custom_effect_chain_base_required_full_targets_max: perf
                .render_plan_custom_effect_chain_base_required_full_targets_max,
            render_plan_custom_effect_chain_optional_mask_max_bytes: perf
                .render_plan_custom_effect_chain_optional_mask_max_bytes,
            render_plan_custom_effect_chain_optional_pyramid_max_bytes: perf
                .render_plan_custom_effect_chain_optional_pyramid_max_bytes,
            render_plan_degradations_budget_zero: perf.render_plan_degradations_budget_zero,
            render_plan_degradations_budget_insufficient: perf
                .render_plan_degradations_budget_insufficient,
            render_plan_degradations_target_exhausted: perf
                .render_plan_degradations_target_exhausted,
            render_plan_degradations_backdrop_noop: perf.render_plan_degradations_backdrop_noop,
            render_plan_degradations_filter_content_disabled: perf
                .render_plan_degradations_filter_content_disabled,
            render_plan_degradations_clip_path_disabled: perf
                .render_plan_degradations_clip_path_disabled,
            render_plan_degradations_composite_group_blend_to_over: perf
                .render_plan_degradations_composite_group_blend_to_over,
            effect_degradations: perf.effect_degradations,
            effect_blur_quality: perf.effect_blur_quality,
            custom_effect_v1_steps_requested: perf.custom_effect_v1_steps_requested,
            custom_effect_v1_passes_emitted: perf.custom_effect_v1_passes_emitted,
            custom_effect_v2_steps_requested: perf.custom_effect_v2_steps_requested,
            custom_effect_v2_passes_emitted: perf.custom_effect_v2_passes_emitted,
            custom_effect_v2_user_image_incompatible_fallbacks: perf
                .custom_effect_v2_user_image_incompatible_fallbacks,
            custom_effect_v3_steps_requested: perf.custom_effect_v3_steps_requested,
            custom_effect_v3_passes_emitted: perf.custom_effect_v3_passes_emitted,
            custom_effect_v3_user0_image_incompatible_fallbacks: perf
                .custom_effect_v3_user0_image_incompatible_fallbacks,
            custom_effect_v3_user1_image_incompatible_fallbacks: perf
                .custom_effect_v3_user1_image_incompatible_fallbacks,
            custom_effect_v3_pyramid_cache_hits: perf.custom_effect_v3_pyramid_cache_hits,
            custom_effect_v3_pyramid_cache_misses: perf.custom_effect_v3_pyramid_cache_misses,
            clip_path_mask_cache_bytes_live: perf.clip_path_mask_cache_bytes_live,
            clip_path_mask_cache_entries_live: perf.clip_path_mask_cache_entries_live,
            clip_path_mask_cache_hits: perf.clip_path_mask_cache_hits,
            clip_path_mask_cache_misses: perf.clip_path_mask_cache_misses,
            draw_calls: perf.draw_calls,
            quad_draw_calls: perf.quad_draw_calls,
            viewport_draw_calls: perf.viewport_draw_calls,
            viewport_draw_calls_ingest_unknown: perf.viewport_draw_calls_ingest_unknown,
            viewport_draw_calls_ingest_owned: perf.viewport_draw_calls_ingest_owned,
            viewport_draw_calls_ingest_external_zero_copy: perf
                .viewport_draw_calls_ingest_external_zero_copy,
            viewport_draw_calls_ingest_gpu_copy: perf.viewport_draw_calls_ingest_gpu_copy,
            viewport_draw_calls_ingest_cpu_upload: perf.viewport_draw_calls_ingest_cpu_upload,
            image_draw_calls: perf.image_draw_calls,
            text_draw_calls: perf.text_draw_calls,
            path_draw_calls: perf.path_draw_calls,
            mask_draw_calls: perf.mask_draw_calls,
            fullscreen_draw_calls: perf.fullscreen_draw_calls,
            clip_mask_draw_calls: perf.clip_mask_draw_calls,
            pipeline_switches: perf.pipeline_switches,
            pipeline_switches_quad: perf.pipeline_switches_quad,
            pipeline_switches_viewport: perf.pipeline_switches_viewport,
            pipeline_switches_mask: perf.pipeline_switches_mask,
            pipeline_switches_text_mask: perf.pipeline_switches_text_mask,
            pipeline_switches_text_color: perf.pipeline_switches_text_color,
            pipeline_switches_text_subpixel: perf.pipeline_switches_text_subpixel,
            pipeline_switches_path: perf.pipeline_switches_path,
            pipeline_switches_path_msaa: perf.pipeline_switches_path_msaa,
            pipeline_switches_composite: perf.pipeline_switches_composite,
            pipeline_switches_fullscreen: perf.pipeline_switches_fullscreen,
            pipeline_switches_clip_mask: perf.pipeline_switches_clip_mask,
            bind_group_switches: perf.bind_group_switches,
            uniform_bind_group_switches: perf.uniform_bind_group_switches,
            texture_bind_group_switches: perf.texture_bind_group_switches,
            scissor_sets: perf.scissor_sets,
            path_msaa_samples_requested: perf.path_msaa_samples_requested,
            path_msaa_samples_effective: perf.path_msaa_samples_effective,
            path_msaa_vulkan_safety_valve_degradations: perf
                .path_msaa_vulkan_safety_valve_degradations,
            uniform_bytes: perf.uniform_bytes,
            instance_bytes: perf.instance_bytes,
            vertex_bytes: perf.vertex_bytes,
            scene_encoding_cache_hits: perf.scene_encoding_cache_hits,
            scene_encoding_cache_misses: perf.scene_encoding_cache_misses,
            scene_encoding_cache_last_miss_reasons: perf.scene_encoding_cache_last_miss_reasons,
            material_quad_ops: perf.material_quad_ops,
            material_sampled_quad_ops: perf.material_sampled_quad_ops,
            material_distinct: perf.material_distinct,
            material_unknown_ids: perf.material_unknown_ids,
            material_degraded_due_to_budget: perf.material_degraded_due_to_budget,
            path_material_paints_degraded_to_solid_base: perf
                .path_material_paints_degraded_to_solid_base,
        };

        self.diagnostics_state.perf = RenderPerfStats::default();
        Some(snap)
    }

    pub fn take_last_frame_perf_snapshot(&mut self) -> Option<RenderPerfSnapshot> {
        self.diagnostics_state.take_last_frame_perf_snapshot()
    }

    pub fn set_svg_perf_enabled(&mut self, enabled: bool) {
        self.svg_raster_state.set_perf_enabled(enabled);
    }

    /// Drop all cached SVG rasterizations (standalone rasters and alpha-mask atlas pages) while
    /// keeping the underlying registered SVG bytes (`SvgId`) intact.
    ///
    /// This is the GPUI-style explicit lifecycle knob: apps can decide when to reclaim memory and
    /// accept the cost of re-rasterizing later.
    pub fn clear_svg_raster_cache(&mut self) {
        let rasters = std::mem::take(&mut self.svg_raster_state.rasters);
        for (_, entry) in rasters {
            if matches!(entry.storage, SvgRasterStorage::Standalone { .. }) {
                let _ = self.unregister_image(entry.image);
            }
        }
        self.svg_raster_state.raster_bytes = 0;

        for idx in 0..self.svg_raster_state.mask_atlas_pages.len() {
            self.evict_svg_mask_atlas_page(idx);
        }
        self.svg_raster_state.mask_atlas_pages.clear();
        self.svg_raster_state.mask_atlas_free.clear();
        self.svg_raster_state.mask_atlas_bytes = 0;
    }

    /// Drop only the SVG alpha-mask atlas pages and their cached entries.
    ///
    /// This is a cheap explicit “defragment/rebuild” knob: the next `SceneOp::SvgMaskIcon` usage
    /// will re-pack masks into fresh pages.
    pub fn clear_svg_mask_atlas_cache(&mut self) {
        self.svg_raster_state
            .rasters
            .retain(|_, entry| matches!(entry.storage, SvgRasterStorage::Standalone { .. }));

        for idx in 0..self.svg_raster_state.mask_atlas_pages.len() {
            self.evict_svg_mask_atlas_page(idx);
        }
        self.svg_raster_state.mask_atlas_pages.clear();
        self.svg_raster_state.mask_atlas_free.clear();
        self.svg_raster_state.mask_atlas_bytes = 0;
    }

    pub fn take_svg_perf_snapshot(&mut self) -> Option<SvgPerfSnapshot> {
        self.svg_raster_state.take_perf_snapshot()
    }

    pub fn svg_raster_budget_bytes(&self) -> u64 {
        self.svg_raster_state.raster_budget_bytes()
    }

    pub fn set_svg_raster_budget_bytes(&mut self, bytes: u64) {
        self.svg_raster_state.set_raster_budget_bytes(bytes);
    }

    pub fn path_msaa_samples(&self) -> u32 {
        self.path_msaa_samples
    }

    fn path_msaa_samples_override_from_env() -> Option<u32> {
        static OVERRIDE: OnceLock<Option<u32>> = OnceLock::new();
        *OVERRIDE.get_or_init(|| {
            let Ok(raw) = std::env::var("FRET_RENDER_WGPU_PATH_MSAA_SAMPLES") else {
                return None;
            };

            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return None;
            }

            match trimmed.parse::<u32>() {
                Ok(samples) => {
                    tracing::warn!(
                        path_msaa_samples = samples,
                        "Renderer path MSAA samples overridden via FRET_RENDER_WGPU_PATH_MSAA_SAMPLES."
                    );
                    Some(samples)
                }
                Err(_) => {
                    tracing::warn!(
                        raw = trimmed,
                        "Invalid FRET_RENDER_WGPU_PATH_MSAA_SAMPLES; ignoring override."
                    );
                    None
                }
            }
        })
    }

    pub fn set_path_msaa_samples(&mut self, samples: u32) {
        let samples = Self::path_msaa_samples_override_from_env().unwrap_or(samples);
        let samples = samples.max(1);
        let samples = samples.min(16);
        let next_samples = if samples == 1 {
            1
        } else {
            // wgpu requires sample counts to be powers of two. Prefer a conservative downgrade to
            // the nearest supported-shape value (rather than rounding up to a potentially
            // unsupported count).
            let pow2_floor = 1u32 << (31 - samples.leading_zeros());
            pow2_floor.max(1)
        };

        if self.path_msaa_samples != next_samples {
            self.path_intermediate = None;
        }
        self.path_msaa_samples = next_samples;
    }

    pub fn debug_offscreen_blit_enabled(&self) -> bool {
        self.debug_offscreen_blit_enabled
    }

    pub fn set_debug_offscreen_blit_enabled(&mut self, enabled: bool) {
        self.debug_offscreen_blit_enabled = enabled;
    }

    pub fn debug_pixelate_scale(&self) -> u32 {
        self.debug_pixelate_scale
    }

    pub fn set_debug_pixelate_scale(&mut self, scale: u32) {
        // 0 disables the debug pixelate path; otherwise clamp to a sane upper bound.
        self.debug_pixelate_scale = scale.min(128);
    }

    pub fn debug_blur_radius(&self) -> u32 {
        self.debug_blur_radius
    }

    pub fn set_debug_blur_radius(&mut self, radius: u32) {
        // 0 disables the debug blur path; otherwise clamp to a sane upper bound.
        self.debug_blur_radius = radius.min(64);
    }

    pub fn debug_blur_scissor(&self) -> Option<(u32, u32, u32, u32)> {
        self.debug_blur_scissor.map(|s| (s.x, s.y, s.w, s.h))
    }

    pub fn set_debug_blur_scissor(&mut self, scissor: Option<(u32, u32, u32, u32)>) {
        self.debug_blur_scissor = scissor.and_then(|(x, y, w, h)| {
            if w == 0 || h == 0 {
                return None;
            }
            Some(ScissorRect { x, y, w, h })
        });
    }

    fn intermediate_budget_override_bytes_from_env() -> Option<u64> {
        static OVERRIDE: OnceLock<Option<u64>> = OnceLock::new();
        *OVERRIDE.get_or_init(|| {
            let Ok(raw) = std::env::var("FRET_RENDER_WGPU_INTERMEDIATE_BUDGET_BYTES") else {
                return None;
            };

            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return None;
            }

            match trimmed.parse::<u64>() {
                Ok(bytes) => {
                    tracing::warn!(
                        intermediate_budget_bytes = bytes,
                        "Renderer intermediate budget overridden via FRET_RENDER_WGPU_INTERMEDIATE_BUDGET_BYTES."
                    );
                    Some(bytes)
                }
                Err(_) => {
                    tracing::warn!(
                        raw = trimmed,
                        "Invalid FRET_RENDER_WGPU_INTERMEDIATE_BUDGET_BYTES; ignoring override."
                    );
                    None
                }
            }
        })
    }

    pub fn intermediate_budget_bytes(&self) -> u64 {
        self.intermediate_state.budget_bytes()
    }

    pub fn set_intermediate_budget_bytes(&mut self, bytes: u64) {
        let bytes = Self::intermediate_budget_override_bytes_from_env().unwrap_or(bytes);

        // Allow 0 for deterministic "no intermediates" modes (diagnostics, conformance, low-end).
        // Otherwise keep a small non-zero floor so callers can't accidentally force unbounded thrash.
        let budget_bytes = if bytes == 0 { 0 } else { bytes.max(1024) };
        self.intermediate_state.set_budget_bytes(budget_bytes);
        self.intermediate_state.pool.enforce_budget(budget_bytes);
        self.clip_path_mask_cache
            .enforce_budget(&mut self.intermediate_state.pool, budget_bytes / 8);
    }

    pub fn material_paint_budget_per_frame(&self) -> u64 {
        self.material_effect_state.material_paint_budget_per_frame
    }

    pub fn set_material_paint_budget_per_frame(&mut self, budget: u64) {
        // Allow 0 to force deterministic solid-color fallbacks in conformance tests or low-end modes.
        self.material_effect_state.material_paint_budget_per_frame = budget;
    }

    pub fn material_distinct_budget_per_frame(&self) -> usize {
        self.material_effect_state
            .material_distinct_budget_per_frame
    }

    pub fn set_material_distinct_budget_per_frame(&mut self, budget: usize) {
        // Allow 0 to force deterministic solid-color fallbacks in conformance tests or low-end modes.
        self.material_effect_state
            .material_distinct_budget_per_frame = budget;
    }

    pub fn set_intermediate_perf_enabled(&mut self, enabled: bool) {
        self.intermediate_state.set_perf_enabled(enabled);
    }

    pub fn take_intermediate_perf_snapshot(&mut self) -> Option<IntermediatePerfSnapshot> {
        self.intermediate_state.take_perf_snapshot()
    }

    pub fn set_text_font_families(&mut self, config: &TextFontFamilyConfig) -> bool {
        self.text_system.set_font_families(config)
    }

    pub fn set_text_quality_settings(&mut self, settings: TextQualitySettings) -> bool {
        self.text_system.set_text_quality_settings(settings)
    }

    pub fn set_text_locale(&mut self, locale_bcp47: Option<&str>) -> bool {
        self.text_system.set_text_locale(locale_bcp47)
    }

    /// Returns a sorted list of available font family names (best-effort).
    pub fn all_font_names(&mut self) -> Vec<String> {
        self.text_system.all_font_names()
    }

    pub fn all_font_catalog_entries(&mut self) -> Vec<crate::FontCatalogEntryMetadata> {
        self.text_system.all_font_catalog_entries()
    }

    /// Adds font bytes (TTF/OTF/TTC) to the renderer text system.
    ///
    /// Returns the number of newly loaded faces.
    pub fn add_fonts(&mut self, fonts: impl IntoIterator<Item = Vec<u8>>) -> usize {
        self.text_system.add_fonts(fonts)
    }

    pub fn system_font_rescan_seed(&self) -> Option<SystemFontRescanSeed> {
        self.text_system.system_font_rescan_seed()
    }

    pub fn apply_system_font_rescan_result(&mut self, result: SystemFontRescanResult) -> bool {
        self.text_system.apply_system_font_rescan_result(result)
    }

    pub fn rescan_system_fonts(&mut self) -> bool {
        self.text_system.rescan_system_fonts()
    }

    pub fn text_font_stack_key(&self) -> u64 {
        self.text_system.font_stack_key()
    }

    pub(super) fn effective_path_msaa_samples(&self, format: wgpu::TextureFormat) -> u32 {
        let requested = self.path_msaa_samples.max(1);
        if requested == 1 {
            return 1;
        }

        // Vulkan path MSAA can be disabled via env var as an emergency escape hatch for driver
        // issues. Prefer the opt-out knob over backend allow/deny lists to match GPUI's default
        // behavior ("enable when supported; disable only when needed").
        if self.adapter.get_info().backend == wgpu::Backend::Vulkan
            && std::env::var_os("FRET_DISABLE_VULKAN_PATH_MSAA").is_some()
        {
            static WARNED: OnceLock<()> = OnceLock::new();
            if WARNED.set(()).is_ok() {
                let info = self.adapter.get_info();
                tracing::warn!(
                    backend = ?info.backend,
                    vendor = info.vendor,
                    device = info.device,
                    driver = info.driver,
                    driver_info = info.driver_info,
                    "Vulkan path MSAA is disabled via FRET_DISABLE_VULKAN_PATH_MSAA=1."
                );
            }
            return 1;
        }

        let features = self.adapter.get_texture_format_features(format);
        if !features
            .allowed_usages
            .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
        {
            return 1;
        }

        // When MSAA is enabled we render into an intermediate and then sample from the resolved
        // texture in the composite pass, so the format must be sampleable and support resolves.
        if !features
            .allowed_usages
            .contains(wgpu::TextureUsages::TEXTURE_BINDING)
            || !features
                .flags
                .contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE)
        {
            return 1;
        }

        for candidate in [16u32, 8, 4, 2] {
            if candidate <= requested && features.flags.sample_count_supported(candidate) {
                return candidate;
            }
        }
        1
    }
}
