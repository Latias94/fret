use super::super::shaders::*;
use super::super::*;

impl Renderer {
    pub(in crate::renderer) fn ensure_color_adjust_pipeline(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) {
        if self.pipelines.color_adjust_pipeline_format == Some(format)
            && self.pipelines.color_adjust_pipeline.is_some()
            && self.pipelines.color_adjust_masked_pipeline.is_some()
            && self.pipelines.color_adjust_bind_group_layout.is_some()
            && self.pipelines.color_adjust_mask_pipeline.is_some()
            && self.pipelines.color_adjust_mask_bind_group_layout.is_some()
        {
            return;
        }

        let create_span = tracing::enabled!(tracing::Level::TRACE)
            .then(|| {
                let reason = if self.pipelines.color_adjust_pipeline_format != Some(format) {
                    "format_changed"
                } else {
                    "missing"
                };
                tracing::trace_span!(
                    "fret.renderer.pipeline.create.color_adjust",
                    format = ?format,
                    reason
                )
            })
            .unwrap_or_else(tracing::Span::none);
        let _create_guard = create_span.enter();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fret color-adjust bind group layout"),
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
                        min_binding_size: Some(std::num::NonZeroU64::new(16).unwrap()),
                    },
                    count: None,
                },
            ],
        });
        let mask_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret color-adjust mask bind group layout"),
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
                            min_binding_size: Some(std::num::NonZeroU64::new(16).unwrap()),
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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret color-adjust shader"),
            source: wgpu::ShaderSource::Wgsl(COLOR_ADJUST_SHADER.into()),
        });
        let masked_src = color_adjust_masked_shader_source();
        let masked_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret color-adjust masked shader"),
            source: wgpu::ShaderSource::Wgsl(masked_src.into()),
        });
        let mask_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret color-adjust mask shader"),
            source: wgpu::ShaderSource::Wgsl(COLOR_ADJUST_MASK_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret color-adjust pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let masked_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("fret color-adjust masked pipeline layout"),
                bind_group_layouts: &[
                    Some(&self.globals.uniform_bind_group_layout),
                    Some(&bind_group_layout),
                ],
                immediate_size: 0,
            });
        let mask_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret color-adjust mask pipeline layout"),
            bind_group_layouts: &[
                Some(&self.globals.uniform_bind_group_layout),
                Some(&mask_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret color-adjust pipeline"),
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
            label: Some("fret color-adjust masked pipeline"),
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
        let mask_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret color-adjust mask pipeline"),
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

        self.pipelines.color_adjust_pipeline_format = Some(format);
        self.pipelines.color_adjust_bind_group_layout = Some(bind_group_layout);
        self.pipelines.color_adjust_mask_bind_group_layout = Some(mask_bind_group_layout);
        self.pipelines.color_adjust_pipeline = Some(pipeline);
        self.pipelines.color_adjust_masked_pipeline = Some(masked_pipeline);
        self.pipelines.color_adjust_mask_pipeline = Some(mask_pipeline);
    }

    pub(in crate::renderer) fn color_adjust_bind_group_layout_ref(&self) -> &wgpu::BindGroupLayout {
        self.pipelines
            .color_adjust_bind_group_layout
            .as_ref()
            .expect("color-adjust bind group layout must exist")
    }

    pub(in crate::renderer) fn color_adjust_mask_bind_group_layout_ref(
        &self,
    ) -> &wgpu::BindGroupLayout {
        self.pipelines
            .color_adjust_mask_bind_group_layout
            .as_ref()
            .expect("color-adjust mask bind group layout must exist")
    }

    pub(in crate::renderer) fn color_adjust_pipeline_ref(&self) -> &wgpu::RenderPipeline {
        self.pipelines
            .color_adjust_pipeline
            .as_ref()
            .expect("color-adjust pipeline must exist")
    }

    pub(in crate::renderer) fn color_adjust_masked_pipeline_ref(&self) -> &wgpu::RenderPipeline {
        self.pipelines
            .color_adjust_masked_pipeline
            .as_ref()
            .expect("color-adjust masked pipeline must exist")
    }

    pub(in crate::renderer) fn color_adjust_mask_pipeline_ref(&self) -> &wgpu::RenderPipeline {
        self.pipelines
            .color_adjust_mask_pipeline
            .as_ref()
            .expect("color-adjust mask pipeline must exist")
    }
}
