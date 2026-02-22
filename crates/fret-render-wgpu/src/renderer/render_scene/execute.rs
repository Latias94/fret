use super::super::frame_targets::{FrameTargets, downsampled_size};
use super::super::*;
use super::executor::{RecordPassCtx, RecordPassResources, RenderSceneExecutor};
use super::helpers::{
    render_plan_pass_render_space, render_plan_pass_trace_kind, render_plan_pass_trace_meta,
    render_plan_trace_fingerprint,
};
use super::quad_vertices::upload_plan_quad_vertices;
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

                self.ensure_viewport_pipeline(device, format);
                self.ensure_quad_pipelines(format);
                self.ensure_text_pipeline(device, format);
                self.ensure_text_color_pipeline(device, format);
                self.ensure_text_subpixel_pipeline(device, format);
                self.ensure_mask_pipeline(device, format);
                self.ensure_path_pipeline(device, format);
                self.ensure_path_clip_mask_pipeline(device);
                let path_samples = self.effective_path_msaa_samples(format);
                span.record("path_samples", path_samples);
                if path_samples > 1 {
                    self.ensure_composite_pipeline(device, format);
                    self.ensure_path_msaa_pipeline(device, format, path_samples);
                    self.ensure_path_intermediate(device, viewport_size, format, path_samples);
                }
                path_samples
            },
        );
        if let Some(ensure_elapsed) = ensure_elapsed {
            frame_perf.ensure_pipelines += ensure_elapsed;
        }

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
            frame_perf.intermediate_budget_bytes = self.intermediate_budget_bytes;
        }
        if let Some(text_prepare_start) = text_prepare_start {
            frame_perf.prepare_text += text_prepare_start.elapsed();
        }
        if self.svg_perf_enabled {
            self.svg_perf.frames = self.svg_perf.frames.saturating_add(1);
        }
        if self.intermediate_perf_enabled {
            self.intermediate_perf.frames = self.intermediate_perf.frames.saturating_add(1);
        }
        self.bump_svg_raster_epoch();
        let svg_prepare_start = self.svg_perf_enabled.then(Instant::now);
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
            self.svg_perf.prepare_svg_ops += svg_prepare_start.elapsed();
        }
        if let Some(perf_svg_prepare_start) = perf_svg_prepare_start {
            frame_perf.prepare_svg += perf_svg_prepare_start.elapsed();
        }

        let (render_targets_generation, images_generation) = self.gpu_resources.generations();
        let key = SceneEncodingCacheKey {
            format,
            viewport_size,
            scale_factor_bits: scale_factor.to_bits(),
            scene_fingerprint: scene.fingerprint(),
            scene_ops_len: scene.ops_len(),
            render_targets_generation,
            images_generation,
            text_atlas_revision,
            text_quality_key: self.text_system.text_quality_key(),
        };

        let cache_hit = self.scene_encoding_cache_key == Some(key);
        render_scene_span.record("encoding_cache_hit", cache_hit);
        if perf_enabled {
            if cache_hit {
                frame_perf.scene_encoding_cache_hits =
                    frame_perf.scene_encoding_cache_hits.saturating_add(1);
            } else {
                frame_perf.scene_encoding_cache_misses =
                    frame_perf.scene_encoding_cache_misses.saturating_add(1);
            }
        }
        let encoding = if cache_hit {
            std::mem::take(&mut self.scene_encoding_cache)
        } else {
            let mut encoding = std::mem::take(&mut self.scene_encoding_scratch);
            encoding.clear();
            let encode_start = perf_enabled.then(Instant::now);
            {
                let encode_span = if trace_enabled {
                    tracing::trace_span!("fret.renderer.scene.encode", frame_index)
                } else {
                    tracing::Span::none()
                };
                let _guard = encode_span.enter();
                self.encode_scene_ops_into(
                    scene,
                    scale_factor,
                    viewport_size,
                    format.is_srgb(),
                    &mut encoding,
                );
            }
            if let Some(encode_start) = encode_start {
                frame_perf.encode_scene += encode_start.elapsed();
            }

            // Preserve the old cache's allocations for reuse.
            self.scene_encoding_scratch = std::mem::take(&mut self.scene_encoding_cache);
            self.scene_encoding_cache_key = Some(key);
            encoding
        };

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

        let postprocess = if self.debug_pixelate_scale > 0 {
            DebugPostprocess::Pixelate {
                scale: self.debug_pixelate_scale,
            }
        } else if self.debug_blur_radius > 0 {
            let radius = self.debug_blur_radius.max(1);
            let budget = self.intermediate_budget_bytes;
            let full = estimate_texture_bytes(viewport_size, format, 1);
            let half = estimate_texture_bytes(downsampled_size(viewport_size, 2), format, 1);
            let quarter = estimate_texture_bytes(downsampled_size(viewport_size, 4), format, 1);

            let required_half = full.saturating_add(half.saturating_mul(2));
            let required_quarter = full.saturating_add(quarter.saturating_mul(2));

            let default_downsample_scale = if radius > 4 { 4 } else { 2 };
            let mut downsample_scale = default_downsample_scale;
            if downsample_scale == 2 && required_half > budget {
                downsample_scale = 4;
                if self.intermediate_perf_enabled {
                    self.intermediate_perf.blur_degraded_to_quarter = self
                        .intermediate_perf
                        .blur_degraded_to_quarter
                        .saturating_add(1);
                }
            }

            if downsample_scale == 4 && required_quarter > budget {
                if self.intermediate_perf_enabled {
                    self.intermediate_perf.blur_disabled_due_to_budget = self
                        .intermediate_perf
                        .blur_disabled_due_to_budget
                        .saturating_add(1);
                }
                DebugPostprocess::None
            } else {
                DebugPostprocess::Blur {
                    radius,
                    downsample_scale,
                    scissor: self.debug_blur_scissor,
                }
            }
        } else if self.debug_offscreen_blit_enabled {
            DebugPostprocess::OffscreenBlit
        } else {
            DebugPostprocess::None
        };
        let (plan, plan_elapsed) = fret_perf::measure_span(
            perf_enabled,
            trace_enabled,
            || tracing::trace_span!("fret.renderer.plan.compile", frame_index),
            || {
                RenderPlan::compile_for_scene(
                    &encoding,
                    viewport_size,
                    format,
                    clear.0,
                    path_samples,
                    postprocess,
                    self.intermediate_budget_bytes,
                )
            },
        );
        if let Some(plan_elapsed) = plan_elapsed {
            frame_perf.plan_compile += plan_elapsed;
        }
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
        if perf_enabled {
            use super::super::render_plan::{
                RenderPlanDegradationKind as DegradationKind,
                RenderPlanDegradationReason as DegradationReason,
            };

            frame_perf.render_plan_estimated_peak_intermediate_bytes =
                plan.compile_stats.estimated_peak_intermediate_bytes;
            frame_perf.render_plan_segments = plan.segments.len() as u64;
            frame_perf.render_plan_degradations = plan.degradations.len() as u64;
            for d in &plan.degradations {
                match d.reason {
                    DegradationReason::BudgetZero => {
                        frame_perf.render_plan_degradations_budget_zero = frame_perf
                            .render_plan_degradations_budget_zero
                            .saturating_add(1);
                    }
                    DegradationReason::BudgetInsufficient => {
                        frame_perf.render_plan_degradations_budget_insufficient = frame_perf
                            .render_plan_degradations_budget_insufficient
                            .saturating_add(1);
                    }
                    DegradationReason::TargetExhausted => {
                        frame_perf.render_plan_degradations_target_exhausted = frame_perf
                            .render_plan_degradations_target_exhausted
                            .saturating_add(1);
                    }
                }

                match d.kind {
                    DegradationKind::BackdropEffectNoOp => {
                        frame_perf.render_plan_degradations_backdrop_noop = frame_perf
                            .render_plan_degradations_backdrop_noop
                            .saturating_add(1);
                    }
                    DegradationKind::FilterContentDisabled => {
                        frame_perf.render_plan_degradations_filter_content_disabled = frame_perf
                            .render_plan_degradations_filter_content_disabled
                            .saturating_add(1);
                    }
                    DegradationKind::ClipPathDisabled => {
                        frame_perf.render_plan_degradations_clip_path_disabled = frame_perf
                            .render_plan_degradations_clip_path_disabled
                            .saturating_add(1);
                    }
                    DegradationKind::CompositeGroupBlendDegradedToOver => {
                        frame_perf.render_plan_degradations_composite_group_blend_to_over =
                            frame_perf
                                .render_plan_degradations_composite_group_blend_to_over
                                .saturating_add(1);
                    }
                }
            }

            let mut scene_draw_range_passes_by_segment: Vec<u32> = vec![0; plan.segments.len()];
            let mut path_msaa_batch_passes_by_segment: Vec<u32> = vec![0; plan.segments.len()];
            for p in &plan.passes {
                match p {
                    RenderPlanPass::SceneDrawRange(p) => {
                        if let Some(c) = scene_draw_range_passes_by_segment.get_mut(p.segment.0) {
                            *c = c.saturating_add(1);
                        }
                    }
                    RenderPlanPass::PathMsaaBatch(p) => {
                        if let Some(c) = path_msaa_batch_passes_by_segment.get_mut(p.segment.0) {
                            *c = c.saturating_add(1);
                        }
                    }
                    _ => {}
                }
            }

            let mut report: Vec<RenderPlanSegmentReport> = Vec::with_capacity(plan.segments.len());
            for (ix, seg) in plan.segments.iter().enumerate() {
                let flags_mask = u8::from(seg.flags.has_quad)
                    | (u8::from(seg.flags.has_viewport) << 1)
                    | (u8::from(seg.flags.has_image) << 2)
                    | (u8::from(seg.flags.has_mask) << 3)
                    | (u8::from(seg.flags.has_text) << 4)
                    | (u8::from(seg.flags.has_path) << 5);
                report.push(RenderPlanSegmentReport {
                    draw_range: (seg.draw_range.start, seg.draw_range.end),
                    start_uniform_fingerprint: seg.start_uniform_fingerprint,
                    flags_mask,
                    scene_draw_range_passes: *scene_draw_range_passes_by_segment
                        .get(ix)
                        .unwrap_or(&0),
                    path_msaa_batch_passes: *path_msaa_batch_passes_by_segment
                        .get(ix)
                        .unwrap_or(&0),
                });
            }

            let mut segments_changed: u64 = 0;
            let mut segments_passes_increased: u64 = 0;
            if let Some(prev) = &self.last_render_plan_segment_report {
                if prev.len() != report.len() {
                    segments_changed = report.len() as u64;
                } else {
                    for (p, c) in prev.iter().zip(report.iter()) {
                        if p.draw_range != c.draw_range
                            || p.start_uniform_fingerprint != c.start_uniform_fingerprint
                            || p.flags_mask != c.flags_mask
                        {
                            segments_changed = segments_changed.saturating_add(1);
                        }

                        let prev_passes = p
                            .scene_draw_range_passes
                            .saturating_add(p.path_msaa_batch_passes);
                        let cur_passes = c
                            .scene_draw_range_passes
                            .saturating_add(c.path_msaa_batch_passes);
                        if cur_passes > prev_passes {
                            segments_passes_increased = segments_passes_increased.saturating_add(1);
                        }
                    }
                }
            }
            frame_perf.render_plan_segments_changed = segments_changed;
            frame_perf.render_plan_segments_passes_increased = segments_passes_increased;
            self.last_render_plan_segment_report = Some(report);
        }
        render_plan_dump::maybe_dump_render_plan_json(
            &plan,
            viewport_size,
            format,
            frame_index,
            postprocess,
            encoding.ordered_draws.len(),
            &encoding.effect_markers,
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

        let needs_scale = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::ScaleNearest(_)));
        let needs_blur = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::Blur(_)));
        let needs_clip_mask = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::ClipMask(_)));
        let needs_blit = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::FullscreenBlit(_)));
        let needs_composite = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::CompositePremul(_)));
        let needs_color_adjust = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::ColorAdjust(_)));
        let needs_backdrop_warp = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::BackdropWarp(_)));
        let needs_color_matrix = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::ColorMatrix(_)));
        let needs_alpha_threshold = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::AlphaThreshold(_)));
        let needs_drop_shadow = plan
            .passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::DropShadow(_)));

        if needs_blit || needs_blur {
            self.ensure_blit_pipeline(device, format);
        }
        if needs_scale {
            self.ensure_scale_nearest_pipelines(device, format);
        }
        if needs_blur {
            self.ensure_blur_pipelines(device, format);
        }
        if needs_clip_mask {
            self.ensure_clip_mask_pipeline(device);
        }
        if needs_composite && path_samples <= 1 {
            self.ensure_composite_pipeline(device, format);
        }
        if needs_backdrop_warp {
            self.ensure_backdrop_warp_pipeline(device, format);
        }
        if needs_color_adjust {
            self.ensure_color_adjust_pipeline(device, format);
        }
        if needs_color_matrix {
            self.ensure_color_matrix_pipeline(device, format);
        }
        if needs_alpha_threshold {
            self.ensure_alpha_threshold_pipeline(device, format);
        }
        if needs_drop_shadow {
            self.ensure_drop_shadow_pipeline(device, format);
        }
        if self.intermediate_perf_enabled {
            self.intermediate_perf.last_frame_release_targets = plan
                .passes
                .iter()
                .filter(|p| matches!(p, RenderPlanPass::ReleaseTarget(_)))
                .count() as u64;
        }

        let scale_pass_count = plan
            .passes
            .iter()
            .filter(|p| matches!(p, RenderPlanPass::ScaleNearest(_)))
            .count();
        self.effect_params
            .ensure_scale_param_capacity(device, scale_pass_count);
        self.ensure_render_space_capacity(device, plan.passes.len());

        self.ensure_uniform_capacity(device, encoding.uniforms.len());
        let uniform_bytes_written =
            self.uniforms
                .write_viewport_uniforms(queue, &encoding.uniforms) as u64;
        if perf_enabled {
            frame_perf.uniform_bytes = frame_perf
                .uniform_bytes
                .saturating_add(uniform_bytes_written);
        }

        self.ensure_clip_capacity(device, encoding.clips.len().max(1));
        let clip_bytes_written = self.uniforms.write_clips(queue, &encoding.clips) as u64;
        if perf_enabled {
            frame_perf.uniform_bytes = frame_perf.uniform_bytes.saturating_add(clip_bytes_written);
        }

        self.ensure_mask_capacity(device, encoding.masks.len().max(1));
        let mask_bytes_written = self.uniforms.write_masks(queue, &encoding.masks) as u64;
        if perf_enabled {
            frame_perf.uniform_bytes = frame_perf.uniform_bytes.saturating_add(mask_bytes_written);
        }

        self.prepare_viewport_bind_groups(device, &encoding.ordered_draws);
        self.prepare_image_bind_groups(device, &encoding.ordered_draws);
        self.prepare_uniform_mask_image_bind_groups(device, &encoding.uniform_mask_images);

        let instances = &encoding.instances;
        let path_paints = &encoding.path_paints;
        let text_paints = &encoding.text_paints;
        let viewport_vertices = &encoding.viewport_vertices;
        let text_vertices = &encoding.text_vertices;
        let path_vertices = &encoding.path_vertices;

        self.quad_instances.ensure_capacity(device, instances.len());
        self.path_paints.ensure_capacity(device, path_paints.len());
        self.text_paints.ensure_capacity(device, text_paints.len());
        self.viewport_vertices
            .ensure_capacity(device, viewport_vertices.len());
        self.text_vertices
            .ensure_capacity(device, text_vertices.len());
        self.path_vertices
            .ensure_capacity(device, path_vertices.len());

        let (instance_buffer, quad_instance_bind_group) = self.quad_instances.next_pair();
        if !instances.is_empty() {
            queue.write_buffer(&instance_buffer, 0, bytemuck::cast_slice(instances));
            if perf_enabled {
                frame_perf.instance_bytes = frame_perf
                    .instance_bytes
                    .saturating_add((std::mem::size_of::<QuadInstance>() * instances.len()) as u64);
            }
        }

        let (path_paint_buffer, path_paint_bind_group) = self.path_paints.next_pair();
        if !path_paints.is_empty() {
            queue.write_buffer(&path_paint_buffer, 0, bytemuck::cast_slice(path_paints));
            if perf_enabled {
                frame_perf.instance_bytes = frame_perf
                    .instance_bytes
                    .saturating_add((std::mem::size_of::<PaintGpu>() * path_paints.len()) as u64);
            }
        }

        let (text_paint_buffer, text_paint_bind_group) = self.text_paints.next_pair();
        if !text_paints.is_empty() {
            queue.write_buffer(&text_paint_buffer, 0, bytemuck::cast_slice(text_paints));
            if perf_enabled {
                frame_perf.instance_bytes = frame_perf
                    .instance_bytes
                    .saturating_add((std::mem::size_of::<PaintGpu>() * text_paints.len()) as u64);
            }
        }

        let viewport_vertex_buffer = self.viewport_vertices.next_buffer();
        if !viewport_vertices.is_empty() {
            queue.write_buffer(
                &viewport_vertex_buffer,
                0,
                bytemuck::cast_slice(viewport_vertices),
            );
            if perf_enabled {
                frame_perf.vertex_bytes = frame_perf.vertex_bytes.saturating_add(
                    (std::mem::size_of::<ViewportVertex>() * viewport_vertices.len()) as u64,
                );
            }
        }

        let text_vertex_buffer = self.text_vertices.next_buffer();
        if !text_vertices.is_empty() {
            queue.write_buffer(&text_vertex_buffer, 0, bytemuck::cast_slice(text_vertices));
            if perf_enabled {
                frame_perf.vertex_bytes = frame_perf.vertex_bytes.saturating_add(
                    (std::mem::size_of::<TextVertex>() * text_vertices.len()) as u64,
                );
            }
        }

        let path_vertex_buffer = self.path_vertices.next_buffer();
        if !path_vertices.is_empty() {
            queue.write_buffer(&path_vertex_buffer, 0, bytemuck::cast_slice(path_vertices));
            if perf_enabled {
                frame_perf.vertex_bytes = frame_perf.vertex_bytes.saturating_add(
                    (std::mem::size_of::<PathVertex>() * path_vertices.len()) as u64,
                );
            }
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fret renderer encoder"),
        });

        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST;
        let mut frame_targets = FrameTargets::default();
        let scale_param_size = std::mem::size_of::<ScaleParamsUniform>() as u64;
        let mut scale_param_cursor: u32 = 0;

        // Some passes draw textured quads (not fullscreen triangles). Upload the vertex payload
        // once per frame and reference it via slices, since multiple `queue.write_buffer()` calls
        // against the same buffer region in a single submission would make all passes observe the
        // final write.
        let quad_vertex_bases =
            upload_plan_quad_vertices(self, device, queue, &plan, viewport_size);

        drop(uploads_guard);
        if let Some(upload_started) = upload_started {
            frame_perf.upload += upload_started.elapsed();
        }

        let quad_vertex_size = std::mem::size_of::<ViewportVertex>() as u64;

        debug_assert!(
            (std::mem::size_of::<RenderSpaceUniform>() as u64) <= self.uniforms.render_space_stride,
            "render_space_stride must fit RenderSpaceUniform"
        );
        let render_space_uniform_size = std::mem::size_of::<RenderSpaceUniform>();
        let render_space_stride = self.uniforms.render_space_stride as usize;
        let mut render_space_bytes =
            vec![0u8; render_space_stride.saturating_mul(plan.passes.len())];
        for (pass_index, planned_pass) in plan.passes.iter().enumerate() {
            let Some((origin, size)) = render_plan_pass_render_space(planned_pass) else {
                continue;
            };
            let offset = render_space_stride.saturating_mul(pass_index);
            render_space_bytes[offset..offset + render_space_uniform_size].copy_from_slice(
                bytemuck::bytes_of(&RenderSpaceUniform {
                    origin_px: [origin.0 as f32, origin.1 as f32],
                    size_px: [size.0.max(1) as f32, size.1.max(1) as f32],
                }),
            );
        }
        if !render_space_bytes.is_empty() {
            let _ = self
                .uniforms
                .write_render_space_bytes(queue, &render_space_bytes);
        }

        let (_, record_elapsed) = fret_perf::measure_span(
            perf_enabled,
            trace_enabled,
            || tracing::trace_span!("fret.renderer.record_passes", frame_index),
            || {
                let render_space_capacity = self.uniforms.render_space_capacity;
                let render_space_stride = self.uniforms.render_space_stride;
                let mut executor = RenderSceneExecutor::new(
                    self,
                    device,
                    queue,
                    frame_index,
                    format,
                    target_view,
                    viewport_size,
                    usage,
                    &mut encoder,
                    &mut frame_targets,
                    &encoding,
                    scale_param_size,
                    &mut scale_param_cursor,
                    quad_vertex_size,
                    &quad_vertex_bases,
                    perf_enabled,
                    &mut frame_perf,
                );
                let resources = RecordPassResources {
                    viewport_vertex_buffer: &viewport_vertex_buffer,
                    text_vertex_buffer: &text_vertex_buffer,
                    path_vertex_buffer: &path_vertex_buffer,
                    quad_instance_bind_group: &quad_instance_bind_group,
                    text_paint_bind_group: &text_paint_bind_group,
                    path_paint_bind_group: &path_paint_bind_group,
                };
                for (pass_index, planned_pass) in plan.passes.iter().enumerate() {
                    debug_assert!(
                        pass_index < render_space_capacity,
                        "render_space_capacity too small for RenderPlan passes"
                    );
                    let pass_span = if trace_enabled {
                        let kind = render_plan_pass_trace_kind(planned_pass);
                        let meta = render_plan_pass_trace_meta(planned_pass);
                        tracing::trace_span!(
                            "fret.renderer.pass",
                            frame_index,
                            pass_index = pass_index as u32,
                            kind,
                            src = ?meta.src,
                            dst = ?meta.dst,
                            load = meta.load.unwrap_or(""),
                            scissor = ?meta.scissor,
                            scissor_space = meta
                                .scissor_space
                                .map(|s| s.as_str())
                                .unwrap_or(""),
                            render_origin = ?meta.render_origin,
                            render_size = ?meta.render_size
                        )
                    } else {
                        tracing::Span::none()
                    };
                    let _pass_guard = pass_span.enter();
                    let render_space_offset =
                        (pass_index as u64).saturating_mul(render_space_stride);
                    let render_space_offset_u32 = render_space_offset as u32;
                    let ctx = RecordPassCtx {
                        plan: &plan,
                        pass_index,
                        render_space_offset_u32,
                    };

                    executor.record_pass(planned_pass, &ctx, &resources);
                }
            },
        );
        if let Some(record_elapsed) = record_elapsed {
            frame_perf.record_passes += record_elapsed;
        }

        let (cmd, finish_elapsed) = fret_perf::measure_span(
            perf_enabled,
            trace_enabled,
            || tracing::trace_span!("fret.renderer.encoder.finish", frame_index),
            || encoder.finish(),
        );
        if let Some(finish_elapsed) = finish_elapsed {
            frame_perf.encoder_finish += finish_elapsed;
        }

        if self.intermediate_perf_enabled {
            self.intermediate_perf.last_frame_in_use_bytes = frame_targets.in_use_bytes();
            self.intermediate_perf.last_frame_peak_in_use_bytes = frame_targets.peak_in_use_bytes();
        }
        if perf_enabled {
            frame_perf.intermediate_in_use_bytes = frame_targets.in_use_bytes();
            frame_perf.intermediate_peak_in_use_bytes = frame_targets.peak_in_use_bytes();
            frame_perf.intermediate_release_targets = plan
                .passes
                .iter()
                .filter(|p| matches!(p, RenderPlanPass::ReleaseTarget(_)))
                .count() as u64;
        }
        if self.intermediate_perf_enabled {
            self.intermediate_perf.last_frame_release_targets = plan
                .passes
                .iter()
                .filter(|p| matches!(p, RenderPlanPass::ReleaseTarget(_)))
                .count() as u64;
        }
        frame_targets.release_all(&mut self.intermediate_pool, self.intermediate_budget_bytes);

        if perf_enabled {
            // Snapshot SVG cache occupancy after `prepare_svg_ops` (which may prune rasters).
            let pages_live = self
                .svg_mask_atlas_pages
                .iter()
                .filter(|p| p.is_some())
                .count();
            let atlas_capacity_px = u64::from(pages_live as u32)
                .saturating_mul(u64::from(SVG_MASK_ATLAS_PAGE_SIZE_PX))
                .saturating_mul(u64::from(SVG_MASK_ATLAS_PAGE_SIZE_PX));
            let atlas_used_px = self
                .svg_rasters
                .values()
                .filter_map(|e| match e.storage {
                    SvgRasterStorage::MaskAtlas { page_index, .. } => Some((page_index, e.size_px)),
                    SvgRasterStorage::Standalone { .. } => None,
                })
                .filter(|(page_index, _)| {
                    self.svg_mask_atlas_pages
                        .get(*page_index)
                        .is_some_and(|p| p.is_some())
                })
                .fold(0u64, |acc, (_, (w, h))| {
                    let pad = u64::from(SVG_MASK_ATLAS_PADDING_PX.saturating_mul(2));
                    let w_pad = u64::from(w).saturating_add(pad);
                    let h_pad = u64::from(h).saturating_add(pad);
                    acc.saturating_add(w_pad.saturating_mul(h_pad))
                });

            frame_perf.svg_raster_budget_bytes = self.svg_raster_budget_bytes;
            frame_perf.svg_rasters_live = self.svg_rasters.len() as u64;
            frame_perf.svg_standalone_bytes_live = self.svg_raster_bytes;
            frame_perf.svg_mask_atlas_pages_live = pages_live as u64;
            frame_perf.svg_mask_atlas_bytes_live = self.svg_mask_atlas_bytes;
            frame_perf.svg_mask_atlas_used_px = atlas_used_px;
            frame_perf.svg_mask_atlas_capacity_px = atlas_capacity_px;
            frame_perf.svg_raster_cache_hits = self.perf_svg_raster_cache_hits;
            frame_perf.svg_raster_cache_misses = self.perf_svg_raster_cache_misses;
            frame_perf.svg_raster_budget_evictions = self.perf_svg_raster_budget_evictions;
            frame_perf.svg_mask_atlas_page_evictions = self.perf_svg_mask_atlas_page_evictions;
            frame_perf.svg_mask_atlas_entries_evicted = self.perf_svg_mask_atlas_entries_evicted;

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
                .saturating_add(
                    frame_perf.render_target_updates_requested_ingest_external_zero_copy,
                );
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
                .saturating_add(
                    frame_perf.render_target_metadata_degradations_color_encoding_dropped,
                );

            self.perf.svg_raster_budget_bytes = frame_perf.svg_raster_budget_bytes;
            self.perf.svg_rasters_live =
                self.perf.svg_rasters_live.max(frame_perf.svg_rasters_live);
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
                render_target_updates_ingest_unknown: frame_perf
                    .render_target_updates_ingest_unknown,
                render_target_updates_ingest_owned: frame_perf.render_target_updates_ingest_owned,
                render_target_updates_ingest_external_zero_copy: frame_perf
                    .render_target_updates_ingest_external_zero_copy,
                render_target_updates_ingest_gpu_copy: frame_perf
                    .render_target_updates_ingest_gpu_copy,
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
                intermediate_in_use_bytes: frame_perf.intermediate_in_use_bytes,
                intermediate_peak_in_use_bytes: frame_perf.intermediate_peak_in_use_bytes,
                intermediate_release_targets: frame_perf.intermediate_release_targets,
                intermediate_pool_allocations: frame_perf.intermediate_pool_allocations,
                intermediate_pool_reuses: frame_perf.intermediate_pool_reuses,
                intermediate_pool_releases: frame_perf.intermediate_pool_releases,
                intermediate_pool_evictions: frame_perf.intermediate_pool_evictions,
                intermediate_pool_free_bytes: frame_perf.intermediate_pool_free_bytes,
                intermediate_pool_free_textures: frame_perf.intermediate_pool_free_textures,
                render_plan_estimated_peak_intermediate_bytes: frame_perf
                    .render_plan_estimated_peak_intermediate_bytes,
                render_plan_segments: frame_perf.render_plan_segments,
                render_plan_segments_changed: frame_perf.render_plan_segments_changed,
                render_plan_segments_passes_increased: frame_perf
                    .render_plan_segments_passes_increased,
                render_plan_degradations: frame_perf.render_plan_degradations,
                render_plan_degradations_budget_zero: frame_perf
                    .render_plan_degradations_budget_zero,
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
                draw_calls: frame_perf.draw_calls,
                quad_draw_calls: frame_perf.quad_draw_calls,
                viewport_draw_calls: frame_perf.viewport_draw_calls,
                viewport_draw_calls_ingest_unknown: frame_perf.viewport_draw_calls_ingest_unknown,
                viewport_draw_calls_ingest_owned: frame_perf.viewport_draw_calls_ingest_owned,
                viewport_draw_calls_ingest_external_zero_copy: frame_perf
                    .viewport_draw_calls_ingest_external_zero_copy,
                viewport_draw_calls_ingest_gpu_copy: frame_perf.viewport_draw_calls_ingest_gpu_copy,
                viewport_draw_calls_ingest_cpu_upload: frame_perf
                    .viewport_draw_calls_ingest_cpu_upload,
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
                uniform_bytes: frame_perf.uniform_bytes,
                instance_bytes: frame_perf.instance_bytes,
                vertex_bytes: frame_perf.vertex_bytes,
                scene_encoding_cache_hits: frame_perf.scene_encoding_cache_hits,
                scene_encoding_cache_misses: frame_perf.scene_encoding_cache_misses,
                material_quad_ops: frame_perf.material_quad_ops,
                material_sampled_quad_ops: frame_perf.material_sampled_quad_ops,
                material_distinct: frame_perf.material_distinct,
                material_unknown_ids: frame_perf.material_unknown_ids,
                material_degraded_due_to_budget: frame_perf.material_degraded_due_to_budget,
                clip_path_mask_cache_bytes_live: frame_perf.clip_path_mask_cache_bytes_live,
                clip_path_mask_cache_entries_live: frame_perf.clip_path_mask_cache_entries_live,
                clip_path_mask_cache_hits: frame_perf.clip_path_mask_cache_hits,
                clip_path_mask_cache_misses: frame_perf.clip_path_mask_cache_misses,
            });
        }

        // Keep the most recent encoding for potential reuse on the next frame.
        if cache_hit {
            self.scene_encoding_cache_key = Some(key);
        }
        self.scene_encoding_cache = encoding;
        cmd
    }
}

// FrameTargets moved to `renderer/frame_targets.rs`.
