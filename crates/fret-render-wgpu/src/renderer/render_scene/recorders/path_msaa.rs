use super::super::super::*;
use super::super::executor::{RecordPassCtx, RecordPassResources, RenderSceneExecutor};
use super::super::helpers::{run_path_msaa_composite_quad_pass, set_scissor_rect_absolute};

pub(in super::super) fn record_path_msaa_batch_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    resources: &RecordPassResources<'_>,
    path_pass: &PathMsaaBatchPass,
) {
    let target_view = exec.target_view;
    let encoding = exec.encoding;
    let perf_enabled = exec.perf_enabled;
    let quad_vertex_bases = exec.quad_vertex_bases;
    let quad_vertex_size = exec.quad_vertex_size;

    debug_assert!(path_pass.segment.0 < ctx.plan.segments.len());
    let target_origin = path_pass.target_origin;
    let target_size = path_pass.target_size;
    let pass_target_view_owned =
        exec.ensure_color_dst_view_owned(path_pass.target, target_size, "PathMsaaBatch");
    let pass_target_view = pass_target_view_owned.as_ref().unwrap_or(target_view);

    let start = path_pass.draw_range.start;
    let end = path_pass.draw_range.end;
    if start >= end {
        return;
    }

    let renderer = &*exec.renderer;
    let Some(intermediate) = renderer.path_intermediate_ref() else {
        return;
    };
    let Some(path_msaa_pipeline) = renderer.path_msaa_pipeline_ref() else {
        return;
    };
    let Some(composite_pipeline) = renderer.pipelines.composite_pipelines
        [fret_core::BlendMode::Over.pipeline_index()]
    .as_ref() else {
        return;
    };

    {
        let mut path_pass_rp = exec.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                    // NOTE: Keep MSAA store as `Store` for Vulkan robustness. Some drivers have
                    // been observed to misbehave when using `Discard` with a resolve target.
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        path_pass_rp.set_pipeline(path_msaa_pipeline);
        if perf_enabled {
            exec.frame_perf.pipeline_switches = exec.frame_perf.pipeline_switches.saturating_add(1);
            exec.frame_perf.pipeline_switches_path_msaa = exec
                .frame_perf
                .pipeline_switches_path_msaa
                .saturating_add(1);
        }
        path_pass_rp.set_bind_group(1, resources.path_paint_bind_group, &[]);
        if perf_enabled {
            exec.frame_perf.bind_group_switches =
                exec.frame_perf.bind_group_switches.saturating_add(1);
        }
        path_pass_rp.set_vertex_buffer(0, resources.path_vertex_buffer.slice(..));

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
                    exec.frame_perf.scissor_sets = exec.frame_perf.scissor_sets.saturating_add(1);
                }
                active_scissor = Some(draw.scissor);
            }
            let uniform_offset = (u64::from(draw.uniform_index) * renderer.uniform_stride()) as u32;
            let mask_image = encoding
                .uniform_mask_images
                .get(draw.uniform_index as usize)
                .copied()
                .flatten();
            let uniform_bind_group = renderer.pick_uniform_bind_group_for_mask_image(mask_image);

            if active_uniform_offset != Some(uniform_offset) || active_mask_image != mask_image {
                path_pass_rp.set_bind_group(
                    0,
                    uniform_bind_group,
                    &[uniform_offset, ctx.render_space_offset_u32],
                );
                if perf_enabled {
                    exec.frame_perf.bind_group_switches =
                        exec.frame_perf.bind_group_switches.saturating_add(1);
                    exec.frame_perf.uniform_bind_group_switches = exec
                        .frame_perf
                        .uniform_bind_group_switches
                        .saturating_add(1);
                }
                active_uniform_offset = Some(uniform_offset);
                active_mask_image = mask_image;
            }
            path_pass_rp.draw(
                draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                draw.paint_index..(draw.paint_index + 1),
            );
            if perf_enabled {
                exec.frame_perf.draw_calls = exec.frame_perf.draw_calls.saturating_add(1);
                exec.frame_perf.path_draw_calls = exec.frame_perf.path_draw_calls.saturating_add(1);
            }
        }
    }

    let union = path_pass.union_scissor;
    if union.0.w == 0 || union.0.h == 0 {
        return;
    }
    let Some(base) = quad_vertex_bases.get(ctx.pass_index).and_then(|v| *v) else {
        return;
    };
    let uniform_offset =
        (u64::from(path_pass.batch_uniform_index) * renderer.uniform_stride()) as u32;
    let mask_image = encoding
        .uniform_mask_images
        .get(path_pass.batch_uniform_index as usize)
        .copied()
        .flatten();
    let uniform_bind_group = renderer.pick_uniform_bind_group_for_mask_image(mask_image);
    let base = u64::from(base) * quad_vertex_size;
    let len = 6 * quad_vertex_size;
    run_path_msaa_composite_quad_pass(
        &mut *exec.encoder,
        "fret renderer pass",
        composite_pipeline,
        pass_target_view,
        path_pass.load,
        uniform_bind_group,
        &[uniform_offset, ctx.render_space_offset_u32],
        &intermediate.bind_group,
        &[],
        renderer.path_composite_vertices_ref(),
        base,
        len,
        union,
        target_origin,
        target_size,
        if perf_enabled {
            Some(&mut *exec.frame_perf)
        } else {
            None
        },
    );
}
