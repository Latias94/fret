use super::super::frame_targets::FrameTargets;
use super::super::*;
use super::ctx::ExecuteCtx;

pub(super) struct RenderSceneExecutor<'a> {
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
            scale_param_size: self.scale_param_size,
            scale_param_cursor: self.scale_param_cursor,
            quad_vertex_size: self.quad_vertex_size,
            quad_vertex_bases: self.quad_vertex_bases,
            perf_enabled: self.perf_enabled,
            frame_perf: self.frame_perf,
        };

        match planned_pass {
            RenderPlanPass::PathClipMask(mask_pass) => {
                self.renderer
                    .record_path_clip_mask_pass(&mut ctx, path_vertex_buffer, mask_pass);
            }
            RenderPlanPass::SceneDrawRange(scene_pass) => {
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
                self.renderer.record_scale_nearest_pass(&mut ctx, pass);
            }
            RenderPlanPass::Blur(pass) => {
                self.renderer.record_blur_pass(&mut ctx, pass);
            }
            RenderPlanPass::FullscreenBlit(pass) => {
                self.renderer.record_fullscreen_blit_pass(&mut ctx, pass);
            }
            RenderPlanPass::BackdropWarp(pass) => {
                self.renderer.record_backdrop_warp_pass(&mut ctx, pass);
            }
            RenderPlanPass::ColorAdjust(pass) => {
                self.renderer.record_color_adjust_pass(&mut ctx, pass);
            }
            RenderPlanPass::ColorMatrix(pass) => {
                self.renderer.record_color_matrix_pass(&mut ctx, pass);
            }
            RenderPlanPass::AlphaThreshold(pass) => {
                self.renderer.record_alpha_threshold_pass(&mut ctx, pass);
            }
            RenderPlanPass::DropShadow(pass) => {
                self.renderer.record_drop_shadow_pass(&mut ctx, pass);
            }
            RenderPlanPass::CompositePremul(pass) => {
                self.renderer
                    .record_composite_premul_pass(&mut ctx, pass_index, pass);
            }
            RenderPlanPass::ClipMask(pass) => {
                self.renderer.record_clip_mask_pass(&mut ctx, pass);
            }
            RenderPlanPass::ReleaseTarget(target) => {
                ctx.frame_targets
                    .release_target(&mut self.renderer.intermediate_pool, *target);
            }
        }
    }
}
