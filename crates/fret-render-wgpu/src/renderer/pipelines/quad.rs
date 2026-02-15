use super::super::shaders::*;
use super::super::*;

impl Renderer {
    pub(in crate::renderer) fn ensure_quad_pipelines(&mut self, format: wgpu::TextureFormat) {
        if self.quad_pipeline_format == Some(format) {
            return;
        }

        self.quad_pipeline_format = Some(format);
        self.quad_pipelines.clear();
    }

    pub(in crate::renderer) fn quad_pipeline(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        key: QuadPipelineKey,
    ) -> &wgpu::RenderPipeline {
        self.ensure_quad_pipelines(format);
        if !self.quad_pipelines.contains_key(&key) {
            let quad_shader_source = quad_shader_source();
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("fret quad shader"),
                source: wgpu::ShaderSource::Wgsl(quad_shader_source.into()),
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("fret quad pipeline layout"),
                bind_group_layouts: &[
                    &self.uniform_bind_group_layout,
                    &self.quad_instance_bind_group_layout,
                ],
                immediate_size: 0,
            });

            let constants = [
                ("FRET_FILL_KIND", f64::from(key.fill_kind)),
                ("FRET_BORDER_KIND", f64::from(key.border_kind)),
                (
                    "FRET_BORDER_PRESENT",
                    if key.border_present { 1.0 } else { 0.0 },
                ),
                (
                    "FRET_DASH_ENABLED",
                    if key.dash_enabled { 1.0 } else { 0.0 },
                ),
            ];
            let compilation_options = wgpu::PipelineCompilationOptions {
                constants: &constants,
                ..wgpu::PipelineCompilationOptions::default()
            };

            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("fret quad pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: compilation_options.clone(),
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
                        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options,
                }),
                multiview_mask: None,
                cache: None,
            });

            self.quad_pipelines.insert(key, pipeline);
        }

        self.quad_pipelines
            .get(&key)
            .expect("quad pipeline must exist")
    }
}
