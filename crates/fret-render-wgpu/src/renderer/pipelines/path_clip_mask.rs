use super::super::shaders::*;
use super::super::*;

impl Renderer {
    pub(in crate::renderer) fn ensure_path_clip_mask_pipeline(&mut self, device: &wgpu::Device) {
        if self.pipelines.path_clip_mask_pipeline.is_some() {
            return;
        }

        let create_span = tracing::enabled!(tracing::Level::TRACE)
            .then(|| tracing::trace_span!("fret.renderer.pipeline.create.path_clip_mask"))
            .unwrap_or_else(tracing::Span::none);
        let _create_guard = create_span.enter();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret path clip-mask shader"),
            source: wgpu::ShaderSource::Wgsl(PATH_CLIP_MASK_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret path clip-mask pipeline layout"),
            bind_group_layouts: &[&self.globals.uniform_bind_group_layout],
            immediate_size: 0,
        });

        let vertex_stride = std::mem::size_of::<PathVertex>() as wgpu::BufferAddress;
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret path clip-mask pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: vertex_stride,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
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
                    format: wgpu::TextureFormat::R8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        self.pipelines.path_clip_mask_pipeline = Some(pipeline);
    }

    pub(in crate::renderer) fn path_clip_mask_pipeline_ref(&self) -> &wgpu::RenderPipeline {
        self.pipelines
            .path_clip_mask_pipeline
            .as_ref()
            .expect("path clip-mask pipeline must exist")
    }
}
