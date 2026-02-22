use super::super::bind_group_builders::UniformMaskImageBindGroupGlobals;
use super::super::bind_group_caches::SamplingBindGroups;
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

                    SamplingBindGroups {
                        linear: bind_group_linear,
                        nearest: bind_group_nearest,
                    }
                });
        }
    }

    pub(super) fn prepare_uniform_mask_image_bind_groups(
        &mut self,
        device: &wgpu::Device,
        uniform_mask_images: &[Option<UniformMaskImageSelection>],
    ) {
        let globals = UniformMaskImageBindGroupGlobals {
            layout: &self.uniform_bind_group_layout,
            uniform_buffer: &self.uniform_buffer,
            clip_buffer: &self.clip_buffer,
            mask_buffer: &self.mask_buffer,
            material_catalog_view: &self.material_catalog_view,
            material_catalog_sampler: &self.material_catalog_sampler,
            render_space_buffer: &self.render_space_buffer,
        };

        for &sel in uniform_mask_images.iter().flatten() {
            let image = sel.image;
            let Some(view) = self.images.get(image) else {
                continue;
            };

            let revision = self.image_revisions.get(&image).copied().unwrap_or(0);
            self.bind_group_caches
                .ensure_uniform_mask_image_bind_groups(image, revision, || {
                    let bind_group_linear = globals.create(
                        device,
                        "fret uniforms bind group (mask image override, linear)",
                        &self.mask_image_sampler,
                        view,
                    );
                    let bind_group_nearest = globals.create(
                        device,
                        "fret uniforms bind group (mask image override, nearest)",
                        &self.mask_image_sampler_nearest,
                        view,
                    );

                    SamplingBindGroups {
                        linear: bind_group_linear,
                        nearest: bind_group_nearest,
                    }
                });
        }
    }
}
