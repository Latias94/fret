use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Overlay3dVertex {
    pub pos: [f32; 3],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Overlay3dLineVertex {
    pub a: [f32; 3],
    pub b: [f32; 3],
    pub t: f32,
    pub side: f32,
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Overlay3dUniforms {
    pub view_proj: [[f32; 4]; 4],
    /// x = viewport_w_px, y = viewport_h_px, z = line_thickness_px, w = unused
    pub viewport_and_thickness: [f32; 4],
}

#[derive(Debug, Clone)]
pub struct Overlay3dPipelines {
    pub uniform: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub tri_pipeline: wgpu::RenderPipeline,
    pub solid_depth_pipeline: wgpu::RenderPipeline,
    pub solid_always_pipeline: wgpu::RenderPipeline,
    pub thick_line_depth_pipeline: wgpu::RenderPipeline,
    pub thick_line_always_pipeline: wgpu::RenderPipeline,
}

#[derive(Debug, Default, Clone)]
pub struct Overlay3dBatch {
    solid_test: Overlay3dBatchSlot<Overlay3dVertex>,
    solid_ghost: Overlay3dBatchSlot<Overlay3dVertex>,
    solid_always: Overlay3dBatchSlot<Overlay3dVertex>,
    line_test: Overlay3dBatchSlot<Overlay3dLineVertex>,
    line_ghost: Overlay3dBatchSlot<Overlay3dLineVertex>,
    line_always: Overlay3dBatchSlot<Overlay3dLineVertex>,
}

impl Overlay3dBatch {
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        solid_test: &[Overlay3dVertex],
        solid_ghost: &[Overlay3dVertex],
        solid_always: &[Overlay3dVertex],
        line_test: &[Overlay3dLineVertex],
        line_ghost: &[Overlay3dLineVertex],
        line_always: &[Overlay3dLineVertex],
    ) {
        self.solid_test
            .upload(device, queue, "fret overlay3d solid vb (test)", solid_test);
        self.solid_ghost.upload(
            device,
            queue,
            "fret overlay3d solid vb (ghost)",
            solid_ghost,
        );
        self.solid_always.upload(
            device,
            queue,
            "fret overlay3d solid vb (always)",
            solid_always,
        );
        self.line_test.upload(
            device,
            queue,
            "fret overlay3d thick line vb (test)",
            line_test,
        );
        self.line_ghost.upload(
            device,
            queue,
            "fret overlay3d thick line vb (ghost)",
            line_ghost,
        );
        self.line_always.upload(
            device,
            queue,
            "fret overlay3d thick line vb (always)",
            line_always,
        );
    }

    pub fn record<'rp>(&self, pipelines: &Overlay3dPipelines, pass: &mut wgpu::RenderPass<'rp>) {
        pass.set_bind_group(0, &pipelines.bind_group, &[]);

        if self.solid_ghost.vertex_count > 0 {
            pass.set_pipeline(&pipelines.solid_always_pipeline);
            pass.set_vertex_buffer(0, self.solid_ghost.buffer_slice());
            pass.draw(0..self.solid_ghost.vertex_count, 0..1);
        }
        if self.line_ghost.vertex_count > 0 {
            pass.set_pipeline(&pipelines.thick_line_always_pipeline);
            pass.set_vertex_buffer(0, self.line_ghost.buffer_slice());
            pass.draw(0..self.line_ghost.vertex_count, 0..1);
        }

        if self.solid_test.vertex_count > 0 {
            pass.set_pipeline(&pipelines.solid_depth_pipeline);
            pass.set_vertex_buffer(0, self.solid_test.buffer_slice());
            pass.draw(0..self.solid_test.vertex_count, 0..1);
        }
        if self.line_test.vertex_count > 0 {
            pass.set_pipeline(&pipelines.thick_line_depth_pipeline);
            pass.set_vertex_buffer(0, self.line_test.buffer_slice());
            pass.draw(0..self.line_test.vertex_count, 0..1);
        }

        if self.solid_always.vertex_count > 0 {
            pass.set_pipeline(&pipelines.solid_always_pipeline);
            pass.set_vertex_buffer(0, self.solid_always.buffer_slice());
            pass.draw(0..self.solid_always.vertex_count, 0..1);
        }
        if self.line_always.vertex_count > 0 {
            pass.set_pipeline(&pipelines.thick_line_always_pipeline);
            pass.set_vertex_buffer(0, self.line_always.buffer_slice());
            pass.draw(0..self.line_always.vertex_count, 0..1);
        }
    }
}

