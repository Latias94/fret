use super::super::shaders::*;
use super::super::*;

impl Renderer {
    pub(in crate::renderer) fn ensure_mip_downsample_box_pipeline(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) {
        if self.pipelines.mip_downsample_box_pipeline_format == Some(format)
            && self.pipelines.mip_downsample_box_pipeline.is_some()
        {
            return;
        }

        let create_span = tracing::enabled!(tracing::Level::TRACE)
            .then(|| {
                let reason = if self.pipelines.mip_downsample_box_pipeline.is_none() {
                    "missing"
                } else {
                    "format_changed"
                };
                tracing::trace_span!(
                    "fret.renderer.pipeline.create.mip_downsample_box",
                    format = ?format,
                    reason
                )
            })
            .unwrap_or_else(tracing::Span::none);
        let _create_guard = create_span.enter();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fret mip downsample box bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                },
                count: None,
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret mip downsample box shader"),
            source: wgpu::ShaderSource::Wgsl(MIP_DOWNSAMPLE_BOX_2X2_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret mip downsample box pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret mip downsample box pipeline"),
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

        self.pipelines.mip_downsample_box_bind_group_layout = Some(bind_group_layout);
        self.pipelines.mip_downsample_box_pipeline_format = Some(format);
        self.pipelines.mip_downsample_box_pipeline = Some(pipeline);
    }

    pub(in crate::renderer) fn mip_downsample_box_pipeline_ref(&self) -> &wgpu::RenderPipeline {
        self.pipelines
            .mip_downsample_box_pipeline
            .as_ref()
            .expect("mip downsample pipeline must exist")
    }

    pub(in crate::renderer) fn mip_downsample_box_bind_group_layout_ref(
        &self,
    ) -> &wgpu::BindGroupLayout {
        self.pipelines
            .mip_downsample_box_bind_group_layout
            .as_ref()
            .expect("mip downsample bind group layout must exist")
    }
}
