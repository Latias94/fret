use super::super::*;
use super::quad_vertices::upload_plan_quad_vertices;

pub(super) struct FrameUploadResources {
    pub(super) quad_instance_bind_group: wgpu::BindGroup,
    pub(super) text_paint_bind_group: wgpu::BindGroup,
    pub(super) path_paint_bind_group: wgpu::BindGroup,
    pub(super) viewport_vertex_buffer: wgpu::Buffer,
    pub(super) text_vertex_buffer: wgpu::Buffer,
    pub(super) path_vertex_buffer: wgpu::Buffer,
    pub(super) quad_vertex_bases: Vec<Option<u32>>,
}

impl Renderer {
    pub(super) fn upload_frame_geometry(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        plan: &RenderPlan,
        viewport_size: (u32, u32),
        instances: &[QuadInstance],
        path_paints: &[PaintGpu],
        text_paints: &[PaintGpu],
        viewport_vertices: &[ViewportVertex],
        text_vertices: &[TextVertex],
        path_vertices: &[PathVertex],
        perf_enabled: bool,
        frame_perf: &mut RenderPerfStats,
    ) -> FrameUploadResources {
        let uploads = self.geometry_upload_state.upload_frame_geometry(
            device,
            queue,
            instances,
            path_paints,
            text_paints,
            viewport_vertices,
            text_vertices,
            path_vertices,
            perf_enabled,
            frame_perf,
        );

        // Some passes draw textured quads (not fullscreen triangles). Upload the vertex payload
        // once per frame and reference it via slices, since multiple `queue.write_buffer()` calls
        // against the same buffer region in a single submission would make all passes observe the
        // final write.
        let quad_vertex_bases = upload_plan_quad_vertices(self, device, queue, plan, viewport_size);

        FrameUploadResources {
            quad_instance_bind_group: uploads.quad_instance_bind_group,
            text_paint_bind_group: uploads.text_paint_bind_group,
            path_paint_bind_group: uploads.path_paint_bind_group,
            viewport_vertex_buffer: uploads.viewport_vertex_buffer,
            text_vertex_buffer: uploads.text_vertex_buffer,
            path_vertex_buffer: uploads.path_vertex_buffer,
            quad_vertex_bases,
        }
    }
}
