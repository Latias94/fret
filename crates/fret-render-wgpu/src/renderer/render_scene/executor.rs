use super::super::frame_targets::FrameTargets;
use super::super::*;
use super::ctx::ExecuteCtx;
use super::recorders::{
    record_alpha_threshold_pass, record_backdrop_warp_pass, record_blur_pass,
    record_color_adjust_pass, record_color_matrix_pass, record_fullscreen_blit_pass,
    record_scale_nearest_pass,
};

pub(super) struct RenderSceneExecutor<'a> {
    pub(super) renderer: &'a mut Renderer,
    pub(super) device: &'a wgpu::Device,
    pub(super) queue: &'a wgpu::Queue,
    pub(super) frame_index: u64,
    pub(super) format: wgpu::TextureFormat,
    pub(super) target_view: &'a wgpu::TextureView,
    pub(super) viewport_size: (u32, u32),
    pub(super) usage: wgpu::TextureUsages,
    pub(super) encoder: &'a mut wgpu::CommandEncoder,
    pub(super) frame_targets: &'a mut FrameTargets,
    pub(super) encoding: &'a SceneEncoding,
    pub(super) scale_param_size: u64,
    pub(super) scale_param_cursor: &'a mut u32,
    pub(super) quad_vertex_size: u64,
    pub(super) quad_vertex_bases: &'a [Option<u32>],
    pub(super) perf_enabled: bool,
    pub(super) frame_perf: &'a mut RenderPerfStats,
}

