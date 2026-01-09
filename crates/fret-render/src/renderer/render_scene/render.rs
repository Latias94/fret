use super::super::frame_targets::{FrameTargets, downsampled_size};
use super::super::*;
use fret_core::time::Instant;

impl Renderer {
    pub fn render_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        params: RenderSceneParams<'_>,
    ) -> wgpu::CommandBuffer {
        let RenderSceneParams {
            format,
            target_view,
            scene,
            clear,
            scale_factor,
            viewport_size,
        } = params;

        #[cfg(debug_assertions)]
        if let Err(e) = scene.validate() {
            panic!("invalid scene: {e}");
        }

        self.ensure_viewport_pipeline(device, format);
        self.ensure_pipeline(device, format);
        self.ensure_text_pipeline(device, format);
        self.ensure_text_color_pipeline(device, format);
        self.ensure_mask_pipeline(device, format);
        self.ensure_path_pipeline(device, format);
        let path_samples = self.effective_path_msaa_samples(format);
        if path_samples > 1 {
            self.ensure_composite_pipeline(device, format);
            self.ensure_path_msaa_pipeline(device, format, path_samples);
            self.ensure_path_intermediate(device, viewport_size, format, path_samples);
        }

        self.text_system.flush_uploads(queue);
        if self.svg_perf_enabled {
            self.svg_perf.frames = self.svg_perf.frames.saturating_add(1);
        }
        if self.intermediate_perf_enabled {
            self.intermediate_perf.frames = self.intermediate_perf.frames.saturating_add(1);
        }
        self.bump_svg_raster_epoch();
        let svg_prepare_start = self.svg_perf_enabled.then(Instant::now);
        self.prepare_svg_ops(device, queue, scene, scale_factor);
        if let Some(svg_prepare_start) = svg_prepare_start {
            self.svg_perf.prepare_svg_ops += svg_prepare_start.elapsed();
        }

        let key = SceneEncodingCacheKey {
            format,
            viewport_size,
            scale_factor_bits: scale_factor.to_bits(),
            scene_fingerprint: scene.fingerprint(),
            scene_ops_len: scene.ops_len(),
            render_targets_generation: self.render_targets_generation,
            images_generation: self.images_generation,
        };

        let cache_hit = self.scene_encoding_cache_key == Some(key);
        let encoding = if cache_hit {
            std::mem::take(&mut self.scene_encoding_cache)
        } else {
            let mut encoding = std::mem::take(&mut self.scene_encoding_scratch);
            encoding.clear();
            self.encode_scene_ops_into(
                scene,
                scale_factor,
                viewport_size,
                format.is_srgb(),
                &mut encoding,
            );

            // Preserve the old cache's allocations for reuse.
            self.scene_encoding_scratch = std::mem::take(&mut self.scene_encoding_cache);
            self.scene_encoding_cache_key = Some(key);
            encoding
        };

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
        let plan = RenderPlan::compile_for_scene(
            &encoding,
            viewport_size,
            format,
            clear.0,
            path_samples,
            postprocess,
            self.intermediate_budget_bytes,
        );

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
        if needs_color_adjust {
            self.ensure_color_adjust_pipeline(device, format);
        }
        if self.intermediate_perf_enabled {
            self.intermediate_perf.last_frame_release_targets = plan
                .passes
                .iter()
                .filter(|p| matches!(p, RenderPlanPass::ReleaseTarget(_)))
                .count() as u64;
        }

        self.ensure_uniform_capacity(device, encoding.uniforms.len());
        let uniform_size = std::mem::size_of::<ViewportUniform>() as u64;
        let mut uniform_bytes =
            vec![0u8; (self.uniform_stride * encoding.uniforms.len() as u64) as usize];
        for (i, u) in encoding.uniforms.iter().enumerate() {
            let offset = (self.uniform_stride * i as u64) as usize;
            uniform_bytes[offset..offset + uniform_size as usize]
                .copy_from_slice(bytemuck::bytes_of(u));
        }
        queue.write_buffer(&self.uniform_buffer, 0, &uniform_bytes);

        self.ensure_clip_capacity(device, encoding.clips.len().max(1));
        if !encoding.clips.is_empty() {
            queue.write_buffer(&self.clip_buffer, 0, bytemuck::cast_slice(&encoding.clips));
        }

        self.prepare_viewport_bind_groups(device, &encoding.ordered_draws);
        self.prepare_image_bind_groups(device, &encoding.ordered_draws);

        let instances = &encoding.instances;
        let viewport_vertices = &encoding.viewport_vertices;
        let text_vertices = &encoding.text_vertices;
        let path_vertices = &encoding.path_vertices;

        self.ensure_instance_capacity(device, instances.len());
        self.ensure_viewport_vertex_capacity(device, viewport_vertices.len());
        self.ensure_text_vertex_capacity(device, text_vertices.len());
        self.ensure_path_vertex_capacity(device, path_vertices.len());

        let instance_buffer_index = self.instance_buffer_index;
        self.instance_buffer_index = (self.instance_buffer_index + 1) % self.instance_buffers.len();
        let instance_buffer = &self.instance_buffers[instance_buffer_index];
        if !instances.is_empty() {
            queue.write_buffer(instance_buffer, 0, bytemuck::cast_slice(instances));
        }

        let viewport_vertex_buffer_index = self.viewport_vertex_buffer_index;
        self.viewport_vertex_buffer_index =
            (self.viewport_vertex_buffer_index + 1) % self.viewport_vertex_buffers.len();
        let viewport_vertex_buffer = &self.viewport_vertex_buffers[viewport_vertex_buffer_index];
        if !viewport_vertices.is_empty() {
            queue.write_buffer(
                viewport_vertex_buffer,
                0,
                bytemuck::cast_slice(viewport_vertices),
            );
        }

        let text_vertex_buffer_index = self.text_vertex_buffer_index;
        self.text_vertex_buffer_index =
            (self.text_vertex_buffer_index + 1) % self.text_vertex_buffers.len();
        let text_vertex_buffer = &self.text_vertex_buffers[text_vertex_buffer_index];
        if !text_vertices.is_empty() {
            queue.write_buffer(text_vertex_buffer, 0, bytemuck::cast_slice(text_vertices));
        }

        let path_vertex_buffer_index = self.path_vertex_buffer_index;
        self.path_vertex_buffer_index =
            (self.path_vertex_buffer_index + 1) % self.path_vertex_buffers.len();
        let path_vertex_buffer = &self.path_vertex_buffers[path_vertex_buffer_index];
        if !path_vertices.is_empty() {
            queue.write_buffer(path_vertex_buffer, 0, bytemuck::cast_slice(path_vertices));
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fret renderer encoder"),
        });

        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let mut frame_targets = FrameTargets::default();

        for planned_pass in &plan.passes {
            match planned_pass {
                RenderPlanPass::SceneDrawRange(scene_pass) => {
                    debug_assert_eq!(scene_pass.segment.0, 0);
                    let load = scene_pass.load;
                    let pass_target_view_owned = match scene_pass.target {
                        PlanTarget::Output => None,
                        PlanTarget::Intermediate0 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            PlanTarget::Intermediate0,
                            viewport_size,
                            format,
                            usage,
                            self.intermediate_budget_bytes,
                        )),
                        PlanTarget::Intermediate1 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            PlanTarget::Intermediate1,
                            viewport_size,
                            format,
                            usage,
                            self.intermediate_budget_bytes,
                        )),
                        PlanTarget::Intermediate2 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            PlanTarget::Intermediate2,
                            viewport_size,
                            format,
                            usage,
                            self.intermediate_budget_bytes,
                        )),
                        PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                            debug_assert!(false, "SceneDrawRange cannot target mask targets");
                            None
                        }
                    };
                    let pass_target_view = pass_target_view_owned.as_ref().unwrap_or(target_view);

                    {
                        enum ActivePipeline {
                            None,
                            Quad,
                            Viewport,
                            TextMask,
                            TextColor,
                            Mask,
                            Path,
                        }

                        let quad_pipeline = self
                            .quad_pipeline
                            .as_ref()
                            .expect("quad pipeline must exist");
                        let viewport_pipeline = self
                            .viewport_pipeline
                            .as_ref()
                            .expect("viewport pipeline must exist");
                        let text_pipeline = self
                            .text_pipeline
                            .as_ref()
                            .expect("text pipeline must exist");
                        let text_color_pipeline = self
                            .text_color_pipeline
                            .as_ref()
                            .expect("text color pipeline must exist");
                        let mask_pipeline = self
                            .mask_pipeline
                            .as_ref()
                            .expect("mask pipeline must exist");
                        let path_pipeline = self
                            .path_pipeline
                            .as_ref()
                            .expect("path pipeline must exist");

                        let mut active_pipeline = ActivePipeline::None;

                        fn begin_main_pass<'a>(
                            encoder: &'a mut wgpu::CommandEncoder,
                            target_view: &'a wgpu::TextureView,
                            load: wgpu::LoadOp<wgpu::Color>,
                        ) -> wgpu::RenderPass<'a> {
                            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("fret renderer pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: target_view,
                                    depth_slice: None,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load,
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                                multiview_mask: None,
                            })
                        }

                        let mut pass = begin_main_pass(&mut encoder, pass_target_view, load);
                        let mut active_uniform_offset: Option<u32> = None;

                        let mut i = scene_pass.draw_range.start;
                        while i < scene_pass.draw_range.end {
                            let item = &encoding.ordered_draws[i];

                            match item {
                                OrderedDraw::Quad(draw) => {
                                    if draw.scissor.w == 0 || draw.scissor.h == 0 {
                                        i += 1;
                                        continue;
                                    }

                                    if !matches!(active_pipeline, ActivePipeline::Quad) {
                                        pass.set_pipeline(quad_pipeline);
                                        pass.set_vertex_buffer(0, instance_buffer.slice(..));
                                        active_pipeline = ActivePipeline::Quad;
                                    }

                                    let uniform_offset = (u64::from(draw.uniform_index)
                                        * self.uniform_stride)
                                        as u32;
                                    if active_uniform_offset != Some(uniform_offset) {
                                        pass.set_bind_group(
                                            0,
                                            &self.uniform_bind_group,
                                            &[uniform_offset],
                                        );
                                        active_uniform_offset = Some(uniform_offset);
                                    }
                                    pass.set_scissor_rect(
                                        draw.scissor.x,
                                        draw.scissor.y,
                                        draw.scissor.w,
                                        draw.scissor.h,
                                    );
                                    pass.draw(
                                        0..6,
                                        draw.first_instance
                                            ..(draw.first_instance + draw.instance_count),
                                    );
                                }
                                OrderedDraw::Viewport(draw) => {
                                    if draw.scissor.w == 0 || draw.scissor.h == 0 {
                                        i += 1;
                                        continue;
                                    }

                                    if !matches!(active_pipeline, ActivePipeline::Viewport) {
                                        pass.set_pipeline(viewport_pipeline);
                                        pass.set_vertex_buffer(0, viewport_vertex_buffer.slice(..));
                                        active_pipeline = ActivePipeline::Viewport;
                                    }

                                    let uniform_offset = (u64::from(draw.uniform_index)
                                        * self.uniform_stride)
                                        as u32;
                                    if active_uniform_offset != Some(uniform_offset) {
                                        pass.set_bind_group(
                                            0,
                                            &self.uniform_bind_group,
                                            &[uniform_offset],
                                        );
                                        active_uniform_offset = Some(uniform_offset);
                                    }
                                    let Some((_, bind_group)) =
                                        self.viewport_bind_groups.get(&draw.target)
                                    else {
                                        // Missing bind group should only happen if the target vanished
                                        // between encoding and rendering.
                                        i += 1;
                                        continue;
                                    };
                                    pass.set_bind_group(1, bind_group, &[]);
                                    pass.set_scissor_rect(
                                        draw.scissor.x,
                                        draw.scissor.y,
                                        draw.scissor.w,
                                        draw.scissor.h,
                                    );
                                    pass.draw(
                                        draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                                        0..1,
                                    );
                                }
                                OrderedDraw::Image(draw) => {
                                    if draw.scissor.w == 0 || draw.scissor.h == 0 {
                                        i += 1;
                                        continue;
                                    }

                                    if !matches!(active_pipeline, ActivePipeline::Viewport) {
                                        pass.set_pipeline(viewport_pipeline);
                                        pass.set_vertex_buffer(0, viewport_vertex_buffer.slice(..));
                                        active_pipeline = ActivePipeline::Viewport;
                                    }

                                    let uniform_offset = (u64::from(draw.uniform_index)
                                        * self.uniform_stride)
                                        as u32;
                                    if active_uniform_offset != Some(uniform_offset) {
                                        pass.set_bind_group(
                                            0,
                                            &self.uniform_bind_group,
                                            &[uniform_offset],
                                        );
                                        active_uniform_offset = Some(uniform_offset);
                                    }
                                    let Some((_, bind_group)) =
                                        self.image_bind_groups.get(&draw.image)
                                    else {
                                        i += 1;
                                        continue;
                                    };
                                    pass.set_bind_group(1, bind_group, &[]);
                                    pass.set_scissor_rect(
                                        draw.scissor.x,
                                        draw.scissor.y,
                                        draw.scissor.w,
                                        draw.scissor.h,
                                    );
                                    pass.draw(
                                        draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                                        0..1,
                                    );
                                }
                                OrderedDraw::Mask(draw) => {
                                    if draw.scissor.w == 0 || draw.scissor.h == 0 {
                                        i += 1;
                                        continue;
                                    }

                                    if !matches!(active_pipeline, ActivePipeline::Mask) {
                                        pass.set_pipeline(mask_pipeline);
                                        pass.set_vertex_buffer(0, text_vertex_buffer.slice(..));
                                        active_pipeline = ActivePipeline::Mask;
                                    }

                                    let uniform_offset = (u64::from(draw.uniform_index)
                                        * self.uniform_stride)
                                        as u32;
                                    if active_uniform_offset != Some(uniform_offset) {
                                        pass.set_bind_group(
                                            0,
                                            &self.uniform_bind_group,
                                            &[uniform_offset],
                                        );
                                        active_uniform_offset = Some(uniform_offset);
                                    }
                                    let Some((_, bind_group)) =
                                        self.image_bind_groups.get(&draw.image)
                                    else {
                                        i += 1;
                                        continue;
                                    };
                                    pass.set_bind_group(1, bind_group, &[]);
                                    pass.set_scissor_rect(
                                        draw.scissor.x,
                                        draw.scissor.y,
                                        draw.scissor.w,
                                        draw.scissor.h,
                                    );
                                    pass.draw(
                                        draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                                        0..1,
                                    );
                                }
                                OrderedDraw::Text(draw) => {
                                    if draw.scissor.w == 0 || draw.scissor.h == 0 {
                                        i += 1;
                                        continue;
                                    }

                                    match draw.kind {
                                        TextDrawKind::Mask => {
                                            if !matches!(active_pipeline, ActivePipeline::TextMask)
                                            {
                                                pass.set_pipeline(text_pipeline);
                                                pass.set_vertex_buffer(
                                                    0,
                                                    text_vertex_buffer.slice(..),
                                                );
                                                pass.set_bind_group(
                                                    1,
                                                    self.text_system.mask_atlas_bind_group(),
                                                    &[],
                                                );
                                                active_pipeline = ActivePipeline::TextMask;
                                            }
                                        }
                                        TextDrawKind::Color => {
                                            if !matches!(active_pipeline, ActivePipeline::TextColor)
                                            {
                                                pass.set_pipeline(text_color_pipeline);
                                                pass.set_vertex_buffer(
                                                    0,
                                                    text_vertex_buffer.slice(..),
                                                );
                                                pass.set_bind_group(
                                                    1,
                                                    self.text_system.color_atlas_bind_group(),
                                                    &[],
                                                );
                                                active_pipeline = ActivePipeline::TextColor;
                                            }
                                        }
                                    }

                                    let uniform_offset = (u64::from(draw.uniform_index)
                                        * self.uniform_stride)
                                        as u32;
                                    if active_uniform_offset != Some(uniform_offset) {
                                        pass.set_bind_group(
                                            0,
                                            &self.uniform_bind_group,
                                            &[uniform_offset],
                                        );
                                        active_uniform_offset = Some(uniform_offset);
                                    }
                                    pass.set_scissor_rect(
                                        draw.scissor.x,
                                        draw.scissor.y,
                                        draw.scissor.w,
                                        draw.scissor.h,
                                    );
                                    pass.draw(
                                        draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                                        0..1,
                                    );
                                }
                                OrderedDraw::Path(draw) => {
                                    if draw.scissor.w == 0 || draw.scissor.h == 0 {
                                        i += 1;
                                        continue;
                                    }

                                    if !matches!(active_pipeline, ActivePipeline::Path) {
                                        pass.set_pipeline(path_pipeline);
                                        pass.set_vertex_buffer(0, path_vertex_buffer.slice(..));
                                        active_pipeline = ActivePipeline::Path;
                                    }

                                    let uniform_offset = (u64::from(draw.uniform_index)
                                        * self.uniform_stride)
                                        as u32;
                                    if active_uniform_offset != Some(uniform_offset) {
                                        pass.set_bind_group(
                                            0,
                                            &self.uniform_bind_group,
                                            &[uniform_offset],
                                        );
                                        active_uniform_offset = Some(uniform_offset);
                                    }
                                    pass.set_scissor_rect(
                                        draw.scissor.x,
                                        draw.scissor.y,
                                        draw.scissor.w,
                                        draw.scissor.h,
                                    );
                                    pass.draw(
                                        draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                                        0..1,
                                    );
                                }
                            }

                            i += 1;
                        }
                    }
                }
                RenderPlanPass::PathMsaaBatch(path_pass) => {
                    debug_assert_eq!(path_pass.segment.0, 0);
                    let pass_target_view_owned = match path_pass.target {
                        PlanTarget::Output => None,
                        PlanTarget::Intermediate0 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            PlanTarget::Intermediate0,
                            viewport_size,
                            format,
                            usage,
                            self.intermediate_budget_bytes,
                        )),
                        PlanTarget::Intermediate1 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            PlanTarget::Intermediate1,
                            viewport_size,
                            format,
                            usage,
                            self.intermediate_budget_bytes,
                        )),
                        PlanTarget::Intermediate2 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            PlanTarget::Intermediate2,
                            viewport_size,
                            format,
                            usage,
                            self.intermediate_budget_bytes,
                        )),
                        PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                            debug_assert!(false, "PathMsaaBatch cannot target mask targets");
                            None
                        }
                    };
                    let pass_target_view = pass_target_view_owned.as_ref().unwrap_or(target_view);

                    let start = path_pass.draw_range.start;
                    let end = path_pass.draw_range.end;
                    if start >= end {
                        continue;
                    }

                    let Some(intermediate) = &self.path_intermediate else {
                        continue;
                    };
                    let Some(path_msaa_pipeline) = self.path_msaa_pipeline.as_ref() else {
                        continue;
                    };
                    let Some(composite_pipeline) = self.composite_pipeline.as_ref() else {
                        continue;
                    };

                    {
                        let mut path_pass_rp =
                            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("fret path intermediate pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: intermediate
                                        .msaa_view
                                        .as_ref()
                                        .unwrap_or(&intermediate.resolved_view),
                                    depth_slice: None,
                                    resolve_target: if intermediate.sample_count > 1 {
                                        Some(&intermediate.resolved_view)
                                    } else {
                                        None
                                    },
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                        store: if intermediate.sample_count > 1 {
                                            wgpu::StoreOp::Discard
                                        } else {
                                            wgpu::StoreOp::Store
                                        },
                                    },
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                                multiview_mask: None,
                            });

                        path_pass_rp.set_pipeline(path_msaa_pipeline);
                        path_pass_rp.set_vertex_buffer(0, path_vertex_buffer.slice(..));

                        let mut active_uniform_offset: Option<u32> = None;
                        for j in start..end {
                            let OrderedDraw::Path(draw) = &encoding.ordered_draws[j] else {
                                unreachable!("PathMsaaBatch pass must reference only Path draws");
                            };
                            if draw.scissor.w == 0 || draw.scissor.h == 0 {
                                continue;
                            }
                            path_pass_rp.set_scissor_rect(
                                draw.scissor.x,
                                draw.scissor.y,
                                draw.scissor.w,
                                draw.scissor.h,
                            );
                            let uniform_offset =
                                (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                            if active_uniform_offset != Some(uniform_offset) {
                                path_pass_rp.set_bind_group(
                                    0,
                                    &self.uniform_bind_group,
                                    &[uniform_offset],
                                );
                                active_uniform_offset = Some(uniform_offset);
                            }
                            path_pass_rp.draw(
                                draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                                0..1,
                            );
                        }
                    }

                    let union = path_pass.union_scissor;
                    if union.w == 0 || union.h == 0 {
                        continue;
                    }

                    let x0 = union.x as f32;
                    let y0 = union.y as f32;
                    let x1 = (union.x + union.w) as f32;
                    let y1 = (union.y + union.h) as f32;

                    let vw = viewport_size.0.max(1) as f32;
                    let vh = viewport_size.1.max(1) as f32;
                    let u0 = x0 / vw;
                    let v0 = y0 / vh;
                    let u1 = x1 / vw;
                    let v1 = y1 / vh;

                    let vertices: [ViewportVertex; 6] = [
                        ViewportVertex {
                            pos_px: [x0, y0],
                            uv: [u0, v0],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y0],
                            uv: [u1, v0],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y1],
                            uv: [u1, v1],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x0, y0],
                            uv: [u0, v0],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y1],
                            uv: [u1, v1],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x0, y1],
                            uv: [u0, v1],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                    ];
                    queue.write_buffer(
                        &self.path_composite_vertices,
                        0,
                        bytemuck::cast_slice(&vertices),
                    );

                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("fret renderer pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: pass_target_view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });

                    pass.set_pipeline(composite_pipeline);
                    let uniform_offset =
                        (u64::from(path_pass.batch_uniform_index) * self.uniform_stride) as u32;
                    pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                    pass.set_bind_group(1, &intermediate.bind_group, &[]);
                    pass.set_vertex_buffer(0, self.path_composite_vertices.slice(..));
                    pass.set_scissor_rect(union.x, union.y, union.w, union.h);
                    pass.draw(0..6, 0..1);
                }
                RenderPlanPass::ScaleNearest(pass) => {
                    let scale = pass.scale.max(1);
                    queue.write_buffer(
                        &self.scale_param_buffer,
                        0,
                        bytemuck::cast_slice(&[scale, 0, 0, 0]),
                    );

                    let src_view = match pass.src {
                        PlanTarget::Output
                        | PlanTarget::Mask0
                        | PlanTarget::Mask1
                        | PlanTarget::Mask2 => {
                            debug_assert!(false, "ScaleNearest src cannot be Output/mask targets");
                            continue;
                        }
                        PlanTarget::Intermediate0
                        | PlanTarget::Intermediate1
                        | PlanTarget::Intermediate2 => {
                            frame_targets.require_target(pass.src, pass.src_size)
                        }
                    };

                    let dst_view_owned = match pass.dst {
                        PlanTarget::Output => None,
                        PlanTarget::Intermediate0
                        | PlanTarget::Intermediate1
                        | PlanTarget::Intermediate2 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            pass.dst,
                            pass.dst_size,
                            format,
                            usage,
                            self.intermediate_budget_bytes,
                        )),
                        PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                            debug_assert!(false, "ScaleNearest dst cannot be mask targets");
                            None
                        }
                    };
                    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

                    if let Some(mask_target) = pass.mask_target {
                        debug_assert!(matches!(pass.mode, ScaleMode::Upscale));
                        debug_assert!(matches!(
                            mask_target,
                            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
                        ));
                        debug_assert_eq!(
                            pass.dst_size, viewport_size,
                            "mask-based scale-nearest expects full-size destination"
                        );

                        let mask_uniform_index = pass
                            .mask_uniform_index
                            .expect("mask pass needs uniform index");
                        let uniform_offset =
                            (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

                        let mask_view = frame_targets.require_target(
                            mask_target,
                            mask_target_size(viewport_size, mask_target),
                        );
                        let mask_layout = self
                            .scale_mask_bind_group_layout
                            .as_ref()
                            .expect("scale mask bind group layout must exist");
                        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("fret scale-nearest mask bind group"),
                            layout: mask_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&src_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: self.scale_param_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: wgpu::BindingResource::TextureView(&mask_view),
                                },
                            ],
                        });

                        let pipeline = self
                            .upscale_mask_pipeline
                            .as_ref()
                            .expect("upscale mask pipeline must exist");
                        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("fret upscale-nearest mask pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: dst_view,
                                depth_slice: None,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: pass.load,
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });
                        rp.set_pipeline(pipeline);
                        rp.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                        rp.set_bind_group(1, &bind_group, &[]);
                        if let Some(scissor) = pass.dst_scissor {
                            if scissor.w != 0 && scissor.h != 0 {
                                rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
                            }
                        }
                        rp.draw(0..3, 0..1);
                    } else if let Some(mask_uniform_index) = pass.mask_uniform_index {
                        debug_assert!(matches!(pass.mode, ScaleMode::Upscale));
                        let pipeline = self
                            .upscale_masked_pipeline
                            .as_ref()
                            .expect("upscale masked pipeline must exist");
                        let uniform_offset =
                            (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

                        let layout = self
                            .scale_bind_group_layout
                            .as_ref()
                            .expect("scale bind group layout must exist");
                        let bind_group = create_texture_uniform_bind_group(
                            device,
                            "fret scale-nearest bind group",
                            layout,
                            &src_view,
                            self.scale_param_buffer.as_entire_binding(),
                        );

                        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("fret upscale-nearest masked pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: dst_view,
                                depth_slice: None,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: pass.load,
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });
                        rp.set_pipeline(pipeline);
                        rp.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                        rp.set_bind_group(1, &bind_group, &[]);
                        if let Some(scissor) = pass.dst_scissor {
                            if scissor.w != 0 && scissor.h != 0 {
                                rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
                            }
                        }
                        rp.draw(0..3, 0..1);
                    } else {
                        let layout = self
                            .scale_bind_group_layout
                            .as_ref()
                            .expect("scale bind group layout must exist");
                        let bind_group = create_texture_uniform_bind_group(
                            device,
                            "fret scale-nearest bind group",
                            layout,
                            &src_view,
                            self.scale_param_buffer.as_entire_binding(),
                        );
                        let (pipeline, label) = match pass.mode {
                            ScaleMode::Downsample => (
                                self.downsample_pipeline
                                    .as_ref()
                                    .expect("downsample pipeline must exist"),
                                "fret downsample-nearest pass",
                            ),
                            ScaleMode::Upscale => (
                                self.upscale_pipeline
                                    .as_ref()
                                    .expect("upscale pipeline must exist"),
                                "fret upscale-nearest pass",
                            ),
                        };
                        run_fullscreen_triangle_pass(
                            &mut encoder,
                            label,
                            pipeline,
                            dst_view,
                            pass.load,
                            &bind_group,
                            pass.dst_scissor,
                        );
                    }
                }
                RenderPlanPass::Blur(pass) => {
                    let src_view = match pass.src {
                        PlanTarget::Output
                        | PlanTarget::Mask0
                        | PlanTarget::Mask1
                        | PlanTarget::Mask2 => {
                            debug_assert!(false, "Blur src cannot be Output/mask targets");
                            continue;
                        }
                        PlanTarget::Intermediate0
                        | PlanTarget::Intermediate1
                        | PlanTarget::Intermediate2 => {
                            frame_targets.require_target(pass.src, pass.src_size)
                        }
                    };

                    let dst_view_owned = match pass.dst {
                        PlanTarget::Output => None,
                        PlanTarget::Intermediate0
                        | PlanTarget::Intermediate1
                        | PlanTarget::Intermediate2 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            pass.dst,
                            pass.dst_size,
                            format,
                            usage,
                            self.intermediate_budget_bytes,
                        )),
                        PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                            debug_assert!(false, "Blur dst cannot be mask targets");
                            None
                        }
                    };
                    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

                    if let Some(mask_target) = pass.mask_target {
                        debug_assert!(matches!(
                            mask_target,
                            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
                        ));
                        debug_assert_eq!(
                            pass.dst_size, viewport_size,
                            "mask-based blur expects full-size destination"
                        );

                        let mask_uniform_index = pass
                            .mask_uniform_index
                            .expect("mask blur needs uniform index");
                        let uniform_offset =
                            (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

                        let mask_view = frame_targets.require_target(
                            mask_target,
                            mask_target_size(viewport_size, mask_target),
                        );
                        let layout = self
                            .blit_mask_bind_group_layout
                            .as_ref()
                            .expect("blit mask bind group layout must exist");
                        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("fret blur mask bind group"),
                            layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&src_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::TextureView(&mask_view),
                                },
                            ],
                        });

                        let (pipeline, label) = match pass.axis {
                            BlurAxis::Horizontal => (
                                self.blur_h_mask_pipeline
                                    .as_ref()
                                    .expect("blur-h mask pipeline must exist"),
                                "fret blur-h mask pass",
                            ),
                            BlurAxis::Vertical => (
                                self.blur_v_mask_pipeline
                                    .as_ref()
                                    .expect("blur-v mask pipeline must exist"),
                                "fret blur-v mask pass",
                            ),
                        };

                        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some(label),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: dst_view,
                                depth_slice: None,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: pass.load,
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });
                        rp.set_pipeline(pipeline);
                        rp.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                        rp.set_bind_group(1, &bind_group, &[]);
                        if let Some(scissor) = pass.dst_scissor {
                            if scissor.w != 0 && scissor.h != 0 {
                                rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
                            }
                        }
                        rp.draw(0..3, 0..1);
                    } else if let Some(mask_uniform_index) = pass.mask_uniform_index {
                        let layout = self
                            .blit_bind_group_layout
                            .as_ref()
                            .expect("blit bind group layout must exist");
                        let bind_group = create_texture_bind_group(
                            device,
                            "fret blur bind group",
                            layout,
                            &src_view,
                        );
                        let (pipeline, label) = match pass.axis {
                            BlurAxis::Horizontal => (
                                self.blur_h_masked_pipeline
                                    .as_ref()
                                    .expect("blur-h masked pipeline must exist"),
                                "fret blur-h masked pass",
                            ),
                            BlurAxis::Vertical => (
                                self.blur_v_masked_pipeline
                                    .as_ref()
                                    .expect("blur-v masked pipeline must exist"),
                                "fret blur-v masked pass",
                            ),
                        };
                        let uniform_offset =
                            (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

                        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some(label),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: dst_view,
                                depth_slice: None,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: pass.load,
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });
                        rp.set_pipeline(pipeline);
                        rp.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                        rp.set_bind_group(1, &bind_group, &[]);
                        if let Some(scissor) = pass.dst_scissor {
                            if scissor.w != 0 && scissor.h != 0 {
                                rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
                            }
                        }
                        rp.draw(0..3, 0..1);
                    } else {
                        let layout = self
                            .blit_bind_group_layout
                            .as_ref()
                            .expect("blit bind group layout must exist");
                        let bind_group = create_texture_bind_group(
                            device,
                            "fret blur bind group",
                            layout,
                            &src_view,
                        );
                        let blur_pipeline = match pass.axis {
                            BlurAxis::Horizontal => self
                                .blur_h_pipeline
                                .as_ref()
                                .expect("blur-h pipeline must exist"),
                            BlurAxis::Vertical => self
                                .blur_v_pipeline
                                .as_ref()
                                .expect("blur-v pipeline must exist"),
                        };
                        let label = match pass.axis {
                            BlurAxis::Horizontal => "fret blur-h pass",
                            BlurAxis::Vertical => "fret blur-v pass",
                        };
                        run_fullscreen_triangle_pass(
                            &mut encoder,
                            label,
                            blur_pipeline,
                            dst_view,
                            pass.load,
                            &bind_group,
                            pass.dst_scissor,
                        );
                    }
                }
                RenderPlanPass::FullscreenBlit(pass) => {
                    let blit_pipeline = self
                        .blit_pipeline
                        .as_ref()
                        .expect("blit pipeline must exist");

                    let layout = self
                        .blit_bind_group_layout
                        .as_ref()
                        .expect("blit bind group layout must exist");
                    let src_view = match pass.src {
                        PlanTarget::Output
                        | PlanTarget::Mask0
                        | PlanTarget::Mask1
                        | PlanTarget::Mask2 => {
                            debug_assert!(
                                false,
                                "FullscreenBlit src cannot be Output/mask targets"
                            );
                            continue;
                        }
                        PlanTarget::Intermediate0
                        | PlanTarget::Intermediate1
                        | PlanTarget::Intermediate2 => {
                            frame_targets.require_target(pass.src, pass.src_size)
                        }
                    };
                    let blit_bind_group = create_texture_bind_group(
                        device,
                        "fret blit bind group",
                        layout,
                        &src_view,
                    );

                    let dst_view_owned = match pass.dst {
                        PlanTarget::Output => None,
                        PlanTarget::Intermediate0
                        | PlanTarget::Intermediate1
                        | PlanTarget::Intermediate2 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            pass.dst,
                            pass.dst_size,
                            format,
                            usage,
                            self.intermediate_budget_bytes,
                        )),
                        PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                            debug_assert!(false, "FullscreenBlit dst cannot be mask targets");
                            None
                        }
                    };
                    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

                    run_fullscreen_triangle_pass(
                        &mut encoder,
                        "fret blit pass",
                        blit_pipeline,
                        dst_view,
                        pass.load,
                        &blit_bind_group,
                        pass.dst_scissor,
                    );
                }
                RenderPlanPass::ColorAdjust(pass) => {
                    queue.write_buffer(
                        &self.color_adjust_param_buffer,
                        0,
                        bytemuck::cast_slice(&[
                            pass.saturation,
                            pass.brightness,
                            pass.contrast,
                            0.0,
                        ]),
                    );

                    let src_view = match pass.src {
                        PlanTarget::Output
                        | PlanTarget::Mask0
                        | PlanTarget::Mask1
                        | PlanTarget::Mask2 => {
                            debug_assert!(false, "ColorAdjust src cannot be Output/mask targets");
                            continue;
                        }
                        PlanTarget::Intermediate0
                        | PlanTarget::Intermediate1
                        | PlanTarget::Intermediate2 => {
                            frame_targets.require_target(pass.src, pass.src_size)
                        }
                    };

                    let dst_view_owned = match pass.dst {
                        PlanTarget::Output => None,
                        PlanTarget::Intermediate0
                        | PlanTarget::Intermediate1
                        | PlanTarget::Intermediate2 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            pass.dst,
                            pass.dst_size,
                            format,
                            usage,
                            self.intermediate_budget_bytes,
                        )),
                        PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                            debug_assert!(false, "ColorAdjust dst cannot be mask targets");
                            None
                        }
                    };
                    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

                    if let Some(mask_target) = pass.mask_target {
                        debug_assert!(matches!(
                            mask_target,
                            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
                        ));
                        debug_assert_eq!(
                            pass.dst_size, viewport_size,
                            "mask-based color-adjust expects full-size destination"
                        );

                        let mask_uniform_index = pass
                            .mask_uniform_index
                            .expect("mask color-adjust needs uniform index");
                        let uniform_offset =
                            (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

                        let mask_view = frame_targets.require_target(
                            mask_target,
                            mask_target_size(viewport_size, mask_target),
                        );
                        let layout = self
                            .color_adjust_mask_bind_group_layout
                            .as_ref()
                            .expect("color-adjust mask bind group layout must exist");
                        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("fret color-adjust mask bind group"),
                            layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&src_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: self.color_adjust_param_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: wgpu::BindingResource::TextureView(&mask_view),
                                },
                            ],
                        });

                        let pipeline = self
                            .color_adjust_mask_pipeline
                            .as_ref()
                            .expect("color-adjust mask pipeline must exist");

                        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("fret color-adjust mask pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: dst_view,
                                depth_slice: None,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: pass.load,
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });
                        rp.set_pipeline(pipeline);
                        rp.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                        rp.set_bind_group(1, &bind_group, &[]);
                        if let Some(scissor) = pass.dst_scissor {
                            if scissor.w != 0 && scissor.h != 0 {
                                rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
                            }
                        }
                        rp.draw(0..3, 0..1);
                    } else if let Some(mask_uniform_index) = pass.mask_uniform_index {
                        let layout = self
                            .color_adjust_bind_group_layout
                            .as_ref()
                            .expect("color-adjust bind group layout must exist");
                        let bind_group = create_texture_uniform_bind_group(
                            device,
                            "fret color-adjust bind group",
                            layout,
                            &src_view,
                            self.color_adjust_param_buffer.as_entire_binding(),
                        );
                        let pipeline = self
                            .color_adjust_masked_pipeline
                            .as_ref()
                            .expect("color-adjust masked pipeline must exist");
                        let uniform_offset =
                            (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

                        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("fret color-adjust masked pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: dst_view,
                                depth_slice: None,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: pass.load,
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });
                        rp.set_pipeline(pipeline);
                        rp.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                        rp.set_bind_group(1, &bind_group, &[]);
                        if let Some(scissor) = pass.dst_scissor {
                            if scissor.w != 0 && scissor.h != 0 {
                                rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
                            }
                        }
                        rp.draw(0..3, 0..1);
                    } else {
                        let layout = self
                            .color_adjust_bind_group_layout
                            .as_ref()
                            .expect("color-adjust bind group layout must exist");
                        let bind_group = create_texture_uniform_bind_group(
                            device,
                            "fret color-adjust bind group",
                            layout,
                            &src_view,
                            self.color_adjust_param_buffer.as_entire_binding(),
                        );
                        let pipeline = self
                            .color_adjust_pipeline
                            .as_ref()
                            .expect("color-adjust pipeline must exist");
                        run_fullscreen_triangle_pass(
                            &mut encoder,
                            "fret color-adjust pass",
                            pipeline,
                            dst_view,
                            pass.load,
                            &bind_group,
                            pass.dst_scissor,
                        );
                    }
                }
                RenderPlanPass::CompositePremul(pass) => {
                    let src_view = match pass.src {
                        PlanTarget::Output
                        | PlanTarget::Mask0
                        | PlanTarget::Mask1
                        | PlanTarget::Mask2 => {
                            debug_assert!(
                                false,
                                "CompositePremul src cannot be Output/mask targets"
                            );
                            continue;
                        }
                        PlanTarget::Intermediate0
                        | PlanTarget::Intermediate1
                        | PlanTarget::Intermediate2 => {
                            frame_targets.require_target(pass.src, pass.src_size)
                        }
                    };

                    let dst_view_owned = match pass.dst {
                        PlanTarget::Output => None,
                        PlanTarget::Intermediate0
                        | PlanTarget::Intermediate1
                        | PlanTarget::Intermediate2 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            pass.dst,
                            pass.dst_size,
                            format,
                            usage,
                            self.intermediate_budget_bytes,
                        )),
                        PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                            debug_assert!(false, "CompositePremul dst cannot be mask targets");
                            None
                        }
                    };
                    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

                    let (composite_pipeline, bind_group) =
                        if let Some(mask_target) = pass.mask_target {
                            debug_assert!(matches!(
                                mask_target,
                                PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
                            ));
                            debug_assert_eq!(
                                pass.dst_size, viewport_size,
                                "mask-based composite expects full-size destination"
                            );

                            let mask_view = frame_targets.require_target(
                                mask_target,
                                mask_target_size(viewport_size, mask_target),
                            );
                            let layout = self
                                .composite_mask_bind_group_layout
                                .as_ref()
                                .expect("composite mask bind group layout must exist");
                            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                                label: Some("fret composite premul mask bind group"),
                                layout,
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: wgpu::BindingResource::Sampler(
                                            &self.viewport_sampler,
                                        ),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: wgpu::BindingResource::TextureView(&src_view),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 2,
                                        resource: wgpu::BindingResource::TextureView(&mask_view),
                                    },
                                ],
                            });

                            (
                                self.composite_mask_pipeline
                                    .as_ref()
                                    .expect("composite premul mask pipeline must exist"),
                                bind_group,
                            )
                        } else {
                            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                                label: Some("fret composite premul bind group"),
                                layout: &self.viewport_bind_group_layout,
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: wgpu::BindingResource::Sampler(
                                            &self.viewport_sampler,
                                        ),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: wgpu::BindingResource::TextureView(&src_view),
                                    },
                                ],
                            });
                            (
                                self.composite_pipeline
                                    .as_ref()
                                    .expect("composite premul pipeline must exist"),
                                bind_group,
                            )
                        };

                    let x0 = 0.0;
                    let y0 = 0.0;
                    let x1 = pass.dst_size.0 as f32;
                    let y1 = pass.dst_size.1 as f32;
                    let vertices = [
                        ViewportVertex {
                            pos_px: [x0, y0],
                            uv: [0.0, 0.0],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y0],
                            uv: [1.0, 0.0],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y1],
                            uv: [1.0, 1.0],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x0, y0],
                            uv: [0.0, 0.0],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y1],
                            uv: [1.0, 1.0],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x0, y1],
                            uv: [0.0, 1.0],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                    ];
                    queue.write_buffer(
                        &self.path_composite_vertices,
                        0,
                        bytemuck::cast_slice(&vertices),
                    );

                    let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("fret composite premul pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: dst_view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: pass.load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    rp.set_pipeline(composite_pipeline);
                    if let Some(mask_uniform_index) = pass.mask_uniform_index {
                        let uniform_offset =
                            (u64::from(mask_uniform_index) * self.uniform_stride) as u32;
                        rp.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                    } else {
                        rp.set_bind_group(0, &self.uniform_bind_group, &[0]);
                    }
                    rp.set_bind_group(1, &bind_group, &[]);
                    rp.set_vertex_buffer(0, self.path_composite_vertices.slice(..));
                    if let Some(scissor) = pass.dst_scissor {
                        if scissor.w != 0 && scissor.h != 0 {
                            rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
                        }
                    }
                    rp.draw(0..6, 0..1);
                }
                RenderPlanPass::ClipMask(pass) => {
                    debug_assert!(matches!(
                        pass.dst,
                        PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
                    ));
                    queue.write_buffer(
                        &self.clip_mask_param_buffer,
                        0,
                        bytemuck::cast_slice(&[
                            pass.dst_size.0 as f32,
                            pass.dst_size.1 as f32,
                            0.0,
                            0.0,
                        ]),
                    );
                    let dst_view = frame_targets.ensure_target(
                        &mut self.intermediate_pool,
                        device,
                        pass.dst,
                        pass.dst_size,
                        wgpu::TextureFormat::R8Unorm,
                        usage,
                        self.intermediate_budget_bytes,
                    );

                    let pipeline = self
                        .clip_mask_pipeline
                        .as_ref()
                        .expect("clip mask pipeline must exist");
                    let uniform_offset =
                        (u64::from(pass.uniform_index) * self.uniform_stride) as u32;

                    let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("fret clip mask pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &dst_view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    rp.set_pipeline(pipeline);
                    rp.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                    rp.set_bind_group(1, &self.clip_mask_param_bind_group, &[]);
                    if let Some(scissor) = pass.dst_scissor {
                        if scissor.w != 0 && scissor.h != 0 {
                            rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
                        }
                    }
                    rp.draw(0..3, 0..1);
                }
                RenderPlanPass::ReleaseTarget(target) => {
                    frame_targets.release_target(
                        &mut self.intermediate_pool,
                        *target,
                        self.intermediate_budget_bytes,
                    );
                }
            }
        }

        let cmd = encoder.finish();

        if self.intermediate_perf_enabled {
            self.intermediate_perf.last_frame_in_use_bytes = frame_targets.in_use_bytes();
            self.intermediate_perf.last_frame_peak_in_use_bytes = frame_targets.peak_in_use_bytes();
        }
        frame_targets.release_all(&mut self.intermediate_pool, self.intermediate_budget_bytes);

        // Keep the most recent encoding for potential reuse on the next frame.
        if cache_hit {
            self.scene_encoding_cache_key = Some(key);
        }
        self.scene_encoding_cache = encoding;
        cmd
    }
}

// FrameTargets moved to `renderer/frame_targets.rs`.
