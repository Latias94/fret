use crate::images::ImageRegistry;
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
