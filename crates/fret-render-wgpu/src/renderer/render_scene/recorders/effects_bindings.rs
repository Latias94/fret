use super::super::super::*;

#[derive(Clone, Copy)]
pub(super) struct CustomEffectV2BindGroupResources<'a> {
    pub(super) src_view: &'a wgpu::TextureView,
    pub(super) param_buffer: &'a wgpu::Buffer,
    pub(super) input_view: &'a wgpu::TextureView,
    pub(super) input_sampler: &'a wgpu::Sampler,
    pub(super) input_meta_buffer: &'a wgpu::Buffer,
}

pub(super) fn create_custom_effect_v2_bind_group(
    device: &wgpu::Device,
    label: &'static str,
    layout: &wgpu::BindGroupLayout,
    resources: CustomEffectV2BindGroupResources<'_>,
    mask_view: Option<&wgpu::TextureView>,
) -> wgpu::BindGroup {
    match mask_view {
        Some(mask_view) => device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(resources.src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: resources.param_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(resources.input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(resources.input_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: resources.input_meta_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(mask_view),
                },
            ],
        }),
        None => device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(resources.src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: resources.param_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(resources.input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(resources.input_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: resources.input_meta_buffer.as_entire_binding(),
                },
            ],
        }),
    }
}

pub(super) fn create_composite_premul_pipeline_and_bind_group<'a>(
    device: &wgpu::Device,
    renderer: &'a Renderer,
    src_view: &wgpu::TextureView,
    mask_view: Option<&wgpu::TextureView>,
    blend_mode: fret_core::scene::BlendMode,
) -> (&'a wgpu::RenderPipeline, wgpu::BindGroup) {
    match mask_view {
        Some(mask_view) => {
            let layout = renderer.composite_mask_bind_group_layout_ref();
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fret composite premul mask bind group"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(
                            &renderer.globals.viewport_sampler,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(src_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(mask_view),
                    },
                ],
            });
            (renderer.composite_mask_pipeline_ref(blend_mode), bind_group)
        }
        None => {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fret composite premul bind group"),
                layout: &renderer.globals.viewport_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(
                            &renderer.globals.viewport_sampler,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(src_view),
                    },
                ],
            });
            (renderer.composite_pipeline_ref(blend_mode), bind_group)
        }
    }
}
