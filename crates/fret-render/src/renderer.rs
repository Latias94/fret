use bytemuck::{Pod, Zeroable};
use fret_core::{
    geometry::{Corners, Edges, Rect},
    scene::{Color, Scene, SceneOp},
};

use crate::targets::{RenderTargetDescriptor, RenderTargetRegistry};

#[derive(Debug, Clone, Copy)]
pub struct ClearColor(pub wgpu::Color);

impl Default for ClearColor {
    fn default() -> Self {
        Self(wgpu::Color {
            r: 0.08,
            g: 0.09,
            b: 0.10,
            a: 1.0,
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ViewportUniform {
    viewport_size: [f32; 2],
    _pad: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct QuadInstance {
    rect: [f32; 4],
    color: [f32; 4],
    corner_radii: [f32; 4],
    border: [f32; 4],
    border_color: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScissorRect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl ScissorRect {
    fn full(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            w: width,
            h: height,
        }
    }
}

fn color_to_linear_rgba_premul(color: Color) -> [f32; 4] {
    let a = color.a;
    [color.r * a, color.g * a, color.b * a, a]
}

fn corners_to_vec4(c: Corners) -> [f32; 4] {
    [
        c.top_left.0,
        c.top_right.0,
        c.bottom_right.0,
        c.bottom_left.0,
    ]
}

fn edges_to_vec4(e: Edges) -> [f32; 4] {
    [e.left.0, e.top.0, e.right.0, e.bottom.0]
}

fn rect_to_pixels(rect: Rect, scale_factor: f32) -> (f32, f32, f32, f32) {
    (
        rect.origin.x.0 * scale_factor,
        rect.origin.y.0 * scale_factor,
        rect.size.width.0 * scale_factor,
        rect.size.height.0 * scale_factor,
    )
}

fn scissor_from_rect(rect: Rect, scale_factor: f32, viewport: (u32, u32)) -> Option<ScissorRect> {
    let (vw, vh) = viewport;
    if vw == 0 || vh == 0 {
        return None;
    }

    let (x, y, w, h) = rect_to_pixels(rect, scale_factor);
    let x0 = x.floor().clamp(0.0, vw as f32) as i32;
    let y0 = y.floor().clamp(0.0, vh as f32) as i32;
    let x1 = (x + w).ceil().clamp(0.0, vw as f32) as i32;
    let y1 = (y + h).ceil().clamp(0.0, vh as f32) as i32;

    let w = (x1 - x0).max(0) as u32;
    let h = (y1 - y0).max(0) as u32;
    if w == 0 || h == 0 {
        return Some(ScissorRect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        });
    }

    Some(ScissorRect {
        x: x0 as u32,
        y: y0 as u32,
        w,
        h,
    })
}

fn intersect_scissor(a: ScissorRect, b: ScissorRect) -> ScissorRect {
    let ax1 = a.x.saturating_add(a.w);
    let ay1 = a.y.saturating_add(a.h);
    let bx1 = b.x.saturating_add(b.w);
    let by1 = b.y.saturating_add(b.h);

    let x0 = a.x.max(b.x);
    let y0 = a.y.max(b.y);
    let x1 = ax1.min(bx1);
    let y1 = ay1.min(by1);

    let w = x1.saturating_sub(x0);
    let h = y1.saturating_sub(y0);
    ScissorRect { x: x0, y: y0, w, h }
}

struct DrawCall {
    scissor: ScissorRect,
    first_instance: u32,
    instance_count: u32,
}

pub struct Renderer {
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    uniform_bind_group_layout: wgpu::BindGroupLayout,

    quad_pipeline_format: Option<wgpu::TextureFormat>,
    quad_pipeline: Option<wgpu::RenderPipeline>,

    instance_buffers: Vec<wgpu::Buffer>,
    instance_buffer_index: usize,
    instance_capacity: usize,

    render_targets: RenderTargetRegistry,
}

impl Renderer {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret quad uniforms layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:
                            Some(
                                std::num::NonZeroU64::new(
                                    std::mem::size_of::<ViewportUniform>() as u64
                                )
                                .unwrap(),
                            ),
                    },
                    count: None,
                }],
            });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret quad uniforms buffer"),
            size: std::mem::size_of::<ViewportUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret quad uniforms bind group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        const FRAMES_IN_FLIGHT: usize = 3;
        let instance_capacity = 1024;
        let instance_buffers = (0..FRAMES_IN_FLIGHT)
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("fret quad instances #{i}")),
                    size: (instance_capacity * std::mem::size_of::<QuadInstance>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();

        Self {
            uniform_buffer,
            uniform_bind_group,
            uniform_bind_group_layout,
            quad_pipeline_format: None,
            quad_pipeline: None,
            instance_buffers,
            instance_buffer_index: 0,
            instance_capacity,
            render_targets: RenderTargetRegistry::default(),
        }
    }

    pub fn register_render_target(
        &mut self,
        desc: RenderTargetDescriptor,
    ) -> fret_core::RenderTargetId {
        self.render_targets.register(desc)
    }

    pub fn update_render_target(
        &mut self,
        id: fret_core::RenderTargetId,
        desc: RenderTargetDescriptor,
    ) -> bool {
        self.render_targets.update(id, desc)
    }

    pub fn unregister_render_target(&mut self, id: fret_core::RenderTargetId) -> bool {
        self.render_targets.unregister(id)
    }

    fn ensure_pipeline(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
        if self.quad_pipeline_format == Some(format) && self.quad_pipeline.is_some() {
            return;
        }

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret quad shader"),
            source: wgpu::ShaderSource::Wgsl(QUAD_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret quad pipeline layout"),
            bind_group_layouts: &[&self.uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let instance_stride = std::mem::size_of::<QuadInstance>() as wgpu::BufferAddress;
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret quad pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: instance_stride,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 32,
                            shader_location: 2,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 48,
                            shader_location: 3,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 64,
                            shader_location: 4,
                        },
                    ],
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
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        self.quad_pipeline_format = Some(format);
        self.quad_pipeline = Some(pipeline);
    }

    fn ensure_instance_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.instance_capacity {
            return;
        }
        let new_capacity = needed.next_power_of_two().max(self.instance_capacity * 2);
        self.instance_buffers = (0..self.instance_buffers.len())
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("fret quad instances (resized) #{i}")),
                    size: (new_capacity * std::mem::size_of::<QuadInstance>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();
        self.instance_buffer_index = 0;
        self.instance_capacity = new_capacity;
    }

    pub fn render_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        target_view: &wgpu::TextureView,
        scene: &Scene,
        clear: ClearColor,
        scale_factor: f32,
        viewport_size: (u32, u32),
    ) -> wgpu::CommandBuffer {
        self.ensure_pipeline(device, format);

        let uniform = ViewportUniform {
            viewport_size: [viewport_size.0 as f32, viewport_size.1 as f32],
            _pad: [0.0, 0.0],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        let mut instances: Vec<QuadInstance> = Vec::new();
        let mut draws: Vec<DrawCall> = Vec::new();

        let mut scissor_stack: Vec<ScissorRect> =
            vec![ScissorRect::full(viewport_size.0, viewport_size.1)];

        let mut current_scissor = *scissor_stack
            .last()
            .expect("scissor stack must be non-empty");
        let mut current_draw_first: u32 = 0;

        for op in &scene.ops {
            match op {
                SceneOp::PushClipRect { rect } => {
                    let Some(new_scissor) = scissor_from_rect(*rect, scale_factor, viewport_size)
                    else {
                        continue;
                    };

                    let combined = intersect_scissor(current_scissor, new_scissor);
                    if combined != current_scissor {
                        let instance_count =
                            (instances.len() as u32).saturating_sub(current_draw_first);
                        if instance_count > 0 {
                            draws.push(DrawCall {
                                scissor: current_scissor,
                                first_instance: current_draw_first,
                                instance_count,
                            });
                            current_draw_first = instances.len() as u32;
                        }
                    }

                    current_scissor = combined;
                    scissor_stack.push(current_scissor);
                }
                SceneOp::PopClip => {
                    if scissor_stack.len() > 1 {
                        scissor_stack.pop();
                        let new_scissor = *scissor_stack
                            .last()
                            .expect("scissor stack must be non-empty");
                        if new_scissor != current_scissor {
                            let instance_count =
                                (instances.len() as u32).saturating_sub(current_draw_first);
                            if instance_count > 0 {
                                draws.push(DrawCall {
                                    scissor: current_scissor,
                                    first_instance: current_draw_first,
                                    instance_count,
                                });
                                current_draw_first = instances.len() as u32;
                            }
                            current_scissor = new_scissor;
                        }
                    }
                }
                SceneOp::Quad {
                    rect,
                    background,
                    border,
                    border_color,
                    corner_radii,
                    ..
                } => {
                    if background.a <= 0.0 && border_color.a <= 0.0 {
                        continue;
                    }
                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    if w <= 0.0 || h <= 0.0 {
                        continue;
                    }
                    instances.push(QuadInstance {
                        rect: [x, y, w, h],
                        color: color_to_linear_rgba_premul(*background),
                        corner_radii: corners_to_vec4(*corner_radii),
                        border: edges_to_vec4(*border),
                        border_color: color_to_linear_rgba_premul(*border_color),
                    });
                }
                SceneOp::Image { .. } | SceneOp::Text { .. } => {
                    // Not implemented yet.
                }
                SceneOp::ViewportSurface { target, .. } => {
                    if self.render_targets.get(*target).is_none() {
                        continue;
                    }
                    // Not implemented yet; target registry exists to fix the contract early.
                }
            }
        }

        let instance_count = (instances.len() as u32).saturating_sub(current_draw_first);
        if instance_count > 0 {
            draws.push(DrawCall {
                scissor: current_scissor,
                first_instance: current_draw_first,
                instance_count,
            });
        }

        self.ensure_instance_capacity(device, instances.len());
        let instance_buffer = &self.instance_buffers[self.instance_buffer_index];
        self.instance_buffer_index =
            (self.instance_buffer_index + 1) % self.instance_buffers.len();
        if !instances.is_empty() {
            queue.write_buffer(instance_buffer, 0, bytemuck::cast_slice(&instances));
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fret renderer encoder"),
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fret renderer pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear.0),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let pipeline = self
                .quad_pipeline
                .as_ref()
                .expect("quad pipeline must exist");

            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            pass.set_vertex_buffer(0, instance_buffer.slice(..));

            for draw in draws {
                if draw.scissor.w == 0 || draw.scissor.h == 0 {
                    continue;
                }
                pass.set_scissor_rect(
                    draw.scissor.x,
                    draw.scissor.y,
                    draw.scissor.w,
                    draw.scissor.h,
                );
                pass.draw(
                    0..6,
                    draw.first_instance..(draw.first_instance + draw.instance_count),
                );
            }
        }

        encoder.finish()
    }
}

