use super::super::*;
use super::dispatch_state::RenderSceneDispatchState;
use super::executor::RecordPassResources;

impl Renderer {
    pub(super) fn dispatch_render_plan(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame_index: u64,
        format: wgpu::TextureFormat,
        target_view: &wgpu::TextureView,
        viewport_size: (u32, u32),
        plan: &RenderPlan,
        encoding: &SceneEncoding,
        quad_vertex_bases: &[Option<u32>],
        viewport_vertex_buffer: &wgpu::Buffer,
        text_vertex_buffer: &wgpu::Buffer,
        path_vertex_buffer: &wgpu::Buffer,
        quad_instance_bind_group: &wgpu::BindGroup,
        text_paint_bind_group: &wgpu::BindGroup,
        path_paint_bind_group: &wgpu::BindGroup,
        perf_enabled: bool,
        trace_enabled: bool,
        frame_perf: &mut RenderPerfStats,
    ) -> wgpu::CommandBuffer {
        let mut dispatch_state = RenderSceneDispatchState::new(device);
        let resources = RecordPassResources {
            viewport_vertex_buffer,
            text_vertex_buffer,
            path_vertex_buffer,
            quad_instance_bind_group,
            text_paint_bind_group,
            path_paint_bind_group,
        };
        dispatch_state.record_passes(
            self,
            device,
            queue,
            frame_index,
            format,
            target_view,
            viewport_size,
            plan,
            encoding,
            quad_vertex_bases,
            &resources,
            perf_enabled,
            trace_enabled,
            frame_perf,
        );
        dispatch_state.finish(
            self,
            frame_index,
            plan,
            perf_enabled,
            trace_enabled,
            frame_perf,
        )
    }
}
