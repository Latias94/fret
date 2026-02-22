use super::super::super::*;
use super::super::executor::RenderSceneExecutor;
use super::super::helpers::{ensure_color_dst_view_owned, require_color_src_view};

pub(in super::super) fn record_fullscreen_blit_pass(
    exec: &mut RenderSceneExecutor<'_>,
    pass: &FullscreenBlitPass,
) {
    let device = exec.device;
    let format = exec.format;
    let target_view = exec.target_view;
    let usage = exec.usage;
    let encoder = &mut *exec.encoder;
    let frame_targets = &mut *exec.frame_targets;
    let perf_enabled = exec.perf_enabled;
    let frame_perf = &mut *exec.frame_perf;

    let renderer = &mut *exec.renderer;

    let Some(src_view) =
        require_color_src_view(frame_targets, pass.src, pass.src_size, "FullscreenBlit")
    else {
        return;
    };

    let dst_view_owned = ensure_color_dst_view_owned(
        frame_targets,
        &mut renderer.intermediate_pool,
        device,
        pass.dst,
        pass.dst_size,
        format,
        usage,
        "FullscreenBlit",
    );
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

    let layout = renderer.blit_bind_group_layout_ref();
    let blit_bind_group =
        create_texture_bind_group(device, "fret blit bind group", layout, &src_view);
    let blit_pipeline = renderer.blit_pipeline_ref();

    run_fullscreen_triangle_pass(
        encoder,
        "fret blit pass",
        blit_pipeline,
        dst_view,
        pass.dst_size,
        pass.load,
        &blit_bind_group,
        &[],
        pass.dst_scissor,
        perf_enabled.then_some(frame_perf),
    );
}
