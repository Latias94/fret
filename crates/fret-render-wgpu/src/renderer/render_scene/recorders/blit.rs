use super::super::super::*;
use super::super::executor::RenderSceneExecutor;

pub(in super::super) fn record_fullscreen_blit_pass(
    exec: &mut RenderSceneExecutor<'_>,
    pass: &FullscreenBlitPass,
) {
    let device = exec.device;
    let target_view = exec.target_view;
    let perf_enabled = exec.perf_enabled;

    let Some(src_view) = exec.require_color_src_view(pass.src, pass.src_size, "FullscreenBlit")
    else {
        return;
    };

    let dst_view_owned =
        exec.ensure_color_dst_view_owned(pass.dst, pass.dst_size, "FullscreenBlit");
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

    let renderer = &*exec.renderer;
    let layout = renderer.blit_bind_group_layout_ref();
    let blit_bind_group =
        create_texture_bind_group(device, "fret blit bind group", layout, &src_view);
    let blit_pipeline = if pass.encode_output_srgb {
        renderer.blit_srgb_encode_pipeline_ref()
    } else {
        renderer.blit_pipeline_ref()
    };

    run_fullscreen_triangle_pass(
        &mut *exec.encoder,
        "fret blit pass",
        blit_pipeline,
        dst_view,
        pass.dst_size,
        pass.load,
        &blit_bind_group,
        &[],
        pass.dst_scissor,
        if perf_enabled {
            Some(&mut *exec.frame_perf)
        } else {
            None
        },
    );
}
