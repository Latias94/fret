use super::bind_group_builders::UniformMaskImageBindGroupGlobals;
use super::bind_group_caches::BindGroupCaches;
use super::gpu_registries::GpuRegistries;
use crate::images::AlphaMode;
use crate::images::ImageDescriptor;
use crate::targets::RenderTargetDescriptor;
use fret_render_core::RenderTargetMetadata;

#[derive(Default)]
pub(super) struct GpuResources {
    registries: GpuRegistries,
    bind_group_caches: BindGroupCaches,
}

impl GpuResources {
    pub(super) fn generations(&self) -> (u64, u64) {
        (
            self.registries.render_targets_generation,
            self.registries.images_generation,
        )
    }

    pub(super) fn render_target_view(
        &self,
        target: fret_core::RenderTargetId,
    ) -> Option<&wgpu::TextureView> {
        self.registries.render_targets.get(target)
    }

    pub(super) fn render_target_revision(&self, target: fret_core::RenderTargetId) -> u64 {
        self.registries
            .render_target_revisions
            .get(&target)
            .copied()
            .unwrap_or(0)
    }

    pub(super) fn render_target_metadata(
        &self,
        target: fret_core::RenderTargetId,
    ) -> Option<RenderTargetMetadata> {
        self.registries.render_targets.metadata(target)
    }

    pub(super) fn image_view(&self, image: fret_core::ImageId) -> Option<&wgpu::TextureView> {
        self.registries.images.get(image)
    }

    pub(super) fn image_revision(&self, image: fret_core::ImageId) -> u64 {
        self.registries
            .image_revisions
            .get(&image)
            .copied()
            .unwrap_or(0)
    }

    pub(super) fn image_size_px(&self, image: fret_core::ImageId) -> Option<(u32, u32)> {
        self.registries.images.size_px(image)
    }

    pub(super) fn image_format(&self, image: fret_core::ImageId) -> Option<wgpu::TextureFormat> {
        self.registries.images.format(image)
    }

    pub(super) fn image_alpha_mode(&self, image: fret_core::ImageId) -> Option<AlphaMode> {
        self.registries.images.alpha_mode(image)
    }

    pub(super) fn caches(&self) -> &BindGroupCaches {
        &self.bind_group_caches
    }

    pub(super) fn caches_mut(&mut self) -> &mut BindGroupCaches {
        &mut self.bind_group_caches
    }

    pub(super) fn ensure_viewport_sampler_texture_bind_group_for_target(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        target: fret_core::RenderTargetId,
    ) {
        let Some(view) = self.registries.render_targets.get(target) else {
            return;
        };
        let revision = self.render_target_revision(target);
        self.bind_group_caches
            .ensure_viewport_sampler_texture_bind_group(
                device, layout, sampler, view, target, revision,
            );
    }

    pub(super) fn ensure_image_sampler_texture_bind_groups_for_image(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler_linear: &wgpu::Sampler,
        sampler_nearest: &wgpu::Sampler,
        image: fret_core::ImageId,
    ) {
        let Some(view) = self.registries.images.get(image) else {
            return;
        };
        let revision = self.image_revision(image);
        self.bind_group_caches
            .ensure_image_sampler_texture_bind_groups(
                device,
                layout,
                sampler_linear,
                sampler_nearest,
                view,
                image,
                revision,
            );
    }

    pub(super) fn ensure_uniform_mask_image_override_bind_groups_for_image(
        &mut self,
        device: &wgpu::Device,
        globals: &UniformMaskImageBindGroupGlobals<'_>,
        sampler_linear: &wgpu::Sampler,
        sampler_nearest: &wgpu::Sampler,
        image: fret_core::ImageId,
        uniforms_revision: u64,
    ) {
        let Some(view) = self.registries.images.get(image) else {
            return;
        };
        let image_revision = self.image_revision(image);
        self.bind_group_caches
            .ensure_uniform_mask_image_override_bind_groups(
                device,
                globals,
                sampler_linear,
                sampler_nearest,
                view,
                image,
                image_revision,
                uniforms_revision,
            );
    }

    pub(super) fn register_render_target(
        &mut self,
        desc: RenderTargetDescriptor,
    ) -> fret_core::RenderTargetId {
        self.registries.register_render_target(desc)
    }

    pub(super) fn update_render_target(
        &mut self,
        id: fret_core::RenderTargetId,
        desc: RenderTargetDescriptor,
    ) -> bool {
        if !self.registries.update_render_target(id, desc) {
            return false;
        }
        self.bind_group_caches.invalidate_render_target(id);
        true
    }

    pub(super) fn unregister_render_target(&mut self, id: fret_core::RenderTargetId) -> bool {
        if !self.registries.unregister_render_target(id) {
            return false;
        }
        self.bind_group_caches.invalidate_render_target(id);
        true
    }

    pub(super) fn register_image(&mut self, desc: ImageDescriptor) -> fret_core::ImageId {
        self.registries.register_image(desc)
    }

    pub(super) fn update_image(&mut self, id: fret_core::ImageId, desc: ImageDescriptor) -> bool {
        if !self.registries.update_image(id, desc) {
            return false;
        }
        self.bind_group_caches.invalidate_image(id);
        true
    }

    pub(super) fn unregister_image(&mut self, id: fret_core::ImageId) -> bool {
        if !self.registries.unregister_image(id) {
            return false;
        }
        self.bind_group_caches.invalidate_image(id);
        true
    }
}
