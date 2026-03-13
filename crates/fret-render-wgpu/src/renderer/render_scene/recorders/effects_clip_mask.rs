use super::super::super::*;
use super::super::executor::{RecordPassCtx, RenderSceneExecutor};

pub(in super::super) fn record_clip_mask_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &ClipMaskPass,
) {
    let queue = exec.queue;
    let perf_enabled = exec.perf_enabled;

    queue.write_buffer(
        &exec.renderer.effect_params.clip_mask_param_buffer,
        0,
        bytemuck::cast_slice(&[pass.dst_size.0 as f32, pass.dst_size.1 as f32, 0.0, 0.0]),
    );
    if perf_enabled {
        exec.frame_perf.uniform_bytes = exec
            .frame_perf
            .uniform_bytes
            .saturating_add(std::mem::size_of::<[f32; 4]>() as u64);
    }
    let Some(dst_view) = exec.ensure_mask_dst_view(pass.dst, pass.dst_size, "ClipMask") else {
        return;
    };

    let encoder = &mut *exec.encoder;
    let encoding = exec.encoding;
    let frame_perf = &mut *exec.frame_perf;
    let renderer = &mut *exec.renderer;
    let pipeline = renderer.clip_mask_pipeline_ref();
    let uniform_offset = (u64::from(pass.uniform_index) * renderer.uniform_stride()) as u32;

    run_clip_mask_triangle_pass(
        encoder,
        "fret clip mask pass",
        pipeline,
        &dst_view,
        pass.load,
        renderer.pick_uniform_bind_group_for_mask_image(
            encoding
                .uniform_mask_images
                .get(pass.uniform_index as usize)
                .copied()
                .flatten(),
        ),
        &[uniform_offset, ctx.render_space_offset_u32],
        &renderer.effect_params.clip_mask_param_bind_group,
        &[],
        pass.dst_scissor,
        pass.dst_size,
        perf_enabled.then_some(frame_perf),
    );
}
