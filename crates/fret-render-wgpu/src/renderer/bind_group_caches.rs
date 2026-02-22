use super::revisioned_cache::RevisionedCache;

pub(super) struct SamplingBindGroups {
    pub(super) linear: wgpu::BindGroup,
    pub(super) nearest: wgpu::BindGroup,
}

impl SamplingBindGroups {
    pub(super) fn pick(&self, sampling: fret_core::scene::ImageSamplingHint) -> &wgpu::BindGroup {
        match sampling {
            fret_core::scene::ImageSamplingHint::Nearest => &self.nearest,
            fret_core::scene::ImageSamplingHint::Default
            | fret_core::scene::ImageSamplingHint::Linear => &self.linear,
        }
    }
}

#[derive(Default)]
pub(super) struct BindGroupCaches {
    viewport: RevisionedCache<fret_core::RenderTargetId, wgpu::BindGroup>,
    images: RevisionedCache<fret_core::ImageId, SamplingBindGroups>,
    uniform_mask_images: RevisionedCache<fret_core::ImageId, SamplingBindGroups>,
}

impl BindGroupCaches {
    pub(super) fn invalidate_render_target(&mut self, id: fret_core::RenderTargetId) {
        self.viewport.remove(id);
    }

    pub(super) fn invalidate_image(&mut self, id: fret_core::ImageId) {
        self.images.remove(id);
        self.uniform_mask_images.remove(id);
    }

    pub(super) fn clear_uniform_mask_images(&mut self) {
        self.uniform_mask_images.clear();
    }

    pub(super) fn ensure_viewport_bind_group(
        &mut self,
        id: fret_core::RenderTargetId,
        revision: u64,
        build: impl FnOnce() -> wgpu::BindGroup,
    ) -> &wgpu::BindGroup {
        self.viewport.ensure(id, revision, build)
    }

    pub(super) fn get_viewport_bind_group(
        &self,
        id: fret_core::RenderTargetId,
    ) -> Option<&wgpu::BindGroup> {
        self.viewport.get(id)
    }

    pub(super) fn ensure_image_bind_groups(
        &mut self,
        id: fret_core::ImageId,
        revision: u64,
        build: impl FnOnce() -> SamplingBindGroups,
    ) -> &SamplingBindGroups {
        self.images.ensure(id, revision, build)
    }

    pub(super) fn get_image_bind_groups(
        &self,
        id: fret_core::ImageId,
    ) -> Option<&SamplingBindGroups> {
        self.images.get(id)
    }

    pub(super) fn ensure_uniform_mask_image_bind_groups(
        &mut self,
        id: fret_core::ImageId,
        revision: u64,
        build: impl FnOnce() -> SamplingBindGroups,
    ) -> &SamplingBindGroups {
        self.uniform_mask_images.ensure(id, revision, build)
    }

    pub(super) fn get_uniform_mask_image_bind_groups(
        &self,
        id: fret_core::ImageId,
    ) -> Option<&SamplingBindGroups> {
        self.uniform_mask_images.get(id)
    }
}