const QUAD_SHADER: &str = r#"
struct Viewport {
  viewport_size: vec2<f32>,
  _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;

struct QuadInstance {
  rect: vec4<f32>,
  color: vec4<f32>,
  corner_radii: vec4<f32>,
  border: vec4<f32>,
  border_color: vec4<f32>,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) pixel_pos: vec2<f32>,
  @location(1) rect_origin: vec2<f32>,
  @location(2) rect_size: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) corner_radii: vec4<f32>,
  @location(5) border: vec4<f32>,
  @location(6) border_color: vec4<f32>,
};

fn quad_vertex_xy(vertex_index: u32) -> vec2<f32> {
  switch vertex_index {
    case 0u: { return vec2<f32>(0.0, 0.0); }
    case 1u: { return vec2<f32>(1.0, 0.0); }
    case 2u: { return vec2<f32>(1.0, 1.0); }
    case 3u: { return vec2<f32>(0.0, 0.0); }
    case 4u: { return vec2<f32>(1.0, 1.0); }
    default: { return vec2<f32>(0.0, 1.0); }
  }
}

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let ndc_x = (pixel_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (pixel_pos.y / viewport.viewport_size.y) * 2.0;
  return vec2<f32>(ndc_x, ndc_y);
}

@vertex
fn vs_main(
  @builtin(vertex_index) vertex_index: u32,
  @location(0) rect: vec4<f32>,
  @location(1) color: vec4<f32>,
  @location(2) corner_radii: vec4<f32>,
  @location(3) border: vec4<f32>,
  @location(4) border_color: vec4<f32>,
) -> VsOut {
  let uv = quad_vertex_xy(vertex_index);
  let pixel_pos = rect.xy + uv * rect.zw;
  let clip_xy = to_clip_space(pixel_pos);

  var out: VsOut;
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.pixel_pos = pixel_pos;
  out.rect_origin = rect.xy;
  out.rect_size = rect.zw;
  out.color = color;
  out.corner_radii = corner_radii;
  out.border = border;
  out.border_color = border_color;
  return out;
}

