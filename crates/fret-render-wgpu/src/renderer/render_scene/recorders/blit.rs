use super::super::super::*;
use super::super::executor::RenderSceneExecutor;

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

    let blit_pipeline = renderer
        .blit_pipeline
        .as_ref()
        .expect("blit pipeline must exist");

    let layout = renderer
        .blit_bind_group_layout
        .as_ref()
        .expect("blit bind group layout must exist");
    let src_view = match pass.src {
        PlanTarget::Output | PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
            debug_assert!(false, "FullscreenBlit src cannot be Output/mask targets");
            return;
        }
        PlanTarget::Intermediate0 | PlanTarget::Intermediate1 | PlanTarget::Intermediate2 => {
            frame_targets.require_target(pass.src, pass.src_size)
        }
    };
    let blit_bind_group =
        create_texture_bind_group(device, "fret blit bind group", layout, &src_view);

    let dst_view_owned = match pass.dst {
        PlanTarget::Output => None,
        PlanTarget::Intermediate0 | PlanTarget::Intermediate1 | PlanTarget::Intermediate2 => {
            Some(frame_targets.ensure_target(
                &mut renderer.intermediate_pool,
                device,
                pass.dst,
                pass.dst_size,
                format,
                usage,
            ))
        }
        PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
            debug_assert!(false, "FullscreenBlit dst cannot be mask targets");
            None
        }
    };
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

    run_fullscreen_triangle_pass(
        encoder,
        "fret blit pass",
        blit_pipeline,
        dst_view,
        pass.load,
        &blit_bind_group,
        &[],
        pass.dst_scissor,
        perf_enabled.then_some(frame_perf),
    );
}
