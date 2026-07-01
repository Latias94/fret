use super::super::*;

impl Renderer {
    pub(super) fn finalize_frame_perf_after_dispatch(&mut self, frame_perf: &mut RenderPerfStats) {
        self.svg_raster_state.write_frame_perf(frame_perf);

        frame_perf.clip_path_mask_cache_bytes_live = self.clip_path_mask_cache.bytes_live();
        frame_perf.clip_path_mask_cache_entries_live = self.clip_path_mask_cache.entries_live();

        let pool_perf = self.intermediate_state.pool.take_perf_snapshot();
        frame_perf.intermediate_pool_allocations = pool_perf.allocations;
        frame_perf.intermediate_pool_reuses = pool_perf.reuses;
        frame_perf.intermediate_pool_releases = pool_perf.releases;
        frame_perf.intermediate_pool_evictions = pool_perf.evictions;
        frame_perf.intermediate_pool_free_bytes = pool_perf.free_bytes;
        frame_perf.intermediate_pool_free_textures = pool_perf.free_textures;
        let perf = &mut self.diagnostics_state.perf;

        perf.frames = perf.frames.saturating_add(frame_perf.frames);
        perf.encode_scene += frame_perf.encode_scene;
        perf.ensure_pipelines += frame_perf.ensure_pipelines;
        perf.plan_compile += frame_perf.plan_compile;
        perf.upload += frame_perf.upload;
        perf.record_passes += frame_perf.record_passes;
        perf.encoder_finish += frame_perf.encoder_finish;
        perf.prepare_svg += frame_perf.prepare_svg;
        perf.prepare_text += frame_perf.prepare_text;
        perf.encode_scene_stack += frame_perf.encode_scene_stack;
        perf.encode_scene_clip += frame_perf.encode_scene_clip;
        perf.encode_scene_mask += frame_perf.encode_scene_mask;
        perf.encode_scene_effect += frame_perf.encode_scene_effect;
        perf.encode_scene_quad += frame_perf.encode_scene_quad;
        perf.encode_scene_image += frame_perf.encode_scene_image;
        perf.encode_scene_text += frame_perf.encode_scene_text;
        perf.encode_scene_path += frame_perf.encode_scene_path;
        perf.encode_scene_viewport += frame_perf.encode_scene_viewport;
        perf.encode_scene_flush += frame_perf.encode_scene_flush;
        perf.encode_scene_text_shadow += frame_perf.encode_scene_text_shadow;
        perf.encode_scene_text_setup += frame_perf.encode_scene_text_setup;
        perf.encode_scene_text_glyphs += frame_perf.encode_scene_text_glyphs;
        perf.encode_scene_text_glyph_transform += frame_perf.encode_scene_text_glyph_transform;
        perf.encode_scene_text_glyph_emit += frame_perf.encode_scene_text_glyph_emit;
        perf.encode_scene_text_group_flush += frame_perf.encode_scene_text_group_flush;
        perf.encode_scene_text_vertex_grow_events = perf
            .encode_scene_text_vertex_grow_events
            .saturating_add(frame_perf.encode_scene_text_vertex_grow_events);
        perf.encode_scene_text_transform_fast_path_glyphs = perf
            .encode_scene_text_transform_fast_path_glyphs
            .saturating_add(frame_perf.encode_scene_text_transform_fast_path_glyphs);
        perf.encode_scene_text_transform_generic_glyphs = perf
            .encode_scene_text_transform_generic_glyphs
            .saturating_add(frame_perf.encode_scene_text_transform_generic_glyphs);
        perf.encode_scene_stack_ops = perf
            .encode_scene_stack_ops
            .saturating_add(frame_perf.encode_scene_stack_ops);
        perf.encode_scene_clip_ops = perf
            .encode_scene_clip_ops
            .saturating_add(frame_perf.encode_scene_clip_ops);
        perf.encode_scene_mask_ops = perf
            .encode_scene_mask_ops
            .saturating_add(frame_perf.encode_scene_mask_ops);
        perf.encode_scene_effect_ops = perf
            .encode_scene_effect_ops
            .saturating_add(frame_perf.encode_scene_effect_ops);
        perf.encode_scene_quad_ops = perf
            .encode_scene_quad_ops
            .saturating_add(frame_perf.encode_scene_quad_ops);
        perf.encode_scene_image_ops = perf
            .encode_scene_image_ops
            .saturating_add(frame_perf.encode_scene_image_ops);
        perf.encode_scene_text_ops = perf
            .encode_scene_text_ops
            .saturating_add(frame_perf.encode_scene_text_ops);
        perf.encode_scene_path_ops = perf
            .encode_scene_path_ops
            .saturating_add(frame_perf.encode_scene_path_ops);
        perf.encode_scene_viewport_ops = perf
            .encode_scene_viewport_ops
            .saturating_add(frame_perf.encode_scene_viewport_ops);
        perf.encode_scene_flushes = perf
            .encode_scene_flushes
            .saturating_add(frame_perf.encode_scene_flushes);

        perf.svg_uploads = perf.svg_uploads.saturating_add(frame_perf.svg_uploads);
        perf.svg_upload_bytes = perf
            .svg_upload_bytes
            .saturating_add(frame_perf.svg_upload_bytes);
        perf.image_uploads = perf.image_uploads.saturating_add(frame_perf.image_uploads);
        perf.image_upload_bytes = perf
            .image_upload_bytes
            .saturating_add(frame_perf.image_upload_bytes);

        perf.render_target_updates_ingest_unknown = perf
            .render_target_updates_ingest_unknown
            .saturating_add(frame_perf.render_target_updates_ingest_unknown);
        perf.render_target_updates_ingest_owned = perf
            .render_target_updates_ingest_owned
            .saturating_add(frame_perf.render_target_updates_ingest_owned);
        perf.render_target_updates_ingest_external_zero_copy = perf
            .render_target_updates_ingest_external_zero_copy
            .saturating_add(frame_perf.render_target_updates_ingest_external_zero_copy);
        perf.render_target_updates_ingest_gpu_copy = perf
            .render_target_updates_ingest_gpu_copy
            .saturating_add(frame_perf.render_target_updates_ingest_gpu_copy);
        perf.render_target_updates_ingest_cpu_upload = perf
            .render_target_updates_ingest_cpu_upload
            .saturating_add(frame_perf.render_target_updates_ingest_cpu_upload);

        perf.render_target_updates_requested_ingest_unknown = perf
            .render_target_updates_requested_ingest_unknown
            .saturating_add(frame_perf.render_target_updates_requested_ingest_unknown);
        perf.render_target_updates_requested_ingest_owned = perf
            .render_target_updates_requested_ingest_owned
            .saturating_add(frame_perf.render_target_updates_requested_ingest_owned);
        perf.render_target_updates_requested_ingest_external_zero_copy = perf
            .render_target_updates_requested_ingest_external_zero_copy
            .saturating_add(frame_perf.render_target_updates_requested_ingest_external_zero_copy);
        perf.render_target_updates_requested_ingest_gpu_copy = perf
            .render_target_updates_requested_ingest_gpu_copy
            .saturating_add(frame_perf.render_target_updates_requested_ingest_gpu_copy);
        perf.render_target_updates_requested_ingest_cpu_upload = perf
            .render_target_updates_requested_ingest_cpu_upload
            .saturating_add(frame_perf.render_target_updates_requested_ingest_cpu_upload);
        perf.render_target_updates_ingest_fallbacks = perf
            .render_target_updates_ingest_fallbacks
            .saturating_add(frame_perf.render_target_updates_ingest_fallbacks);
        perf.render_target_metadata_degradations_color_encoding_dropped = perf
            .render_target_metadata_degradations_color_encoding_dropped
            .saturating_add(frame_perf.render_target_metadata_degradations_color_encoding_dropped);

        perf.svg_raster_budget_bytes = frame_perf.svg_raster_budget_bytes;
        perf.svg_rasters_live = perf.svg_rasters_live.max(frame_perf.svg_rasters_live);
        perf.svg_standalone_bytes_live = perf
            .svg_standalone_bytes_live
            .max(frame_perf.svg_standalone_bytes_live);
        perf.svg_mask_atlas_pages_live = perf
            .svg_mask_atlas_pages_live
            .max(frame_perf.svg_mask_atlas_pages_live);
        perf.svg_mask_atlas_bytes_live = perf
            .svg_mask_atlas_bytes_live
            .max(frame_perf.svg_mask_atlas_bytes_live);
        perf.svg_mask_atlas_used_px = perf
            .svg_mask_atlas_used_px
            .max(frame_perf.svg_mask_atlas_used_px);
        perf.svg_mask_atlas_capacity_px = perf
            .svg_mask_atlas_capacity_px
            .max(frame_perf.svg_mask_atlas_capacity_px);
        perf.svg_raster_cache_hits = perf
            .svg_raster_cache_hits
            .saturating_add(frame_perf.svg_raster_cache_hits);
        perf.svg_raster_cache_misses = perf
            .svg_raster_cache_misses
            .saturating_add(frame_perf.svg_raster_cache_misses);
        perf.svg_raster_budget_evictions = perf
            .svg_raster_budget_evictions
            .saturating_add(frame_perf.svg_raster_budget_evictions);
        perf.svg_mask_atlas_page_evictions = perf
            .svg_mask_atlas_page_evictions
            .saturating_add(frame_perf.svg_mask_atlas_page_evictions);
        perf.svg_mask_atlas_entries_evicted = perf
            .svg_mask_atlas_entries_evicted
            .saturating_add(frame_perf.svg_mask_atlas_entries_evicted);

        perf.text_atlas_revision = frame_perf.text_atlas_revision;
        perf.text_atlas_uploads = perf
            .text_atlas_uploads
            .saturating_add(frame_perf.text_atlas_uploads);
        perf.text_atlas_upload_bytes = perf
            .text_atlas_upload_bytes
            .saturating_add(frame_perf.text_atlas_upload_bytes);
        perf.text_atlas_evicted_glyphs = perf
            .text_atlas_evicted_glyphs
            .saturating_add(frame_perf.text_atlas_evicted_glyphs);
        perf.text_atlas_evicted_pages = perf
            .text_atlas_evicted_pages
            .saturating_add(frame_perf.text_atlas_evicted_pages);
        perf.text_atlas_evicted_page_glyphs = perf
            .text_atlas_evicted_page_glyphs
            .saturating_add(frame_perf.text_atlas_evicted_page_glyphs);
        perf.text_atlas_resets = perf
            .text_atlas_resets
            .saturating_add(frame_perf.text_atlas_resets);

        perf.intermediate_budget_bytes = frame_perf.intermediate_budget_bytes;
        perf.intermediate_full_target_bytes = frame_perf.intermediate_full_target_bytes;
        perf.intermediate_in_use_bytes = perf
            .intermediate_in_use_bytes
            .max(frame_perf.intermediate_in_use_bytes);
        perf.intermediate_peak_in_use_bytes = perf
            .intermediate_peak_in_use_bytes
            .max(frame_perf.intermediate_peak_in_use_bytes);
        perf.intermediate_release_targets = perf
            .intermediate_release_targets
            .saturating_add(frame_perf.intermediate_release_targets);
        perf.intermediate_pool_allocations = perf
            .intermediate_pool_allocations
            .saturating_add(frame_perf.intermediate_pool_allocations);
        perf.intermediate_pool_reuses = perf
            .intermediate_pool_reuses
            .saturating_add(frame_perf.intermediate_pool_reuses);
        perf.intermediate_pool_releases = perf
            .intermediate_pool_releases
            .saturating_add(frame_perf.intermediate_pool_releases);
        perf.intermediate_pool_evictions = perf
            .intermediate_pool_evictions
            .saturating_add(frame_perf.intermediate_pool_evictions);
        perf.intermediate_pool_free_bytes = pool_perf.free_bytes;
        perf.intermediate_pool_free_textures = pool_perf.free_textures;
        perf.scene_chunk_input_chunks = perf
            .scene_chunk_input_chunks
            .max(frame_perf.scene_chunk_input_chunks);
        perf.scene_chunk_input_ops = perf
            .scene_chunk_input_ops
            .max(frame_perf.scene_chunk_input_ops);
        if frame_perf.scene_chunk_input_fingerprint != 0 {
            perf.scene_chunk_input_fingerprint = frame_perf.scene_chunk_input_fingerprint;
        }
        perf.render_plan_estimated_peak_intermediate_bytes = perf
            .render_plan_estimated_peak_intermediate_bytes
            .max(frame_perf.render_plan_estimated_peak_intermediate_bytes);
        perf.render_plan_segments = perf
            .render_plan_segments
            .max(frame_perf.render_plan_segments);
        perf.render_plan_scene_chunk_candidates = perf
            .render_plan_scene_chunk_candidates
            .max(frame_perf.render_plan_scene_chunk_candidates);
        perf.render_plan_scene_chunk_candidate_draws = perf
            .render_plan_scene_chunk_candidate_draws
            .max(frame_perf.render_plan_scene_chunk_candidate_draws);
        perf.render_plan_scene_chunk_candidate_upload_bytes_estimate = perf
            .render_plan_scene_chunk_candidate_upload_bytes_estimate
            .max(frame_perf.render_plan_scene_chunk_candidate_upload_bytes_estimate);
        perf.render_plan_degradations = perf
            .render_plan_degradations
            .saturating_add(frame_perf.render_plan_degradations);
        perf.render_plan_effect_chain_budget_samples =
            frame_perf.render_plan_effect_chain_budget_samples;
        perf.render_plan_effect_chain_effective_budget_min_bytes =
            frame_perf.render_plan_effect_chain_effective_budget_min_bytes;
        perf.render_plan_effect_chain_effective_budget_max_bytes =
            frame_perf.render_plan_effect_chain_effective_budget_max_bytes;
        perf.render_plan_effect_chain_other_live_max_bytes =
            frame_perf.render_plan_effect_chain_other_live_max_bytes;
        perf.render_plan_custom_effect_chain_budget_samples =
            frame_perf.render_plan_custom_effect_chain_budget_samples;
        perf.render_plan_custom_effect_chain_effective_budget_min_bytes =
            frame_perf.render_plan_custom_effect_chain_effective_budget_min_bytes;
        perf.render_plan_custom_effect_chain_effective_budget_max_bytes =
            frame_perf.render_plan_custom_effect_chain_effective_budget_max_bytes;
        perf.render_plan_custom_effect_chain_other_live_max_bytes =
            frame_perf.render_plan_custom_effect_chain_other_live_max_bytes;
        perf.render_plan_custom_effect_chain_base_required_max_bytes =
            frame_perf.render_plan_custom_effect_chain_base_required_max_bytes;
        perf.render_plan_custom_effect_chain_optional_required_max_bytes =
            frame_perf.render_plan_custom_effect_chain_optional_required_max_bytes;
        perf.render_plan_custom_effect_chain_base_required_full_targets_max =
            frame_perf.render_plan_custom_effect_chain_base_required_full_targets_max;
        perf.render_plan_custom_effect_chain_optional_mask_max_bytes =
            frame_perf.render_plan_custom_effect_chain_optional_mask_max_bytes;
        perf.render_plan_custom_effect_chain_optional_pyramid_max_bytes =
            frame_perf.render_plan_custom_effect_chain_optional_pyramid_max_bytes;
        perf.render_plan_segments_changed = perf
            .render_plan_segments_changed
            .saturating_add(frame_perf.render_plan_segments_changed);
        perf.render_plan_segments_passes_increased = perf
            .render_plan_segments_passes_increased
            .saturating_add(frame_perf.render_plan_segments_passes_increased);
        perf.render_plan_scene_chunk_candidates_stable = perf
            .render_plan_scene_chunk_candidates_stable
            .saturating_add(frame_perf.render_plan_scene_chunk_candidates_stable);
        perf.render_plan_scene_chunk_candidates_changed = perf
            .render_plan_scene_chunk_candidates_changed
            .saturating_add(frame_perf.render_plan_scene_chunk_candidates_changed);
        perf.render_plan_scene_chunk_candidate_stream_ranges_changed = perf
            .render_plan_scene_chunk_candidate_stream_ranges_changed
            .saturating_add(frame_perf.render_plan_scene_chunk_candidate_stream_ranges_changed);
        perf.render_plan_degradations_budget_zero = perf
            .render_plan_degradations_budget_zero
            .saturating_add(frame_perf.render_plan_degradations_budget_zero);
        perf.render_plan_degradations_budget_insufficient = perf
            .render_plan_degradations_budget_insufficient
            .saturating_add(frame_perf.render_plan_degradations_budget_insufficient);
        perf.render_plan_degradations_target_exhausted = perf
            .render_plan_degradations_target_exhausted
            .saturating_add(frame_perf.render_plan_degradations_target_exhausted);
        perf.render_plan_degradations_backdrop_noop = perf
            .render_plan_degradations_backdrop_noop
            .saturating_add(frame_perf.render_plan_degradations_backdrop_noop);
        perf.render_plan_degradations_filter_content_disabled = perf
            .render_plan_degradations_filter_content_disabled
            .saturating_add(frame_perf.render_plan_degradations_filter_content_disabled);
        perf.render_plan_degradations_clip_path_disabled = perf
            .render_plan_degradations_clip_path_disabled
            .saturating_add(frame_perf.render_plan_degradations_clip_path_disabled);
        perf.render_plan_degradations_composite_group_blend_to_over = perf
            .render_plan_degradations_composite_group_blend_to_over
            .saturating_add(frame_perf.render_plan_degradations_composite_group_blend_to_over);
        perf.effect_degradations
            .saturating_add_assign(frame_perf.effect_degradations);
        perf.effect_blur_quality
            .saturating_add_assign(frame_perf.effect_blur_quality);
        perf.custom_effect_v1_steps_requested = perf
            .custom_effect_v1_steps_requested
            .saturating_add(frame_perf.custom_effect_v1_steps_requested);
        perf.custom_effect_v1_passes_emitted = perf
            .custom_effect_v1_passes_emitted
            .saturating_add(frame_perf.custom_effect_v1_passes_emitted);
        perf.custom_effect_v2_steps_requested = perf
            .custom_effect_v2_steps_requested
            .saturating_add(frame_perf.custom_effect_v2_steps_requested);
        perf.custom_effect_v2_passes_emitted = perf
            .custom_effect_v2_passes_emitted
            .saturating_add(frame_perf.custom_effect_v2_passes_emitted);
        perf.custom_effect_v2_user_image_incompatible_fallbacks = perf
            .custom_effect_v2_user_image_incompatible_fallbacks
            .saturating_add(frame_perf.custom_effect_v2_user_image_incompatible_fallbacks);
        perf.custom_effect_v3_pyramid_cache_hits = perf
            .custom_effect_v3_pyramid_cache_hits
            .saturating_add(frame_perf.custom_effect_v3_pyramid_cache_hits);
        perf.custom_effect_v3_pyramid_cache_misses = perf
            .custom_effect_v3_pyramid_cache_misses
            .saturating_add(frame_perf.custom_effect_v3_pyramid_cache_misses);
        perf.custom_effect_v3_steps_requested = perf
            .custom_effect_v3_steps_requested
            .saturating_add(frame_perf.custom_effect_v3_steps_requested);
        perf.custom_effect_v3_passes_emitted = perf
            .custom_effect_v3_passes_emitted
            .saturating_add(frame_perf.custom_effect_v3_passes_emitted);
        perf.custom_effect_v3_user0_image_incompatible_fallbacks = perf
            .custom_effect_v3_user0_image_incompatible_fallbacks
            .saturating_add(frame_perf.custom_effect_v3_user0_image_incompatible_fallbacks);
        perf.custom_effect_v3_user1_image_incompatible_fallbacks = perf
            .custom_effect_v3_user1_image_incompatible_fallbacks
            .saturating_add(frame_perf.custom_effect_v3_user1_image_incompatible_fallbacks);

        perf.clip_path_mask_cache_bytes_live = perf
            .clip_path_mask_cache_bytes_live
            .max(frame_perf.clip_path_mask_cache_bytes_live);
        perf.clip_path_mask_cache_entries_live = perf
            .clip_path_mask_cache_entries_live
            .max(frame_perf.clip_path_mask_cache_entries_live);
        perf.clip_path_mask_cache_hits = perf
            .clip_path_mask_cache_hits
            .saturating_add(frame_perf.clip_path_mask_cache_hits);
        perf.clip_path_mask_cache_misses = perf
            .clip_path_mask_cache_misses
            .saturating_add(frame_perf.clip_path_mask_cache_misses);

        perf.draw_calls = perf.draw_calls.saturating_add(frame_perf.draw_calls);
        perf.quad_draw_calls = perf
            .quad_draw_calls
            .saturating_add(frame_perf.quad_draw_calls);
        perf.viewport_draw_calls = perf
            .viewport_draw_calls
            .saturating_add(frame_perf.viewport_draw_calls);
        perf.viewport_draw_calls_ingest_unknown = perf
            .viewport_draw_calls_ingest_unknown
            .saturating_add(frame_perf.viewport_draw_calls_ingest_unknown);
        perf.viewport_draw_calls_ingest_owned = perf
            .viewport_draw_calls_ingest_owned
            .saturating_add(frame_perf.viewport_draw_calls_ingest_owned);
        perf.viewport_draw_calls_ingest_external_zero_copy = perf
            .viewport_draw_calls_ingest_external_zero_copy
            .saturating_add(frame_perf.viewport_draw_calls_ingest_external_zero_copy);
        perf.viewport_draw_calls_ingest_gpu_copy = perf
            .viewport_draw_calls_ingest_gpu_copy
            .saturating_add(frame_perf.viewport_draw_calls_ingest_gpu_copy);
        perf.viewport_draw_calls_ingest_cpu_upload = perf
            .viewport_draw_calls_ingest_cpu_upload
            .saturating_add(frame_perf.viewport_draw_calls_ingest_cpu_upload);
        perf.image_draw_calls = perf
            .image_draw_calls
            .saturating_add(frame_perf.image_draw_calls);
        perf.text_draw_calls = perf
            .text_draw_calls
            .saturating_add(frame_perf.text_draw_calls);
        perf.path_draw_calls = perf
            .path_draw_calls
            .saturating_add(frame_perf.path_draw_calls);
        perf.mask_draw_calls = perf
            .mask_draw_calls
            .saturating_add(frame_perf.mask_draw_calls);
        perf.fullscreen_draw_calls = perf
            .fullscreen_draw_calls
            .saturating_add(frame_perf.fullscreen_draw_calls);
        perf.clip_mask_draw_calls = perf
            .clip_mask_draw_calls
            .saturating_add(frame_perf.clip_mask_draw_calls);
        perf.pipeline_switches = perf
            .pipeline_switches
            .saturating_add(frame_perf.pipeline_switches);
        perf.pipeline_switches_quad = perf
            .pipeline_switches_quad
            .saturating_add(frame_perf.pipeline_switches_quad);
        perf.pipeline_switches_viewport = perf
            .pipeline_switches_viewport
            .saturating_add(frame_perf.pipeline_switches_viewport);
        perf.pipeline_switches_mask = perf
            .pipeline_switches_mask
            .saturating_add(frame_perf.pipeline_switches_mask);
        perf.pipeline_switches_text_mask = perf
            .pipeline_switches_text_mask
            .saturating_add(frame_perf.pipeline_switches_text_mask);
        perf.pipeline_switches_text_color = perf
            .pipeline_switches_text_color
            .saturating_add(frame_perf.pipeline_switches_text_color);
        perf.pipeline_switches_text_subpixel = perf
            .pipeline_switches_text_subpixel
            .saturating_add(frame_perf.pipeline_switches_text_subpixel);
        perf.pipeline_switches_path = perf
            .pipeline_switches_path
            .saturating_add(frame_perf.pipeline_switches_path);
        perf.pipeline_switches_path_msaa = perf
            .pipeline_switches_path_msaa
            .saturating_add(frame_perf.pipeline_switches_path_msaa);
        perf.pipeline_switches_composite = perf
            .pipeline_switches_composite
            .saturating_add(frame_perf.pipeline_switches_composite);
        perf.pipeline_switches_fullscreen = perf
            .pipeline_switches_fullscreen
            .saturating_add(frame_perf.pipeline_switches_fullscreen);
        perf.pipeline_switches_clip_mask = perf
            .pipeline_switches_clip_mask
            .saturating_add(frame_perf.pipeline_switches_clip_mask);
        perf.bind_group_switches = perf
            .bind_group_switches
            .saturating_add(frame_perf.bind_group_switches);
        perf.uniform_bind_group_switches = perf
            .uniform_bind_group_switches
            .saturating_add(frame_perf.uniform_bind_group_switches);
        perf.texture_bind_group_switches = perf
            .texture_bind_group_switches
            .saturating_add(frame_perf.texture_bind_group_switches);
        perf.scissor_sets = perf.scissor_sets.saturating_add(frame_perf.scissor_sets);
        perf.path_msaa_samples_requested = frame_perf.path_msaa_samples_requested;
        perf.path_msaa_samples_effective = frame_perf.path_msaa_samples_effective;
        perf.path_msaa_vulkan_safety_valve_degradations = perf
            .path_msaa_vulkan_safety_valve_degradations
            .saturating_add(frame_perf.path_msaa_vulkan_safety_valve_degradations);
        perf.uniform_bytes = perf.uniform_bytes.saturating_add(frame_perf.uniform_bytes);
        perf.instance_bytes = perf
            .instance_bytes
            .saturating_add(frame_perf.instance_bytes);
        perf.vertex_bytes = perf.vertex_bytes.saturating_add(frame_perf.vertex_bytes);
        perf.geometry_upload
            .saturating_add_assign(frame_perf.geometry_upload);
        perf.scene_encoding_cache_hits = perf
            .scene_encoding_cache_hits
            .saturating_add(frame_perf.scene_encoding_cache_hits);
        perf.scene_encoding_cache_misses = perf
            .scene_encoding_cache_misses
            .saturating_add(frame_perf.scene_encoding_cache_misses);
        if frame_perf.scene_encoding_cache_last_miss_reasons != 0 {
            perf.scene_encoding_cache_last_miss_reasons =
                frame_perf.scene_encoding_cache_last_miss_reasons;
        }
        perf.scene_encoding_cache_miss_histogram
            .saturating_add_assign(frame_perf.scene_encoding_cache_miss_histogram);
        perf.material_quad_ops = perf
            .material_quad_ops
            .saturating_add(frame_perf.material_quad_ops);
        perf.material_sampled_quad_ops = perf
            .material_sampled_quad_ops
            .saturating_add(frame_perf.material_sampled_quad_ops);
        perf.material_distinct = perf
            .material_distinct
            .saturating_add(frame_perf.material_distinct);
        perf.material_unknown_ids = perf
            .material_unknown_ids
            .saturating_add(frame_perf.material_unknown_ids);
        perf.material_degraded_due_to_budget = perf
            .material_degraded_due_to_budget
            .saturating_add(frame_perf.material_degraded_due_to_budget);
        perf.path_material_paints_degraded_to_solid_base = perf
            .path_material_paints_degraded_to_solid_base
            .saturating_add(frame_perf.path_material_paints_degraded_to_solid_base);

        let registry_est = self.gpu_resources.diagnostics_estimated_bytes();
        let path_intermediate_msaa_bytes_estimate =
            self.path_state.intermediate_msaa_bytes_estimate();
        let path_intermediate_resolved_bytes_estimate =
            self.path_state.intermediate_resolved_bytes_estimate();
        let path_intermediate_bytes_estimate = self.path_state.intermediate_bytes_estimate();
        let custom_effect_v3_pyramid_scratch_bytes_estimate =
            self.custom_effect_v3_pyramid.scratch_bytes_estimate();

        self.diagnostics_state.last_frame_perf = Some(RenderPerfSnapshot {
            frames: frame_perf.frames,
            encode_scene_us: frame_perf.encode_scene.as_micros() as u64,
            ensure_pipelines_us: frame_perf.ensure_pipelines.as_micros() as u64,
            plan_compile_us: frame_perf.plan_compile.as_micros() as u64,
            upload_us: frame_perf.upload.as_micros() as u64,
            record_passes_us: frame_perf.record_passes.as_micros() as u64,
            encoder_finish_us: frame_perf.encoder_finish.as_micros() as u64,
            prepare_svg_us: frame_perf.prepare_svg.as_micros() as u64,
            prepare_text_us: frame_perf.prepare_text.as_micros() as u64,
            prepare_text_collect_pin_keys_us: frame_perf.prepare_text_collect_pin_keys.as_micros()
                as u64,
            prepare_text_bucket_delta_us: frame_perf.prepare_text_bucket_delta.as_micros() as u64,
            prepare_text_prewarm_us: frame_perf.prepare_text_prewarm.as_micros() as u64,
            prepare_text_pin_bucket_update_us: frame_perf.prepare_text_pin_bucket_update.as_micros()
                as u64,
            prepare_text_flush_uploads_us: frame_perf.prepare_text_flush_uploads.as_micros() as u64,
            prepare_text_scene_text_blobs: frame_perf.prepare_text_scene_text_blobs,
            prepare_text_fast_scene_bucket_reuses: frame_perf.prepare_text_fast_scene_bucket_reuses,
            prepare_text_pinned_glyph_keys: frame_perf.prepare_text_pinned_glyph_keys,
            prepare_text_prewarm_glyph_keys: frame_perf.prepare_text_prewarm_glyph_keys,
            prepare_text_retained_glyph_keys: frame_perf.prepare_text_retained_glyph_keys,
            prepare_text_added_glyph_keys: frame_perf.prepare_text_added_glyph_keys,
            prepare_text_removed_glyph_keys: frame_perf.prepare_text_removed_glyph_keys,
            encode_scene_stack_us: frame_perf.encode_scene_stack.as_micros() as u64,
            encode_scene_clip_us: frame_perf.encode_scene_clip.as_micros() as u64,
            encode_scene_mask_us: frame_perf.encode_scene_mask.as_micros() as u64,
            encode_scene_effect_us: frame_perf.encode_scene_effect.as_micros() as u64,
            encode_scene_quad_us: frame_perf.encode_scene_quad.as_micros() as u64,
            encode_scene_image_us: frame_perf.encode_scene_image.as_micros() as u64,
            encode_scene_text_us: frame_perf.encode_scene_text.as_micros() as u64,
            encode_scene_path_us: frame_perf.encode_scene_path.as_micros() as u64,
            encode_scene_viewport_us: frame_perf.encode_scene_viewport.as_micros() as u64,
            encode_scene_flush_us: frame_perf.encode_scene_flush.as_micros() as u64,
            encode_scene_text_shadow_us: frame_perf.encode_scene_text_shadow.as_micros() as u64,
            encode_scene_text_setup_us: frame_perf.encode_scene_text_setup.as_micros() as u64,
            encode_scene_text_glyphs_us: frame_perf.encode_scene_text_glyphs.as_micros() as u64,
            encode_scene_text_glyph_transform_us: frame_perf
                .encode_scene_text_glyph_transform
                .as_micros() as u64,
            encode_scene_text_glyph_emit_us: frame_perf.encode_scene_text_glyph_emit.as_micros()
                as u64,
            encode_scene_text_group_flush_us: frame_perf.encode_scene_text_group_flush.as_micros()
                as u64,
            encode_scene_text_vertex_grow_events: frame_perf.encode_scene_text_vertex_grow_events,
            encode_scene_text_transform_fast_path_glyphs: frame_perf
                .encode_scene_text_transform_fast_path_glyphs,
            encode_scene_text_transform_generic_glyphs: frame_perf
                .encode_scene_text_transform_generic_glyphs,
            encode_scene_stack_ops: frame_perf.encode_scene_stack_ops,
            encode_scene_clip_ops: frame_perf.encode_scene_clip_ops,
            encode_scene_mask_ops: frame_perf.encode_scene_mask_ops,
            encode_scene_effect_ops: frame_perf.encode_scene_effect_ops,
            encode_scene_quad_ops: frame_perf.encode_scene_quad_ops,
            encode_scene_image_ops: frame_perf.encode_scene_image_ops,
            encode_scene_text_ops: frame_perf.encode_scene_text_ops,
            encode_scene_path_ops: frame_perf.encode_scene_path_ops,
            encode_scene_viewport_ops: frame_perf.encode_scene_viewport_ops,
            encode_scene_flushes: frame_perf.encode_scene_flushes,
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
            scene_chunk_input_chunks: frame_perf.scene_chunk_input_chunks,
            scene_chunk_input_ops: frame_perf.scene_chunk_input_ops,
            scene_chunk_input_fingerprint: frame_perf.scene_chunk_input_fingerprint,
            render_plan_estimated_peak_intermediate_bytes: frame_perf
                .render_plan_estimated_peak_intermediate_bytes,
            render_plan_segments: frame_perf.render_plan_segments,
            render_plan_segments_changed: frame_perf.render_plan_segments_changed,
            render_plan_segments_passes_increased: frame_perf.render_plan_segments_passes_increased,
            render_plan_scene_chunk_candidates: frame_perf.render_plan_scene_chunk_candidates,
            render_plan_scene_chunk_candidate_draws: frame_perf
                .render_plan_scene_chunk_candidate_draws,
            render_plan_scene_chunk_candidates_stable: frame_perf
                .render_plan_scene_chunk_candidates_stable,
            render_plan_scene_chunk_candidates_changed: frame_perf
                .render_plan_scene_chunk_candidates_changed,
            render_plan_scene_chunk_candidate_upload_bytes_estimate: frame_perf
                .render_plan_scene_chunk_candidate_upload_bytes_estimate,
            render_plan_scene_chunk_candidate_stream_ranges_changed: frame_perf
                .render_plan_scene_chunk_candidate_stream_ranges_changed,
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
            geometry_upload: frame_perf.geometry_upload,
            scene_encoding_cache_hits: frame_perf.scene_encoding_cache_hits,
            scene_encoding_cache_misses: frame_perf.scene_encoding_cache_misses,
            scene_encoding_cache_last_miss_reasons: frame_perf
                .scene_encoding_cache_last_miss_reasons,
            scene_encoding_cache_miss_histogram: frame_perf.scene_encoding_cache_miss_histogram,
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
