use super::super::*;

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
            self.viewport_bind_groups.ensure(target, revision, || {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("fret viewport texture bind group"),
                    layout: &self.viewport_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Sampler(&self.viewport_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(view),
                        },
                    ],
                })
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
            self.image_bind_groups.ensure(image, revision, || {
                let bind_group_linear = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("fret image texture bind group (linear)"),
                    layout: &self.viewport_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Sampler(&self.viewport_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(view),
                        },
                    ],
                });
                let bind_group_nearest = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("fret image texture bind group (nearest)"),
                    layout: &self.viewport_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Sampler(&self.image_sampler_nearest),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(view),
                        },
                    ],
                });

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
            self.uniform_mask_image_bind_groups
                .ensure(image, revision, || {
                    let bind_group_linear = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("fret uniforms bind group (mask image override, linear)"),
                        layout: &self.uniform_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &self.uniform_buffer,
                                    offset: 0,
                                    size: Some(std::num::NonZeroU64::new(uniform_size).unwrap()),
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &self.clip_buffer,
                                    offset: 0,
                                    size: None,
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &self.mask_buffer,
                                    offset: 0,
                                    size: None,
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: wgpu::BindingResource::TextureView(
                                    &self.material_catalog_view,
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 4,
                                resource: wgpu::BindingResource::Sampler(
                                    &self.material_catalog_sampler,
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 5,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &self.render_space_buffer,
                                    offset: 0,
                                    size: Some(
                                        std::num::NonZeroU64::new(render_space_size).unwrap(),
                                    ),
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 6,
                                resource: wgpu::BindingResource::Sampler(&self.mask_image_sampler),
                            },
                            wgpu::BindGroupEntry {
                                binding: 7,
                                resource: wgpu::BindingResource::TextureView(view),
                            },
                        ],
                    });
                    let bind_group_nearest = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("fret uniforms bind group (mask image override, nearest)"),
                        layout: &self.uniform_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &self.uniform_buffer,
                                    offset: 0,
                                    size: Some(std::num::NonZeroU64::new(uniform_size).unwrap()),
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &self.clip_buffer,
                                    offset: 0,
                                    size: None,
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &self.mask_buffer,
                                    offset: 0,
                                    size: None,
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: wgpu::BindingResource::TextureView(
                                    &self.material_catalog_view,
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 4,
                                resource: wgpu::BindingResource::Sampler(
                                    &self.material_catalog_sampler,
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 5,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &self.render_space_buffer,
                                    offset: 0,
                                    size: Some(
                                        std::num::NonZeroU64::new(render_space_size).unwrap(),
                                    ),
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 6,
                                resource: wgpu::BindingResource::Sampler(
                                    &self.mask_image_sampler_nearest,
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 7,
                                resource: wgpu::BindingResource::TextureView(view),
                            },
                        ],
                    });

                    (bind_group_linear, bind_group_nearest)
                });
        }
    }
}
