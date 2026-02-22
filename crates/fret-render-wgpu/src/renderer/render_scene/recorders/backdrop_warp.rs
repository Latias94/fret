use super::super::super::frame_targets::FrameTargets;
use super::super::super::*;

impl Renderer {
    pub(in super::super) fn record_backdrop_warp_pass(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
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
        pass: &BackdropWarpPass,
    ) {
        #[repr(C)]
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct BackdropWarpParams {
            origin_px: [f32; 2],
            bounds_size_px: [f32; 2],
            strength_px: f32,
            scale_px: f32,
            phase: f32,
            chroma_px: f32,
            kind: u32,
            warp_encoding: u32,
            warp_sampling: u32,
            _pad0: u32,
            uv0: [f32; 2],
            uv1: [f32; 2],
        }

        let kind = match pass.kind {
            fret_core::scene::BackdropWarpKindV1::Wave => 0u32,
            fret_core::scene::BackdropWarpKindV1::LensReserved => 1u32,
        };
        let warp_encoding = match pass.warp_encoding {
            fret_core::scene::WarpMapEncodingV1::RgSigned => 1u32,
            fret_core::scene::WarpMapEncodingV1::NormalRgb => 2u32,
        };
        let warp_sampling = match pass.warp_sampling {
            fret_core::scene::ImageSamplingHint::Nearest => 2u32,
            fret_core::scene::ImageSamplingHint::Default
            | fret_core::scene::ImageSamplingHint::Linear => 1u32,
        };
        queue.write_buffer(
            &self.backdrop_warp_param_buffer,
            0,
            bytemuck::bytes_of(&BackdropWarpParams {
                origin_px: [pass.origin_px.0 as f32, pass.origin_px.1 as f32],
                bounds_size_px: [pass.bounds_size_px.0 as f32, pass.bounds_size_px.1 as f32],
                strength_px: pass.strength_px,
                scale_px: pass.scale_px,
                phase: pass.phase,
                chroma_px: pass.chromatic_aberration_px,
                kind,
                warp_encoding,
                warp_sampling,
                _pad0: 0,
                uv0: [pass.warp_uv.u0, pass.warp_uv.v0],
                uv1: [pass.warp_uv.u1, pass.warp_uv.v1],
            }),
        );
        if perf_enabled {
            frame_perf.uniform_bytes = frame_perf
                .uniform_bytes
                .saturating_add(std::mem::size_of::<BackdropWarpParams>() as u64);
        }

        let warp_view = pass.warp_image.and_then(|image| self.images.get(image));

        let src_view = match pass.src {
            PlanTarget::Output | PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                debug_assert!(false, "BackdropWarp src cannot be Output/mask targets");
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
                debug_assert!(false, "BackdropWarp dst cannot be mask targets");
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
                "mask-based backdrop-warp expects full-size destination"
            );

            let mask_uniform_index = pass
                .mask_uniform_index
                .expect("mask backdrop-warp needs uniform index");
            let uniform_offset = (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

            let mask_view = frame_targets.require_target(mask.target, mask.size);

            let (label, bind_group, pipeline) = if let Some(warp_view) = warp_view {
                let layout = self
                    .backdrop_warp_image_mask_bind_group_layout
                    .as_ref()
                    .expect("backdrop-warp image mask bind group layout must exist");
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("fret backdrop-warp image mask bind group"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&src_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: self.backdrop_warp_param_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(warp_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(&mask_view),
                        },
                    ],
                });
                let pipeline = self
                    .backdrop_warp_image_mask_pipeline
                    .as_ref()
                    .expect("backdrop-warp image mask pipeline must exist");
                ("fret backdrop-warp image mask pass", bind_group, pipeline)
            } else {
                let layout = self
                    .backdrop_warp_mask_bind_group_layout
                    .as_ref()
                    .expect("backdrop-warp mask bind group layout must exist");
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("fret backdrop-warp mask bind group"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&src_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: self.backdrop_warp_param_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&mask_view),
                        },
                    ],
                });
                let pipeline = self
                    .backdrop_warp_mask_pipeline
                    .as_ref()
                    .expect("backdrop-warp mask pipeline must exist");
                ("fret backdrop-warp mask pass", bind_group, pipeline)
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
            let (bind_group, pipeline) = if let Some(warp_view) = warp_view {
                let layout = self
                    .backdrop_warp_image_bind_group_layout
                    .as_ref()
                    .expect("backdrop-warp image bind group layout must exist");
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("fret backdrop-warp image masked bind group"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&src_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: self.backdrop_warp_param_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(warp_view),
                        },
                    ],
                });
                let pipeline = self
                    .backdrop_warp_image_masked_pipeline
                    .as_ref()
                    .expect("backdrop-warp image masked pipeline must exist");
                (bind_group, pipeline)
            } else {
                let layout = self
                    .backdrop_warp_bind_group_layout
                    .as_ref()
                    .expect("backdrop-warp bind group layout must exist");
                let bind_group = create_texture_uniform_bind_group(
                    device,
                    "fret backdrop-warp bind group",
                    layout,
                    &src_view,
                    self.backdrop_warp_param_buffer.as_entire_binding(),
                );
                let pipeline = self
                    .backdrop_warp_masked_pipeline
                    .as_ref()
                    .expect("backdrop-warp masked pipeline must exist");
                (bind_group, pipeline)
            };
            let uniform_offset = (u64::from(mask_uniform_index) * self.uniform_stride) as u32;

            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fret backdrop-warp masked pass"),
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
            let (bind_group, pipeline) = if let Some(warp_view) = warp_view {
                let layout = self
                    .backdrop_warp_image_bind_group_layout
                    .as_ref()
                    .expect("backdrop-warp image bind group layout must exist");
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("fret backdrop-warp image bind group"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&src_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: self.backdrop_warp_param_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(warp_view),
                        },
                    ],
                });
                let pipeline = self
                    .backdrop_warp_image_pipeline
                    .as_ref()
                    .expect("backdrop-warp image pipeline must exist");
                (bind_group, pipeline)
            } else {
                let layout = self
                    .backdrop_warp_bind_group_layout
                    .as_ref()
                    .expect("backdrop-warp bind group layout must exist");
                let bind_group = create_texture_uniform_bind_group(
                    device,
                    "fret backdrop-warp bind group",
                    layout,
                    &src_view,
                    self.backdrop_warp_param_buffer.as_entire_binding(),
                );
                let pipeline = self
                    .backdrop_warp_pipeline
                    .as_ref()
                    .expect("backdrop-warp pipeline must exist");
                (bind_group, pipeline)
            };

            run_fullscreen_triangle_pass(
                encoder,
                "fret backdrop-warp pass",
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
}
