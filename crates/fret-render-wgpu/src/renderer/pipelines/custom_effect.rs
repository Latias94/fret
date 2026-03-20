use super::super::gpu_pipelines::CustomEffectPipelines;
use super::super::*;

impl Renderer {
    pub(in crate::renderer) fn ensure_custom_effect_pipelines(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        effect: fret_core::EffectId,
    ) {
        if self.pipelines.custom_effect_pipeline_format != Some(format) {
            self.pipelines.custom_effect_pipeline_format = Some(format);
            self.pipelines.custom_effect_pipelines.clear();
        }

        if self.pipelines.custom_effect_pipelines.contains_key(&effect) {
            return;
        }

        let Some(entry) = self.material_effect_state.custom_effects.get(effect) else {
            return;
        };

        if self.pipelines.custom_effect_bind_group_layout.is_none() {
            let bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("fret custom-effect bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: Some(std::num::NonZeroU64::new(64).unwrap()),
                            },
                            count: None,
                        },
                    ],
                });
            self.pipelines.custom_effect_bind_group_layout = Some(bind_group_layout);
        }
        if self
            .pipelines
            .custom_effect_mask_bind_group_layout
            .is_none()
        {
            let mask_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("fret custom-effect mask bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: Some(std::num::NonZeroU64::new(64).unwrap()),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                    ],
                });
            self.pipelines.custom_effect_mask_bind_group_layout = Some(mask_bind_group_layout);
        }

        let bind_group_layout = self.custom_effect_bind_group_layout_ref();
        let mask_bind_group_layout = self.custom_effect_mask_bind_group_layout_ref();

        let unmasked_shader_label = format!("fret custom-effect unmasked shader {effect:?}");
        let unmasked_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(unmasked_shader_label.as_str()),
            source: wgpu::ShaderSource::Wgsl(entry.wgsl_unmasked.as_ref().into()),
        });

        let masked_shader_label = format!("fret custom-effect masked shader {effect:?}");
        let masked_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(masked_shader_label.as_str()),
            source: wgpu::ShaderSource::Wgsl(entry.wgsl_masked.as_ref().into()),
        });

        let mask_shader_label = format!("fret custom-effect mask shader {effect:?}");
        let mask_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(mask_shader_label.as_str()),
            source: wgpu::ShaderSource::Wgsl(entry.wgsl_mask.as_ref().into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret custom-effect pipeline layout"),
            bind_group_layouts: &[
                Some(&self.globals.uniform_bind_group_layout),
                Some(bind_group_layout),
            ],
            immediate_size: 0,
        });
        let masked_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("fret custom-effect masked pipeline layout"),
                bind_group_layouts: &[
                    Some(&self.globals.uniform_bind_group_layout),
                    Some(bind_group_layout),
                ],
                immediate_size: 0,
            });
        let mask_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret custom-effect mask pipeline layout"),
            bind_group_layouts: &[
                Some(&self.globals.uniform_bind_group_layout),
                Some(mask_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let make_pipeline =
            |label: &str, layout: &wgpu::PipelineLayout, shader: &wgpu::ShaderModule| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(layout),
                    vertex: wgpu::VertexState {
                        module: shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    multiview_mask: None,
                    cache: None,
                })
            };

        let masked_blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };

        let make_masked_pipeline =
            |label: &str, layout: &wgpu::PipelineLayout, shader: &wgpu::ShaderModule| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(layout),
                    vertex: wgpu::VertexState {
                        module: shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: Some(masked_blend),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    multiview_mask: None,
                    cache: None,
                })
            };

        let unmasked_label = format!("fret custom-effect unmasked pipeline {effect:?}");
        let masked_label = format!("fret custom-effect masked pipeline {effect:?}");
        let mask_label = format!("fret custom-effect mask pipeline {effect:?}");

        let pipelines = CustomEffectPipelines {
            unmasked: make_pipeline(unmasked_label.as_str(), &pipeline_layout, &unmasked_shader),
            masked: make_masked_pipeline(
                masked_label.as_str(),
                &masked_pipeline_layout,
                &masked_shader,
            ),
            mask: make_masked_pipeline(mask_label.as_str(), &mask_pipeline_layout, &mask_shader),
        };
        self.pipelines
            .custom_effect_pipelines
            .insert(effect, pipelines);
    }

    pub(in crate::renderer) fn ensure_custom_effect_v2_pipelines(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        effect: fret_core::EffectId,
    ) {
        if self.pipelines.custom_effect_v2_pipeline_format != Some(format) {
            self.pipelines.custom_effect_v2_pipeline_format = Some(format);
            self.pipelines.custom_effect_v2_pipelines.clear();
        }

        if self
            .pipelines
            .custom_effect_v2_pipelines
            .contains_key(&effect)
        {
            return;
        }

        let Some(entry) = self.material_effect_state.custom_effects.get(effect) else {
            return;
        };
        if entry.abi != CustomEffectAbi::V2 {
            return;
        }

        if self.pipelines.custom_effect_v2_bind_group_layout.is_none() {
            let bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("fret custom-effect v2 bind group layout"),
                    entries: &[
                        // src_texture (unfilterable, nearest via textureLoad)
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        // params (64B uniform)
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: Some(std::num::NonZeroU64::new(64).unwrap()),
                            },
                            count: None,
                        },
                        // input_texture (filterable, sampled via sampler)
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        // input_sampler (nearest/linear, clamp-to-edge)
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        // input_meta (uv rect), 16 bytes (vec4<f32>)
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: Some(std::num::NonZeroU64::new(16).unwrap()),
                            },
                            count: None,
                        },
                    ],
                });
            self.pipelines.custom_effect_v2_bind_group_layout = Some(bind_group_layout);
        }
        if self
            .pipelines
            .custom_effect_v2_mask_bind_group_layout
            .is_none()
        {
            let mask_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("fret custom-effect v2 mask bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: Some(std::num::NonZeroU64::new(64).unwrap()),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: Some(std::num::NonZeroU64::new(16).unwrap()),
                            },
                            count: None,
                        },
                        // mask_texture for mask pass emission (unfilterable)
                        wgpu::BindGroupLayoutEntry {
                            binding: 5,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                    ],
                });
            self.pipelines.custom_effect_v2_mask_bind_group_layout = Some(mask_bind_group_layout);
        }

        let bind_group_layout = self.custom_effect_v2_bind_group_layout_ref();
        let mask_bind_group_layout = self.custom_effect_v2_mask_bind_group_layout_ref();

        let unmasked_shader_label = format!("fret custom-effect v2 unmasked shader {effect:?}");
        let unmasked_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(unmasked_shader_label.as_str()),
            source: wgpu::ShaderSource::Wgsl(entry.wgsl_unmasked.as_ref().into()),
        });

        let masked_shader_label = format!("fret custom-effect v2 masked shader {effect:?}");
        let masked_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(masked_shader_label.as_str()),
            source: wgpu::ShaderSource::Wgsl(entry.wgsl_masked.as_ref().into()),
        });

        let mask_shader_label = format!("fret custom-effect v2 mask shader {effect:?}");
        let mask_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(mask_shader_label.as_str()),
            source: wgpu::ShaderSource::Wgsl(entry.wgsl_mask.as_ref().into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret custom-effect v2 pipeline layout"),
            bind_group_layouts: &[
                Some(&self.globals.uniform_bind_group_layout),
                Some(bind_group_layout),
            ],
            immediate_size: 0,
        });
        let masked_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("fret custom-effect v2 masked pipeline layout"),
                bind_group_layouts: &[
                    Some(&self.globals.uniform_bind_group_layout),
                    Some(bind_group_layout),
                ],
                immediate_size: 0,
            });
        let mask_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret custom-effect v2 mask pipeline layout"),
            bind_group_layouts: &[
                Some(&self.globals.uniform_bind_group_layout),
                Some(mask_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let make_pipeline =
            |label: &str, layout: &wgpu::PipelineLayout, shader: &wgpu::ShaderModule| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(layout),
                    vertex: wgpu::VertexState {
                        module: shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    multiview_mask: None,
                    cache: None,
                })
            };

        let masked_blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };

        let make_masked_pipeline =
            |label: &str, layout: &wgpu::PipelineLayout, shader: &wgpu::ShaderModule| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(layout),
                    vertex: wgpu::VertexState {
                        module: shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: Some(masked_blend),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    multiview_mask: None,
                    cache: None,
                })
            };

        let unmasked_label = format!("fret custom-effect v2 unmasked pipeline {effect:?}");
        let masked_label = format!("fret custom-effect v2 masked pipeline {effect:?}");
        let mask_label = format!("fret custom-effect v2 mask pipeline {effect:?}");

        let pipelines = CustomEffectPipelines {
            unmasked: make_pipeline(unmasked_label.as_str(), &pipeline_layout, &unmasked_shader),
            masked: make_masked_pipeline(
                masked_label.as_str(),
                &masked_pipeline_layout,
                &masked_shader,
            ),
            mask: make_masked_pipeline(mask_label.as_str(), &mask_pipeline_layout, &mask_shader),
        };
        self.pipelines
            .custom_effect_v2_pipelines
            .insert(effect, pipelines);
    }

    pub(in crate::renderer) fn ensure_custom_effect_v3_pipelines(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        effect: fret_core::EffectId,
    ) {
        if self.pipelines.custom_effect_v3_pipeline_format != Some(format) {
            self.pipelines.custom_effect_v3_pipeline_format = Some(format);
            self.pipelines.custom_effect_v3_pipelines.clear();
        }

        if self
            .pipelines
            .custom_effect_v3_pipelines
            .contains_key(&effect)
        {
            return;
        }

        let Some(entry) = self.material_effect_state.custom_effects.get(effect) else {
            return;
        };
        if entry.abi != CustomEffectAbi::V3 {
            return;
        }

        if self.pipelines.custom_effect_v3_bind_group_layout.is_none() {
            let bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("fret custom-effect v3 bind group layout"),
                    entries: &[
                        // src_texture (unfilterable, nearest via textureLoad)
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        // params (64B uniform)
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: Some(std::num::NonZeroU64::new(64).unwrap()),
                            },
                            count: None,
                        },
                        // src_raw_texture (unfilterable)
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        // src_pyramid_texture (unfilterable; mip levels are sampled via textureLoad)
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        // meta: pyramid levels + user UV rects (48 bytes)
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: Some(std::num::NonZeroU64::new(48).unwrap()),
                            },
                            count: None,
                        },
                        // user0 texture + sampler (filterable)
                        wgpu::BindGroupLayoutEntry {
                            binding: 5,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 6,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        // user1 texture + sampler (filterable)
                        wgpu::BindGroupLayoutEntry {
                            binding: 7,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 8,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });
            self.pipelines.custom_effect_v3_bind_group_layout = Some(bind_group_layout);
        }

        if self
            .pipelines
            .custom_effect_v3_mask_bind_group_layout
            .is_none()
        {
            let mask_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("fret custom-effect v3 mask bind group layout"),
                    entries: &[
                        // src_texture
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        // params
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: Some(std::num::NonZeroU64::new(64).unwrap()),
                            },
                            count: None,
                        },
                        // src_raw_texture
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        // src_pyramid_texture
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        // meta
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: Some(std::num::NonZeroU64::new(48).unwrap()),
                            },
                            count: None,
                        },
                        // user0 texture + sampler
                        wgpu::BindGroupLayoutEntry {
                            binding: 5,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 6,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        // user1 texture + sampler
                        wgpu::BindGroupLayoutEntry {
                            binding: 7,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 8,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        // mask_texture
                        wgpu::BindGroupLayoutEntry {
                            binding: 9,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                    ],
                });
            self.pipelines.custom_effect_v3_mask_bind_group_layout = Some(mask_bind_group_layout);
        }

        let bind_group_layout = self.custom_effect_v3_bind_group_layout_ref();
        let mask_bind_group_layout = self.custom_effect_v3_mask_bind_group_layout_ref();

        let unmasked_shader_label = format!("fret custom-effect v3 unmasked shader {effect:?}");
        let unmasked_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(unmasked_shader_label.as_str()),
            source: wgpu::ShaderSource::Wgsl(entry.wgsl_unmasked.as_ref().into()),
        });
        let masked_shader_label = format!("fret custom-effect v3 masked shader {effect:?}");
        let masked_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(masked_shader_label.as_str()),
            source: wgpu::ShaderSource::Wgsl(entry.wgsl_masked.as_ref().into()),
        });
        let mask_shader_label = format!("fret custom-effect v3 mask shader {effect:?}");
        let mask_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(mask_shader_label.as_str()),
            source: wgpu::ShaderSource::Wgsl(entry.wgsl_mask.as_ref().into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret custom-effect v3 pipeline layout"),
            bind_group_layouts: &[
                Some(&self.globals.uniform_bind_group_layout),
                Some(bind_group_layout),
            ],
            immediate_size: 0,
        });
        let masked_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("fret custom-effect v3 masked pipeline layout"),
                bind_group_layouts: &[
                    Some(&self.globals.uniform_bind_group_layout),
                    Some(bind_group_layout),
                ],
                immediate_size: 0,
            });
        let mask_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret custom-effect v3 mask pipeline layout"),
            bind_group_layouts: &[
                Some(&self.globals.uniform_bind_group_layout),
                Some(mask_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let make_pipeline =
            |label: &str, layout: &wgpu::PipelineLayout, shader: &wgpu::ShaderModule| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(layout),
                    vertex: wgpu::VertexState {
                        module: shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    multiview_mask: None,
                    cache: None,
                })
            };

        let masked_blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };

        let make_masked_pipeline =
            |label: &str, layout: &wgpu::PipelineLayout, shader: &wgpu::ShaderModule| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(layout),
                    vertex: wgpu::VertexState {
                        module: shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: Some(masked_blend),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    multiview_mask: None,
                    cache: None,
                })
            };

        let unmasked_label = format!("fret custom-effect v3 unmasked pipeline {effect:?}");
        let masked_label = format!("fret custom-effect v3 masked pipeline {effect:?}");
        let mask_label = format!("fret custom-effect v3 mask pipeline {effect:?}");

        let pipelines = CustomEffectPipelines {
            unmasked: make_pipeline(unmasked_label.as_str(), &pipeline_layout, &unmasked_shader),
            masked: make_masked_pipeline(
                masked_label.as_str(),
                &masked_pipeline_layout,
                &masked_shader,
            ),
            mask: make_masked_pipeline(mask_label.as_str(), &mask_pipeline_layout, &mask_shader),
        };
        self.pipelines
            .custom_effect_v3_pipelines
            .insert(effect, pipelines);
    }

    pub(in crate::renderer) fn custom_effect_bind_group_layout_ref(
        &self,
    ) -> &wgpu::BindGroupLayout {
        self.pipelines
            .custom_effect_bind_group_layout
            .as_ref()
            .expect("custom-effect bind group layout must exist")
    }

    pub(in crate::renderer) fn custom_effect_mask_bind_group_layout_ref(
        &self,
    ) -> &wgpu::BindGroupLayout {
        self.pipelines
            .custom_effect_mask_bind_group_layout
            .as_ref()
            .expect("custom-effect mask bind group layout must exist")
    }

    pub(in crate::renderer) fn custom_effect_pipeline_ref(
        &self,
        effect: fret_core::EffectId,
    ) -> &wgpu::RenderPipeline {
        &self
            .pipelines
            .custom_effect_pipelines
            .get(&effect)
            .expect("custom-effect pipelines must exist")
            .unmasked
    }

    pub(in crate::renderer) fn custom_effect_masked_pipeline_ref(
        &self,
        effect: fret_core::EffectId,
    ) -> &wgpu::RenderPipeline {
        &self
            .pipelines
            .custom_effect_pipelines
            .get(&effect)
            .expect("custom-effect pipelines must exist")
            .masked
    }

    pub(in crate::renderer) fn custom_effect_mask_pipeline_ref(
        &self,
        effect: fret_core::EffectId,
    ) -> &wgpu::RenderPipeline {
        &self
            .pipelines
            .custom_effect_pipelines
            .get(&effect)
            .expect("custom-effect pipelines must exist")
            .mask
    }

    pub(in crate::renderer) fn custom_effect_v2_pipeline_ref(
        &self,
        effect: fret_core::EffectId,
    ) -> &wgpu::RenderPipeline {
        &self
            .pipelines
            .custom_effect_v2_pipelines
            .get(&effect)
            .expect("custom-effect v2 pipelines must exist")
            .unmasked
    }

    pub(in crate::renderer) fn custom_effect_v2_masked_pipeline_ref(
        &self,
        effect: fret_core::EffectId,
    ) -> &wgpu::RenderPipeline {
        &self
            .pipelines
            .custom_effect_v2_pipelines
            .get(&effect)
            .expect("custom-effect v2 pipelines must exist")
            .masked
    }

    pub(in crate::renderer) fn custom_effect_v2_mask_pipeline_ref(
        &self,
        effect: fret_core::EffectId,
    ) -> &wgpu::RenderPipeline {
        &self
            .pipelines
            .custom_effect_v2_pipelines
            .get(&effect)
            .expect("custom-effect v2 pipelines must exist")
            .mask
    }

    pub(in crate::renderer) fn custom_effect_v2_bind_group_layout_ref(
        &self,
    ) -> &wgpu::BindGroupLayout {
        self.pipelines
            .custom_effect_v2_bind_group_layout
            .as_ref()
            .expect("custom-effect v2 bind group layout must exist")
    }

    pub(in crate::renderer) fn custom_effect_v2_mask_bind_group_layout_ref(
        &self,
    ) -> &wgpu::BindGroupLayout {
        self.pipelines
            .custom_effect_v2_mask_bind_group_layout
            .as_ref()
            .expect("custom-effect v2 mask bind group layout must exist")
    }

    pub(in crate::renderer) fn custom_effect_v3_pipeline_ref(
        &self,
        effect: fret_core::EffectId,
    ) -> &wgpu::RenderPipeline {
        &self
            .pipelines
            .custom_effect_v3_pipelines
            .get(&effect)
            .expect("custom-effect v3 pipelines must exist")
            .unmasked
    }

    pub(in crate::renderer) fn custom_effect_v3_masked_pipeline_ref(
        &self,
        effect: fret_core::EffectId,
    ) -> &wgpu::RenderPipeline {
        &self
            .pipelines
            .custom_effect_v3_pipelines
            .get(&effect)
            .expect("custom-effect v3 pipelines must exist")
            .masked
    }

    pub(in crate::renderer) fn custom_effect_v3_mask_pipeline_ref(
        &self,
        effect: fret_core::EffectId,
    ) -> &wgpu::RenderPipeline {
        &self
            .pipelines
            .custom_effect_v3_pipelines
            .get(&effect)
            .expect("custom-effect v3 pipelines must exist")
            .mask
    }

    pub(in crate::renderer) fn custom_effect_v3_bind_group_layout_ref(
        &self,
    ) -> &wgpu::BindGroupLayout {
        self.pipelines
            .custom_effect_v3_bind_group_layout
            .as_ref()
            .expect("custom-effect v3 bind group layout must exist")
    }

    pub(in crate::renderer) fn custom_effect_v3_mask_bind_group_layout_ref(
        &self,
    ) -> &wgpu::BindGroupLayout {
        self.pipelines
            .custom_effect_v3_mask_bind_group_layout
            .as_ref()
            .expect("custom-effect v3 mask bind group layout must exist")
    }
}
