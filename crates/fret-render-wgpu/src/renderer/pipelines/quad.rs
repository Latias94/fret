use super::super::shaders::*;
use super::super::*;

impl Renderer {
    pub(in crate::renderer) fn ensure_quad_pipelines(&mut self, format: wgpu::TextureFormat) {
        self.pipelines.ensure_quad_pipelines(format);
    }

    pub(in crate::renderer) fn quad_pipeline(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        key: QuadPipelineKey,
    ) -> &wgpu::RenderPipeline {
        self.ensure_quad_pipelines(format);
        if self.pipelines.quad_pipeline_ref(&key).is_none() {
            let create_span = tracing::enabled!(tracing::Level::TRACE)
                .then(|| {
                    tracing::trace_span!(
                        "fret.renderer.pipeline.create.quad",
                        format = ?format,
                        fill_kind = key.fill_kind,
                        border_kind = key.border_kind,
                        border_present = key.border_present,
                        dash_enabled = key.dash_enabled,
                        fill_material_sampled = key.fill_material_sampled,
                        border_material_sampled = key.border_material_sampled,
                    )
                })
                .unwrap_or_else(tracing::Span::none);
            let _create_guard = create_span.enter();

            let quad_shader_source = quad_shader_source();
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("fret quad shader"),
                source: wgpu::ShaderSource::Wgsl(quad_shader_source.into()),
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("fret quad pipeline layout"),
                bind_group_layouts: &[
                    Some(&self.globals.uniform_bind_group_layout),
                    Some(self.geometry_upload_state.quad_instances_layout()),
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
                (
                    "FRET_FILL_MATERIAL_SAMPLED",
                    if key.fill_material_sampled { 1.0 } else { 0.0 },
                ),
                (
                    "FRET_BORDER_MATERIAL_SAMPLED",
                    if key.border_material_sampled {
                        1.0
                    } else {
                        0.0
                    },
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

            return self.pipelines.quad_pipeline_inserted(key, pipeline);
        }

        self.pipelines
            .quad_pipeline_ref(&key)
            .expect("quad pipeline must exist")
    }

    pub(in crate::renderer) fn quad_pipeline_ref(
        &self,
        key: &QuadPipelineKey,
    ) -> &wgpu::RenderPipeline {
        self.pipelines
            .quad_pipeline_ref(key)
            .expect("quad pipeline must exist")
    }
}
