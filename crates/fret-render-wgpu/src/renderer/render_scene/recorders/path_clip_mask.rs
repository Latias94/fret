use super::super::super::*;
use super::super::executor::{RecordPassCtx, RecordPassResources, RenderSceneExecutor};
use super::super::helpers::set_scissor_rect_absolute;

pub(in super::super) fn record_path_clip_mask_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    resources: &RecordPassResources<'_>,
    mask_pass: &PathClipMaskPass,
) {
    let encoding = exec.encoding;
    let perf_enabled = exec.perf_enabled;

    let target_size = mask_pass.dst_size;
    let Some((pass_target_view, cache_hit)) = exec.ensure_clip_path_mask_target_and_try_cache_copy(
        mask_pass.cache_key,
        mask_pass.dst,
        target_size,
    ) else {
        return;
    };
    if cache_hit {
        return;
    }

    let renderer = &*exec.renderer;
    let uniform_offset = (mask_pass.uniform_index as u64).saturating_mul(renderer.uniform_stride());

    let vertex_size = std::mem::size_of::<PathVertex>() as u64;
    let first = (mask_pass.first_vertex as u64).saturating_mul(vertex_size);
    let size = (mask_pass.vertex_count as u64).saturating_mul(vertex_size);

    {
        let mut rp = exec.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fret path clip-mask pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &pass_target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: mask_pass.load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let pipeline = renderer.path_clip_mask_pipeline_ref();
        rp.set_pipeline(pipeline);
        let mask_image = encoding
            .uniform_mask_images
            .get(mask_pass.uniform_index as usize)
            .copied()
            .flatten();
        let uniform_bind_group = renderer.pick_uniform_bind_group_for_mask_image(mask_image);
        rp.set_bind_group(
            0,
            uniform_bind_group,
            &[uniform_offset as u32, ctx.render_space_offset_u32],
        );

        if size != 0 {
            rp.set_vertex_buffer(0, resources.path_vertex_buffer.slice(first..first + size));
            let _ = set_scissor_rect_absolute(
                &mut rp,
                mask_pass.scissor.0,
                mask_pass.dst_origin,
                mask_pass.dst_size,
            );
            rp.draw(0..mask_pass.vertex_count, 0..1);
        }
    }

    if perf_enabled {
        exec.frame_perf.clip_mask_draw_calls =
            exec.frame_perf.clip_mask_draw_calls.saturating_add(1);
        exec.frame_perf.clip_path_mask_cache_misses = exec
            .frame_perf
            .clip_path_mask_cache_misses
            .saturating_add(1);
    }

    exec.store_clip_path_mask_cache_from_target(mask_pass.cache_key, mask_pass.dst, target_size);
}
