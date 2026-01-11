use glam::Mat4;

mod immediate_3d;
pub use immediate_3d::*;

#[derive(Debug, Clone, Copy)]
pub struct ViewportOverlay3dContext {
    pub view_proj: Mat4,
    pub viewport_px: (u32, u32),
}

pub type ViewportOverlay3d<'a> =
    Box<dyn for<'rp> FnMut(&mut wgpu::RenderPass<'rp>, &ViewportOverlay3dContext) + 'a>;

pub fn run_overlays<'rp>(
    pass: &mut wgpu::RenderPass<'rp>,
    ctx: &ViewportOverlay3dContext,
    overlays: &mut [ViewportOverlay3d<'_>],
) {
    for overlay in overlays {
        overlay(pass, ctx);
    }
}
