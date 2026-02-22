use super::super::super::*;
use super::super::ctx::ExecuteCtx;
use super::super::helpers::set_scissor_rect_absolute;

impl Renderer {
    pub(in super::super) fn record_color_adjust_pass(
        &mut self,
        ctx: &mut ExecuteCtx<'_>,
        pass: &ColorAdjustPass,
    ) {
        let device = ctx.device;
        let queue = ctx.queue;
        let format = ctx.format;
        let target_view = ctx.target_view;
        let viewport_size = ctx.viewport_size;
        let usage = ctx.usage;
        let encoder = &mut *ctx.encoder;
        let frame_targets = &mut *ctx.frame_targets;
        let encoding = ctx.encoding;
        let render_space_offset_u32 = ctx.render_space_offset_u32;
        let perf_enabled = ctx.perf_enabled;
        let frame_perf = &mut *ctx.frame_perf;

        queue.write_buffer(
            &self.color_adjust_param_buffer,
            0,
            bytemuck::cast_slice(&[pass.saturation, pass.brightness, pass.contrast, 0.0]),
        );
        if perf_enabled {
            frame_perf.uniform_bytes = frame_perf
                .uniform_bytes
                .saturating_add(std::mem::size_of::<[f32; 4]>() as u64);
        }

        let src_view = match pass.src {
            PlanTarget::Output | PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                debug_assert!(false, "ColorAdjust src cannot be Output/mask targets");
                return;
            }
            PlanTarget::Intermediate0 | PlanTarget::Intermediate1 | PlanTarget::Intermediate2 => {
                frame_targets.require_target(pass.src, pass.src_size)
            }
        };

