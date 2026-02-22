use super::bind_group_caches::BindGroupCaches;
use super::gpu_registries::GpuRegistries;
use crate::images::ImageDescriptor;
use crate::targets::RenderTargetDescriptor;

#[derive(Default)]
pub(super) struct GpuResources {
    pub(super) registries: GpuRegistries,
    pub(super) bind_group_caches: BindGroupCaches,
}

impl GpuResources {
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
