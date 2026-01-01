use super::super::*;

impl Renderer {
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

        let msaa_view = if sample_count > 1 {
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
            Some(msaa_texture.create_view(&wgpu::TextureViewDescriptor::default()))
        } else {
            None
        };

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
            resolved_view,
            msaa_view,
            bind_group,
        });
    }
}
