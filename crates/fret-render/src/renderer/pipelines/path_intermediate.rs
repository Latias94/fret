use super::super::*;

impl Renderer {
    pub(in crate::renderer) fn ensure_path_composite_vertex_buffer(
        &mut self,
        device: &wgpu::Device,
        required_vertices: usize,
    ) {
        if required_vertices == 0 {
            return;
        }

        if self.path_composite_vertex_capacity >= required_vertices {
            return;
        }

        let next_capacity = required_vertices
            .next_power_of_two()
            .max(self.path_composite_vertex_capacity.max(64 * 6));

        self.path_composite_vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret path composite vertices"),
            size: (next_capacity * std::mem::size_of::<ViewportVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.path_composite_vertex_capacity = next_capacity;
    }

    pub(in crate::renderer) fn ensure_path_intermediate(
        &mut self,
        device: &wgpu::Device,
        viewport_size: (u32, u32),
        format: wgpu::TextureFormat,
        sample_count: u32,
    ) {
        let needs_rebuild = match &self.path_intermediate {
            Some(cur) => {
                cur.size != viewport_size
                    || cur.format != format
                    || cur.sample_count != sample_count
            }
            None => true,
        };
        if !needs_rebuild {
            return;
        }

        let (msaa_texture, msaa_view) = if sample_count > 1 {
            let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("fret path intermediate msaa"),
                size: wgpu::Extent3d {
                    width: viewport_size.0,
                    height: viewport_size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let msaa_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());
            (Some(msaa_texture), Some(msaa_view))
        } else {
            (None, None)
        };

        let resolved_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fret path intermediate resolved"),
            size: wgpu::Extent3d {
                width: viewport_size.0,
                height: viewport_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let resolved_view = resolved_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret path intermediate bind group"),
            layout: &self.viewport_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&self.viewport_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&resolved_view),
                },
            ],
        });

        self.path_intermediate = Some(PathIntermediate {
            size: viewport_size,
            format,
            sample_count,
            _msaa_texture: msaa_texture,
            msaa_view,
            _resolved_texture: resolved_texture,
            resolved_view,
            bind_group,
        });
    }
}
