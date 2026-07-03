use fret_core::Scene;
use fret_render::{
    ClearColor, RenderSceneParams, RenderSceneSourcePolicy, Renderer, SurfaceState, WgpuContext,
    select_render_scene_source,
};

use super::redraw_hitch::{RedrawPhase, measure_redraw_phase};

pub(super) struct WindowRedrawRenderSceneInput<'a, 'window> {
    pub(super) renderer: &'a mut Renderer,
    pub(super) context: &'a WgpuContext,
    pub(super) surface: &'a SurfaceState<'window>,
    pub(super) target_view: &'a wgpu::TextureView,
    pub(super) scene: &'a Scene,
    pub(super) scene_chunks: &'a fret_core::SceneChunkManifest,
    pub(super) clear_color: ClearColor,
    pub(super) scale_factor: f32,
}

pub(super) fn record_window_redraw_render_scene(
    input: WindowRedrawRenderSceneInput<'_, '_>,
) -> wgpu::CommandBuffer {
    let (ui_cmd, _) = measure_redraw_phase(RedrawPhase::RenderScene, false, || {
        input.renderer.render_scene(
            &input.context.device,
            &input.context.queue,
            RenderSceneParams {
                format: input.surface.format(),
                target_view: input.target_view,
                source: select_render_scene_source(
                    input.scene,
                    input.scene_chunks,
                    RenderSceneSourcePolicy::flat_compat(),
                ),
                clear: input.clear_color,
                scale_factor: input.scale_factor,
                viewport_size: input.surface.size(),
            },
        )
    });
    ui_cmd
}