        let dst_view_owned = match pass.dst {
            PlanTarget::Output => None,
            PlanTarget::Intermediate0 | PlanTarget::Intermediate1 | PlanTarget::Intermediate2 => {
                Some(frame_targets.ensure_target(
                    &mut self.intermediate_pool,
                    device,
                    pass.dst,
                    pass.dst_size,
                    format,
                    usage,
                ))
            }
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                debug_assert!(false, "ColorAdjust dst cannot be mask targets");
                None
            }
        };
        let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

        if let Some(mask) = pass.mask {
            debug_assert!(matches!(
                mask.target,
                PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
            ));
            debug_assert_eq!(
                pass.dst_size, viewport_size,
                "mask-based color-adjust expects full-size destination"
            );

            let mask_uniform_index = pass
                .mask_uniform_index
                .expect("mask color-adjust needs uniform index");
            let uniform_offset = (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

            let mask_view = frame_targets.require_target(mask.target, mask.size);
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
            if perf_enabled {
                frame_perf.pipeline_switches = frame_perf.pipeline_switches.saturating_add(1);
                frame_perf.pipeline_switches_fullscreen =
                    frame_perf.pipeline_switches_fullscreen.saturating_add(1);
            }
            rp.set_bind_group(
                0,
                self.pick_uniform_bind_group_for_mask_image(
                    encoding
                        .uniform_mask_images
                        .get(mask_uniform_index as usize)
                        .copied()
                        .flatten(),
                ),
                &[uniform_offset, render_space_offset_u32],
            );
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
                frame_perf.uniform_bind_group_switches =
                    frame_perf.uniform_bind_group_switches.saturating_add(1);
            }
            rp.set_bind_group(1, &bind_group, &[]);
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
                frame_perf.texture_bind_group_switches =
                    frame_perf.texture_bind_group_switches.saturating_add(1);
            }
            if let Some(scissor) = pass.dst_scissor
                && scissor.w != 0
                && scissor.h != 0
            {
                rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
                if perf_enabled {
                    frame_perf.scissor_sets = frame_perf.scissor_sets.saturating_add(1);
                }
            }
            rp.draw(0..3, 0..1);
            if perf_enabled {
                frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
                frame_perf.fullscreen_draw_calls =
                    frame_perf.fullscreen_draw_calls.saturating_add(1);
            }
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
            let uniform_offset = (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

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
            if perf_enabled {
                frame_perf.pipeline_switches = frame_perf.pipeline_switches.saturating_add(1);
                frame_perf.pipeline_switches_fullscreen =
                    frame_perf.pipeline_switches_fullscreen.saturating_add(1);
            }
            rp.set_bind_group(
                0,
                self.pick_uniform_bind_group_for_mask_image(
                    encoding
                        .uniform_mask_images
                        .get(mask_uniform_index as usize)
                        .copied()
                        .flatten(),
                ),
                &[uniform_offset, render_space_offset_u32],
            );
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
                frame_perf.uniform_bind_group_switches =
                    frame_perf.uniform_bind_group_switches.saturating_add(1);
            }
            rp.set_bind_group(1, &bind_group, &[]);
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
                frame_perf.texture_bind_group_switches =
                    frame_perf.texture_bind_group_switches.saturating_add(1);
            }
            if let Some(scissor) = pass.dst_scissor
                && scissor.w != 0
                && scissor.h != 0
            {
                rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
                if perf_enabled {
                    frame_perf.scissor_sets = frame_perf.scissor_sets.saturating_add(1);
                }
            }
            rp.draw(0..3, 0..1);
            if perf_enabled {
                frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
                frame_perf.fullscreen_draw_calls =
                    frame_perf.fullscreen_draw_calls.saturating_add(1);
            }
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
                encoder,
                "fret color-adjust pass",
                pipeline,
                dst_view,
                pass.load,
                &bind_group,
                &[],
                pass.dst_scissor,
                perf_enabled.then_some(frame_perf),
            );
        }
    }

    pub(in super::super) fn record_color_matrix_pass(
        &mut self,
        ctx: &mut ExecuteCtx<'_>,
        pass: &ColorMatrixPass,
    ) {
        let device = ctx.device;
        let queue = ctx.queue;
        let format = ctx.format;
        let target_view = ctx.target_view;
        let viewport_size = ctx.viewport_size;
        let usage = ctx.usage;
        let encoder = &mut *ctx.encoder;
        let frame_targets = &mut *ctx.frame_targets;
        let encoding = ctx.encoding;
        let render_space_offset_u32 = ctx.render_space_offset_u32;
        let perf_enabled = ctx.perf_enabled;
        let frame_perf = &mut *ctx.frame_perf;

        let m = pass.matrix;
        let packed: [f32; 20] = [
            // row0 (col0..3)
            m[0], m[1], m[2], m[3], // row1 (col0..3)
            m[5], m[6], m[7], m[8], // row2 (col0..3)
            m[10], m[11], m[12], m[13], // row3 (col0..3)
            m[15], m[16], m[17], m[18], // bias (col4)
            m[4], m[9], m[14], m[19],
        ];
        queue.write_buffer(
            &self.color_matrix_param_buffer,
            0,
            bytemuck::cast_slice(&packed),
        );
        if perf_enabled {
            frame_perf.uniform_bytes = frame_perf
                .uniform_bytes
                .saturating_add(std::mem::size_of::<[f32; 20]>() as u64);
        }

        let src_view = match pass.src {
            PlanTarget::Output | PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                debug_assert!(false, "ColorMatrix src cannot be Output/mask targets");
                return;
            }
            PlanTarget::Intermediate0 | PlanTarget::Intermediate1 | PlanTarget::Intermediate2 => {
                frame_targets.require_target(pass.src, pass.src_size)
            }
        };

        let dst_view_owned = match pass.dst {
            PlanTarget::Output => None,
            PlanTarget::Intermediate0 | PlanTarget::Intermediate1 | PlanTarget::Intermediate2 => {
                Some(frame_targets.ensure_target(
                    &mut self.intermediate_pool,
                    device,
                    pass.dst,
                    pass.dst_size,
                    format,
                    usage,
                ))
            }
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                debug_assert!(false, "ColorMatrix dst cannot be mask targets");
                None
            }
        };
        let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

        if let Some(mask) = pass.mask {
            debug_assert!(matches!(
                mask.target,
                PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
            ));
            debug_assert_eq!(
                pass.dst_size, viewport_size,
                "mask-based color-matrix expects full-size destination"
            );

            let mask_uniform_index = pass
                .mask_uniform_index
                .expect("mask color-matrix needs uniform index");
            let uniform_offset = (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

            let mask_view = frame_targets.require_target(mask.target, mask.size);
            let layout = self
                .color_matrix_mask_bind_group_layout
                .as_ref()
                .expect("color-matrix mask bind group layout must exist");
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fret color-matrix mask bind group"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&src_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.color_matrix_param_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&mask_view),
                    },
                ],
            });
            let pipeline = self
                .color_matrix_mask_pipeline
                .as_ref()
                .expect("color-matrix mask pipeline must exist");

            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fret color-matrix mask pass"),
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
            if perf_enabled {
                frame_perf.pipeline_switches = frame_perf.pipeline_switches.saturating_add(1);
                frame_perf.pipeline_switches_fullscreen =
                    frame_perf.pipeline_switches_fullscreen.saturating_add(1);
            }
            rp.set_bind_group(
                0,
                self.pick_uniform_bind_group_for_mask_image(
                    encoding
                        .uniform_mask_images
                        .get(mask_uniform_index as usize)
                        .copied()
                        .flatten(),
                ),
                &[uniform_offset, render_space_offset_u32],
            );
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
                frame_perf.uniform_bind_group_switches =
                    frame_perf.uniform_bind_group_switches.saturating_add(1);
            }
            rp.set_bind_group(1, &bind_group, &[]);
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
                frame_perf.texture_bind_group_switches =
                    frame_perf.texture_bind_group_switches.saturating_add(1);
            }
            if let Some(scissor) = pass.dst_scissor
                && scissor.w != 0
                && scissor.h != 0
            {
                rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
                if perf_enabled {
                    frame_perf.scissor_sets = frame_perf.scissor_sets.saturating_add(1);
                }
            }
            rp.draw(0..3, 0..1);
            if perf_enabled {
                frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
                frame_perf.fullscreen_draw_calls =
                    frame_perf.fullscreen_draw_calls.saturating_add(1);
            }
        } else if let Some(mask_uniform_index) = pass.mask_uniform_index {
            let layout = self
                .color_matrix_bind_group_layout
                .as_ref()
                .expect("color-matrix bind group layout must exist");
            let bind_group = create_texture_uniform_bind_group(
                device,
                "fret color-matrix bind group",
                layout,
                &src_view,
                self.color_matrix_param_buffer.as_entire_binding(),
            );
            let pipeline = self
                .color_matrix_masked_pipeline
                .as_ref()
                .expect("color-matrix masked pipeline must exist");
            let uniform_offset = (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fret color-matrix masked pass"),
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
            if perf_enabled {
                frame_perf.pipeline_switches = frame_perf.pipeline_switches.saturating_add(1);
                frame_perf.pipeline_switches_fullscreen =
                    frame_perf.pipeline_switches_fullscreen.saturating_add(1);
            }
            rp.set_bind_group(
                0,
                self.pick_uniform_bind_group_for_mask_image(
                    encoding
                        .uniform_mask_images
                        .get(mask_uniform_index as usize)
                        .copied()
                        .flatten(),
                ),
                &[uniform_offset, render_space_offset_u32],
            );
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
                frame_perf.uniform_bind_group_switches =
                    frame_perf.uniform_bind_group_switches.saturating_add(1);
            }
            rp.set_bind_group(1, &bind_group, &[]);
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
                frame_perf.texture_bind_group_switches =
                    frame_perf.texture_bind_group_switches.saturating_add(1);
            }
            if let Some(scissor) = pass.dst_scissor
                && scissor.w != 0
                && scissor.h != 0
            {
                rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
                if perf_enabled {
                    frame_perf.scissor_sets = frame_perf.scissor_sets.saturating_add(1);
                }
            }
            rp.draw(0..3, 0..1);
            if perf_enabled {
                frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
                frame_perf.fullscreen_draw_calls =
                    frame_perf.fullscreen_draw_calls.saturating_add(1);
            }
        } else {
            let layout = self
                .color_matrix_bind_group_layout
                .as_ref()
                .expect("color-matrix bind group layout must exist");
            let bind_group = create_texture_uniform_bind_group(
                device,
                "fret color-matrix bind group",
                layout,
                &src_view,
                self.color_matrix_param_buffer.as_entire_binding(),
            );
            let pipeline = self
                .color_matrix_pipeline
                .as_ref()
                .expect("color-matrix pipeline must exist");
            run_fullscreen_triangle_pass(
                encoder,
                "fret color-matrix pass",
                pipeline,
                dst_view,
                pass.load,
                &bind_group,
                &[],
                pass.dst_scissor,
                perf_enabled.then_some(frame_perf),
            );
        }
    }

    pub(in super::super) fn record_alpha_threshold_pass(
        &mut self,
        ctx: &mut ExecuteCtx<'_>,
        pass: &AlphaThresholdPass,
    ) {
        let device = ctx.device;
        let queue = ctx.queue;
        let format = ctx.format;
        let target_view = ctx.target_view;
        let viewport_size = ctx.viewport_size;
        let usage = ctx.usage;
        let encoder = &mut *ctx.encoder;
        let frame_targets = &mut *ctx.frame_targets;
        let encoding = ctx.encoding;
        let render_space_offset_u32 = ctx.render_space_offset_u32;
        let perf_enabled = ctx.perf_enabled;
        let frame_perf = &mut *ctx.frame_perf;

        queue.write_buffer(
            &self.alpha_threshold_param_buffer,
            0,
            bytemuck::cast_slice(&[pass.cutoff, pass.soft, 0.0, 0.0]),
        );
        if perf_enabled {
            frame_perf.uniform_bytes = frame_perf
                .uniform_bytes
                .saturating_add(std::mem::size_of::<[f32; 4]>() as u64);
        }

        let src_view = match pass.src {
            PlanTarget::Output | PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                debug_assert!(false, "AlphaThreshold src cannot be Output/mask targets");
                return;
            }
            PlanTarget::Intermediate0 | PlanTarget::Intermediate1 | PlanTarget::Intermediate2 => {
                frame_targets.require_target(pass.src, pass.src_size)
            }
        };

        let dst_view_owned = match pass.dst {
            PlanTarget::Output => None,
            PlanTarget::Intermediate0 | PlanTarget::Intermediate1 | PlanTarget::Intermediate2 => {
                Some(frame_targets.ensure_target(
                    &mut self.intermediate_pool,
                    device,
                    pass.dst,
                    pass.dst_size,
                    format,
                    usage,
                ))
            }
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                debug_assert!(false, "AlphaThreshold dst cannot be mask targets");
                None
            }
        };
        let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

        if let Some(mask) = pass.mask {
            debug_assert!(matches!(
                mask.target,
                PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
            ));
            debug_assert_eq!(
                pass.dst_size, viewport_size,
                "mask-based alpha-threshold expects full-size destination"
            );

            let mask_uniform_index = pass
                .mask_uniform_index
                .expect("mask alpha-threshold needs uniform index");
            let uniform_offset = (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

            let mask_view = frame_targets.require_target(mask.target, mask.size);
            let layout = self
                .alpha_threshold_mask_bind_group_layout
                .as_ref()
                .expect("alpha-threshold mask bind group layout must exist");
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fret alpha-threshold mask bind group"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&src_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.alpha_threshold_param_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&mask_view),
                    },
                ],
            });
            let pipeline = self
                .alpha_threshold_mask_pipeline
                .as_ref()
                .expect("alpha-threshold mask pipeline must exist");

            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fret alpha-threshold mask pass"),
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
            if perf_enabled {
                frame_perf.pipeline_switches = frame_perf.pipeline_switches.saturating_add(1);
                frame_perf.pipeline_switches_fullscreen =
                    frame_perf.pipeline_switches_fullscreen.saturating_add(1);
            }
            rp.set_bind_group(
                0,
                self.pick_uniform_bind_group_for_mask_image(
                    encoding
                        .uniform_mask_images
                        .get(mask_uniform_index as usize)
                        .copied()
                        .flatten(),
                ),
                &[uniform_offset, render_space_offset_u32],
            );
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
                frame_perf.uniform_bind_group_switches =
                    frame_perf.uniform_bind_group_switches.saturating_add(1);
            }
            rp.set_bind_group(1, &bind_group, &[]);
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
                frame_perf.texture_bind_group_switches =
                    frame_perf.texture_bind_group_switches.saturating_add(1);
            }
            if let Some(scissor) = pass.dst_scissor
                && scissor.w != 0
                && scissor.h != 0
            {
                rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
                if perf_enabled {
                    frame_perf.scissor_sets = frame_perf.scissor_sets.saturating_add(1);
                }
            }
            rp.draw(0..3, 0..1);
            if perf_enabled {
                frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
                frame_perf.fullscreen_draw_calls =
                    frame_perf.fullscreen_draw_calls.saturating_add(1);
            }
        } else if let Some(mask_uniform_index) = pass.mask_uniform_index {
            let layout = self
                .alpha_threshold_bind_group_layout
                .as_ref()
                .expect("alpha-threshold bind group layout must exist");
            let bind_group = create_texture_uniform_bind_group(
                device,
                "fret alpha-threshold bind group",
                layout,
                &src_view,
                self.alpha_threshold_param_buffer.as_entire_binding(),
            );
            let pipeline = self
                .alpha_threshold_masked_pipeline
                .as_ref()
                .expect("alpha-threshold masked pipeline must exist");
            let uniform_offset = (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fret alpha-threshold masked pass"),
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
            if perf_enabled {
                frame_perf.pipeline_switches = frame_perf.pipeline_switches.saturating_add(1);
                frame_perf.pipeline_switches_fullscreen =
                    frame_perf.pipeline_switches_fullscreen.saturating_add(1);
            }
            rp.set_bind_group(
                0,
                self.pick_uniform_bind_group_for_mask_image(
                    encoding
                        .uniform_mask_images
                        .get(mask_uniform_index as usize)
                        .copied()
                        .flatten(),
                ),
                &[uniform_offset, render_space_offset_u32],
            );
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
                frame_perf.uniform_bind_group_switches =
                    frame_perf.uniform_bind_group_switches.saturating_add(1);
            }
            rp.set_bind_group(1, &bind_group, &[]);
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
                frame_perf.texture_bind_group_switches =
                    frame_perf.texture_bind_group_switches.saturating_add(1);
            }
            if let Some(scissor) = pass.dst_scissor
                && scissor.w != 0
                && scissor.h != 0
            {
                rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
                if perf_enabled {
                    frame_perf.scissor_sets = frame_perf.scissor_sets.saturating_add(1);
                }
            }
            rp.draw(0..3, 0..1);
            if perf_enabled {
                frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
                frame_perf.fullscreen_draw_calls =
                    frame_perf.fullscreen_draw_calls.saturating_add(1);
            }
        } else {
            let layout = self
                .alpha_threshold_bind_group_layout
                .as_ref()
                .expect("alpha-threshold bind group layout must exist");
            let bind_group = create_texture_uniform_bind_group(
                device,
                "fret alpha-threshold bind group",
                layout,
                &src_view,
                self.alpha_threshold_param_buffer.as_entire_binding(),
            );
            let pipeline = self
                .alpha_threshold_pipeline
                .as_ref()
                .expect("alpha-threshold pipeline must exist");
            run_fullscreen_triangle_pass(
                encoder,
                "fret alpha-threshold pass",
                pipeline,
                dst_view,
                pass.load,
                &bind_group,
                &[],
                pass.dst_scissor,
                perf_enabled.then_some(frame_perf),
            );
        }
    }

    pub(in super::super) fn record_drop_shadow_pass(
        &mut self,
        ctx: &mut ExecuteCtx<'_>,
        pass: &DropShadowPass,
    ) {
        let device = ctx.device;
        let queue = ctx.queue;
        let format = ctx.format;
        let target_view = ctx.target_view;
        let viewport_size = ctx.viewport_size;
        let usage = ctx.usage;
        let encoder = &mut *ctx.encoder;
        let frame_targets = &mut *ctx.frame_targets;
        let encoding = ctx.encoding;
        let render_space_offset_u32 = ctx.render_space_offset_u32;
        let perf_enabled = ctx.perf_enabled;
        let frame_perf = &mut *ctx.frame_perf;

        queue.write_buffer(
            &self.drop_shadow_param_buffer,
            0,
            bytemuck::cast_slice(&[
                pass.offset_px.0,
                pass.offset_px.1,
                0.0,
                0.0,
                pass.color.r,
                pass.color.g,
                pass.color.b,
                pass.color.a,
            ]),
        );
        if perf_enabled {
            frame_perf.uniform_bytes = frame_perf
                .uniform_bytes
                .saturating_add(std::mem::size_of::<[f32; 8]>() as u64);
        }

        let src_view = match pass.src {
            PlanTarget::Output | PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                debug_assert!(false, "DropShadow src cannot be Output/mask targets");
                return;
            }
            PlanTarget::Intermediate0 | PlanTarget::Intermediate1 | PlanTarget::Intermediate2 => {
                frame_targets.require_target(pass.src, pass.src_size)
            }
        };

        let dst_view_owned = match pass.dst {
            PlanTarget::Output => None,
            PlanTarget::Intermediate0 | PlanTarget::Intermediate1 | PlanTarget::Intermediate2 => {
                Some(frame_targets.ensure_target(
                    &mut self.intermediate_pool,
                    device,
                    pass.dst,
                    pass.dst_size,
                    format,
                    usage,
                ))
            }
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                debug_assert!(false, "DropShadow dst cannot be mask targets");
                None
            }
        };
        let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

        if let Some(mask) = pass.mask {
            debug_assert!(matches!(
                mask.target,
                PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
            ));
            debug_assert_eq!(
                pass.dst_size, viewport_size,
                "mask-based drop-shadow expects full-size destination"
            );

            let mask_uniform_index = pass
                .mask_uniform_index
                .expect("mask drop-shadow needs uniform index");
            let uniform_offset = (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

            let mask_view = frame_targets.require_target(mask.target, mask.size);
            let layout = self
                .drop_shadow_mask_bind_group_layout
                .as_ref()
                .expect("drop-shadow mask bind group layout must exist");
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fret drop-shadow mask bind group"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&src_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.drop_shadow_param_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&mask_view),
                    },
                ],
            });

            let pipeline = self
                .drop_shadow_mask_pipeline
                .as_ref()
                .expect("drop-shadow mask pipeline must exist");

            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fret drop-shadow mask pass"),
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
            if perf_enabled {
                frame_perf.pipeline_switches = frame_perf.pipeline_switches.saturating_add(1);
                frame_perf.pipeline_switches_fullscreen =
                    frame_perf.pipeline_switches_fullscreen.saturating_add(1);
            }
            rp.set_bind_group(
                0,
                self.pick_uniform_bind_group_for_mask_image(
                    encoding
                        .uniform_mask_images
                        .get(mask_uniform_index as usize)
                        .copied()
                        .flatten(),
                ),
                &[uniform_offset, render_space_offset_u32],
            );
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
                frame_perf.uniform_bind_group_switches =
                    frame_perf.uniform_bind_group_switches.saturating_add(1);
            }
            rp.set_bind_group(1, &bind_group, &[]);
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
                frame_perf.texture_bind_group_switches =
                    frame_perf.texture_bind_group_switches.saturating_add(1);
            }
            if let Some(scissor) = pass.dst_scissor
                && scissor.w != 0
                && scissor.h != 0
            {
                rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
                if perf_enabled {
                    frame_perf.scissor_sets = frame_perf.scissor_sets.saturating_add(1);
                }
            }
            rp.draw(0..3, 0..1);
            if perf_enabled {
                frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
                frame_perf.fullscreen_draw_calls =
                    frame_perf.fullscreen_draw_calls.saturating_add(1);
            }
        } else if let Some(mask_uniform_index) = pass.mask_uniform_index {
            let layout = self
                .drop_shadow_bind_group_layout
                .as_ref()
                .expect("drop-shadow bind group layout must exist");
            let bind_group = create_texture_uniform_bind_group(
                device,
                "fret drop-shadow bind group",
                layout,
                &src_view,
                self.drop_shadow_param_buffer.as_entire_binding(),
            );
            let pipeline = self
                .drop_shadow_masked_pipeline
                .as_ref()
                .expect("drop-shadow masked pipeline must exist");
            let uniform_offset = (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fret drop-shadow masked pass"),
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
            if perf_enabled {
                frame_perf.pipeline_switches = frame_perf.pipeline_switches.saturating_add(1);
                frame_perf.pipeline_switches_fullscreen =
                    frame_perf.pipeline_switches_fullscreen.saturating_add(1);
            }
            rp.set_bind_group(
                0,
                self.pick_uniform_bind_group_for_mask_image(
                    encoding
                        .uniform_mask_images
                        .get(mask_uniform_index as usize)
                        .copied()
                        .flatten(),
                ),
                &[uniform_offset, render_space_offset_u32],
            );
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
                frame_perf.uniform_bind_group_switches =
                    frame_perf.uniform_bind_group_switches.saturating_add(1);
            }
            rp.set_bind_group(1, &bind_group, &[]);
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
                frame_perf.texture_bind_group_switches =
                    frame_perf.texture_bind_group_switches.saturating_add(1);
            }
            if let Some(scissor) = pass.dst_scissor
                && scissor.w != 0
                && scissor.h != 0
            {
                rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
                if perf_enabled {
                    frame_perf.scissor_sets = frame_perf.scissor_sets.saturating_add(1);
                }
            }
            rp.draw(0..3, 0..1);
            if perf_enabled {
                frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
                frame_perf.fullscreen_draw_calls =
                    frame_perf.fullscreen_draw_calls.saturating_add(1);
            }
        } else {
            let layout = self
                .drop_shadow_bind_group_layout
                .as_ref()
                .expect("drop-shadow bind group layout must exist");
            let bind_group = create_texture_uniform_bind_group(
                device,
                "fret drop-shadow bind group",
                layout,
                &src_view,
                self.drop_shadow_param_buffer.as_entire_binding(),
            );
            let pipeline = self
                .drop_shadow_pipeline
                .as_ref()
                .expect("drop-shadow pipeline must exist");
            run_fullscreen_triangle_pass(
                encoder,
                "fret drop-shadow pass",
                pipeline,
                dst_view,
                pass.load,
                &bind_group,
                &[],
                pass.dst_scissor,
                perf_enabled.then_some(frame_perf),
            );
        }
    }

    pub(in super::super) fn record_composite_premul_pass(
        &mut self,
        ctx: &mut ExecuteCtx<'_>,
        pass_index: usize,
        quad_vertex_bases: &[Option<u32>],
        quad_vertex_size: u64,
        pass: &CompositePremulPass,
    ) {
        let device = ctx.device;
        let format = ctx.format;
        let target_view = ctx.target_view;
        let viewport_size = ctx.viewport_size;
        let usage = ctx.usage;
        let encoder = &mut *ctx.encoder;
        let frame_targets = &mut *ctx.frame_targets;
        let encoding = ctx.encoding;
        let render_space_offset_u32 = ctx.render_space_offset_u32;
        let perf_enabled = ctx.perf_enabled;
        let frame_perf = &mut *ctx.frame_perf;

        let pipeline_ix = pass.blend_mode.pipeline_index();

        let src_view = match pass.src {
            PlanTarget::Output | PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                debug_assert!(false, "CompositePremul src cannot be Output/mask targets");
                return;
            }
            PlanTarget::Intermediate0 | PlanTarget::Intermediate1 | PlanTarget::Intermediate2 => {
                frame_targets.require_target(pass.src, pass.src_size)
            }
        };

        let dst_view_owned = match pass.dst {
            PlanTarget::Output => None,
            PlanTarget::Intermediate0 | PlanTarget::Intermediate1 | PlanTarget::Intermediate2 => {
                Some(frame_targets.ensure_target(
                    &mut self.intermediate_pool,
                    device,
                    pass.dst,
                    pass.dst_size,
                    format,
                    usage,
                ))
            }
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                debug_assert!(false, "CompositePremul dst cannot be mask targets");
                None
            }
        };
        let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

        let (composite_pipeline, bind_group) = if let Some(mask) = pass.mask {
            debug_assert!(matches!(
                mask.target,
                PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
            ));
            debug_assert_eq!(
                pass.dst_size, viewport_size,
                "mask-based composite expects full-size destination"
            );

            let mask_view = frame_targets.require_target(mask.target, mask.size);
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
                        resource: wgpu::BindingResource::Sampler(&self.viewport_sampler),
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
                self.composite_mask_pipelines[pipeline_ix]
                    .as_ref()
                    .expect("composite mask pipeline must exist"),
                bind_group,
            )
        } else {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fret composite premul bind group"),
                layout: &self.viewport_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&self.viewport_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&src_view),
                    },
                ],
            });
            (
                self.composite_pipelines[pipeline_ix]
                    .as_ref()
                    .expect("composite pipeline must exist"),
                bind_group,
            )
        };
        let Some(base) = quad_vertex_bases.get(pass_index).and_then(|v| *v) else {
            return;
        };

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
        if perf_enabled {
            frame_perf.pipeline_switches = frame_perf.pipeline_switches.saturating_add(1);
            frame_perf.pipeline_switches_composite =
                frame_perf.pipeline_switches_composite.saturating_add(1);
        }
        if let Some(mask_uniform_index) = pass.mask_uniform_index {
            let uniform_offset = (u64::from(mask_uniform_index) * self.uniform_stride) as u32;
            rp.set_bind_group(
                0,
                self.pick_uniform_bind_group_for_mask_image(
                    encoding
                        .uniform_mask_images
                        .get(mask_uniform_index as usize)
                        .copied()
                        .flatten(),
                ),
                &[uniform_offset, render_space_offset_u32],
            );
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
            }
        } else {
            rp.set_bind_group(0, &self.uniform_bind_group, &[0, render_space_offset_u32]);
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
            }
        }
        rp.set_bind_group(1, &bind_group, &[]);
        if perf_enabled {
            frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
            frame_perf.texture_bind_group_switches =
                frame_perf.texture_bind_group_switches.saturating_add(1);
        }
        let base = u64::from(base) * quad_vertex_size;
        let len = 6 * quad_vertex_size;
        rp.set_vertex_buffer(0, self.path_composite_vertices.slice(base..base + len));
        if let Some(scissor) = pass.dst_scissor
            && scissor.w != 0
            && scissor.h != 0
        {
            let _ = set_scissor_rect_absolute(&mut rp, scissor, pass.dst_origin, pass.dst_size);
        }
        rp.draw(0..6, 0..1);
        if perf_enabled {
            frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
        }
    }

    pub(in super::super) fn record_clip_mask_pass(
        &mut self,
        ctx: &mut ExecuteCtx<'_>,
        pass: &ClipMaskPass,
    ) {
        let device = ctx.device;
        let queue = ctx.queue;
        let usage = ctx.usage;
        let encoder = &mut *ctx.encoder;
        let frame_targets = &mut *ctx.frame_targets;
        let encoding = ctx.encoding;
        let render_space_offset_u32 = ctx.render_space_offset_u32;
        let perf_enabled = ctx.perf_enabled;
        let frame_perf = &mut *ctx.frame_perf;

        debug_assert!(matches!(
            pass.dst,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
        ));
        queue.write_buffer(
            &self.clip_mask_param_buffer,
            0,
            bytemuck::cast_slice(&[pass.dst_size.0 as f32, pass.dst_size.1 as f32, 0.0, 0.0]),
        );
        if perf_enabled {
            frame_perf.uniform_bytes = frame_perf
                .uniform_bytes
                .saturating_add(std::mem::size_of::<[f32; 4]>() as u64);
        }
        let dst_view = frame_targets.ensure_target(
            &mut self.intermediate_pool,
            device,
            pass.dst,
            pass.dst_size,
            wgpu::TextureFormat::R8Unorm,
            usage,
        );

        let pipeline = self
            .clip_mask_pipeline
            .as_ref()
            .expect("clip mask pipeline must exist");
        let uniform_offset = (u64::from(pass.uniform_index) * self.uniform_stride) as u32;

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
        if perf_enabled {
            frame_perf.pipeline_switches = frame_perf.pipeline_switches.saturating_add(1);
            frame_perf.pipeline_switches_clip_mask =
                frame_perf.pipeline_switches_clip_mask.saturating_add(1);
        }
        rp.set_bind_group(
            0,
            self.pick_uniform_bind_group_for_mask_image(
                encoding
                    .uniform_mask_images
                    .get(pass.uniform_index as usize)
                    .copied()
                    .flatten(),
            ),
            &[uniform_offset, render_space_offset_u32],
        );
        if perf_enabled {
            frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
            frame_perf.uniform_bind_group_switches =
                frame_perf.uniform_bind_group_switches.saturating_add(1);
        }
        rp.set_bind_group(1, &self.clip_mask_param_bind_group, &[]);
        if perf_enabled {
            frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
            frame_perf.texture_bind_group_switches =
                frame_perf.texture_bind_group_switches.saturating_add(1);
        }
        if let Some(scissor) = pass.dst_scissor
            && scissor.w != 0
            && scissor.h != 0
        {
            rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
            if perf_enabled {
                frame_perf.scissor_sets = frame_perf.scissor_sets.saturating_add(1);
            }
        }
        rp.draw(0..3, 0..1);
        if perf_enabled {
            frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
            frame_perf.clip_mask_draw_calls = frame_perf.clip_mask_draw_calls.saturating_add(1);
        }
    }
}
