use super::*;

pub(super) struct UniformBindGroupGlobals<'a> {
    pub(super) layout: &'a wgpu::BindGroupLayout,
    pub(super) material_catalog_view: &'a wgpu::TextureView,
    pub(super) material_catalog_sampler: &'a wgpu::Sampler,
    pub(super) mask_image_sampler: &'a wgpu::Sampler,
    pub(super) mask_image_identity_view: &'a wgpu::TextureView,
}

impl<'a> UniformBindGroupGlobals<'a> {
    pub(super) fn create(
        &self,
        device: &wgpu::Device,
        label: &'static str,
        uniform_buffer: &wgpu::Buffer,
        clip_buffer: &wgpu::Buffer,
        mask_buffer: &wgpu::Buffer,
        render_space_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        let uniform_size = std::mem::size_of::<ViewportUniform>() as u64;
        let render_space_size = std::mem::size_of::<RenderSpaceUniform>() as u64;

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: uniform_buffer,
                        offset: 0,
                        size: Some(std::num::NonZeroU64::new(uniform_size).unwrap()),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: clip_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: mask_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(self.material_catalog_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(self.material_catalog_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: render_space_buffer,
                        offset: 0,
                        size: Some(std::num::NonZeroU64::new(render_space_size).unwrap()),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(self.mask_image_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(self.mask_image_identity_view),
                },
            ],
        })
    }
}

pub(super) struct UniformMaskImageBindGroupGlobals<'a> {
    pub(super) layout: &'a wgpu::BindGroupLayout,
    pub(super) uniform_buffer: &'a wgpu::Buffer,
    pub(super) clip_buffer: &'a wgpu::Buffer,
    pub(super) mask_buffer: &'a wgpu::Buffer,
    pub(super) material_catalog_view: &'a wgpu::TextureView,
    pub(super) material_catalog_sampler: &'a wgpu::Sampler,
    pub(super) render_space_buffer: &'a wgpu::Buffer,
}

impl<'a> UniformMaskImageBindGroupGlobals<'a> {
    pub(super) fn create(
        &self,
        device: &wgpu::Device,
        label: &'static str,
        mask_image_sampler: &wgpu::Sampler,
        mask_image_view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        let uniform_size = std::mem::size_of::<ViewportUniform>() as u64;
        let render_space_size = std::mem::size_of::<RenderSpaceUniform>() as u64;

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: self.uniform_buffer,
                        offset: 0,
                        size: Some(std::num::NonZeroU64::new(uniform_size).unwrap()),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: self.clip_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: self.mask_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(self.material_catalog_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(self.material_catalog_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: self.render_space_buffer,
                        offset: 0,
                        size: Some(std::num::NonZeroU64::new(render_space_size).unwrap()),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(mask_image_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(mask_image_view),
                },
            ],
        })
    }
}
