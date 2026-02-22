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
            self.gpu_resources
                .ensure_viewport_sampler_texture_bind_group_for_target(
                    device,
                    &self.globals.viewport_bind_group_layout,
                    &self.globals.viewport_sampler,
                    target,
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
            self.gpu_resources
                .ensure_image_sampler_texture_bind_groups_for_image(
                    device,
                    &self.globals.viewport_bind_group_layout,
                    &self.globals.viewport_sampler,
                    &self.globals.image_sampler_nearest,
                    image,
                );
        }
    }

    pub(super) fn prepare_uniform_mask_image_bind_groups(
        &mut self,
        device: &wgpu::Device,
        uniform_mask_images: &[Option<UniformMaskImageSelection>],
    ) {
        let globals = UniformMaskImageBindGroupGlobals {
            layout: &self.globals.uniform_bind_group_layout,
            uniform_buffer: &self.uniforms.uniform_buffer,
            clip_buffer: &self.uniforms.clip_buffer,
            mask_buffer: &self.uniforms.mask_buffer,
            material_catalog_view: &self.globals.material_catalog_view,
            material_catalog_sampler: &self.globals.material_catalog_sampler,
            render_space_buffer: &self.uniforms.render_space_buffer,
        };

        for &sel in uniform_mask_images.iter().flatten() {
            let image = sel.image;
            self.gpu_resources
                .ensure_uniform_mask_image_override_bind_groups_for_image(
                    device,
                    &globals,
                    &self.globals.mask_image_sampler,
                    &self.globals.mask_image_sampler_nearest,
                    image,
                    self.uniforms.revision(),
                );
        }
    }
}
