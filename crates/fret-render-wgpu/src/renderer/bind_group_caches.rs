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
    pub(super) fn ensure_viewport_sampler_texture_bind_group(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        view: &wgpu::TextureView,
        id: fret_core::RenderTargetId,
        revision: u64,
    ) -> &wgpu::BindGroup {
        self.viewport.ensure(id, revision, || {
            super::bind_group_builders::create_sampler_texture_bind_group(
                device,
                layout,
                sampler,
                view,
                "fret viewport texture bind group",
            )
        })
    }

    pub(super) fn ensure_image_sampler_texture_bind_groups(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        linear_sampler: &wgpu::Sampler,
        nearest_sampler: &wgpu::Sampler,
        view: &wgpu::TextureView,
        id: fret_core::ImageId,
        revision: u64,
    ) -> &SamplingBindGroups {
        self.images.ensure(id, revision, || {
            let linear = super::bind_group_builders::create_sampler_texture_bind_group(
                device,
                layout,
                linear_sampler,
                view,
                "fret image texture bind group (linear)",
            );
            let nearest = super::bind_group_builders::create_sampler_texture_bind_group(
                device,
                layout,
                nearest_sampler,
                view,
                "fret image texture bind group (nearest)",
            );
            SamplingBindGroups { linear, nearest }
        })
    }

    pub(super) fn ensure_uniform_mask_image_override_bind_groups(
        &mut self,
        device: &wgpu::Device,
        globals: &super::bind_group_builders::UniformMaskImageBindGroupGlobals<'_>,
        linear_sampler: &wgpu::Sampler,
        nearest_sampler: &wgpu::Sampler,
        view: &wgpu::TextureView,
        id: fret_core::ImageId,
        revision: u64,
    ) -> &SamplingBindGroups {
        self.uniform_mask_images.ensure(id, revision, || {
            let linear = globals.create(
                device,
                "fret uniforms bind group (mask image override, linear)",
                linear_sampler,
                view,
            );
            let nearest = globals.create(
                device,
                "fret uniforms bind group (mask image override, nearest)",
                nearest_sampler,
                view,
            );
            SamplingBindGroups { linear, nearest }
        })
    }

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

    pub(super) fn get_viewport_bind_group(
        &self,
        id: fret_core::RenderTargetId,
    ) -> Option<&wgpu::BindGroup> {
        self.viewport.get(id)
    }

    pub(super) fn get_image_bind_groups(
        &self,
        id: fret_core::ImageId,
    ) -> Option<&SamplingBindGroups> {
        self.images.get(id)
    }

    pub(super) fn get_uniform_mask_image_bind_groups(
        &self,
        id: fret_core::ImageId,
    ) -> Option<&SamplingBindGroups> {
        self.uniform_mask_images.get(id)
    }
}
