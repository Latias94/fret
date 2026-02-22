use super::super::*;

fn create_sampler_texture_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    view: &wgpu::TextureView,
    label: &'static str,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(view),
            },
        ],
    })
}

fn create_uniform_mask_image_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    label: &'static str,
    uniform_buffer: &wgpu::Buffer,
    clip_buffer: &wgpu::Buffer,
    mask_buffer: &wgpu::Buffer,
    material_catalog_view: &wgpu::TextureView,
    material_catalog_sampler: &wgpu::Sampler,
    render_space_buffer: &wgpu::Buffer,
    uniform_size: u64,
    render_space_size: u64,
    mask_image_sampler: &wgpu::Sampler,
    mask_image_view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
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
                resource: wgpu::BindingResource::TextureView(material_catalog_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(material_catalog_sampler),
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
                resource: wgpu::BindingResource::Sampler(mask_image_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: wgpu::BindingResource::TextureView(mask_image_view),
            },
        ],
    })
}

impl Renderer {
    pub(super) fn prepare_viewport_bind_groups(
        &mut self,
        device: &wgpu::Device,
        draws: &[OrderedDraw],
    ) {
        for item in draws {
            let OrderedDraw::Viewport(draw) = item else {
                continue;
            };

            let target = draw.target;
            let Some(view) = self.render_targets.get(target) else {
                continue;
            };

            let revision = self
                .render_target_revisions
                .get(&target)
                .copied()
                .unwrap_or(0);
            self.bind_group_caches
                .ensure_viewport_bind_group(target, revision, || {
                    create_sampler_texture_bind_group(
                        device,
                        &self.viewport_bind_group_layout,
                        &self.viewport_sampler,
                        view,
                        "fret viewport texture bind group",
                    )
                });
        }
    }

    pub(super) fn prepare_image_bind_groups(
        &mut self,
        device: &wgpu::Device,
        draws: &[OrderedDraw],
    ) {
        for item in draws {
            let image = match item {
                OrderedDraw::Image(draw) => draw.image,
                OrderedDraw::Mask(draw) => draw.image,
                _ => continue,
            };
            let Some(view) = self.images.get(image) else {
                continue;
            };

            let revision = self.image_revisions.get(&image).copied().unwrap_or(0);
            self.bind_group_caches
                .ensure_image_bind_groups(image, revision, || {
                    let bind_group_linear = create_sampler_texture_bind_group(
                        device,
                        &self.viewport_bind_group_layout,
                        &self.viewport_sampler,
                        view,
                        "fret image texture bind group (linear)",
                    );
                    let bind_group_nearest = create_sampler_texture_bind_group(
                        device,
                        &self.viewport_bind_group_layout,
                        &self.image_sampler_nearest,
                        view,
                        "fret image texture bind group (nearest)",
                    );

                    (bind_group_linear, bind_group_nearest)
                });
        }
    }

    pub(super) fn prepare_uniform_mask_image_bind_groups(
        &mut self,
        device: &wgpu::Device,
        uniform_mask_images: &[Option<UniformMaskImageSelection>],
    ) {
        let uniform_size = std::mem::size_of::<ViewportUniform>() as u64;
        let render_space_size = std::mem::size_of::<RenderSpaceUniform>() as u64;

        for &sel in uniform_mask_images.iter().flatten() {
            let image = sel.image;
            let Some(view) = self.images.get(image) else {
                continue;
            };

            let revision = self.image_revisions.get(&image).copied().unwrap_or(0);
            self.bind_group_caches
                .ensure_uniform_mask_image_bind_groups(image, revision, || {
                    let bind_group_linear = create_uniform_mask_image_bind_group(
                        device,
                        &self.uniform_bind_group_layout,
                        "fret uniforms bind group (mask image override, linear)",
                        &self.uniform_buffer,
                        &self.clip_buffer,
                        &self.mask_buffer,
                        &self.material_catalog_view,
                        &self.material_catalog_sampler,
                        &self.render_space_buffer,
                        uniform_size,
                        render_space_size,
                        &self.mask_image_sampler,
                        view,
                    );
                    let bind_group_nearest = create_uniform_mask_image_bind_group(
                        device,
                        &self.uniform_bind_group_layout,
                        "fret uniforms bind group (mask image override, nearest)",
                        &self.uniform_buffer,
                        &self.clip_buffer,
                        &self.mask_buffer,
                        &self.material_catalog_view,
                        &self.material_catalog_sampler,
                        &self.render_space_buffer,
                        uniform_size,
                        render_space_size,
                        &self.mask_image_sampler_nearest,
                        view,
                    );

                    (bind_group_linear, bind_group_nearest)
                });
        }
    }
}
