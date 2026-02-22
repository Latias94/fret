use super::super::super::*;
use super::super::ctx::ExecuteCtx;
use super::super::helpers::set_scissor_rect_absolute;

impl Renderer {
    pub(in super::super) fn record_path_msaa_batch_pass(
        &mut self,
        ctx: &mut ExecuteCtx<'_>,
        plan: &RenderPlan,
        pass_index: usize,
        path_vertex_buffer: &wgpu::Buffer,
        path_paint_bind_group: &wgpu::BindGroup,
        path_pass: &PathMsaaBatchPass,
    ) {
        let device = ctx.device;
        let format = ctx.format;
        let target_view = ctx.target_view;
        let usage = ctx.usage;
        let frame_targets = &mut *ctx.frame_targets;
        let encoder = &mut *ctx.encoder;
        let encoding = ctx.encoding;
        let render_space_offset_u32 = ctx.render_space_offset_u32;
        let perf_enabled = ctx.perf_enabled;
        let frame_perf = &mut *ctx.frame_perf;
        let quad_vertex_bases = ctx.quad_vertex_bases;
        let quad_vertex_size = ctx.quad_vertex_size;

        debug_assert!(path_pass.segment.0 < plan.segments.len());
        let target_origin = path_pass.target_origin;
        let target_size = path_pass.target_size;
        let pass_target_view_owned = match path_pass.target {
            PlanTarget::Output => None,
            PlanTarget::Intermediate0 => Some(frame_targets.ensure_target(
                &mut self.intermediate_pool,
                device,
                PlanTarget::Intermediate0,
                target_size,
                format,
                usage,
            )),
            PlanTarget::Intermediate1 => Some(frame_targets.ensure_target(
                &mut self.intermediate_pool,
                device,
                PlanTarget::Intermediate1,
                target_size,
                format,
                usage,
            )),
            PlanTarget::Intermediate2 => Some(frame_targets.ensure_target(
                &mut self.intermediate_pool,
                device,
                PlanTarget::Intermediate2,
                target_size,
                format,
                usage,
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
            return;
        }

        let Some(intermediate) = &self.path_intermediate else {
            return;
        };
        let Some(path_msaa_pipeline) = self.path_msaa_pipeline.as_ref() else {
            return;
        };
        let Some(composite_pipeline) = self.composite_pipelines[0].as_ref() else {
            return;
        };

        {
            let mut path_pass_rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            if perf_enabled {
                frame_perf.pipeline_switches = frame_perf.pipeline_switches.saturating_add(1);
                frame_perf.pipeline_switches_path_msaa =
                    frame_perf.pipeline_switches_path_msaa.saturating_add(1);
            }
            path_pass_rp.set_bind_group(1, path_paint_bind_group, &[]);
            if perf_enabled {
                frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
            }
            path_pass_rp.set_vertex_buffer(0, path_vertex_buffer.slice(..));

            let mut active_uniform_offset: Option<u32> = None;
            let mut active_mask_image: Option<UniformMaskImageSelection> = None;
            let mut active_scissor: Option<ScissorRect> = None;
            for j in start..end {
                let OrderedDraw::Path(draw) = &encoding.ordered_draws[j] else {
                    unreachable!("PathMsaaBatch pass must reference only Path draws");
                };
                if draw.scissor.w == 0 || draw.scissor.h == 0 {
                    continue;
                }
                if active_scissor != Some(draw.scissor) {
                    if set_scissor_rect_absolute(
                        &mut path_pass_rp,
                        draw.scissor,
                        target_origin,
                        target_size,
                    ) && perf_enabled
                    {
                        frame_perf.scissor_sets = frame_perf.scissor_sets.saturating_add(1);
                    }
                    active_scissor = Some(draw.scissor);
                }
                let uniform_offset = (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                let mask_image = encoding
                    .uniform_mask_images
                    .get(draw.uniform_index as usize)
                    .copied()
                    .flatten();
                let uniform_bind_group = self.pick_uniform_bind_group_for_mask_image(mask_image);

                if active_uniform_offset != Some(uniform_offset) || active_mask_image != mask_image
                {
                    path_pass_rp.set_bind_group(
                        0,
                        uniform_bind_group,
                        &[uniform_offset, render_space_offset_u32],
                    );
                    if perf_enabled {
                        frame_perf.bind_group_switches =
                            frame_perf.bind_group_switches.saturating_add(1);
                        frame_perf.uniform_bind_group_switches =
                            frame_perf.uniform_bind_group_switches.saturating_add(1);
                    }
                    active_uniform_offset = Some(uniform_offset);
                    active_mask_image = mask_image;
                }
                path_pass_rp.draw(
                    draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                    draw.paint_index..(draw.paint_index + 1),
                );
                if perf_enabled {
                    frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
                    frame_perf.path_draw_calls = frame_perf.path_draw_calls.saturating_add(1);
                }
            }
        }

        let union = path_pass.union_scissor;
        if union.w == 0 || union.h == 0 {
            return;
        }
        let Some(base) = quad_vertex_bases.get(pass_index).and_then(|v| *v) else {
            return;
        };

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
        if perf_enabled {
            frame_perf.pipeline_switches = frame_perf.pipeline_switches.saturating_add(1);
            frame_perf.pipeline_switches_composite =
                frame_perf.pipeline_switches_composite.saturating_add(1);
        }
        let uniform_offset =
            (u64::from(path_pass.batch_uniform_index) * self.uniform_stride) as u32;
        let mask_image = encoding
            .uniform_mask_images
            .get(path_pass.batch_uniform_index as usize)
            .copied()
            .flatten();
        let uniform_bind_group = self.pick_uniform_bind_group_for_mask_image(mask_image);
        pass.set_bind_group(
            0,
            uniform_bind_group,
            &[uniform_offset, render_space_offset_u32],
        );
        if perf_enabled {
            frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
            frame_perf.uniform_bind_group_switches =
                frame_perf.uniform_bind_group_switches.saturating_add(1);
        }
        pass.set_bind_group(1, &intermediate.bind_group, &[]);
        if perf_enabled {
            frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
            frame_perf.texture_bind_group_switches =
                frame_perf.texture_bind_group_switches.saturating_add(1);
        }
        let base = u64::from(base) * quad_vertex_size;
        let len = 6 * quad_vertex_size;
        pass.set_vertex_buffer(0, self.path_composite_vertices.slice(base..base + len));
        if set_scissor_rect_absolute(&mut pass, union, target_origin, target_size) && perf_enabled {
            frame_perf.scissor_sets = frame_perf.scissor_sets.saturating_add(1);
        }
        pass.draw(0..6, 0..1);
        if perf_enabled {
            frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
            frame_perf.fullscreen_draw_calls = frame_perf.fullscreen_draw_calls.saturating_add(1);
        }
    }
}