#[derive(Debug, Clone)]
struct Overlay3dBatchSlot<T> {
    buffer: Option<wgpu::Buffer>,
    capacity_bytes: u64,
    vertex_count: u32,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Default for Overlay3dBatchSlot<T> {
    fn default() -> Self {
        Self {
            buffer: None,
            capacity_bytes: 0,
            vertex_count: 0,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Pod> Overlay3dBatchSlot<T> {
    fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, label: &str, data: &[T]) {
        self.vertex_count = data.len().min(u32::MAX as usize) as u32;
        if self.vertex_count == 0 {
            return;
        }

        let bytes = bytemuck::cast_slice(data);
        let needed = bytes.len() as u64;
        self.ensure_capacity(device, label, needed);

        if let Some(buffer) = self.buffer.as_ref() {
            queue.write_buffer(buffer, 0, bytes);
        }
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device, label: &str, needed: u64) {
        if self.buffer.is_some() && self.capacity_bytes >= needed {
            return;
        }

        let min_capacity = 256u64;
        let mut cap = needed.max(min_capacity);
        if let Some(next) = cap.checked_next_power_of_two() {
            cap = next;
        }

        self.capacity_bytes = cap;
        self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: cap,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
    }

    fn buffer_slice(&self) -> wgpu::BufferSlice<'_> {
        self.buffer
            .as_ref()
            .expect("overlay3d batch buffer is missing")
            .slice(..)
    }
}

impl Overlay3dPipelines {
    pub fn new(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret viewport overlay3d shader"),
            source: wgpu::ShaderSource::Wgsl(OVERLAY3D_WGSL.into()),
        });

        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret viewport overlay3d uniform"),
            size: std::mem::size_of::<Overlay3dUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fret viewport overlay3d bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret viewport overlay3d bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret viewport overlay3d pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let vertex_layouts = [wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Overlay3dVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    shader_location: 0,
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    shader_location: 1,
                    offset: 12,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }];

        let line_vertex_layouts = [wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Overlay3dLineVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    shader_location: 0,
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    shader_location: 1,
                    offset: 12,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    shader_location: 2,
                    offset: 24,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    shader_location: 3,
                    offset: 28,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    shader_location: 4,
                    offset: 32,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }];

        let tri_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret viewport overlay3d tri pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main_tri"),
                buffers: &vertex_layouts,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let depth_state = wgpu::DepthStencilState {
            format: depth_format,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

        let solid_depth_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret viewport overlay3d solid depth pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main_tri"),
                buffers: &vertex_layouts,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                depth_write_enabled: false,
                bias: wgpu::DepthBiasState {
                    constant: -2,
                    slope_scale: -1.0,
                    clamp: 0.0,
                },
                ..depth_state.clone()
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let solid_always_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("fret viewport overlay3d solid always pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main_tri"),
                    buffers: &vertex_layouts,
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: color_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: depth_format,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        let thick_line_depth_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("fret viewport overlay3d thick line depth pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main_thick_line"),
                    buffers: &line_vertex_layouts,
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: color_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    depth_write_enabled: false,
                    bias: wgpu::DepthBiasState {
                        constant: -2,
                        slope_scale: -1.0,
                        clamp: 0.0,
                    },
                    ..depth_state.clone()
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        let thick_line_always_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("fret viewport overlay3d thick line always pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main_thick_line"),
                    buffers: &line_vertex_layouts,
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: color_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: depth_format,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        Self {
            uniform,
            bind_group,
            tri_pipeline,
            solid_depth_pipeline,
            solid_always_pipeline,
            thick_line_depth_pipeline,
            thick_line_always_pipeline,
        }
    }
}

const OVERLAY3D_WGSL: &str = r#"
struct Globals {
  view_proj: mat4x4f,
  viewport_and_thickness: vec4f,
};

@group(0) @binding(0)
var<uniform> globals: Globals;

struct VsIn {
  @location(0) pos: vec3f,
  @location(1) color: vec4f,
};

struct VsOut {
  @builtin(position) pos: vec4f,
  @location(0) color: vec4f,
};

@vertex
fn vs_main_tri(in: VsIn) -> VsOut {
  var out: VsOut;
  out.pos = globals.view_proj * vec4f(in.pos, 1.0);
  out.color = in.color;
  return out;
}

struct LineVsIn {
  @location(0) a: vec3f,
  @location(1) b: vec3f,
  @location(2) t: f32,
  @location(3) side: f32,
  @location(4) color: vec4f,
};

@vertex
fn vs_main_thick_line(in: LineVsIn) -> VsOut {
  let clip_a = globals.view_proj * vec4f(in.a, 1.0);
  let clip_b = globals.view_proj * vec4f(in.b, 1.0);

  let viewport = globals.viewport_and_thickness.xy;
  let thickness_px = globals.viewport_and_thickness.z;

  let ndc_a = clip_a.xy / clip_a.w;
  let ndc_b = clip_b.xy / clip_b.w;
  let dir_px = (ndc_b - ndc_a) * viewport;

  var offset_ndc = vec2f(0.0, 0.0);
  if dot(dir_px, dir_px) > 1e-8 && thickness_px > 0.0 {
    let dir_px_norm = normalize(dir_px);
    let normal_px = vec2f(-dir_px_norm.y, dir_px_norm.x);
    offset_ndc = normal_px * (thickness_px / viewport) * 0.5;
  }

  let clip = mix(clip_a, clip_b, in.t);
  let ndc = clip.xy / clip.w;
  let ndc_out = ndc + offset_ndc * in.side;

  var out: VsOut;
  out.pos = vec4f(ndc_out * clip.w, clip.z, clip.w);
  out.color = in.color;
  return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4f {
  return in.color;
}
"#;
