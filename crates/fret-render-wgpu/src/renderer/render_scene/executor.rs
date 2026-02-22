use super::super::frame_targets::FrameTargets;
use super::super::*;
use super::recorders::{
    SceneDrawRangePassArgs, record_alpha_threshold_pass, record_backdrop_warp_pass,
    record_blur_pass, record_clip_mask_pass, record_color_adjust_pass, record_color_matrix_pass,
    record_composite_premul_pass, record_drop_shadow_pass, record_fullscreen_blit_pass,
    record_path_clip_mask_pass, record_path_msaa_batch_pass, record_scale_nearest_pass,
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

pub(super) struct RecordPassResources<'a> {
    pub(super) viewport_vertex_buffer: &'a wgpu::Buffer,
    pub(super) text_vertex_buffer: &'a wgpu::Buffer,
    pub(super) path_vertex_buffer: &'a wgpu::Buffer,
    pub(super) quad_instance_bind_group: &'a wgpu::BindGroup,
    pub(super) text_paint_bind_group: &'a wgpu::BindGroup,
    pub(super) path_paint_bind_group: &'a wgpu::BindGroup,
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

    pub(super) fn record_pass(
        &mut self,
        plan: &RenderPlan,
        pass_index: usize,
        planned_pass: &RenderPlanPass,
        render_space_offset_u32: u32,
        resources: &RecordPassResources<'_>,
    ) {
        match planned_pass {
            RenderPlanPass::PathClipMask(mask_pass) => {
                record_path_clip_mask_pass(
                    self,
                    resources.path_vertex_buffer,
                    mask_pass,
                    render_space_offset_u32,
                );
            }
            RenderPlanPass::SceneDrawRange(scene_pass) => {
                let device = self.device;
                let format = self.format;
                let target_view = self.target_view;
                let usage = self.usage;
                let encoding = self.encoding;
                let perf_enabled = self.perf_enabled;

                let renderer = &mut *self.renderer;
                let encoder = &mut *self.encoder;
                let frame_targets = &mut *self.frame_targets;
                let frame_perf = &mut *self.frame_perf;

                let mut args = SceneDrawRangePassArgs {
                    device,
                    format,
                    target_view,
                    usage,
                    encoder,
                    frame_targets,
                    encoding,
                    render_space_offset_u32,
                    perf_enabled,
                    frame_perf,
                    plan,
                    scene_pass,
                    viewport_vertex_buffer: resources.viewport_vertex_buffer,
                    text_vertex_buffer: resources.text_vertex_buffer,
                    path_vertex_buffer: resources.path_vertex_buffer,
                    quad_instance_bind_group: resources.quad_instance_bind_group,
                    text_paint_bind_group: resources.text_paint_bind_group,
                    path_paint_bind_group: resources.path_paint_bind_group,
                };
                renderer.record_scene_draw_range_pass(&mut args);
            }
            RenderPlanPass::PathMsaaBatch(path_pass) => {
                record_path_msaa_batch_pass(
                    self,
                    plan,
                    pass_index,
                    resources.path_vertex_buffer,
                    resources.path_paint_bind_group,
                    path_pass,
                    render_space_offset_u32,
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
                record_drop_shadow_pass(self, pass, render_space_offset_u32);
            }
            RenderPlanPass::CompositePremul(pass) => {
                record_composite_premul_pass(self, pass_index, pass, render_space_offset_u32);
            }
            RenderPlanPass::ClipMask(pass) => {
                record_clip_mask_pass(self, pass, render_space_offset_u32);
            }
            RenderPlanPass::ReleaseTarget(target) => {
                self.frame_targets
                    .release_target(&mut self.renderer.intermediate_pool, *target);
            }
        }
    }
}
