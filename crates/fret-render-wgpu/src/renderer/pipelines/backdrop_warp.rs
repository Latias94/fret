use super::super::shaders::*;
use super::super::*;

impl Renderer {
    pub(in crate::renderer) fn ensure_backdrop_warp_pipeline(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) {
        if self.pipelines.backdrop_warp_pipeline_format == Some(format)
            && self.pipelines.backdrop_warp_pipeline.is_some()
            && self.pipelines.backdrop_warp_masked_pipeline.is_some()
            && self.pipelines.backdrop_warp_bind_group_layout.is_some()
            && self.pipelines.backdrop_warp_mask_pipeline.is_some()
            && self
                .pipelines
                .backdrop_warp_mask_bind_group_layout
                .is_some()
            && self.pipelines.backdrop_warp_image_pipeline.is_some()
            && self.pipelines.backdrop_warp_image_masked_pipeline.is_some()
            && self
                .pipelines
                .backdrop_warp_image_bind_group_layout
                .is_some()
            && self.pipelines.backdrop_warp_image_mask_pipeline.is_some()
            && self
                .pipelines
                .backdrop_warp_image_mask_bind_group_layout
                .is_some()
        {
            return;
        }

        let create_span = tracing::enabled!(tracing::Level::TRACE)
            .then(|| {
                let reason = if self.pipelines.backdrop_warp_pipeline_format != Some(format) {
                    "format_changed"
                } else {
                    "missing"
                };
                tracing::trace_span!(
                    "fret.renderer.pipeline.create.backdrop_warp",
                    format = ?format,
                    reason
                )
            })
            .unwrap_or_else(tracing::Span::none);
        let _create_guard = create_span.enter();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fret backdrop-warp bind group layout"),
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
        let mask_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret backdrop-warp mask bind group layout"),
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

        let image_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret backdrop-warp image bind group layout"),
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
        let image_mask_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret backdrop-warp image mask bind group layout"),
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
                ],
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret backdrop-warp shader"),
            source: wgpu::ShaderSource::Wgsl(BACKDROP_WARP_SHADER.into()),
        });
        let image_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret backdrop-warp image shader"),
            source: wgpu::ShaderSource::Wgsl(BACKDROP_WARP_IMAGE_SHADER.into()),
        });
        let masked_src = backdrop_warp_masked_shader_source();
        let masked_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret backdrop-warp masked shader"),
            source: wgpu::ShaderSource::Wgsl(masked_src.into()),
        });
        let image_masked_src = backdrop_warp_image_masked_shader_source();
        let image_masked_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret backdrop-warp image masked shader"),
            source: wgpu::ShaderSource::Wgsl(image_masked_src.into()),
        });
        let mask_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret backdrop-warp mask shader"),
            source: wgpu::ShaderSource::Wgsl(BACKDROP_WARP_MASK_SHADER.into()),
        });
        let image_mask_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret backdrop-warp image mask shader"),
            source: wgpu::ShaderSource::Wgsl(BACKDROP_WARP_IMAGE_MASK_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret backdrop-warp pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let masked_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("fret backdrop-warp masked pipeline layout"),
                bind_group_layouts: &[
                    Some(&self.globals.uniform_bind_group_layout),
                    Some(&bind_group_layout),
                ],
                immediate_size: 0,
            });
        let mask_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret backdrop-warp mask pipeline layout"),
            bind_group_layouts: &[
                Some(&self.globals.uniform_bind_group_layout),
                Some(&mask_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let image_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("fret backdrop-warp image pipeline layout"),
                bind_group_layouts: &[Some(&image_bind_group_layout)],
                immediate_size: 0,
            });
        let image_masked_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("fret backdrop-warp image masked pipeline layout"),
                bind_group_layouts: &[
                    Some(&self.globals.uniform_bind_group_layout),
                    Some(&image_bind_group_layout),
                ],
                immediate_size: 0,
            });
        let image_mask_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("fret backdrop-warp image mask pipeline layout"),
                bind_group_layouts: &[
                    Some(&self.globals.uniform_bind_group_layout),
                    Some(&image_mask_bind_group_layout),
                ],
                immediate_size: 0,
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret backdrop-warp pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
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
                module: &shader,
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
        });

        let image_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret backdrop-warp image pipeline"),
            layout: Some(&image_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &image_shader,
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
                module: &image_shader,
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
        });

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
        let masked_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret backdrop-warp masked pipeline"),
            layout: Some(&masked_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &masked_shader,
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
                module: &masked_shader,
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
        });

        let image_masked_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("fret backdrop-warp image masked pipeline"),
                layout: Some(&image_masked_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &image_masked_shader,
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
                    module: &image_masked_shader,
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
            });

        let mask_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret backdrop-warp mask pipeline"),
            layout: Some(&mask_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &mask_shader,
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
                module: &mask_shader,
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
        });

        let image_mask_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret backdrop-warp image mask pipeline"),
            layout: Some(&image_mask_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &image_mask_shader,
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
                module: &image_mask_shader,
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
        });

        self.pipelines.backdrop_warp_pipeline_format = Some(format);
        self.pipelines.backdrop_warp_bind_group_layout = Some(bind_group_layout);
        self.pipelines.backdrop_warp_mask_bind_group_layout = Some(mask_bind_group_layout);
        self.pipelines.backdrop_warp_pipeline = Some(pipeline);
        self.pipelines.backdrop_warp_masked_pipeline = Some(masked_pipeline);
        self.pipelines.backdrop_warp_mask_pipeline = Some(mask_pipeline);

        self.pipelines.backdrop_warp_image_bind_group_layout = Some(image_bind_group_layout);
        self.pipelines.backdrop_warp_image_mask_bind_group_layout =
            Some(image_mask_bind_group_layout);
        self.pipelines.backdrop_warp_image_pipeline = Some(image_pipeline);
        self.pipelines.backdrop_warp_image_masked_pipeline = Some(image_masked_pipeline);
        self.pipelines.backdrop_warp_image_mask_pipeline = Some(image_mask_pipeline);
    }

    pub(in crate::renderer) fn backdrop_warp_bind_group_layout_ref(
        &self,
    ) -> &wgpu::BindGroupLayout {
        self.pipelines
            .backdrop_warp_bind_group_layout
            .as_ref()
            .expect("backdrop-warp bind group layout must exist")
    }

    pub(in crate::renderer) fn backdrop_warp_mask_bind_group_layout_ref(
        &self,
    ) -> &wgpu::BindGroupLayout {
        self.pipelines
            .backdrop_warp_mask_bind_group_layout
            .as_ref()
            .expect("backdrop-warp mask bind group layout must exist")
    }

    pub(in crate::renderer) fn backdrop_warp_image_bind_group_layout_ref(
        &self,
    ) -> &wgpu::BindGroupLayout {
        self.pipelines
            .backdrop_warp_image_bind_group_layout
            .as_ref()
            .expect("backdrop-warp image bind group layout must exist")
    }

    pub(in crate::renderer) fn backdrop_warp_image_mask_bind_group_layout_ref(
        &self,
    ) -> &wgpu::BindGroupLayout {
        self.pipelines
            .backdrop_warp_image_mask_bind_group_layout
            .as_ref()
            .expect("backdrop-warp image mask bind group layout must exist")
    }

    pub(in crate::renderer) fn backdrop_warp_pipeline_ref(&self) -> &wgpu::RenderPipeline {
        self.pipelines
            .backdrop_warp_pipeline
            .as_ref()
            .expect("backdrop-warp pipeline must exist")
    }

    pub(in crate::renderer) fn backdrop_warp_masked_pipeline_ref(&self) -> &wgpu::RenderPipeline {
        self.pipelines
            .backdrop_warp_masked_pipeline
            .as_ref()
            .expect("backdrop-warp masked pipeline must exist")
    }

    pub(in crate::renderer) fn backdrop_warp_mask_pipeline_ref(&self) -> &wgpu::RenderPipeline {
        self.pipelines
            .backdrop_warp_mask_pipeline
            .as_ref()
            .expect("backdrop-warp mask pipeline must exist")
    }

    pub(in crate::renderer) fn backdrop_warp_image_pipeline_ref(&self) -> &wgpu::RenderPipeline {
        self.pipelines
            .backdrop_warp_image_pipeline
            .as_ref()
            .expect("backdrop-warp image pipeline must exist")
    }

    pub(in crate::renderer) fn backdrop_warp_image_masked_pipeline_ref(
        &self,
    ) -> &wgpu::RenderPipeline {
        self.pipelines
            .backdrop_warp_image_masked_pipeline
            .as_ref()
            .expect("backdrop-warp image masked pipeline must exist")
    }

    pub(in crate::renderer) fn backdrop_warp_image_mask_pipeline_ref(
        &self,
    ) -> &wgpu::RenderPipeline {
        self.pipelines
            .backdrop_warp_image_mask_pipeline
            .as_ref()
            .expect("backdrop-warp image mask pipeline must exist")
    }
}
