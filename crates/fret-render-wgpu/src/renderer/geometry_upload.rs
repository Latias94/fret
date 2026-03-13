use super::*;

pub(super) struct GeometryUploadState {
    quad_instances: buffers::StorageRingBuffer<QuadInstance>,
    path_paints: buffers::StorageRingBuffer<PaintGpu>,
    text_paints: buffers::StorageRingBuffer<PaintGpu>,
    viewport_vertices: buffers::RingBuffer<ViewportVertex>,
    text_vertices: buffers::RingBuffer<TextVertex>,
    path_vertices: buffers::RingBuffer<PathVertex>,
}

pub(super) struct FrameGeometryUploads {
    pub(super) quad_instance_bind_group: wgpu::BindGroup,
    pub(super) text_paint_bind_group: wgpu::BindGroup,
    pub(super) path_paint_bind_group: wgpu::BindGroup,
    pub(super) viewport_vertex_buffer: wgpu::Buffer,
    pub(super) text_vertex_buffer: wgpu::Buffer,
    pub(super) path_vertex_buffer: wgpu::Buffer,
}

impl GeometryUploadState {
    pub(super) fn new(device: &wgpu::Device) -> Self {
        const FRAMES_IN_FLIGHT: usize = 3;

        let quad_instance_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret quad instances bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let instance_usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        let quad_instances = buffers::StorageRingBuffer::<QuadInstance>::new(
            device,
            FRAMES_IN_FLIGHT,
            1024,
            quad_instance_bind_group_layout,
            "fret quad instances",
            instance_usage,
        );

        let path_paint_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret path paints bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let paint_usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        let path_paints = buffers::StorageRingBuffer::<PaintGpu>::new(
            device,
            FRAMES_IN_FLIGHT,
            1024,
            path_paint_bind_group_layout,
            "fret path paints",
            paint_usage,
        );

        let text_paint_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret text paints bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let text_paints = buffers::StorageRingBuffer::<PaintGpu>::new(
            device,
            FRAMES_IN_FLIGHT,
            1024,
            text_paint_bind_group_layout,
            "fret text paints",
            paint_usage,
        );

        let vertex_usage = wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST;
        let viewport_vertices = buffers::RingBuffer::<ViewportVertex>::new(
            device,
            FRAMES_IN_FLIGHT,
            64 * 6,
            "fret viewport vertices",
            vertex_usage,
        );
        let text_vertices = buffers::RingBuffer::<TextVertex>::new(
            device,
            FRAMES_IN_FLIGHT,
            512 * 6,
            "fret text vertices",
            vertex_usage,
        );
        let path_vertices = buffers::RingBuffer::<PathVertex>::new(
            device,
            FRAMES_IN_FLIGHT,
            1024,
            "fret path vertices",
            vertex_usage,
        );

        Self {
            quad_instances,
            path_paints,
            text_paints,
            viewport_vertices,
            text_vertices,
            path_vertices,
        }
    }

    pub(super) fn quad_instances_layout(&self) -> &wgpu::BindGroupLayout {
        self.quad_instances.layout()
    }

    pub(super) fn path_paints_layout(&self) -> &wgpu::BindGroupLayout {
        self.path_paints.layout()
    }

    pub(super) fn text_paints_layout(&self) -> &wgpu::BindGroupLayout {
        self.text_paints.layout()
    }

    pub(super) fn upload_frame_geometry(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[QuadInstance],
        path_paints: &[PaintGpu],
        text_paints: &[PaintGpu],
        viewport_vertices: &[ViewportVertex],
        text_vertices: &[TextVertex],
        path_vertices: &[PathVertex],
        perf_enabled: bool,
        frame_perf: &mut RenderPerfStats,
    ) -> FrameGeometryUploads {
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

        FrameGeometryUploads {
            quad_instance_bind_group,
            text_paint_bind_group,
            path_paint_bind_group,
            viewport_vertex_buffer,
            text_vertex_buffer,
            path_vertex_buffer,
        }
    }
}
