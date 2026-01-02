use super::super::*;
use std::time::Instant;

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
                TextMask,
                TextColor,
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
            let text_color_pipeline = self
                .text_color_pipeline
                .as_ref()
                .expect("text color pipeline must exist");
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

                        match draw.kind {
                            TextDrawKind::Mask => {
                                if !matches!(active_pipeline, ActivePipeline::TextMask) {
                                    pass.set_pipeline(text_pipeline);
                                    pass.set_vertex_buffer(0, text_vertex_buffer.slice(..));
                                    pass.set_bind_group(
                                        1,
                                        self.text_system.mask_atlas_bind_group(),
                                        &[],
                                    );
                                    active_pipeline = ActivePipeline::TextMask;
                                }
                            }
                            TextDrawKind::Color => {
                                if !matches!(active_pipeline, ActivePipeline::TextColor) {
                                    pass.set_pipeline(text_color_pipeline);
                                    pass.set_vertex_buffer(0, text_vertex_buffer.slice(..));
                                    pass.set_bind_group(
                                        1,
                                        self.text_system.color_atlas_bind_group(),
                                        &[],
                                    );
                                    active_pipeline = ActivePipeline::TextColor;
                                }
                            }
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
}
