use super::super::*;
use super::helpers::render_plan_trace_fingerprint;
use super::uploads::FrameUploadResources;
use fret_core::time::Instant;

impl Renderer {
    pub(super) fn render_scene_execute(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        params: RenderSceneParams<'_>,
    ) -> wgpu::CommandBuffer {
        self.render_scene_frame_index = self.render_scene_frame_index.saturating_add(1);
        let frame_index = self.render_scene_frame_index;

        let RenderSceneParams {
            format,
            target_view,
            scene,
            clear,
            scale_factor,
            viewport_size,
        } = params;

        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        let render_scene_span = if trace_enabled {
            tracing::trace_span!(
                "fret.renderer.render_scene",
                frame_index,
                ops = scene.ops_len(),
                viewport_w = viewport_size.0,
                viewport_h = viewport_size.1,
                scale_factor,
                format = ?format,
                encoding_cache_hit = tracing::field::Empty,
                plan_passes = tracing::field::Empty,
                plan_segments = tracing::field::Empty,
                plan_degradations = tracing::field::Empty,
                plan_estimated_peak_intermediate_bytes = tracing::field::Empty,
                plan_fingerprint = tracing::field::Empty,
            )
        } else {
            tracing::Span::none()
        };
        let _render_scene_guard = render_scene_span.enter();

        let perf_enabled = self.perf_enabled;
        let mut frame_perf = RenderPerfStats::default();
        if perf_enabled {
            frame_perf.frames = 1;
            self.perf_svg_raster_cache_hits = 0;
            self.perf_svg_raster_cache_misses = 0;
            self.perf_svg_raster_budget_evictions = 0;
            self.perf_svg_mask_atlas_page_evictions = 0;
            self.perf_svg_mask_atlas_entries_evicted = 0;
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
            frame_perf.render_target_updates_requested_ingest_external_zero_copy =
                pending_requested[2];
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

        #[cfg(debug_assertions)]
        if let Err(e) = scene.validate() {
            panic!("invalid scene: {e}");
        }

        let path_samples = self.ensure_frame_pipelines_and_path_samples(
            device,
            queue,
            format,
            viewport_size,
            perf_enabled,
            trace_enabled,
            &mut frame_perf,
        );

        let text_atlas_revision = self.prepare_text_for_frame(
            queue,
            scene,
            frame_index,
            perf_enabled,
            trace_enabled,
            &mut frame_perf,
        );
        self.prepare_svg_for_frame(
            device,
            queue,
            scene,
            scale_factor,
            frame_index,
            perf_enabled,
            trace_enabled,
            &mut frame_perf,
        );

        let key = self.build_scene_encoding_cache_key(
            format,
            viewport_size,
            scale_factor,
            scene,
            text_atlas_revision,
        );

        let (encoding, cache_hit) = self.acquire_scene_encoding_for_frame(
            key,
            frame_index,
            scene,
            scale_factor,
            viewport_size,
            format.is_srgb(),
            perf_enabled,
            trace_enabled,
            &render_scene_span,
            &mut frame_perf,
        );

        if perf_enabled {
            frame_perf.material_quad_ops = frame_perf
                .material_quad_ops
                .saturating_add(encoding.material_quad_ops);
            frame_perf.material_sampled_quad_ops = frame_perf
                .material_sampled_quad_ops
                .saturating_add(encoding.material_sampled_quad_ops);
            frame_perf.material_distinct = frame_perf
                .material_distinct
                .saturating_add(encoding.material_distinct);
            frame_perf.material_unknown_ids = frame_perf
                .material_unknown_ids
                .saturating_add(encoding.material_unknown_ids);
            frame_perf.material_degraded_due_to_budget = frame_perf
                .material_degraded_due_to_budget
                .saturating_add(encoding.material_degraded_due_to_budget);
        }

        let postprocess = self.pick_debug_postprocess(viewport_size, format);
        let plan = self.compile_render_plan_for_scene(
            frame_index,
            perf_enabled,
            trace_enabled,
            &encoding,
            viewport_size,
            format,
            clear.0,
            path_samples,
            postprocess,
            &mut frame_perf,
        );
        plan.debug_validate();
        render_scene_span.record("plan_passes", plan.passes.len() as u64);
        render_scene_span.record("plan_segments", plan.segments.len() as u64);
        render_scene_span.record("plan_degradations", plan.degradations.len() as u64);
        render_scene_span.record(
            "plan_estimated_peak_intermediate_bytes",
            plan.compile_stats.estimated_peak_intermediate_bytes,
        );
        if trace_enabled {
            render_scene_span.record(
                "plan_fingerprint",
                render_plan_trace_fingerprint(&plan.passes),
            );
        }
        self.record_render_plan_diagnostics_for_frame(
            perf_enabled,
            &plan,
            viewport_size,
            format,
            frame_index,
            postprocess,
            encoding.ordered_draws.len(),
            &encoding.effect_markers,
            &mut frame_perf,
        );

        let upload_started = perf_enabled.then(Instant::now);
        let uploads_span = if trace_enabled {
            tracing::trace_span!(
                "fret.renderer.upload",
                frame_index,
                passes = plan.passes.len() as u32,
                uniforms = encoding.uniforms.len() as u32,
                clips = encoding.clips.len() as u32,
                masks = encoding.masks.len() as u32,
            )
        } else {
            tracing::Span::none()
        };
        let uploads_guard = uploads_span.enter();

        self.ensure_effect_pipelines_for_plan(device, format, path_samples, &plan);
        self.upload_frame_uniforms_and_prepare_bind_groups(
            device,
            queue,
            &encoding.uniforms,
            &encoding.clips,
            &encoding.masks,
            &encoding.ordered_draws,
            &encoding.uniform_mask_images,
            perf_enabled,
            &mut frame_perf,
        );

        let FrameUploadResources {
            quad_instance_bind_group,
            text_paint_bind_group,
            path_paint_bind_group,
            viewport_vertex_buffer,
            text_vertex_buffer,
            path_vertex_buffer,
            quad_vertex_bases,
        } = self.upload_frame_geometry(
            device,
            queue,
            &plan,
            viewport_size,
            &encoding.instances,
            &encoding.path_paints,
            &encoding.text_paints,
            &encoding.viewport_vertices,
            &encoding.text_vertices,
            &encoding.path_vertices,
            perf_enabled,
            &mut frame_perf,
        );

        drop(uploads_guard);
        if let Some(upload_started) = upload_started {
            frame_perf.upload += upload_started.elapsed();
        }

        self.upload_render_space_uniforms_for_plan(queue, &plan);

        let cmd = self.dispatch_render_plan(
            device,
            queue,
            frame_index,
            format,
            target_view,
            viewport_size,
            &plan,
            &encoding,
            &quad_vertex_bases,
            &viewport_vertex_buffer,
            &text_vertex_buffer,
            &path_vertex_buffer,
            &quad_instance_bind_group,
            &text_paint_bind_group,
            &path_paint_bind_group,
            perf_enabled,
            trace_enabled,
            &mut frame_perf,
        );

        if perf_enabled {
            self.finalize_frame_perf_after_dispatch(&mut frame_perf);
        }

        self.scene_encoding_cache
            .store_after_frame(key, cache_hit, encoding);
        cmd
    }
}

// FrameTargets moved to `renderer/frame_targets.rs`.