fn pick_corner_radius(center_to_point: vec2<f32>, radii: vec4<f32>) -> f32 {
  if (center_to_point.x < 0.0) {
    if (center_to_point.y < 0.0) { return radii.x; }
    return radii.w;
  }
  if (center_to_point.y < 0.0) { return radii.y; }
  return radii.z;
}

fn quad_sdf_impl(corner_center_to_point: vec2<f32>, corner_radius: f32) -> f32 {
  if (corner_radius == 0.0) {
    return max(corner_center_to_point.x, corner_center_to_point.y);
  }
  let signed_distance_to_inset_quad =
    length(max(vec2<f32>(0.0), corner_center_to_point)) +
    min(0.0, max(corner_center_to_point.x, corner_center_to_point.y));
  return signed_distance_to_inset_quad - corner_radius;
}

fn quad_sdf(point: vec2<f32>, rect_origin: vec2<f32>, rect_size: vec2<f32>, corner_radii: vec4<f32>) -> f32 {
  let center = rect_origin + rect_size * 0.5;
  let center_to_point = point - center;
  let half_size = rect_size * 0.5;
  let corner_radius = pick_corner_radius(center_to_point, corner_radii);
  let corner_to_point = abs(center_to_point) - half_size;
  let corner_center_to_point = corner_to_point + corner_radius;
  return quad_sdf_impl(corner_center_to_point, corner_radius);
}

fn saturate(x: f32) -> f32 {
  return clamp(x, 0.0, 1.0);
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let antialias_threshold = 0.5;
  let sdf = quad_sdf(input.pixel_pos, input.rect_origin, input.rect_size, input.corner_radii);

  // TODO: border rendering (dash / per-edge widths) and stroke alignment.
  let alpha = saturate(antialias_threshold - sdf);

  return vec4<f32>(input.color.rgb, input.color.a) * alpha;
}
"#;
