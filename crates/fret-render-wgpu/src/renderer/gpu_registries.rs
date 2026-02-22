use crate::images::{ImageDescriptor, ImageRegistry};
use crate::targets::RenderTargetDescriptor;
use crate::targets::RenderTargetRegistry;
use std::collections::HashMap;

pub(super) struct GpuRegistries {
    pub(super) render_targets: RenderTargetRegistry,
    pub(super) images: ImageRegistry,

    pub(super) render_target_revisions: HashMap<fret_core::RenderTargetId, u64>,
    pub(super) render_targets_generation: u64,

    pub(super) image_revisions: HashMap<fret_core::ImageId, u64>,
    pub(super) images_generation: u64,
}

impl Default for GpuRegistries {
    fn default() -> Self {
        Self {
            render_targets: RenderTargetRegistry::default(),
            images: ImageRegistry::default(),
            render_target_revisions: HashMap::new(),
            render_targets_generation: 0,
            image_revisions: HashMap::new(),
            images_generation: 0,
        }
    }
}

impl GpuRegistries {
    pub(super) fn register_render_target(
        &mut self,
        desc: RenderTargetDescriptor,
    ) -> fret_core::RenderTargetId {
        let id = self.render_targets.register(desc);
        self.render_target_revisions.insert(id, 1);
        self.render_targets_generation = self.render_targets_generation.saturating_add(1);
        id
    }

    pub(super) fn update_render_target(
        &mut self,
        id: fret_core::RenderTargetId,
        desc: RenderTargetDescriptor,
    ) -> bool {
        if !self.render_targets.update(id, desc) {
            return false;
        }
        let next = self.render_target_revisions.get(&id).copied().unwrap_or(0) + 1;
        self.render_target_revisions.insert(id, next);
        self.render_targets_generation = self.render_targets_generation.saturating_add(1);
        true
    }

    pub(super) fn unregister_render_target(&mut self, id: fret_core::RenderTargetId) -> bool {
        if !self.render_targets.unregister(id) {
            return false;
        }
        self.render_target_revisions.remove(&id);
        self.render_targets_generation = self.render_targets_generation.saturating_add(1);
        true
    }

    pub(super) fn register_image(&mut self, desc: ImageDescriptor) -> fret_core::ImageId {
        let id = self.images.register(desc);
        self.image_revisions.insert(id, 1);
        self.images_generation = self.images_generation.saturating_add(1);
        id
    }

    pub(super) fn update_image(&mut self, id: fret_core::ImageId, desc: ImageDescriptor) -> bool {
        if !self.images.update(id, desc) {
            return false;
        }
        let next = self.image_revisions.get(&id).copied().unwrap_or(0) + 1;
        self.image_revisions.insert(id, next);
        self.images_generation = self.images_generation.saturating_add(1);
        true
    }

    pub(super) fn unregister_image(&mut self, id: fret_core::ImageId) -> bool {
        if !self.images.unregister(id) {
            return false;
        }
        self.image_revisions.remove(&id);
        self.images_generation = self.images_generation.saturating_add(1);
        true
    }
}
