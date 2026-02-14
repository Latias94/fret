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
            let needs_rebuild = match self.viewport_bind_groups.get(&target) {
                Some((cached, _)) => *cached != revision,
                None => true,
            };
            if !needs_rebuild {
                continue;
            }

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            });

            self.viewport_bind_groups
                .insert(target, (revision, bind_group));
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
            let needs_rebuild = match self.image_bind_groups.get(&image) {
                Some((cached, _)) => *cached != revision,
                None => true,
            };
            if !needs_rebuild {
                continue;
            }

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fret image texture bind group"),
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

            self.image_bind_groups.insert(image, (revision, bind_group));
        }
    }

    pub(super) fn prepare_uniform_mask_image_bind_groups(
        &mut self,
        device: &wgpu::Device,
        uniform_mask_images: &[Option<fret_core::ImageId>],
    ) {
        let uniform_size = std::mem::size_of::<ViewportUniform>() as u64;
        let render_space_size = std::mem::size_of::<RenderSpaceUniform>() as u64;

        let material_catalog_view =
            self.material_catalog_texture
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("fret material catalog texture array view (uniform mask image)"),
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    ..Default::default()
                });
        let material_catalog_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fret material catalog sampler (uniform mask image)"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        for &image in uniform_mask_images.iter().flatten() {
            let Some(view) = self.images.get(image) else {
                continue;
            };

            let revision = self.image_revisions.get(&image).copied().unwrap_or(0);
            let needs_rebuild = match self.uniform_mask_image_bind_groups.get(&image) {
                Some((cached, _)) => *cached != revision,
                None => true,
            };
            if !needs_rebuild {
                continue;
            }

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fret uniforms bind group (mask image override)"),
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
                        resource: wgpu::BindingResource::TextureView(&material_catalog_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&material_catalog_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.render_space_buffer,
                            offset: 0,
                            size: Some(std::num::NonZeroU64::new(render_space_size).unwrap()),
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

            self.uniform_mask_image_bind_groups
                .insert(image, (revision, bind_group));
        }
    }
}
