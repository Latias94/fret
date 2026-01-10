use super::super::shaders::*;
use super::super::*;

impl Renderer {
    pub(in crate::renderer) fn ensure_blur_pipelines(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) {
        if self.blur_pipeline_format == Some(format)
            && self.blur_h_pipeline.is_some()
            && self.blur_v_pipeline.is_some()
            && self.blur_h_masked_pipeline.is_some()
            && self.blur_v_masked_pipeline.is_some()
            && self.blur_h_mask_pipeline.is_some()
            && self.blur_v_mask_pipeline.is_some()
            && self.blit_mask_bind_group_layout.is_some()
        {
            return;
        }

        let layout = self
            .blit_bind_group_layout
            .as_ref()
            .expect("blit bind group layout must exist");
        let mask_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fret blit mask bind group layout"),
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
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
            ],
        });

        let blur_h_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret blur-h shader"),
            source: wgpu::ShaderSource::Wgsl(BLUR_H_SHADER.into()),
        });
        let blur_v_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret blur-v shader"),
            source: wgpu::ShaderSource::Wgsl(BLUR_V_SHADER.into()),
        });
        let blur_h_masked_src = blur_h_masked_shader_source();
        let blur_h_masked_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret blur-h masked shader"),
            source: wgpu::ShaderSource::Wgsl(blur_h_masked_src.into()),
        });
        let blur_v_masked_src = blur_v_masked_shader_source();
        let blur_v_masked_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret blur-v masked shader"),
            source: wgpu::ShaderSource::Wgsl(blur_v_masked_src.into()),
        });
        let blur_h_mask_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret blur-h mask shader"),
            source: wgpu::ShaderSource::Wgsl(BLUR_H_MASK_SHADER.into()),
        });
        let blur_v_mask_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret blur-v mask shader"),
            source: wgpu::ShaderSource::Wgsl(BLUR_V_MASK_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret blur pipeline layout"),
            bind_group_layouts: &[layout],
            immediate_size: 0,
        });
        let masked_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("fret blur masked pipeline layout"),
                bind_group_layouts: &[&self.uniform_bind_group_layout, layout],
                immediate_size: 0,
            });
        let mask_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret blur mask pipeline layout"),
            bind_group_layouts: &[&self.uniform_bind_group_layout, &mask_layout],
            immediate_size: 0,
        });

        let make_pipeline =
            |label: &'static str, shader: &wgpu::ShaderModule| -> wgpu::RenderPipeline {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(&pipeline_layout),
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
            |label: &'static str, shader: &wgpu::ShaderModule| -> wgpu::RenderPipeline {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(&masked_pipeline_layout),
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

        let blur_h_pipeline = make_pipeline("fret blur-h pipeline", &blur_h_shader);
        let blur_v_pipeline = make_pipeline("fret blur-v pipeline", &blur_v_shader);
        let blur_h_masked_pipeline =
            make_masked_pipeline("fret blur-h masked pipeline", &blur_h_masked_shader);
        let blur_v_masked_pipeline =
            make_masked_pipeline("fret blur-v masked pipeline", &blur_v_masked_shader);
        let blur_h_mask_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret blur-h mask pipeline"),
            layout: Some(&mask_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blur_h_mask_shader,
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
                module: &blur_h_mask_shader,
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
        let blur_v_mask_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret blur-v mask pipeline"),
            layout: Some(&mask_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blur_v_mask_shader,
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
                module: &blur_v_mask_shader,
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

        self.blur_pipeline_format = Some(format);
        self.blur_h_pipeline = Some(blur_h_pipeline);
        self.blur_v_pipeline = Some(blur_v_pipeline);
        self.blur_h_masked_pipeline = Some(blur_h_masked_pipeline);
        self.blur_v_masked_pipeline = Some(blur_v_masked_pipeline);
        self.blur_h_mask_pipeline = Some(blur_h_mask_pipeline);
        self.blur_v_mask_pipeline = Some(blur_v_mask_pipeline);
        self.blit_mask_bind_group_layout = Some(mask_layout);
    }
}
