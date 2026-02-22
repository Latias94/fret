pub(super) struct GpuGlobals {
    pub(super) uniform_bind_group_layout: wgpu::BindGroupLayout,

    pub(super) viewport_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) viewport_sampler: wgpu::Sampler,
    pub(super) image_sampler_nearest: wgpu::Sampler,

    pub(super) material_catalog_view: wgpu::TextureView,
    pub(super) material_catalog_sampler: wgpu::Sampler,

    pub(super) mask_image_sampler: wgpu::Sampler,
    pub(super) mask_image_sampler_nearest: wgpu::Sampler,
    pub(super) mask_image_identity_view: wgpu::TextureView,
}
