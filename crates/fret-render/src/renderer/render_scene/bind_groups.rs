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
}