impl<'a> RenderSceneExecutor<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        renderer: &'a mut Renderer,
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        frame_index: u64,
        format: wgpu::TextureFormat,
        target_view: &'a wgpu::TextureView,
        viewport_size: (u32, u32),
        usage: wgpu::TextureUsages,
        encoder: &'a mut wgpu::CommandEncoder,
        frame_targets: &'a mut FrameTargets,
        encoding: &'a SceneEncoding,
        scale_param_size: u64,
        scale_param_cursor: &'a mut u32,
        quad_vertex_size: u64,
        quad_vertex_bases: &'a [Option<u32>],
        perf_enabled: bool,
        frame_perf: &'a mut RenderPerfStats,
    ) -> Self {
        Self {
            renderer,
            device,
            queue,
            frame_index,
            format,
            target_view,
            viewport_size,
            usage,
            encoder,
            frame_targets,
            encoding,
            scale_param_size,
            scale_param_cursor,
            quad_vertex_size,
            quad_vertex_bases,
            perf_enabled,
            frame_perf,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn record_pass(
        &mut self,
        plan: &RenderPlan,
        pass_index: usize,
        planned_pass: &RenderPlanPass,
        render_space_offset_u32: u32,
        viewport_vertex_buffer: &wgpu::Buffer,
        text_vertex_buffer: &wgpu::Buffer,
        path_vertex_buffer: &wgpu::Buffer,
        quad_instance_bind_group: &wgpu::BindGroup,
        text_paint_bind_group: &wgpu::BindGroup,
        path_paint_bind_group: &wgpu::BindGroup,
    ) {
        match planned_pass {
            RenderPlanPass::PathClipMask(mask_pass) => {
                let mut ctx = ExecuteCtx {
                    device: self.device,
                    queue: self.queue,
                    frame_index: self.frame_index,
                    format: self.format,
                    target_view: self.target_view,
                    viewport_size: self.viewport_size,
                    usage: self.usage,
                    encoder: self.encoder,
                    frame_targets: self.frame_targets,
                    encoding: self.encoding,
                    render_space_offset_u32,
                    quad_vertex_size: self.quad_vertex_size,
                    quad_vertex_bases: self.quad_vertex_bases,
                    perf_enabled: self.perf_enabled,
                    frame_perf: self.frame_perf,
                };
                self.renderer
                    .record_path_clip_mask_pass(&mut ctx, path_vertex_buffer, mask_pass);
            }
            RenderPlanPass::SceneDrawRange(scene_pass) => {
                let mut ctx = ExecuteCtx {
                    device: self.device,
                    queue: self.queue,
                    frame_index: self.frame_index,
                    format: self.format,
                    target_view: self.target_view,
                    viewport_size: self.viewport_size,
                    usage: self.usage,
                    encoder: self.encoder,
                    frame_targets: self.frame_targets,
                    encoding: self.encoding,
                    render_space_offset_u32,
                    quad_vertex_size: self.quad_vertex_size,
                    quad_vertex_bases: self.quad_vertex_bases,
                    perf_enabled: self.perf_enabled,
                    frame_perf: self.frame_perf,
                };
                self.renderer.record_scene_draw_range_pass(
                    &mut ctx,
                    plan,
                    scene_pass,
                    viewport_vertex_buffer,
                    text_vertex_buffer,
                    path_vertex_buffer,
                    quad_instance_bind_group,
                    text_paint_bind_group,
                    path_paint_bind_group,
                );
            }
            RenderPlanPass::PathMsaaBatch(path_pass) => {
                let mut ctx = ExecuteCtx {
                    device: self.device,
                    queue: self.queue,
                    frame_index: self.frame_index,
                    format: self.format,
                    target_view: self.target_view,
                    viewport_size: self.viewport_size,
                    usage: self.usage,
                    encoder: self.encoder,
                    frame_targets: self.frame_targets,
                    encoding: self.encoding,
                    render_space_offset_u32,
                    quad_vertex_size: self.quad_vertex_size,
                    quad_vertex_bases: self.quad_vertex_bases,
                    perf_enabled: self.perf_enabled,
                    frame_perf: self.frame_perf,
                };
                self.renderer.record_path_msaa_batch_pass(
                    &mut ctx,
                    plan,
                    pass_index,
                    path_vertex_buffer,
                    path_paint_bind_group,
                    path_pass,
                );
            }
            RenderPlanPass::ScaleNearest(pass) => {
                record_scale_nearest_pass(self, pass, render_space_offset_u32);
            }
            RenderPlanPass::Blur(pass) => {
                record_blur_pass(self, pass, render_space_offset_u32);
            }
            RenderPlanPass::FullscreenBlit(pass) => {
                record_fullscreen_blit_pass(self, pass);
            }
            RenderPlanPass::BackdropWarp(pass) => {
                record_backdrop_warp_pass(self, pass, render_space_offset_u32);
            }
            RenderPlanPass::ColorAdjust(pass) => {
                record_color_adjust_pass(self, pass, render_space_offset_u32);
            }
            RenderPlanPass::ColorMatrix(pass) => {
                record_color_matrix_pass(self, pass, render_space_offset_u32);
            }
            RenderPlanPass::AlphaThreshold(pass) => {
                record_alpha_threshold_pass(self, pass, render_space_offset_u32);
            }
            RenderPlanPass::DropShadow(pass) => {
                let mut ctx = ExecuteCtx {
                    device: self.device,
                    queue: self.queue,
                    frame_index: self.frame_index,
                    format: self.format,
                    target_view: self.target_view,
                    viewport_size: self.viewport_size,
                    usage: self.usage,
                    encoder: self.encoder,
                    frame_targets: self.frame_targets,
                    encoding: self.encoding,
                    render_space_offset_u32,
                    quad_vertex_size: self.quad_vertex_size,
                    quad_vertex_bases: self.quad_vertex_bases,
                    perf_enabled: self.perf_enabled,
                    frame_perf: self.frame_perf,
                };
                self.renderer.record_drop_shadow_pass(&mut ctx, pass);
            }
            RenderPlanPass::CompositePremul(pass) => {
                let mut ctx = ExecuteCtx {
                    device: self.device,
                    queue: self.queue,
                    frame_index: self.frame_index,
                    format: self.format,
                    target_view: self.target_view,
                    viewport_size: self.viewport_size,
                    usage: self.usage,
                    encoder: self.encoder,
                    frame_targets: self.frame_targets,
                    encoding: self.encoding,
                    render_space_offset_u32,
                    quad_vertex_size: self.quad_vertex_size,
                    quad_vertex_bases: self.quad_vertex_bases,
                    perf_enabled: self.perf_enabled,
                    frame_perf: self.frame_perf,
                };
                self.renderer
                    .record_composite_premul_pass(&mut ctx, pass_index, pass);
            }
            RenderPlanPass::ClipMask(pass) => {
                let mut ctx = ExecuteCtx {
                    device: self.device,
                    queue: self.queue,
                    frame_index: self.frame_index,
                    format: self.format,
                    target_view: self.target_view,
                    viewport_size: self.viewport_size,
                    usage: self.usage,
                    encoder: self.encoder,
                    frame_targets: self.frame_targets,
                    encoding: self.encoding,
                    render_space_offset_u32,
                    quad_vertex_size: self.quad_vertex_size,
                    quad_vertex_bases: self.quad_vertex_bases,
                    perf_enabled: self.perf_enabled,
                    frame_perf: self.frame_perf,
                };
                self.renderer.record_clip_mask_pass(&mut ctx, pass);
            }
            RenderPlanPass::ReleaseTarget(target) => {
                self.frame_targets
                    .release_target(&mut self.renderer.intermediate_pool, *target);
            }
        }
    }
}
