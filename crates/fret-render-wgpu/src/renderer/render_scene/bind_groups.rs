use super::super::bind_group_builders::UniformMaskImageBindGroupGlobals;
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
            self.bind_group_caches
                .ensure_viewport_sampler_texture_bind_group(
                    device,
                    &self.viewport_bind_group_layout,
                    &self.viewport_sampler,
                    view,
                    target,
                    revision,
                );
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
                .ensure_image_sampler_texture_bind_groups(
                    device,
                    &self.viewport_bind_group_layout,
                    &self.viewport_sampler,
                    &self.image_sampler_nearest,
                    view,
                    image,
                    revision,
                );
        }
    }

    pub(super) fn prepare_uniform_mask_image_bind_groups(
        &mut self,
        device: &wgpu::Device,
        uniform_mask_images: &[Option<UniformMaskImageSelection>],
    ) {
        let globals = UniformMaskImageBindGroupGlobals {
            layout: &self.uniform_bind_group_layout,
            uniform_buffer: &self.uniforms.uniform_buffer,
            clip_buffer: &self.uniforms.clip_buffer,
            mask_buffer: &self.uniforms.mask_buffer,
            material_catalog_view: &self.material_catalog_view,
            material_catalog_sampler: &self.material_catalog_sampler,
            render_space_buffer: &self.uniforms.render_space_buffer,
        };

        for &sel in uniform_mask_images.iter().flatten() {
            let image = sel.image;
            let Some(view) = self.images.get(image) else {
                continue;
            };

            let image_revision = self.image_revisions.get(&image).copied().unwrap_or(0);
            self.bind_group_caches
                .ensure_uniform_mask_image_override_bind_groups(
                    device,
                    &globals,
                    &self.mask_image_sampler,
                    &self.mask_image_sampler_nearest,
                    view,
                    image,
                    image_revision,
                    self.uniforms.revision(),
                );
        }
    }
}
