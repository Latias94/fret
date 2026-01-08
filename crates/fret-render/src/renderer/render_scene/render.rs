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
        } else if self.debug_offscreen_blit_enabled {
            DebugPostprocess::OffscreenBlit
        } else {
            DebugPostprocess::None
        };
        if !matches!(postprocess, DebugPostprocess::None) {
            self.ensure_blit_pipeline(device, format);
        }
        if matches!(postprocess, DebugPostprocess::Pixelate { .. }) {
            self.ensure_scale_nearest_pipelines(device, format);
        }
        let plan = RenderPlan::compile_for_scene(&encoding, clear.0, path_samples, postprocess);

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
                        )),
                        PlanTarget::Intermediate1 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            PlanTarget::Intermediate1,
                            viewport_size,
                            format,
                            usage,
                        )),
                        PlanTarget::Intermediate2 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            PlanTarget::Intermediate2,
                            viewport_size,
                            format,
                            usage,
                        )),
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
                        )),
                        PlanTarget::Intermediate1 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            PlanTarget::Intermediate1,
                            viewport_size,
                            format,
                            usage,
                        )),
                        PlanTarget::Intermediate2 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            PlanTarget::Intermediate2,
                            viewport_size,
                            format,
                            usage,
                        )),
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
                    let full_size = viewport_size;
                    let down_size = (
                        full_size.0.max(1).div_ceil(scale),
                        full_size.1.max(1).div_ceil(scale),
                    );

                    let target_size = |target: PlanTarget| match target {
                        PlanTarget::Output => full_size,
                        PlanTarget::Intermediate2 => down_size,
                        PlanTarget::Intermediate0 | PlanTarget::Intermediate1 => full_size,
                    };

                    queue.write_buffer(
                        &self.scale_param_buffer,
                        0,
                        bytemuck::cast_slice(&[scale, 0, 0, 0]),
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
                    let layout = self
                        .scale_bind_group_layout
                        .as_ref()
                        .expect("scale bind group layout must exist");
                    let bind_group = {
                        let src_view = match pass.src {
                            PlanTarget::Output => {
                                debug_assert!(false, "ScaleNearest src cannot be Output");
                                continue;
                            }
                            PlanTarget::Intermediate0 => frame_targets.ensure_target(
                                &mut self.intermediate_pool,
                                device,
                                PlanTarget::Intermediate0,
                                target_size(PlanTarget::Intermediate0),
                                format,
                                usage,
                            ),
                            PlanTarget::Intermediate1 => frame_targets.ensure_target(
                                &mut self.intermediate_pool,
                                device,
                                PlanTarget::Intermediate1,
                                target_size(PlanTarget::Intermediate1),
                                format,
                                usage,
                            ),
                            PlanTarget::Intermediate2 => frame_targets.ensure_target(
                                &mut self.intermediate_pool,
                                device,
                                PlanTarget::Intermediate2,
                                target_size(PlanTarget::Intermediate2),
                                format,
                                usage,
                            ),
                        };

                        device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("fret scale-nearest bind group"),
                            layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&src_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: self.scale_param_buffer.as_entire_binding(),
                                },
                            ],
                        })
                    };

                    let dst_view_owned = match pass.dst {
                        PlanTarget::Output => None,
                        PlanTarget::Intermediate0 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            PlanTarget::Intermediate0,
                            target_size(PlanTarget::Intermediate0),
                            format,
                            usage,
                        )),
                        PlanTarget::Intermediate1 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            PlanTarget::Intermediate1,
                            target_size(PlanTarget::Intermediate1),
                            format,
                            usage,
                        )),
                        PlanTarget::Intermediate2 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            PlanTarget::Intermediate2,
                            target_size(PlanTarget::Intermediate2),
                            format,
                            usage,
                        )),
                    };
                    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

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
                    rp.set_bind_group(0, &bind_group, &[]);
                    rp.draw(0..3, 0..1);
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
                    let blit_bind_group = {
                        let src_view = match pass.src {
                            PlanTarget::Output => {
                                debug_assert!(false, "FullscreenBlit src cannot be Output");
                                continue;
                            }
                            PlanTarget::Intermediate0 => frame_targets.ensure_target(
                                &mut self.intermediate_pool,
                                device,
                                PlanTarget::Intermediate0,
                                viewport_size,
                                format,
                                usage,
                            ),
                            PlanTarget::Intermediate1 => frame_targets.ensure_target(
                                &mut self.intermediate_pool,
                                device,
                                PlanTarget::Intermediate1,
                                viewport_size,
                                format,
                                usage,
                            ),
                            PlanTarget::Intermediate2 => frame_targets.ensure_target(
                                &mut self.intermediate_pool,
                                device,
                                PlanTarget::Intermediate2,
                                viewport_size,
                                format,
                                usage,
                            ),
                        };

                        device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("fret blit bind group"),
                            layout,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&src_view),
                            }],
                        })
                    };

                    let dst_view_owned = match pass.dst {
                        PlanTarget::Output => None,
                        PlanTarget::Intermediate0 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            PlanTarget::Intermediate0,
                            viewport_size,
                            format,
                            usage,
                        )),
                        PlanTarget::Intermediate1 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            PlanTarget::Intermediate1,
                            viewport_size,
                            format,
                            usage,
                        )),
                        PlanTarget::Intermediate2 => Some(frame_targets.ensure_target(
                            &mut self.intermediate_pool,
                            device,
                            PlanTarget::Intermediate2,
                            viewport_size,
                            format,
                            usage,
                        )),
                    };
                    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

                    let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("fret blit pass"),
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
                    rp.set_pipeline(blit_pipeline);
                    rp.set_bind_group(0, &blit_bind_group, &[]);
                    rp.draw(0..3, 0..1);
                }
            }
        }

        let cmd = encoder.finish();

        frame_targets.release_all(&mut self.intermediate_pool);

        // Keep the most recent encoding for potential reuse on the next frame.
        if cache_hit {
            self.scene_encoding_cache_key = Some(key);
        }
        self.scene_encoding_cache = encoding;
        cmd
    }
}

