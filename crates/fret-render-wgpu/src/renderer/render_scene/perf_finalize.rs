use super::super::*;

impl Renderer {
    pub(super) fn finalize_frame_perf_after_dispatch(&mut self, frame_perf: &mut RenderPerfStats) {
        self.svg_raster_state.write_frame_perf(frame_perf);

        frame_perf.clip_path_mask_cache_bytes_live = self.clip_path_mask_cache.bytes_live();
        frame_perf.clip_path_mask_cache_entries_live = self.clip_path_mask_cache.entries_live();

        let pool_perf = self.intermediate_pool.take_perf_snapshot();
        frame_perf.intermediate_pool_allocations = pool_perf.allocations;
        frame_perf.intermediate_pool_reuses = pool_perf.reuses;
        frame_perf.intermediate_pool_releases = pool_perf.releases;
        frame_perf.intermediate_pool_evictions = pool_perf.evictions;
        frame_perf.intermediate_pool_free_bytes = pool_perf.free_bytes;
        frame_perf.intermediate_pool_free_textures = pool_perf.free_textures;

        self.perf.frames = self.perf.frames.saturating_add(frame_perf.frames);
        self.perf.encode_scene += frame_perf.encode_scene;
        self.perf.ensure_pipelines += frame_perf.ensure_pipelines;
        self.perf.plan_compile += frame_perf.plan_compile;
        self.perf.upload += frame_perf.upload;
        self.perf.record_passes += frame_perf.record_passes;
        self.perf.encoder_finish += frame_perf.encoder_finish;
        self.perf.prepare_svg += frame_perf.prepare_svg;
        self.perf.prepare_text += frame_perf.prepare_text;

        self.perf.svg_uploads = self.perf.svg_uploads.saturating_add(frame_perf.svg_uploads);
        self.perf.svg_upload_bytes = self
            .perf
            .svg_upload_bytes
            .saturating_add(frame_perf.svg_upload_bytes);
        self.perf.image_uploads = self
            .perf
            .image_uploads
            .saturating_add(frame_perf.image_uploads);
        self.perf.image_upload_bytes = self
            .perf
            .image_upload_bytes
            .saturating_add(frame_perf.image_upload_bytes);

        self.perf.render_target_updates_ingest_unknown = self
            .perf
            .render_target_updates_ingest_unknown
            .saturating_add(frame_perf.render_target_updates_ingest_unknown);
        self.perf.render_target_updates_ingest_owned = self
            .perf
            .render_target_updates_ingest_owned
            .saturating_add(frame_perf.render_target_updates_ingest_owned);
        self.perf.render_target_updates_ingest_external_zero_copy = self
            .perf
            .render_target_updates_ingest_external_zero_copy
            .saturating_add(frame_perf.render_target_updates_ingest_external_zero_copy);
        self.perf.render_target_updates_ingest_gpu_copy = self
            .perf
            .render_target_updates_ingest_gpu_copy
            .saturating_add(frame_perf.render_target_updates_ingest_gpu_copy);
        self.perf.render_target_updates_ingest_cpu_upload = self
            .perf
            .render_target_updates_ingest_cpu_upload
            .saturating_add(frame_perf.render_target_updates_ingest_cpu_upload);

        self.perf.render_target_updates_requested_ingest_unknown = self
            .perf
            .render_target_updates_requested_ingest_unknown
            .saturating_add(frame_perf.render_target_updates_requested_ingest_unknown);
        self.perf.render_target_updates_requested_ingest_owned = self
            .perf
            .render_target_updates_requested_ingest_owned
            .saturating_add(frame_perf.render_target_updates_requested_ingest_owned);
        self.perf
            .render_target_updates_requested_ingest_external_zero_copy = self
            .perf
            .render_target_updates_requested_ingest_external_zero_copy
            .saturating_add(frame_perf.render_target_updates_requested_ingest_external_zero_copy);
        self.perf.render_target_updates_requested_ingest_gpu_copy = self
            .perf
            .render_target_updates_requested_ingest_gpu_copy
            .saturating_add(frame_perf.render_target_updates_requested_ingest_gpu_copy);
        self.perf.render_target_updates_requested_ingest_cpu_upload = self
            .perf
            .render_target_updates_requested_ingest_cpu_upload
            .saturating_add(frame_perf.render_target_updates_requested_ingest_cpu_upload);
        self.perf.render_target_updates_ingest_fallbacks = self
            .perf
            .render_target_updates_ingest_fallbacks
            .saturating_add(frame_perf.render_target_updates_ingest_fallbacks);
        self.perf
            .render_target_metadata_degradations_color_encoding_dropped = self
            .perf
            .render_target_metadata_degradations_color_encoding_dropped
            .saturating_add(frame_perf.render_target_metadata_degradations_color_encoding_dropped);

        self.perf.svg_raster_budget_bytes = frame_perf.svg_raster_budget_bytes;
        self.perf.svg_rasters_live = self.perf.svg_rasters_live.max(frame_perf.svg_rasters_live);
        self.perf.svg_standalone_bytes_live = self
            .perf
            .svg_standalone_bytes_live
            .max(frame_perf.svg_standalone_bytes_live);
        self.perf.svg_mask_atlas_pages_live = self
            .perf
            .svg_mask_atlas_pages_live
            .max(frame_perf.svg_mask_atlas_pages_live);
        self.perf.svg_mask_atlas_bytes_live = self
            .perf
            .svg_mask_atlas_bytes_live
            .max(frame_perf.svg_mask_atlas_bytes_live);
        self.perf.svg_mask_atlas_used_px = self
            .perf
            .svg_mask_atlas_used_px
            .max(frame_perf.svg_mask_atlas_used_px);
        self.perf.svg_mask_atlas_capacity_px = self
            .perf
            .svg_mask_atlas_capacity_px
            .max(frame_perf.svg_mask_atlas_capacity_px);
        self.perf.svg_raster_cache_hits = self
            .perf
            .svg_raster_cache_hits
            .saturating_add(frame_perf.svg_raster_cache_hits);
        self.perf.svg_raster_cache_misses = self
            .perf
            .svg_raster_cache_misses
            .saturating_add(frame_perf.svg_raster_cache_misses);
        self.perf.svg_raster_budget_evictions = self
            .perf
            .svg_raster_budget_evictions
            .saturating_add(frame_perf.svg_raster_budget_evictions);
        self.perf.svg_mask_atlas_page_evictions = self
            .perf
            .svg_mask_atlas_page_evictions
            .saturating_add(frame_perf.svg_mask_atlas_page_evictions);
        self.perf.svg_mask_atlas_entries_evicted = self
            .perf
            .svg_mask_atlas_entries_evicted
            .saturating_add(frame_perf.svg_mask_atlas_entries_evicted);

        self.perf.text_atlas_revision = frame_perf.text_atlas_revision;
        self.perf.text_atlas_uploads = self
            .perf
            .text_atlas_uploads
            .saturating_add(frame_perf.text_atlas_uploads);
        self.perf.text_atlas_upload_bytes = self
            .perf
            .text_atlas_upload_bytes
            .saturating_add(frame_perf.text_atlas_upload_bytes);
        self.perf.text_atlas_evicted_glyphs = self
            .perf
            .text_atlas_evicted_glyphs
            .saturating_add(frame_perf.text_atlas_evicted_glyphs);
        self.perf.text_atlas_evicted_pages = self
            .perf
            .text_atlas_evicted_pages
            .saturating_add(frame_perf.text_atlas_evicted_pages);
        self.perf.text_atlas_evicted_page_glyphs = self
            .perf
            .text_atlas_evicted_page_glyphs
            .saturating_add(frame_perf.text_atlas_evicted_page_glyphs);
        self.perf.text_atlas_resets = self
            .perf
            .text_atlas_resets
            .saturating_add(frame_perf.text_atlas_resets);

        self.perf.intermediate_budget_bytes = frame_perf.intermediate_budget_bytes;
        self.perf.intermediate_full_target_bytes = frame_perf.intermediate_full_target_bytes;
        self.perf.intermediate_in_use_bytes = self
            .perf
            .intermediate_in_use_bytes
            .max(frame_perf.intermediate_in_use_bytes);
        self.perf.intermediate_peak_in_use_bytes = self
            .perf
            .intermediate_peak_in_use_bytes
            .max(frame_perf.intermediate_peak_in_use_bytes);
        self.perf.intermediate_release_targets = self
            .perf
            .intermediate_release_targets
            .saturating_add(frame_perf.intermediate_release_targets);
        self.perf.intermediate_pool_allocations = self
            .perf
            .intermediate_pool_allocations
            .saturating_add(frame_perf.intermediate_pool_allocations);
        self.perf.intermediate_pool_reuses = self
            .perf
            .intermediate_pool_reuses
            .saturating_add(frame_perf.intermediate_pool_reuses);
        self.perf.intermediate_pool_releases = self
            .perf
            .intermediate_pool_releases
            .saturating_add(frame_perf.intermediate_pool_releases);
        self.perf.intermediate_pool_evictions = self
            .perf
            .intermediate_pool_evictions
            .saturating_add(frame_perf.intermediate_pool_evictions);
        self.perf.intermediate_pool_free_bytes = pool_perf.free_bytes;
        self.perf.intermediate_pool_free_textures = pool_perf.free_textures;
        self.perf.render_plan_estimated_peak_intermediate_bytes = self
            .perf
            .render_plan_estimated_peak_intermediate_bytes
            .max(frame_perf.render_plan_estimated_peak_intermediate_bytes);
        self.perf.render_plan_segments = self
            .perf
            .render_plan_segments
            .max(frame_perf.render_plan_segments);
        self.perf.render_plan_degradations = self
            .perf
            .render_plan_degradations
            .saturating_add(frame_perf.render_plan_degradations);
        self.perf.render_plan_effect_chain_budget_samples =
            frame_perf.render_plan_effect_chain_budget_samples;
        self.perf
            .render_plan_effect_chain_effective_budget_min_bytes =
            frame_perf.render_plan_effect_chain_effective_budget_min_bytes;
        self.perf
            .render_plan_effect_chain_effective_budget_max_bytes =
            frame_perf.render_plan_effect_chain_effective_budget_max_bytes;
        self.perf.render_plan_effect_chain_other_live_max_bytes =
            frame_perf.render_plan_effect_chain_other_live_max_bytes;
        self.perf.render_plan_custom_effect_chain_budget_samples =
            frame_perf.render_plan_custom_effect_chain_budget_samples;
        self.perf
            .render_plan_custom_effect_chain_effective_budget_min_bytes =
            frame_perf.render_plan_custom_effect_chain_effective_budget_min_bytes;
        self.perf
            .render_plan_custom_effect_chain_effective_budget_max_bytes =
            frame_perf.render_plan_custom_effect_chain_effective_budget_max_bytes;
        self.perf
            .render_plan_custom_effect_chain_other_live_max_bytes =
            frame_perf.render_plan_custom_effect_chain_other_live_max_bytes;
        self.perf
            .render_plan_custom_effect_chain_base_required_max_bytes =
            frame_perf.render_plan_custom_effect_chain_base_required_max_bytes;
        self.perf
            .render_plan_custom_effect_chain_optional_required_max_bytes =
            frame_perf.render_plan_custom_effect_chain_optional_required_max_bytes;
        self.perf
            .render_plan_custom_effect_chain_base_required_full_targets_max =
            frame_perf.render_plan_custom_effect_chain_base_required_full_targets_max;
        self.perf
            .render_plan_custom_effect_chain_optional_mask_max_bytes =
            frame_perf.render_plan_custom_effect_chain_optional_mask_max_bytes;
        self.perf
            .render_plan_custom_effect_chain_optional_pyramid_max_bytes =
            frame_perf.render_plan_custom_effect_chain_optional_pyramid_max_bytes;
        self.perf.render_plan_segments_changed = self
            .perf
            .render_plan_segments_changed
            .saturating_add(frame_perf.render_plan_segments_changed);
        self.perf.render_plan_segments_passes_increased = self
            .perf
            .render_plan_segments_passes_increased
            .saturating_add(frame_perf.render_plan_segments_passes_increased);
        self.perf.render_plan_degradations_budget_zero = self
            .perf
            .render_plan_degradations_budget_zero
            .saturating_add(frame_perf.render_plan_degradations_budget_zero);
        self.perf.render_plan_degradations_budget_insufficient = self
            .perf
            .render_plan_degradations_budget_insufficient
            .saturating_add(frame_perf.render_plan_degradations_budget_insufficient);
        self.perf.render_plan_degradations_target_exhausted = self
            .perf
            .render_plan_degradations_target_exhausted
            .saturating_add(frame_perf.render_plan_degradations_target_exhausted);
        self.perf.render_plan_degradations_backdrop_noop = self
            .perf
            .render_plan_degradations_backdrop_noop
            .saturating_add(frame_perf.render_plan_degradations_backdrop_noop);
        self.perf.render_plan_degradations_filter_content_disabled = self
            .perf
            .render_plan_degradations_filter_content_disabled
            .saturating_add(frame_perf.render_plan_degradations_filter_content_disabled);
        self.perf.render_plan_degradations_clip_path_disabled = self
            .perf
            .render_plan_degradations_clip_path_disabled
            .saturating_add(frame_perf.render_plan_degradations_clip_path_disabled);
        self.perf
            .render_plan_degradations_composite_group_blend_to_over = self
            .perf
            .render_plan_degradations_composite_group_blend_to_over
            .saturating_add(frame_perf.render_plan_degradations_composite_group_blend_to_over);
        self.perf
            .effect_degradations
            .saturating_add_assign(frame_perf.effect_degradations);
        self.perf
            .effect_blur_quality
            .saturating_add_assign(frame_perf.effect_blur_quality);
        self.perf.custom_effect_v1_steps_requested = self
            .perf
            .custom_effect_v1_steps_requested
            .saturating_add(frame_perf.custom_effect_v1_steps_requested);
        self.perf.custom_effect_v1_passes_emitted = self
            .perf
            .custom_effect_v1_passes_emitted
            .saturating_add(frame_perf.custom_effect_v1_passes_emitted);
        self.perf.custom_effect_v2_steps_requested = self
            .perf
            .custom_effect_v2_steps_requested
            .saturating_add(frame_perf.custom_effect_v2_steps_requested);
        self.perf.custom_effect_v2_passes_emitted = self
            .perf
            .custom_effect_v2_passes_emitted
            .saturating_add(frame_perf.custom_effect_v2_passes_emitted);
        self.perf.custom_effect_v2_user_image_incompatible_fallbacks = self
            .perf
            .custom_effect_v2_user_image_incompatible_fallbacks
            .saturating_add(frame_perf.custom_effect_v2_user_image_incompatible_fallbacks);
        self.perf.custom_effect_v3_pyramid_cache_hits = self
            .perf
            .custom_effect_v3_pyramid_cache_hits
            .saturating_add(frame_perf.custom_effect_v3_pyramid_cache_hits);
        self.perf.custom_effect_v3_pyramid_cache_misses = self
            .perf
            .custom_effect_v3_pyramid_cache_misses
            .saturating_add(frame_perf.custom_effect_v3_pyramid_cache_misses);
        self.perf.custom_effect_v3_steps_requested = self
            .perf
            .custom_effect_v3_steps_requested
            .saturating_add(frame_perf.custom_effect_v3_steps_requested);
        self.perf.custom_effect_v3_passes_emitted = self
            .perf
            .custom_effect_v3_passes_emitted
            .saturating_add(frame_perf.custom_effect_v3_passes_emitted);
        self.perf
            .custom_effect_v3_user0_image_incompatible_fallbacks = self
            .perf
            .custom_effect_v3_user0_image_incompatible_fallbacks
            .saturating_add(frame_perf.custom_effect_v3_user0_image_incompatible_fallbacks);
        self.perf
            .custom_effect_v3_user1_image_incompatible_fallbacks = self
            .perf
            .custom_effect_v3_user1_image_incompatible_fallbacks
            .saturating_add(frame_perf.custom_effect_v3_user1_image_incompatible_fallbacks);

        self.perf.clip_path_mask_cache_bytes_live = self
            .perf
            .clip_path_mask_cache_bytes_live
            .max(frame_perf.clip_path_mask_cache_bytes_live);
        self.perf.clip_path_mask_cache_entries_live = self
            .perf
            .clip_path_mask_cache_entries_live
            .max(frame_perf.clip_path_mask_cache_entries_live);
        self.perf.clip_path_mask_cache_hits = self
            .perf
            .clip_path_mask_cache_hits
            .saturating_add(frame_perf.clip_path_mask_cache_hits);
        self.perf.clip_path_mask_cache_misses = self
            .perf
            .clip_path_mask_cache_misses
            .saturating_add(frame_perf.clip_path_mask_cache_misses);

        self.perf.draw_calls = self.perf.draw_calls.saturating_add(frame_perf.draw_calls);
        self.perf.quad_draw_calls = self
            .perf
            .quad_draw_calls
            .saturating_add(frame_perf.quad_draw_calls);
        self.perf.viewport_draw_calls = self
            .perf
            .viewport_draw_calls
            .saturating_add(frame_perf.viewport_draw_calls);
        self.perf.viewport_draw_calls_ingest_unknown = self
            .perf
            .viewport_draw_calls_ingest_unknown
            .saturating_add(frame_perf.viewport_draw_calls_ingest_unknown);
        self.perf.viewport_draw_calls_ingest_owned = self
            .perf
            .viewport_draw_calls_ingest_owned
            .saturating_add(frame_perf.viewport_draw_calls_ingest_owned);
        self.perf.viewport_draw_calls_ingest_external_zero_copy = self
            .perf
            .viewport_draw_calls_ingest_external_zero_copy
            .saturating_add(frame_perf.viewport_draw_calls_ingest_external_zero_copy);
        self.perf.viewport_draw_calls_ingest_gpu_copy = self
            .perf
            .viewport_draw_calls_ingest_gpu_copy
            .saturating_add(frame_perf.viewport_draw_calls_ingest_gpu_copy);
        self.perf.viewport_draw_calls_ingest_cpu_upload = self
            .perf
            .viewport_draw_calls_ingest_cpu_upload
            .saturating_add(frame_perf.viewport_draw_calls_ingest_cpu_upload);
        self.perf.image_draw_calls = self
            .perf
            .image_draw_calls
            .saturating_add(frame_perf.image_draw_calls);
        self.perf.text_draw_calls = self
            .perf
            .text_draw_calls
            .saturating_add(frame_perf.text_draw_calls);
        self.perf.path_draw_calls = self
            .perf
            .path_draw_calls
            .saturating_add(frame_perf.path_draw_calls);
        self.perf.mask_draw_calls = self
            .perf
            .mask_draw_calls
            .saturating_add(frame_perf.mask_draw_calls);
        self.perf.fullscreen_draw_calls = self
            .perf
            .fullscreen_draw_calls
            .saturating_add(frame_perf.fullscreen_draw_calls);
        self.perf.clip_mask_draw_calls = self
            .perf
            .clip_mask_draw_calls
            .saturating_add(frame_perf.clip_mask_draw_calls);
        self.perf.pipeline_switches = self
            .perf
            .pipeline_switches
            .saturating_add(frame_perf.pipeline_switches);
        self.perf.pipeline_switches_quad = self
            .perf
            .pipeline_switches_quad
            .saturating_add(frame_perf.pipeline_switches_quad);
        self.perf.pipeline_switches_viewport = self
            .perf
            .pipeline_switches_viewport
            .saturating_add(frame_perf.pipeline_switches_viewport);
        self.perf.pipeline_switches_mask = self
            .perf
            .pipeline_switches_mask
            .saturating_add(frame_perf.pipeline_switches_mask);
        self.perf.pipeline_switches_text_mask = self
            .perf
            .pipeline_switches_text_mask
            .saturating_add(frame_perf.pipeline_switches_text_mask);
        self.perf.pipeline_switches_text_color = self
            .perf
            .pipeline_switches_text_color
            .saturating_add(frame_perf.pipeline_switches_text_color);
        self.perf.pipeline_switches_text_subpixel = self
            .perf
            .pipeline_switches_text_subpixel
            .saturating_add(frame_perf.pipeline_switches_text_subpixel);
        self.perf.pipeline_switches_path = self
            .perf
            .pipeline_switches_path
            .saturating_add(frame_perf.pipeline_switches_path);
        self.perf.pipeline_switches_path_msaa = self
            .perf
            .pipeline_switches_path_msaa
            .saturating_add(frame_perf.pipeline_switches_path_msaa);
        self.perf.pipeline_switches_composite = self
            .perf
            .pipeline_switches_composite
            .saturating_add(frame_perf.pipeline_switches_composite);
        self.perf.pipeline_switches_fullscreen = self
            .perf
            .pipeline_switches_fullscreen
            .saturating_add(frame_perf.pipeline_switches_fullscreen);
        self.perf.pipeline_switches_clip_mask = self
            .perf
            .pipeline_switches_clip_mask
            .saturating_add(frame_perf.pipeline_switches_clip_mask);
        self.perf.bind_group_switches = self
            .perf
            .bind_group_switches
            .saturating_add(frame_perf.bind_group_switches);
        self.perf.uniform_bind_group_switches = self
            .perf
            .uniform_bind_group_switches
            .saturating_add(frame_perf.uniform_bind_group_switches);
        self.perf.texture_bind_group_switches = self
            .perf
            .texture_bind_group_switches
            .saturating_add(frame_perf.texture_bind_group_switches);
        self.perf.scissor_sets = self
            .perf
            .scissor_sets
            .saturating_add(frame_perf.scissor_sets);
        self.perf.path_msaa_samples_requested = frame_perf.path_msaa_samples_requested;
        self.perf.path_msaa_samples_effective = frame_perf.path_msaa_samples_effective;
        self.perf.path_msaa_vulkan_safety_valve_degradations = self
            .perf
            .path_msaa_vulkan_safety_valve_degradations
            .saturating_add(frame_perf.path_msaa_vulkan_safety_valve_degradations);
        self.perf.uniform_bytes = self
            .perf
            .uniform_bytes
            .saturating_add(frame_perf.uniform_bytes);
        self.perf.instance_bytes = self
            .perf
            .instance_bytes
            .saturating_add(frame_perf.instance_bytes);
        self.perf.vertex_bytes = self
            .perf
            .vertex_bytes
            .saturating_add(frame_perf.vertex_bytes);
        self.perf.scene_encoding_cache_hits = self
            .perf
            .scene_encoding_cache_hits
            .saturating_add(frame_perf.scene_encoding_cache_hits);
        self.perf.scene_encoding_cache_misses = self
            .perf
            .scene_encoding_cache_misses
            .saturating_add(frame_perf.scene_encoding_cache_misses);
        if frame_perf.scene_encoding_cache_last_miss_reasons != 0 {
            self.perf.scene_encoding_cache_last_miss_reasons =
                frame_perf.scene_encoding_cache_last_miss_reasons;
        }
        self.perf.material_quad_ops = self
            .perf
            .material_quad_ops
            .saturating_add(frame_perf.material_quad_ops);
        self.perf.material_sampled_quad_ops = self
            .perf
            .material_sampled_quad_ops
            .saturating_add(frame_perf.material_sampled_quad_ops);
        self.perf.material_distinct = self
            .perf
            .material_distinct
            .saturating_add(frame_perf.material_distinct);
        self.perf.material_unknown_ids = self
            .perf
            .material_unknown_ids
            .saturating_add(frame_perf.material_unknown_ids);
        self.perf.material_degraded_due_to_budget = self
            .perf
            .material_degraded_due_to_budget
            .saturating_add(frame_perf.material_degraded_due_to_budget);
        self.perf.path_material_paints_degraded_to_solid_base = self
            .perf
            .path_material_paints_degraded_to_solid_base
            .saturating_add(frame_perf.path_material_paints_degraded_to_solid_base);

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

        self.last_frame_perf = Some(RenderPerfSnapshot {
            frames: frame_perf.frames,
            encode_scene_us: frame_perf.encode_scene.as_micros() as u64,
            ensure_pipelines_us: frame_perf.ensure_pipelines.as_micros() as u64,
            plan_compile_us: frame_perf.plan_compile.as_micros() as u64,
            upload_us: frame_perf.upload.as_micros() as u64,
            record_passes_us: frame_perf.record_passes.as_micros() as u64,
            encoder_finish_us: frame_perf.encoder_finish.as_micros() as u64,
            prepare_svg_us: frame_perf.prepare_svg.as_micros() as u64,
            prepare_text_us: frame_perf.prepare_text.as_micros() as u64,
            svg_uploads: frame_perf.svg_uploads,
            svg_upload_bytes: frame_perf.svg_upload_bytes,
            image_uploads: frame_perf.image_uploads,
            image_upload_bytes: frame_perf.image_upload_bytes,
            render_target_updates_ingest_unknown: frame_perf.render_target_updates_ingest_unknown,
            render_target_updates_ingest_owned: frame_perf.render_target_updates_ingest_owned,
            render_target_updates_ingest_external_zero_copy: frame_perf
                .render_target_updates_ingest_external_zero_copy,
            render_target_updates_ingest_gpu_copy: frame_perf.render_target_updates_ingest_gpu_copy,
            render_target_updates_ingest_cpu_upload: frame_perf
                .render_target_updates_ingest_cpu_upload,
            render_target_updates_requested_ingest_unknown: frame_perf
                .render_target_updates_requested_ingest_unknown,
            render_target_updates_requested_ingest_owned: frame_perf
                .render_target_updates_requested_ingest_owned,
            render_target_updates_requested_ingest_external_zero_copy: frame_perf
                .render_target_updates_requested_ingest_external_zero_copy,
            render_target_updates_requested_ingest_gpu_copy: frame_perf
                .render_target_updates_requested_ingest_gpu_copy,
            render_target_updates_requested_ingest_cpu_upload: frame_perf
                .render_target_updates_requested_ingest_cpu_upload,
            render_target_updates_ingest_fallbacks: frame_perf
                .render_target_updates_ingest_fallbacks,
            render_target_metadata_degradations_color_encoding_dropped: frame_perf
                .render_target_metadata_degradations_color_encoding_dropped,
            svg_raster_budget_bytes: frame_perf.svg_raster_budget_bytes,
            svg_rasters_live: frame_perf.svg_rasters_live,
            svg_standalone_bytes_live: frame_perf.svg_standalone_bytes_live,
            svg_mask_atlas_pages_live: frame_perf.svg_mask_atlas_pages_live,
            svg_mask_atlas_bytes_live: frame_perf.svg_mask_atlas_bytes_live,
            svg_mask_atlas_used_px: frame_perf.svg_mask_atlas_used_px,
            svg_mask_atlas_capacity_px: frame_perf.svg_mask_atlas_capacity_px,
            svg_raster_cache_hits: frame_perf.svg_raster_cache_hits,
            svg_raster_cache_misses: frame_perf.svg_raster_cache_misses,
            svg_raster_budget_evictions: frame_perf.svg_raster_budget_evictions,
            svg_mask_atlas_page_evictions: frame_perf.svg_mask_atlas_page_evictions,
            svg_mask_atlas_entries_evicted: frame_perf.svg_mask_atlas_entries_evicted,
            text_atlas_revision: frame_perf.text_atlas_revision,
            text_atlas_uploads: frame_perf.text_atlas_uploads,
            text_atlas_upload_bytes: frame_perf.text_atlas_upload_bytes,
            text_atlas_evicted_glyphs: frame_perf.text_atlas_evicted_glyphs,
            text_atlas_evicted_pages: frame_perf.text_atlas_evicted_pages,
            text_atlas_evicted_page_glyphs: frame_perf.text_atlas_evicted_page_glyphs,
            text_atlas_resets: frame_perf.text_atlas_resets,
            intermediate_budget_bytes: frame_perf.intermediate_budget_bytes,
            intermediate_full_target_bytes: frame_perf.intermediate_full_target_bytes,
            intermediate_in_use_bytes: frame_perf.intermediate_in_use_bytes,
            intermediate_peak_in_use_bytes: frame_perf.intermediate_peak_in_use_bytes,
            intermediate_release_targets: frame_perf.intermediate_release_targets,
            intermediate_pool_allocations: frame_perf.intermediate_pool_allocations,
            intermediate_pool_reuses: frame_perf.intermediate_pool_reuses,
            intermediate_pool_releases: frame_perf.intermediate_pool_releases,
            intermediate_pool_evictions: frame_perf.intermediate_pool_evictions,
            intermediate_pool_free_bytes: frame_perf.intermediate_pool_free_bytes,
            intermediate_pool_free_textures: frame_perf.intermediate_pool_free_textures,
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
            render_plan_estimated_peak_intermediate_bytes: frame_perf
                .render_plan_estimated_peak_intermediate_bytes,
            render_plan_segments: frame_perf.render_plan_segments,
            render_plan_segments_changed: frame_perf.render_plan_segments_changed,
            render_plan_segments_passes_increased: frame_perf.render_plan_segments_passes_increased,
            render_plan_degradations: frame_perf.render_plan_degradations,
            render_plan_effect_chain_budget_samples: frame_perf
                .render_plan_effect_chain_budget_samples,
            render_plan_effect_chain_effective_budget_min_bytes: frame_perf
                .render_plan_effect_chain_effective_budget_min_bytes,
            render_plan_effect_chain_effective_budget_max_bytes: frame_perf
                .render_plan_effect_chain_effective_budget_max_bytes,
            render_plan_effect_chain_other_live_max_bytes: frame_perf
                .render_plan_effect_chain_other_live_max_bytes,
            render_plan_custom_effect_chain_budget_samples: frame_perf
                .render_plan_custom_effect_chain_budget_samples,
            render_plan_custom_effect_chain_effective_budget_min_bytes: frame_perf
                .render_plan_custom_effect_chain_effective_budget_min_bytes,
            render_plan_custom_effect_chain_effective_budget_max_bytes: frame_perf
                .render_plan_custom_effect_chain_effective_budget_max_bytes,
            render_plan_custom_effect_chain_other_live_max_bytes: frame_perf
                .render_plan_custom_effect_chain_other_live_max_bytes,
            render_plan_custom_effect_chain_base_required_max_bytes: frame_perf
                .render_plan_custom_effect_chain_base_required_max_bytes,
            render_plan_custom_effect_chain_optional_required_max_bytes: frame_perf
                .render_plan_custom_effect_chain_optional_required_max_bytes,
            render_plan_custom_effect_chain_base_required_full_targets_max: frame_perf
                .render_plan_custom_effect_chain_base_required_full_targets_max,
            render_plan_custom_effect_chain_optional_mask_max_bytes: frame_perf
                .render_plan_custom_effect_chain_optional_mask_max_bytes,
            render_plan_custom_effect_chain_optional_pyramid_max_bytes: frame_perf
                .render_plan_custom_effect_chain_optional_pyramid_max_bytes,
            render_plan_degradations_budget_zero: frame_perf.render_plan_degradations_budget_zero,
            render_plan_degradations_budget_insufficient: frame_perf
                .render_plan_degradations_budget_insufficient,
            render_plan_degradations_target_exhausted: frame_perf
                .render_plan_degradations_target_exhausted,
            render_plan_degradations_backdrop_noop: frame_perf
                .render_plan_degradations_backdrop_noop,
            render_plan_degradations_filter_content_disabled: frame_perf
                .render_plan_degradations_filter_content_disabled,
            render_plan_degradations_clip_path_disabled: frame_perf
                .render_plan_degradations_clip_path_disabled,
            render_plan_degradations_composite_group_blend_to_over: frame_perf
                .render_plan_degradations_composite_group_blend_to_over,
            effect_degradations: frame_perf.effect_degradations,
            effect_blur_quality: frame_perf.effect_blur_quality,
            custom_effect_v1_steps_requested: frame_perf.custom_effect_v1_steps_requested,
            custom_effect_v1_passes_emitted: frame_perf.custom_effect_v1_passes_emitted,
            custom_effect_v2_steps_requested: frame_perf.custom_effect_v2_steps_requested,
            custom_effect_v2_passes_emitted: frame_perf.custom_effect_v2_passes_emitted,
            custom_effect_v2_user_image_incompatible_fallbacks: frame_perf
                .custom_effect_v2_user_image_incompatible_fallbacks,
            custom_effect_v3_steps_requested: frame_perf.custom_effect_v3_steps_requested,
            custom_effect_v3_passes_emitted: frame_perf.custom_effect_v3_passes_emitted,
            custom_effect_v3_user0_image_incompatible_fallbacks: frame_perf
                .custom_effect_v3_user0_image_incompatible_fallbacks,
            custom_effect_v3_user1_image_incompatible_fallbacks: frame_perf
                .custom_effect_v3_user1_image_incompatible_fallbacks,
            custom_effect_v3_pyramid_cache_hits: frame_perf.custom_effect_v3_pyramid_cache_hits,
            custom_effect_v3_pyramid_cache_misses: frame_perf.custom_effect_v3_pyramid_cache_misses,
            draw_calls: frame_perf.draw_calls,
            quad_draw_calls: frame_perf.quad_draw_calls,
            viewport_draw_calls: frame_perf.viewport_draw_calls,
            viewport_draw_calls_ingest_unknown: frame_perf.viewport_draw_calls_ingest_unknown,
            viewport_draw_calls_ingest_owned: frame_perf.viewport_draw_calls_ingest_owned,
            viewport_draw_calls_ingest_external_zero_copy: frame_perf
                .viewport_draw_calls_ingest_external_zero_copy,
            viewport_draw_calls_ingest_gpu_copy: frame_perf.viewport_draw_calls_ingest_gpu_copy,
            viewport_draw_calls_ingest_cpu_upload: frame_perf.viewport_draw_calls_ingest_cpu_upload,
            image_draw_calls: frame_perf.image_draw_calls,
            text_draw_calls: frame_perf.text_draw_calls,
            path_draw_calls: frame_perf.path_draw_calls,
            mask_draw_calls: frame_perf.mask_draw_calls,
            fullscreen_draw_calls: frame_perf.fullscreen_draw_calls,
            clip_mask_draw_calls: frame_perf.clip_mask_draw_calls,
            pipeline_switches: frame_perf.pipeline_switches,
            pipeline_switches_quad: frame_perf.pipeline_switches_quad,
            pipeline_switches_viewport: frame_perf.pipeline_switches_viewport,
            pipeline_switches_mask: frame_perf.pipeline_switches_mask,
            pipeline_switches_text_mask: frame_perf.pipeline_switches_text_mask,
            pipeline_switches_text_color: frame_perf.pipeline_switches_text_color,
            pipeline_switches_text_subpixel: frame_perf.pipeline_switches_text_subpixel,
            pipeline_switches_path: frame_perf.pipeline_switches_path,
            pipeline_switches_path_msaa: frame_perf.pipeline_switches_path_msaa,
            pipeline_switches_composite: frame_perf.pipeline_switches_composite,
            pipeline_switches_fullscreen: frame_perf.pipeline_switches_fullscreen,
            pipeline_switches_clip_mask: frame_perf.pipeline_switches_clip_mask,
            bind_group_switches: frame_perf.bind_group_switches,
            uniform_bind_group_switches: frame_perf.uniform_bind_group_switches,
            texture_bind_group_switches: frame_perf.texture_bind_group_switches,
            scissor_sets: frame_perf.scissor_sets,
            path_msaa_samples_requested: frame_perf.path_msaa_samples_requested,
            path_msaa_samples_effective: frame_perf.path_msaa_samples_effective,
            path_msaa_vulkan_safety_valve_degradations: frame_perf
                .path_msaa_vulkan_safety_valve_degradations,
            uniform_bytes: frame_perf.uniform_bytes,
            instance_bytes: frame_perf.instance_bytes,
            vertex_bytes: frame_perf.vertex_bytes,
            scene_encoding_cache_hits: frame_perf.scene_encoding_cache_hits,
            scene_encoding_cache_misses: frame_perf.scene_encoding_cache_misses,
            scene_encoding_cache_last_miss_reasons: frame_perf
                .scene_encoding_cache_last_miss_reasons,
            material_quad_ops: frame_perf.material_quad_ops,
            material_sampled_quad_ops: frame_perf.material_sampled_quad_ops,
            material_distinct: frame_perf.material_distinct,
            material_unknown_ids: frame_perf.material_unknown_ids,
            material_degraded_due_to_budget: frame_perf.material_degraded_due_to_budget,
            path_material_paints_degraded_to_solid_base: frame_perf
                .path_material_paints_degraded_to_solid_base,
            clip_path_mask_cache_bytes_live: frame_perf.clip_path_mask_cache_bytes_live,
            clip_path_mask_cache_entries_live: frame_perf.clip_path_mask_cache_entries_live,
            clip_path_mask_cache_hits: frame_perf.clip_path_mask_cache_hits,
            clip_path_mask_cache_misses: frame_perf.clip_path_mask_cache_misses,
        });
    }
}
