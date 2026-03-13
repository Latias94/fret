use super::super::super::*;
use super::super::executor::{RecordPassCtx, RenderSceneExecutor};
use super::super::helpers::run_composite_premul_quad_pass;
use super::effects_bindings::create_composite_premul_pipeline_and_bind_group;

pub(in super::super) fn record_composite_premul_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &CompositePremulPass,
) {
    let Some(src_view) = exec.require_color_src_view(pass.src, pass.src_size, "CompositePremul")
    else {
        return;
    };
    let dst_view_owned =
        exec.ensure_color_dst_view_owned(pass.dst, pass.dst_size, "CompositePremul");
    let mask_view = if let Some(mask) = pass.mask {
        debug_assert!(matches!(
            mask.target,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
        ));
        debug_assert_eq!(
            pass.dst_size, exec.viewport_size,
            "mask-based composite expects full-size destination"
        );
        let Some(mask_view) = exec.require_mask_view(mask.target, mask.size, "CompositePremul")
        else {
            return;
        };
        Some(mask_view)
    } else {
        None
    };

    let device = exec.device;
    let target_view = exec.target_view;
    let encoder = &mut *exec.encoder;
    let encoding = exec.encoding;
    let perf_enabled = exec.perf_enabled;
    let frame_perf = &mut *exec.frame_perf;
    let quad_vertex_bases = exec.quad_vertex_bases;
    let quad_vertex_size = exec.quad_vertex_size;

    let renderer = &*exec.renderer;
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

    let (composite_pipeline, bind_group) = create_composite_premul_pipeline_and_bind_group(
        device,
        renderer,
        &src_view,
        mask_view.as_ref(),
        pass.blend_mode,
    );
    let Some(base) = quad_vertex_bases.get(ctx.pass_index).and_then(|v| *v) else {
        return;
    };

    let (uniform_bind_group, uniform_offsets) =
        if let Some(mask_uniform_index) = pass.mask_uniform_index {
            let uniform_offset = (u64::from(mask_uniform_index) * renderer.uniform_stride()) as u32;
            let mask_image = encoding
                .uniform_mask_images
                .get(mask_uniform_index as usize)
                .copied()
                .flatten();
            (
                renderer.pick_uniform_bind_group_for_mask_image(mask_image),
                [uniform_offset, ctx.render_space_offset_u32],
            )
        } else {
            (
                renderer.base_uniform_bind_group(),
                [0, ctx.render_space_offset_u32],
            )
        };

    let base = u64::from(base) * quad_vertex_size;
    let len = 6 * quad_vertex_size;
    run_composite_premul_quad_pass(
        encoder,
        "fret composite premul pass",
        composite_pipeline,
        dst_view,
        pass.load,
        uniform_bind_group,
        &uniform_offsets,
        &bind_group,
        &[],
        renderer.path_composite_vertices_ref(),
        base,
        len,
        pass.dst_scissor,
        pass.dst_origin,
        pass.dst_size,
        perf_enabled.then_some(frame_perf),
    );
}