#[derive(Default)]
struct FrameTargets {
    intermediate0: Option<FrameTarget>,
    intermediate1: Option<FrameTarget>,
    intermediate2: Option<FrameTarget>,
}

struct FrameTarget {
    size: (u32, u32),
    texture: PooledTexture,
    view: wgpu::TextureView,
}

impl FrameTargets {
    fn ensure_target(
        &mut self,
        pool: &mut IntermediatePool,
        device: &wgpu::Device,
        target: PlanTarget,
        size: (u32, u32),
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> wgpu::TextureView {
        let size = (size.0.max(1), size.1.max(1));
        let slot = match target {
            PlanTarget::Intermediate0 => &mut self.intermediate0,
            PlanTarget::Intermediate1 => &mut self.intermediate1,
            PlanTarget::Intermediate2 => &mut self.intermediate2,
            PlanTarget::Output => unreachable!("Output is not an intermediate target"),
        };

        if slot.as_ref().is_some_and(|existing| existing.size == size) {
            return slot.as_ref().unwrap().view.clone();
        }

        if let Some(existing) = slot.take() {
            pool.release(existing.texture);
        }

        let texture =
            pool.acquire_texture(device, "fret intermediate target", size, format, usage, 1);
        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        slot.replace(FrameTarget {
            size,
            texture,
            view,
        });
        slot.as_ref().unwrap().view.clone()
    }

    fn release_all(&mut self, pool: &mut IntermediatePool) {
        if let Some(t) = self.intermediate0.take() {
            pool.release(t.texture);
        }
        if let Some(t) = self.intermediate1.take() {
            pool.release(t.texture);
        }
        if let Some(t) = self.intermediate2.take() {
            pool.release(t.texture);
        }
    }
}
