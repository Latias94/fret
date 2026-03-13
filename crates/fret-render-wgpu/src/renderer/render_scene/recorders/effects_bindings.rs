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

#[derive(Clone, Copy)]
pub(super) struct CustomEffectV3BindGroupResources<'a> {
    pub(super) src_view: &'a wgpu::TextureView,
    pub(super) src_raw_view: &'a wgpu::TextureView,
    pub(super) src_pyramid_view: &'a wgpu::TextureView,
    pub(super) param_buffer: &'a wgpu::Buffer,
    pub(super) meta_buffer: &'a wgpu::Buffer,
    pub(super) user0_view: &'a wgpu::TextureView,
    pub(super) user0_sampler: &'a wgpu::Sampler,
    pub(super) user1_view: &'a wgpu::TextureView,
    pub(super) user1_sampler: &'a wgpu::Sampler,
}

pub(super) enum CustomEffectV3BindGroupMode<'a> {
    Unmasked,
    UniformMask,
    TextureMask(&'a wgpu::TextureView),
}

pub(super) fn create_custom_effect_v3_pipeline_and_bind_group<'a>(
    device: &wgpu::Device,
    renderer: &'a Renderer,
    effect: fret_core::EffectId,
    resources: CustomEffectV3BindGroupResources<'_>,
    mode: CustomEffectV3BindGroupMode<'_>,
) -> (&'a wgpu::RenderPipeline, wgpu::BindGroup) {
    match mode {
        CustomEffectV3BindGroupMode::TextureMask(mask_view) => {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fret custom-effect v3 mask bind group"),
                layout: renderer.custom_effect_v3_mask_bind_group_layout_ref(),
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
                        resource: wgpu::BindingResource::TextureView(resources.src_raw_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(resources.src_pyramid_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: resources.meta_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(resources.user0_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::Sampler(resources.user0_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::TextureView(resources.user1_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::Sampler(resources.user1_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: wgpu::BindingResource::TextureView(mask_view),
                    },
                ],
            });
            (
                renderer.custom_effect_v3_mask_pipeline_ref(effect),
                bind_group,
            )
        }
        CustomEffectV3BindGroupMode::UniformMask => {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fret custom-effect v3 bind group"),
                layout: renderer.custom_effect_v3_bind_group_layout_ref(),
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
                        resource: wgpu::BindingResource::TextureView(resources.src_raw_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(resources.src_pyramid_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: resources.meta_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(resources.user0_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::Sampler(resources.user0_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::TextureView(resources.user1_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::Sampler(resources.user1_sampler),
                    },
                ],
            });
            (
                renderer.custom_effect_v3_masked_pipeline_ref(effect),
                bind_group,
            )
        }
        CustomEffectV3BindGroupMode::Unmasked => {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fret custom-effect v3 bind group"),
                layout: renderer.custom_effect_v3_bind_group_layout_ref(),
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
                        resource: wgpu::BindingResource::TextureView(resources.src_raw_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(resources.src_pyramid_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: resources.meta_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(resources.user0_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::Sampler(resources.user0_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::TextureView(resources.user1_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::Sampler(resources.user1_sampler),
                    },
                ],
            });
            (renderer.custom_effect_v3_pipeline_ref(effect), bind_group)
        }
    }
}
