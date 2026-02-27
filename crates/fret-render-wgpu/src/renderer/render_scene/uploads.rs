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
        self.quad_instances.ensure_capacity(device, instances.len());
        self.path_paints.ensure_capacity(device, path_paints.len());
        self.text_paints.ensure_capacity(device, text_paints.len());
        self.viewport_vertices
            .ensure_capacity(device, viewport_vertices.len());
        self.text_vertices
            .ensure_capacity(device, text_vertices.len());
        self.path_vertices
            .ensure_capacity(device, path_vertices.len());

        let (instance_buffer, quad_instance_bind_group) = self.quad_instances.next_pair();
        if !instances.is_empty() {
            queue.write_buffer(&instance_buffer, 0, bytemuck::cast_slice(instances));
            if perf_enabled {
                frame_perf.instance_bytes = frame_perf
                    .instance_bytes
                    .saturating_add(std::mem::size_of_val(instances) as u64);
            }
        }

        let (path_paint_buffer, path_paint_bind_group) = self.path_paints.next_pair();
        if !path_paints.is_empty() {
            queue.write_buffer(&path_paint_buffer, 0, bytemuck::cast_slice(path_paints));
            if perf_enabled {
                frame_perf.instance_bytes = frame_perf
                    .instance_bytes
                    .saturating_add(std::mem::size_of_val(path_paints) as u64);
            }
        }

        let (text_paint_buffer, text_paint_bind_group) = self.text_paints.next_pair();
        if !text_paints.is_empty() {
            queue.write_buffer(&text_paint_buffer, 0, bytemuck::cast_slice(text_paints));
            if perf_enabled {
                frame_perf.instance_bytes = frame_perf
                    .instance_bytes
                    .saturating_add(std::mem::size_of_val(text_paints) as u64);
            }
        }

        let viewport_vertex_buffer = self.viewport_vertices.next_buffer();
        if !viewport_vertices.is_empty() {
            queue.write_buffer(
                &viewport_vertex_buffer,
                0,
                bytemuck::cast_slice(viewport_vertices),
            );
            if perf_enabled {
                frame_perf.vertex_bytes = frame_perf
                    .vertex_bytes
                    .saturating_add(std::mem::size_of_val(viewport_vertices) as u64);
            }
        }

        let text_vertex_buffer = self.text_vertices.next_buffer();
        if !text_vertices.is_empty() {
            queue.write_buffer(&text_vertex_buffer, 0, bytemuck::cast_slice(text_vertices));
            if perf_enabled {
                frame_perf.vertex_bytes = frame_perf
                    .vertex_bytes
                    .saturating_add(std::mem::size_of_val(text_vertices) as u64);
            }
        }

        let path_vertex_buffer = self.path_vertices.next_buffer();
        if !path_vertices.is_empty() {
            queue.write_buffer(&path_vertex_buffer, 0, bytemuck::cast_slice(path_vertices));
            if perf_enabled {
                frame_perf.vertex_bytes = frame_perf
                    .vertex_bytes
                    .saturating_add(std::mem::size_of_val(path_vertices) as u64);
            }
        }

        // Some passes draw textured quads (not fullscreen triangles). Upload the vertex payload
        // once per frame and reference it via slices, since multiple `queue.write_buffer()` calls
        // against the same buffer region in a single submission would make all passes observe the
        // final write.
        let quad_vertex_bases = upload_plan_quad_vertices(self, device, queue, plan, viewport_size);

        FrameUploadResources {
            quad_instance_bind_group,
            text_paint_bind_group,
            path_paint_bind_group,
            viewport_vertex_buffer,
            text_vertex_buffer,
            path_vertex_buffer,
            quad_vertex_bases,
        }
    }
}
