use super::super::super::frame_targets::FrameTargets;
use super::super::super::*;

impl Renderer {
    pub(in super::super) fn record_blur_pass(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        target_view: &wgpu::TextureView,
        viewport_size: (u32, u32),
        usage: wgpu::TextureUsages,
        encoder: &mut wgpu::CommandEncoder,
        frame_targets: &mut FrameTargets,
        encoding: &SceneEncoding,
        render_space_offset_u32: u32,
        perf_enabled: bool,
        frame_perf: &mut RenderPerfStats,
        pass: &BlurPass,
    ) {
        let src_view = match pass.src {
            PlanTarget::Output | PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                debug_assert!(false, "Blur src cannot be Output/mask targets");
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
                debug_assert!(false, "Blur dst cannot be mask targets");
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
                "mask-based blur expects full-size destination"
            );

            let mask_uniform_index = pass
                .mask_uniform_index
                .expect("mask blur needs uniform index");
            let uniform_offset = (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

            let mask_view = frame_targets.require_target(mask.target, mask.size);
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
                .blit_bind_group_layout
                .as_ref()
                .expect("blit bind group layout must exist");
            let bind_group =
                create_texture_bind_group(device, "fret blur bind group", layout, &src_view);
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
            let uniform_offset = (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

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
                .blit_bind_group_layout
                .as_ref()
                .expect("blit bind group layout must exist");
            let bind_group =
                create_texture_bind_group(device, "fret blur bind group", layout, &src_view);
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
                encoder,
                label,
                blur_pipeline,
                dst_view,
                pass.load,
                &bind_group,
                &[],
                pass.dst_scissor,
                if perf_enabled { Some(frame_perf) } else { None },
            );
        }
    }
}
