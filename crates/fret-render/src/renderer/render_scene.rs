use super::*;
use std::time::Instant;

use crate::svg::SMOOTH_SVG_SCALE_FACTOR;

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
        let svg_prepare_start = Instant::now();
        self.prepare_svg_ops(device, queue, scene, scale_factor);
        if self.svg_perf_enabled {
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
            self.encode_scene_ops_into(scene, scale_factor, viewport_size, &mut encoding);

            // Preserve the old cache's allocations for reuse.
            self.scene_encoding_scratch = std::mem::take(&mut self.scene_encoding_cache);
            self.scene_encoding_cache_key = Some(key);
            encoding
        };

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

        {
            enum ActivePipeline {
                None,
                Quad,
                Viewport,
                Text,
                Mask,
                Composite,
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
            let mask_pipeline = self
                .mask_pipeline
                .as_ref()
                .expect("mask pipeline must exist");
            let composite_pipeline = self
                .composite_pipeline
                .as_ref()
                .expect("composite pipeline must exist");
            let path_pipeline = self
                .path_pipeline
                .as_ref()
                .expect("path pipeline must exist");
            let path_msaa_pipeline = self.path_msaa_pipeline.as_ref();

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

            let mut pass = begin_main_pass(&mut encoder, target_view, wgpu::LoadOp::Clear(clear.0));
            let mut active_uniform_offset: Option<u32> = None;

            let mut i = 0usize;
            while i < encoding.ordered_draws.len() {
                let item = &encoding.ordered_draws[i];

                if let OrderedDraw::Path(first) = item
                    && path_samples > 1
                {
                    let mut union = first.scissor;
                    let batch_uniform_index = first.uniform_index;
                    let mut end = i + 1;
                    while end < encoding.ordered_draws.len() {
                        match &encoding.ordered_draws[end] {
                            OrderedDraw::Path(d) if d.uniform_index == batch_uniform_index => {
                                union = union_scissor(union, d.scissor);
                                end += 1;
                            }
                            _ => break,
                        }
                    }

                    // Render the path batch to an intermediate MSAA target, then composite into the
                    // main pass to preserve strict op ordering.
                    drop(pass);

                    let Some(intermediate) = &self.path_intermediate else {
                        pass = begin_main_pass(&mut encoder, target_view, wgpu::LoadOp::Load);
                        i = end;
                        continue;
                    };

                    {
                        let Some(path_msaa_pipeline) = path_msaa_pipeline else {
                            pass = begin_main_pass(&mut encoder, target_view, wgpu::LoadOp::Load);
                            i = end;
                            continue;
                        };

                        let mut path_pass =
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

                        path_pass.set_pipeline(path_msaa_pipeline);
                        path_pass.set_vertex_buffer(0, path_vertex_buffer.slice(..));

                        let mut active_uniform_offset: Option<u32> = None;
                        for j in i..end {
                            let OrderedDraw::Path(draw) = &encoding.ordered_draws[j] else {
                                unreachable!();
                            };
                            if draw.scissor.w == 0 || draw.scissor.h == 0 {
                                continue;
                            }
                            path_pass.set_scissor_rect(
                                draw.scissor.x,
                                draw.scissor.y,
                                draw.scissor.w,
                                draw.scissor.h,
                            );
                            let uniform_offset =
                                (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                            if active_uniform_offset != Some(uniform_offset) {
                                path_pass.set_bind_group(
                                    0,
                                    &self.uniform_bind_group,
                                    &[uniform_offset],
                                );
                                active_uniform_offset = Some(uniform_offset);
                            }
                            path_pass.draw(
                                draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                                0..1,
                            );
                        }
                    }

                    pass = begin_main_pass(&mut encoder, target_view, wgpu::LoadOp::Load);
                    active_pipeline = ActivePipeline::None;
                    active_uniform_offset = None;

                    if union.w > 0 && union.h > 0 {
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

                        pass.set_pipeline(composite_pipeline);
                        let uniform_offset =
                            (u64::from(batch_uniform_index) * self.uniform_stride) as u32;
                        pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                        pass.set_bind_group(1, &intermediate.bind_group, &[]);
                        pass.set_vertex_buffer(0, self.path_composite_vertices.slice(..));
                        pass.set_scissor_rect(union.x, union.y, union.w, union.h);
                        pass.draw(0..6, 0..1);
                        active_pipeline = ActivePipeline::Composite;
                    }

                    i = end;
                    continue;
                }

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

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
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
                            draw.first_instance..(draw.first_instance + draw.instance_count),
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

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                            active_uniform_offset = Some(uniform_offset);
                        }
                        let Some((_, bind_group)) = self.viewport_bind_groups.get(&draw.target)
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

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                            active_uniform_offset = Some(uniform_offset);
                        }
                        let Some((_, bind_group)) = self.image_bind_groups.get(&draw.image) else {
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

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                            active_uniform_offset = Some(uniform_offset);
                        }
                        let Some((_, bind_group)) = self.image_bind_groups.get(&draw.image) else {
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

                        if !matches!(active_pipeline, ActivePipeline::Text) {
                            pass.set_pipeline(text_pipeline);
                            pass.set_vertex_buffer(0, text_vertex_buffer.slice(..));
                            pass.set_bind_group(1, self.text_system.atlas_bind_group(), &[]);
                            active_pipeline = ActivePipeline::Text;
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
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

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
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

        let cmd = encoder.finish();

        // Keep the most recent encoding for potential reuse on the next frame.
        if cache_hit {
            self.scene_encoding_cache_key = Some(key);
        }
        self.scene_encoding_cache = encoding;
        cmd
    }

    fn prepare_viewport_bind_groups(&mut self, device: &wgpu::Device, draws: &[OrderedDraw]) {
        for item in draws {
            let OrderedDraw::Viewport(draw) = item else {
                continue;
            };

            let target = draw.target;
            let Some(view) = self.render_targets.get(target) else {
                continue;
            };

            let revision = self
                .render_target_revisions
                .get(&target)
                .copied()
                .unwrap_or(0);
            let needs_rebuild = match self.viewport_bind_groups.get(&target) {
                Some((cached, _)) => *cached != revision,
                None => true,
            };
            if !needs_rebuild {
                continue;
            }

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fret viewport texture bind group"),
                layout: &self.viewport_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&self.viewport_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(view),
                    },
                ],
            });

            self.viewport_bind_groups
                .insert(target, (revision, bind_group));
        }
    }

    fn prepare_image_bind_groups(&mut self, device: &wgpu::Device, draws: &[OrderedDraw]) {
        for item in draws {
            let image = match item {
                OrderedDraw::Image(draw) => draw.image,
                OrderedDraw::Mask(draw) => draw.image,
                _ => continue,
            };
            let Some(view) = self.images.get(image) else {
                continue;
            };

            let revision = self.image_revisions.get(&image).copied().unwrap_or(0);
            let needs_rebuild = match self.image_bind_groups.get(&image) {
                Some((cached, _)) => *cached != revision,
                None => true,
            };
            if !needs_rebuild {
                continue;
            }

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fret image texture bind group"),
                layout: &self.viewport_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&self.viewport_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(view),
                    },
                ],
            });

            self.image_bind_groups.insert(image, (revision, bind_group));
        }
    }

    fn encode_scene_ops_into(
        &mut self,
        scene: &Scene,
        scale_factor: f32,
        viewport_size: (u32, u32),
        encoding: &mut SceneEncoding,
    ) {
        encoding.clear();
        let instances = &mut encoding.instances;
        let viewport_vertices = &mut encoding.viewport_vertices;
        let text_vertices = &mut encoding.text_vertices;
        let path_vertices = &mut encoding.path_vertices;
        let clips = &mut encoding.clips;
        let uniforms = &mut encoding.uniforms;
        let ordered_draws = &mut encoding.ordered_draws;

        let mut scissor_stack: Vec<ScissorRect> =
            vec![ScissorRect::full(viewport_size.0, viewport_size.1)];

        let mut current_scissor = *scissor_stack
            .last()
            .expect("scissor stack must be non-empty");

        #[derive(Clone, Copy)]
        enum ClipPop {
            NoShader,
            Shader { prev_head: u32 },
        }

        let mut clip_pop_stack: Vec<ClipPop> = Vec::new();
        let mut clip_head: u32 = 0;
        let mut clip_count: u32 = 0;

        let mut push_uniform_snapshot = |clip_head: u32, clip_count: u32| -> u32 {
            let uniform_index = uniforms.len() as u32;
            uniforms.push(ViewportUniform {
                viewport_size: [viewport_size.0 as f32, viewport_size.1 as f32],
                clip_head,
                clip_count,
            });
            uniform_index
        };
        let mut current_uniform_index: u32 = push_uniform_snapshot(0, 0);

        let mut quad_batch: Option<(ScissorRect, u32, u32)> = None;

        macro_rules! flush_quad_batch {
            () => {{
                if let Some((scissor, uniform_index, first_instance)) = quad_batch.take() {
                    let instance_count = (instances.len() as u32).saturating_sub(first_instance);
                    if instance_count > 0 {
                        ordered_draws.push(OrderedDraw::Quad(DrawCall {
                            scissor,
                            uniform_index,
                            first_instance,
                            instance_count,
                        }));
                    }
                }
            }};
        }

        let mut transform_stack: Vec<Transform2D> = vec![Transform2D::IDENTITY];
        let mut opacity_stack: Vec<f32> = vec![1.0];

        let to_physical_px = |t: Transform2D| t.to_physical_px(scale_factor);
        let current_transform_px = |stack: &[Transform2D]| {
            to_physical_px(*stack.last().expect("transform stack must be non-empty"))
        };

        let current_transform_max_scale = |t: Transform2D| -> f32 {
            if let Some((s, _)) = t.as_translation_uniform_scale()
                && s.is_finite()
                && s > 0.0
            {
                return s;
            }

            let sx = (t.a * t.a + t.b * t.b).sqrt();
            let sy = (t.c * t.c + t.d * t.d).sqrt();
            let s = sx.max(sy);
            if s.is_finite() && s > 0.0 { s } else { 1.0 }
        };

        let transform_rows = |t_px: Transform2D| -> ([f32; 4], [f32; 4]) {
            (
                [t_px.a, t_px.c, t_px.tx, 0.0],
                [t_px.b, t_px.d, t_px.ty, 0.0],
            )
        };

        let apply_transform_px = |t_px: Transform2D, x: f32, y: f32| -> (f32, f32) {
            let p = t_px.apply_point(Point::new(Px(x), Px(y)));
            (p.x.0, p.y.0)
        };

        let transform_quad_points_px =
            |t_px: Transform2D, x: f32, y: f32, w: f32, h: f32| -> [(f32, f32); 4] {
                let (x0, y0) = (x, y);
                let (x1, y1) = (x + w, y + h);
                [
                    apply_transform_px(t_px, x0, y0),    // TL
                    apply_transform_px(t_px, x + w, y0), // TR
                    apply_transform_px(t_px, x1, y1),    // BR
                    apply_transform_px(t_px, x0, y1),    // BL
                ]
            };

        let bounds_of_quad_points = |pts: &[(f32, f32); 4]| -> (f32, f32, f32, f32) {
            let mut min_x = pts[0].0;
            let mut max_x = pts[0].0;
            let mut min_y = pts[0].1;
            let mut max_y = pts[0].1;
            for (x, y) in pts.iter().copied() {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
            (min_x, min_y, max_x, max_y)
        };

        let color_with_opacity = |mut c: Color, opacity: f32| -> Color {
            c.a = (c.a * opacity).clamp(0.0, 1.0);
            c
        };

        for op in scene.ops() {
            match op {
                SceneOp::PushTransform { transform } => {
                    let current = *transform_stack
                        .last()
                        .expect("transform stack must be non-empty");
                    transform_stack.push(current * *transform);
                }
                SceneOp::PopTransform => {
                    if transform_stack.len() > 1 {
                        transform_stack.pop();
                    }
                }
                SceneOp::PushOpacity { opacity } => {
                    let current = *opacity_stack
                        .last()
                        .expect("opacity stack must be non-empty");
                    opacity_stack.push((current * opacity).clamp(0.0, 1.0));
                }
                SceneOp::PopOpacity => {
                    if opacity_stack.len() > 1 {
                        opacity_stack.pop();
                    }
                }
                SceneOp::PushLayer { .. } | SceneOp::PopLayer => {
                    flush_quad_batch!();
                }
                SceneOp::PushClipRect { rect } => {
                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    let new_scissor = if w <= 0.0 || h <= 0.0 {
                        Some(ScissorRect {
                            x: 0,
                            y: 0,
                            w: 0,
                            h: 0,
                        })
                    } else {
                        let t_px = to_physical_px(
                            *transform_stack
                                .last()
                                .expect("transform stack must be non-empty"),
                        );
                        let quad = transform_quad_points_px(t_px, x, y, w, h);
                        let (min_x, min_y, max_x, max_y) = bounds_of_quad_points(&quad);
                        scissor_from_bounds_px(min_x, min_y, max_x, max_y, viewport_size)
                    };
                    let Some(new_scissor) = new_scissor else {
                        continue;
                    };

                    let combined = intersect_scissor(current_scissor, new_scissor);
                    if combined != current_scissor {
                        flush_quad_batch!();
                    }

                    current_scissor = combined;
                    scissor_stack.push(current_scissor);

                    if w <= 0.0 || h <= 0.0 {
                        clip_pop_stack.push(ClipPop::NoShader);
                        continue;
                    }

                    let t_px = current_transform_px(&transform_stack);
                    let is_axis_aligned = t_px.b == 0.0 && t_px.c == 0.0;
                    if is_axis_aligned {
                        clip_pop_stack.push(ClipPop::NoShader);
                        continue;
                    }

                    let Some(inv_px) = t_px.inverse() else {
                        clip_pop_stack.push(ClipPop::NoShader);
                        continue;
                    };

                    flush_quad_batch!();
                    let prev_head = if clip_count > 0 { clip_head } else { u32::MAX };
                    let node_index = clips.len() as u32;
                    let parent_bits = f32::from_bits(prev_head);
                    clips.push(ClipRRectUniform {
                        rect: [x, y, w, h],
                        corner_radii: [0.0; 4],
                        inv0: [inv_px.a, inv_px.c, inv_px.tx, parent_bits],
                        inv1: [inv_px.b, inv_px.d, inv_px.ty, 0.0],
                    });
                    clip_head = node_index;
                    clip_count = clip_count.saturating_add(1);
                    current_uniform_index = push_uniform_snapshot(clip_head, clip_count);
                    clip_pop_stack.push(ClipPop::Shader { prev_head });
                }
                SceneOp::PushClipRRect { rect, corner_radii } => {
                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    let radii = corners_to_vec4(*corner_radii).map(|r| r * scale_factor);
                    let radii = if w > 0.0 && h > 0.0 {
                        clamp_corner_radii_for_rect(w, h, radii)
                    } else {
                        [0.0; 4]
                    };

                    let new_scissor = if w <= 0.0 || h <= 0.0 {
                        Some(ScissorRect {
                            x: 0,
                            y: 0,
                            w: 0,
                            h: 0,
                        })
                    } else {
                        let t_px = current_transform_px(&transform_stack);
                        let quad = transform_quad_points_px(t_px, x, y, w, h);
                        let (min_x, min_y, max_x, max_y) = bounds_of_quad_points(&quad);
                        scissor_from_bounds_px(min_x, min_y, max_x, max_y, viewport_size)
                    };
                    let Some(new_scissor) = new_scissor else {
                        continue;
                    };

                    let combined = intersect_scissor(current_scissor, new_scissor);
                    if combined != current_scissor {
                        flush_quad_batch!();
                    }

                    current_scissor = combined;
                    scissor_stack.push(current_scissor);

                    if w <= 0.0 || h <= 0.0 {
                        clip_pop_stack.push(ClipPop::NoShader);
                        continue;
                    }

                    let t_px = current_transform_px(&transform_stack);
                    let is_axis_aligned = t_px.b == 0.0 && t_px.c == 0.0;
                    let is_rect = radii.iter().all(|r| *r <= 0.0);
                    if is_axis_aligned && is_rect {
                        clip_pop_stack.push(ClipPop::NoShader);
                        continue;
                    }

                    let Some(inv_px) = t_px.inverse() else {
                        clip_pop_stack.push(ClipPop::NoShader);
                        continue;
                    };

                    flush_quad_batch!();
                    let prev_head = if clip_count > 0 { clip_head } else { u32::MAX };
                    let node_index = clips.len() as u32;
                    let parent_bits = f32::from_bits(prev_head);
                    clips.push(ClipRRectUniform {
                        rect: [x, y, w, h],
                        corner_radii: radii,
                        inv0: [inv_px.a, inv_px.c, inv_px.tx, parent_bits],
                        inv1: [inv_px.b, inv_px.d, inv_px.ty, 0.0],
                    });
                    clip_head = node_index;
                    clip_count = clip_count.saturating_add(1);
                    current_uniform_index = push_uniform_snapshot(clip_head, clip_count);
                    clip_pop_stack.push(ClipPop::Shader { prev_head });
                }
                SceneOp::PopClip => {
                    if scissor_stack.len() > 1 {
                        scissor_stack.pop();
                        let new_scissor = *scissor_stack
                            .last()
                            .expect("scissor stack must be non-empty");
                        if new_scissor != current_scissor {
                            flush_quad_batch!();
                            current_scissor = new_scissor;
                        }
                    }

                    if let Some(ClipPop::Shader { prev_head }) = clip_pop_stack.pop() {
                        flush_quad_batch!();
                        clip_count = clip_count.saturating_sub(1);
                        clip_head = if clip_count == 0 || prev_head == u32::MAX {
                            0
                        } else {
                            prev_head
                        };
                        current_uniform_index = push_uniform_snapshot(clip_head, clip_count);
                    }
                }
                SceneOp::Quad {
                    rect,
                    background,
                    border,
                    border_color,
                    corner_radii,
                    ..
                } => {
                    let opacity = *opacity_stack
                        .last()
                        .expect("opacity stack must be non-empty");

                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    let background = color_with_opacity(*background, opacity);
                    let border_color = color_with_opacity(*border_color, opacity);

                    if background.a <= 0.0 && border_color.a <= 0.0 {
                        continue;
                    }
                    if w <= 0.0 || h <= 0.0 {
                        continue;
                    }

                    let needs_new_batch = match quad_batch {
                        Some((scissor, uniform_index, _)) => {
                            scissor != current_scissor || uniform_index != current_uniform_index
                        }
                        None => true,
                    };

                    if needs_new_batch {
                        flush_quad_batch!();
                        quad_batch = Some((
                            current_scissor,
                            current_uniform_index,
                            instances.len() as u32,
                        ));
                    }

                    let t_px = to_physical_px(
                        *transform_stack
                            .last()
                            .expect("transform stack must be non-empty"),
                    );
                    let (transform0, transform1) = transform_rows(t_px);

                    let corner_radii = corners_to_vec4(*corner_radii).map(|r| r * scale_factor);
                    let corner_radii = clamp_corner_radii_for_rect(w, h, corner_radii);
                    let border = edges_to_vec4(*border).map(|e| e * scale_factor);
                    instances.push(QuadInstance {
                        rect: [x, y, w, h],
                        transform0,
                        transform1,
                        color: color_to_linear_rgba_premul(background),
                        corner_radii,
                        border,
                        border_color: color_to_linear_rgba_premul(border_color),
                    });
                }
                SceneOp::Image { .. } => {
                    flush_quad_batch!();
                    let SceneOp::Image {
                        rect,
                        image,
                        opacity,
                        ..
                    } = op
                    else {
                        unreachable!();
                    };
                    let group_opacity = *opacity_stack
                        .last()
                        .expect("opacity stack must be non-empty");

                    if *opacity <= 0.0 || group_opacity <= 0.0 {
                        continue;
                    }
                    if self.images.get(*image).is_none() {
                        continue;
                    }
                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    if w <= 0.0 || h <= 0.0 {
                        continue;
                    }
                    let t_px = current_transform_px(&transform_stack);
                    let quad = transform_quad_points_px(t_px, x, y, w, h);

                    let first_vertex = viewport_vertices.len() as u32;
                    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);

                    viewport_vertices.extend_from_slice(&[
                        ViewportVertex {
                            pos_px: [quad[0].0, quad[0].1],
                            uv: [0.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[1].0, quad[1].1],
                            uv: [1.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[2].0, quad[2].1],
                            uv: [1.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[0].0, quad[0].1],
                            uv: [0.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[2].0, quad[2].1],
                            uv: [1.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[3].0, quad[3].1],
                            uv: [0.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                    ]);

                    ordered_draws.push(OrderedDraw::Image(ImageDraw {
                        scissor: current_scissor,
                        uniform_index: current_uniform_index,
                        first_vertex,
                        vertex_count: 6,
                        image: *image,
                    }));
                }
                SceneOp::ImageRegion { .. } => {
                    flush_quad_batch!();
                    let SceneOp::ImageRegion {
                        rect,
                        image,
                        uv,
                        opacity,
                        ..
                    } = op
                    else {
                        unreachable!();
                    };
                    let group_opacity = *opacity_stack
                        .last()
                        .expect("opacity stack must be non-empty");

                    if *opacity <= 0.0 || group_opacity <= 0.0 {
                        continue;
                    }
                    if self.images.get(*image).is_none() {
                        continue;
                    }
                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    if w <= 0.0 || h <= 0.0 {
                        continue;
                    }
                    let t_px = current_transform_px(&transform_stack);
                    let quad = transform_quad_points_px(t_px, x, y, w, h);

                    let first_vertex = viewport_vertices.len() as u32;
                    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);

                    let (u0, v0, u1, v1) = (uv.u0, uv.v0, uv.u1, uv.v1);
                    viewport_vertices.extend_from_slice(&[
                        ViewportVertex {
                            pos_px: [quad[0].0, quad[0].1],
                            uv: [u0, v0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[1].0, quad[1].1],
                            uv: [u1, v0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[2].0, quad[2].1],
                            uv: [u1, v1],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[0].0, quad[0].1],
                            uv: [u0, v0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[2].0, quad[2].1],
                            uv: [u1, v1],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[3].0, quad[3].1],
                            uv: [u0, v1],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                    ]);

                    ordered_draws.push(OrderedDraw::Image(ImageDraw {
                        scissor: current_scissor,
                        uniform_index: current_uniform_index,
                        first_vertex,
                        vertex_count: 6,
                        image: *image,
                    }));
                }
                SceneOp::MaskImage { .. } => {
                    flush_quad_batch!();
                    let SceneOp::MaskImage {
                        rect,
                        image,
                        uv,
                        color,
                        opacity,
                        ..
                    } = op
                    else {
                        unreachable!();
                    };
                    let group_opacity = *opacity_stack
                        .last()
                        .expect("opacity stack must be non-empty");

                    if *opacity <= 0.0 || group_opacity <= 0.0 || color.a <= 0.0 {
                        continue;
                    }
                    if self.images.get(*image).is_none() {
                        continue;
                    }
                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    if w <= 0.0 || h <= 0.0 {
                        continue;
                    }
                    let t_px = current_transform_px(&transform_stack);
                    let quad = transform_quad_points_px(t_px, x, y, w, h);

                    let first_vertex = text_vertices.len() as u32;
                    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);
                    let mut premul = color_to_linear_rgba_premul(*color);
                    premul = premul.map(|c| c * o);

                    let (u0, v0, u1, v1) = (uv.u0, uv.v0, uv.u1, uv.v1);
                    text_vertices.extend_from_slice(&[
                        TextVertex {
                            pos_px: [quad[0].0, quad[0].1],
                            uv: [u0, v0],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [quad[1].0, quad[1].1],
                            uv: [u1, v0],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [quad[2].0, quad[2].1],
                            uv: [u1, v1],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [quad[0].0, quad[0].1],
                            uv: [u0, v0],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [quad[2].0, quad[2].1],
                            uv: [u1, v1],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [quad[3].0, quad[3].1],
                            uv: [u0, v1],
                            color: premul,
                        },
                    ]);

                    ordered_draws.push(OrderedDraw::Mask(MaskDraw {
                        scissor: current_scissor,
                        uniform_index: current_uniform_index,
                        first_vertex,
                        vertex_count: 6,
                        image: *image,
                    }));
                }
                SceneOp::SvgMaskIcon { .. } => {
                    flush_quad_batch!();
                    let SceneOp::SvgMaskIcon {
                        rect,
                        svg,
                        fit,
                        color,
                        opacity,
                        ..
                    } = op
                    else {
                        unreachable!();
                    };
                    let group_opacity = *opacity_stack
                        .last()
                        .expect("opacity stack must be non-empty");

                    if *opacity <= 0.0 || group_opacity <= 0.0 || color.a <= 0.0 {
                        continue;
                    }

                    let t = *transform_stack
                        .last()
                        .expect("transform stack must be non-empty");
                    let s = current_transform_max_scale(t);
                    let key_rect = Rect::new(
                        rect.origin,
                        Size::new(Px(rect.size.width.0 * s), Px(rect.size.height.0 * s)),
                    );
                    let key = Self::svg_raster_key(
                        *svg,
                        key_rect,
                        scale_factor,
                        SvgRasterKind::AlphaMask,
                        *fit,
                    );
                    let Some(entry) = self.svg_rasters.get(&key) else {
                        continue;
                    };
                    if self.images.get(entry.image).is_none() {
                        continue;
                    }

                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    if w <= 0.0 || h <= 0.0 {
                        continue;
                    }

                    let first_vertex = text_vertices.len() as u32;
                    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);
                    let mut premul = color_to_linear_rgba_premul(*color);
                    premul = premul.map(|c| c * o);

                    let (lx0, ly0, lx1, ly1) =
                        svg_draw_rect_px(x, y, w, h, entry.size_px, SMOOTH_SVG_SCALE_FACTOR, *fit);
                    let t_px = current_transform_px(&transform_stack);
                    let quad = [
                        apply_transform_px(t_px, lx0, ly0),
                        apply_transform_px(t_px, lx1, ly0),
                        apply_transform_px(t_px, lx1, ly1),
                        apply_transform_px(t_px, lx0, ly1),
                    ];

                    let (u0, v0, u1, v1) = (entry.uv.u0, entry.uv.v0, entry.uv.u1, entry.uv.v1);
                    text_vertices.extend_from_slice(&[
                        TextVertex {
                            pos_px: [quad[0].0, quad[0].1],
                            uv: [u0, v0],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [quad[1].0, quad[1].1],
                            uv: [u1, v0],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [quad[2].0, quad[2].1],
                            uv: [u1, v1],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [quad[0].0, quad[0].1],
                            uv: [u0, v0],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [quad[2].0, quad[2].1],
                            uv: [u1, v1],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [quad[3].0, quad[3].1],
                            uv: [u0, v1],
                            color: premul,
                        },
                    ]);

                    let (min_x, min_y, max_x, max_y) = bounds_of_quad_points(&quad);
                    let svg_scissor =
                        scissor_from_bounds_px(min_x, min_y, max_x, max_y, viewport_size)
                            .map(|s| intersect_scissor(current_scissor, s))
                            .unwrap_or(current_scissor);
                    ordered_draws.push(OrderedDraw::Mask(MaskDraw {
                        scissor: svg_scissor,
                        uniform_index: current_uniform_index,
                        first_vertex,
                        vertex_count: 6,
                        image: entry.image,
                    }));
                }
                SceneOp::SvgImage { .. } => {
                    flush_quad_batch!();
                    let SceneOp::SvgImage {
                        rect,
                        svg,
                        fit,
                        opacity,
                        ..
                    } = op
                    else {
                        unreachable!();
                    };
                    let group_opacity = *opacity_stack
                        .last()
                        .expect("opacity stack must be non-empty");

                    if *opacity <= 0.0 || group_opacity <= 0.0 {
                        continue;
                    }

                    let t = *transform_stack
                        .last()
                        .expect("transform stack must be non-empty");
                    let s = current_transform_max_scale(t);
                    let key_rect = Rect::new(
                        rect.origin,
                        Size::new(Px(rect.size.width.0 * s), Px(rect.size.height.0 * s)),
                    );

                    let key = Self::svg_raster_key(
                        *svg,
                        key_rect,
                        scale_factor,
                        SvgRasterKind::Rgba,
                        *fit,
                    );
                    let Some(entry) = self.svg_rasters.get(&key) else {
                        continue;
                    };
                    if self.images.get(entry.image).is_none() {
                        continue;
                    }

                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    if w <= 0.0 || h <= 0.0 {
                        continue;
                    }

                    let first_vertex = viewport_vertices.len() as u32;
                    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);

                    let (lx0, ly0, lx1, ly1) =
                        svg_draw_rect_px(x, y, w, h, entry.size_px, SMOOTH_SVG_SCALE_FACTOR, *fit);
                    let t_px = current_transform_px(&transform_stack);
                    let quad = [
                        apply_transform_px(t_px, lx0, ly0),
                        apply_transform_px(t_px, lx1, ly0),
                        apply_transform_px(t_px, lx1, ly1),
                        apply_transform_px(t_px, lx0, ly1),
                    ];

                    viewport_vertices.extend_from_slice(&[
                        ViewportVertex {
                            pos_px: [quad[0].0, quad[0].1],
                            uv: [0.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[1].0, quad[1].1],
                            uv: [1.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[2].0, quad[2].1],
                            uv: [1.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[0].0, quad[0].1],
                            uv: [0.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[2].0, quad[2].1],
                            uv: [1.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[3].0, quad[3].1],
                            uv: [0.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                    ]);

                    let (min_x, min_y, max_x, max_y) = bounds_of_quad_points(&quad);
                    let svg_scissor =
                        scissor_from_bounds_px(min_x, min_y, max_x, max_y, viewport_size)
                            .map(|s| intersect_scissor(current_scissor, s))
                            .unwrap_or(current_scissor);
                    ordered_draws.push(OrderedDraw::Image(ImageDraw {
                        scissor: svg_scissor,
                        uniform_index: current_uniform_index,
                        first_vertex,
                        vertex_count: 6,
                        image: entry.image,
                    }));
                }
                SceneOp::Text {
                    origin,
                    text,
                    color,
                    ..
                } => {
                    flush_quad_batch!();

                    let Some(blob) = self.text_system.blob(*text) else {
                        continue;
                    };

                    let group_opacity = *opacity_stack
                        .last()
                        .expect("opacity stack must be non-empty");
                    if group_opacity <= 0.0 || color.a <= 0.0 {
                        continue;
                    }

                    let t_px = current_transform_px(&transform_stack);

                    let first_vertex = text_vertices.len() as u32;

                    let base_x = origin.x.0 * scale_factor;
                    let base_y = origin.y.0 * scale_factor;
                    let premul =
                        color_to_linear_rgba_premul(color_with_opacity(*color, group_opacity));

                    for g in &blob.glyphs {
                        let lx0 = base_x + g.rect[0] * scale_factor;
                        let ly0 = base_y + g.rect[1] * scale_factor;
                        let lx1 = lx0 + g.rect[2] * scale_factor;
                        let ly1 = ly0 + g.rect[3] * scale_factor;
                        let quad = [
                            apply_transform_px(t_px, lx0, ly0),
                            apply_transform_px(t_px, lx1, ly0),
                            apply_transform_px(t_px, lx1, ly1),
                            apply_transform_px(t_px, lx0, ly1),
                        ];

                        let (u0, v0, u1, v1) = (g.uv[0], g.uv[1], g.uv[2], g.uv[3]);

                        text_vertices.extend_from_slice(&[
                            TextVertex {
                                pos_px: [quad[0].0, quad[0].1],
                                uv: [u0, v0],
                                color: premul,
                            },
                            TextVertex {
                                pos_px: [quad[1].0, quad[1].1],
                                uv: [u1, v0],
                                color: premul,
                            },
                            TextVertex {
                                pos_px: [quad[2].0, quad[2].1],
                                uv: [u1, v1],
                                color: premul,
                            },
                            TextVertex {
                                pos_px: [quad[0].0, quad[0].1],
                                uv: [u0, v0],
                                color: premul,
                            },
                            TextVertex {
                                pos_px: [quad[2].0, quad[2].1],
                                uv: [u1, v1],
                                color: premul,
                            },
                            TextVertex {
                                pos_px: [quad[3].0, quad[3].1],
                                uv: [u0, v1],
                                color: premul,
                            },
                        ]);
                    }

                    let vertex_count = (text_vertices.len() as u32).saturating_sub(first_vertex);
                    if vertex_count > 0 {
                        ordered_draws.push(OrderedDraw::Text(TextDraw {
                            scissor: current_scissor,
                            uniform_index: current_uniform_index,
                            first_vertex,
                            vertex_count,
                        }));
                    }
                }
                SceneOp::Path { .. } => {
                    flush_quad_batch!();
                    let SceneOp::Path {
                        origin,
                        path,
                        color,
                        ..
                    } = op
                    else {
                        unreachable!();
                    };
                    let group_opacity = *opacity_stack
                        .last()
                        .expect("opacity stack must be non-empty");
                    if color.a <= 0.0 || group_opacity <= 0.0 {
                        continue;
                    }
                    let Some(prepared) = self.paths.get(*path) else {
                        continue;
                    };
                    if prepared.triangles.is_empty() {
                        continue;
                    }
                    let t_px = current_transform_px(&transform_stack);

                    let local_bounds = Rect::new(
                        fret_core::Point::new(
                            origin.x + prepared.metrics.bounds.origin.x,
                            origin.y + prepared.metrics.bounds.origin.y,
                        ),
                        prepared.metrics.bounds.size,
                    );
                    let (bx, by, bw, bh) = rect_to_pixels(local_bounds, scale_factor);
                    let bounds_quad = transform_quad_points_px(t_px, bx, by, bw, bh);
                    let (min_x, min_y, max_x, max_y) = bounds_of_quad_points(&bounds_quad);
                    let Some(bounds_scissor) =
                        scissor_from_bounds_px(min_x, min_y, max_x, max_y, viewport_size)
                    else {
                        continue;
                    };
                    let clipped_scissor = intersect_scissor(current_scissor, bounds_scissor);
                    if clipped_scissor.w == 0 || clipped_scissor.h == 0 {
                        continue;
                    }

                    let first_vertex = path_vertices.len() as u32;
                    let ox = origin.x.0 * scale_factor;
                    let oy = origin.y.0 * scale_factor;
                    let premul =
                        color_to_linear_rgba_premul(color_with_opacity(*color, group_opacity));

                    for p in &prepared.triangles {
                        let lx = ox + p[0] * scale_factor;
                        let ly = oy + p[1] * scale_factor;
                        let (wx, wy) = apply_transform_px(t_px, lx, ly);
                        path_vertices.push(PathVertex {
                            pos_px: [wx, wy],
                            color: premul,
                        });
                    }

                    let vertex_count = (path_vertices.len() as u32).saturating_sub(first_vertex);
                    if vertex_count > 0 {
                        ordered_draws.push(OrderedDraw::Path(PathDraw {
                            scissor: clipped_scissor,
                            uniform_index: current_uniform_index,
                            first_vertex,
                            vertex_count,
                        }));
                    }
                }
                SceneOp::ViewportSurface {
                    rect,
                    target,
                    opacity,
                    ..
                } => {
                    flush_quad_batch!();
                    let group_opacity = *opacity_stack
                        .last()
                        .expect("opacity stack must be non-empty");

                    if *opacity <= 0.0 || group_opacity <= 0.0 {
                        continue;
                    }
                    if self.render_targets.get(*target).is_none() {
                        continue;
                    }
                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    if w <= 0.0 || h <= 0.0 {
                        continue;
                    }
                    let t_px = current_transform_px(&transform_stack);
                    let quad = transform_quad_points_px(t_px, x, y, w, h);

                    let first_vertex = viewport_vertices.len() as u32;
                    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);

                    viewport_vertices.extend_from_slice(&[
                        ViewportVertex {
                            pos_px: [quad[0].0, quad[0].1],
                            uv: [0.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[1].0, quad[1].1],
                            uv: [1.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[2].0, quad[2].1],
                            uv: [1.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[0].0, quad[0].1],
                            uv: [0.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[2].0, quad[2].1],
                            uv: [1.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [quad[3].0, quad[3].1],
                            uv: [0.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                    ]);

                    ordered_draws.push(OrderedDraw::Viewport(ViewportDraw {
                        scissor: current_scissor,
                        uniform_index: current_uniform_index,
                        first_vertex,
                        vertex_count: 6,
                        target: *target,
                    }));
                }
            }
        }

        flush_quad_batch!();
    }
}
