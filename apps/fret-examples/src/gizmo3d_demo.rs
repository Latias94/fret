use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{
    AppWindowId, Event, RenderTargetId, ViewportFit, ViewportInputEvent, ViewportInputKind,
};
use fret_gizmo::{
    DepthMode, Gizmo, GizmoConfig, GizmoInput, GizmoMode, GizmoOrientation, GizmoPhase,
    Transform3d, ViewportRect,
};
use fret_launch::{
    EngineFrameUpdate, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_plot3d::retained::{Plot3dCanvas, Plot3dModel, Plot3dStyle, Plot3dViewport};
use fret_render::{RenderTargetColorSpace, RenderTargetDescriptor, Renderer, WgpuContext};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use glam::{Mat4, Quat, Vec2, Vec3};
use std::collections::HashMap;
use wgpu::util::DeviceExt as _;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Vertex {
    pos: [f32; 3],
    color: [f32; 4],
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct LineVertex {
    a: [f32; 3],
    b: [f32; 3],
    t: f32,
    side: f32,
    color: [f32; 4],
}

unsafe impl bytemuck::Zeroable for LineVertex {}
unsafe impl bytemuck::Pod for LineVertex {}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    /// x = viewport_w_px, y = viewport_h_px, z = line_thickness_px, w = unused
    viewport_and_thickness: [f32; 4],
}

unsafe impl bytemuck::Zeroable for Uniforms {}
unsafe impl bytemuck::Pod for Uniforms {}

fn push_thick_line_quad(out: &mut Vec<LineVertex>, a: [f32; 3], b: [f32; 3], color: [f32; 4]) {
    // Two triangles (6 vertices) for a screen-space thick line quad.
    out.extend_from_slice(&[
        LineVertex {
            a,
            b,
            t: 0.0,
            side: -1.0,
            color,
        },
        LineVertex {
            a,
            b,
            t: 0.0,
            side: 1.0,
            color,
        },
        LineVertex {
            a,
            b,
            t: 1.0,
            side: 1.0,
            color,
        },
        LineVertex {
            a,
            b,
            t: 0.0,
            side: -1.0,
            color,
        },
        LineVertex {
            a,
            b,
            t: 1.0,
            side: 1.0,
            color,
        },
        LineVertex {
            a,
            b,
            t: 1.0,
            side: -1.0,
            color,
        },
    ]);
}

struct Gizmo3dDemoTarget {
    id: RenderTargetId,
    size: (u32, u32),
    color: wgpu::Texture,
    depth: wgpu::Texture,
}

struct Gizmo3dDemoGpu {
    uniform: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    tri_pipeline: wgpu::RenderPipeline,
    thick_line_depth_pipeline: wgpu::RenderPipeline,
    thick_line_always_pipeline: wgpu::RenderPipeline,
    cube_vb: wgpu::Buffer,
    cube_ib: wgpu::Buffer,
    cube_index_count: u32,
}

#[derive(Debug)]
struct Gizmo3dDemoModel {
    viewport_target: RenderTargetId,
    viewport_px: (u32, u32),
    gizmo: Gizmo,
    target: Transform3d,
    drag_start_target: Option<Transform3d>,
    input: GizmoInput,
}

impl Default for Gizmo3dDemoModel {
    fn default() -> Self {
        let mut gizmo_cfg = GizmoConfig::default();
        gizmo_cfg.translate_snap_step = Some(0.25);
        Self {
            viewport_target: RenderTargetId::default(),
            viewport_px: (960, 540),
            gizmo: Gizmo::new(gizmo_cfg),
            target: Transform3d {
                translation: Vec3::new(0.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            drag_start_target: None,
            input: GizmoInput {
                cursor_px: Vec2::ZERO,
                hovered: true,
                drag_started: false,
                dragging: false,
                snap: false,
                cancel: false,
            },
        }
    }
}

#[derive(Default)]
struct Gizmo3dDemoService {
    per_window: HashMap<AppWindowId, fret_runtime::Model<Gizmo3dDemoModel>>,
}

struct Gizmo3dDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: fret_runtime::Model<Plot3dModel>,
    demo: fret_runtime::Model<Gizmo3dDemoModel>,
    target: Option<Gizmo3dDemoTarget>,
    gpu: Option<Gizmo3dDemoGpu>,
}

#[derive(Default)]
struct Gizmo3dDemoDriver;

impl Gizmo3dDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> Gizmo3dDemoWindowState {
        let plot = app.models_mut().insert(Plot3dModel {
            viewport: Plot3dViewport {
                target: RenderTargetId::default(),
                target_px_size: (960, 540),
                fit: ViewportFit::Contain,
                opacity: 1.0,
            },
        });

        let demo = app.models_mut().insert(Gizmo3dDemoModel::default());

        app.with_global_mut(Gizmo3dDemoService::default, |svc, _app| {
            svc.per_window.insert(window, demo.clone());
        });

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Gizmo3dDemoWindowState {
            ui,
            root: None,
            plot,
            demo,
            target: None,
            gpu: None,
        }
    }

    fn ensure_target(
        app: &mut App,
        window: AppWindowId,
        state: &mut Gizmo3dDemoWindowState,
        context: &WgpuContext,
        renderer: &mut Renderer,
    ) -> (
        RenderTargetId,
        wgpu::TextureView,
        wgpu::TextureView,
        (u32, u32),
    ) {
        let desired_size = state
            .plot
            .read(app, |_app, m| m.viewport.target_px_size)
            .unwrap_or((960, 540));

        let needs_new = state.target.as_ref().is_none_or(|t| t.size != desired_size);

        if needs_new {
            let (w, h) = desired_size;
            let w = w.max(1);
            let h = h.max(1);
            let size = (w, h);

            let color = context.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("gizmo3d demo color target"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let depth = context.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("gizmo3d demo depth target"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24Plus,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            let view_for_registry = color.create_view(&wgpu::TextureViewDescriptor::default());

            let id = if let Some(prev) = state.target.take() {
                renderer.update_render_target(
                    prev.id,
                    RenderTargetDescriptor {
                        view: view_for_registry,
                        size,
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        color_space: RenderTargetColorSpace::Srgb,
                    },
                );
                prev.id
            } else {
                renderer.register_render_target(RenderTargetDescriptor {
                    view: view_for_registry,
                    size,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    color_space: RenderTargetColorSpace::Srgb,
                })
            };

            state.target = Some(Gizmo3dDemoTarget {
                id,
                size,
                color,
                depth,
            });

            let _ = state.plot.update(app, |m, _cx| {
                m.viewport.target = id;
                m.viewport.target_px_size = size;
            });
            let _ = state.demo.update(app, |m, _cx| {
                m.viewport_target = id;
                m.viewport_px = size;
            });

            app.request_redraw(window);
        }

        let target = state.target.as_ref().expect("target ensured");
        let color_view = target
            .color
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = target
            .depth
            .create_view(&wgpu::TextureViewDescriptor::default());
        (target.id, color_view, depth_view, target.size)
    }

    fn ensure_gpu(state: &mut Gizmo3dDemoWindowState, context: &WgpuContext) {
        if state.gpu.is_some() {
            return;
        }

        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("gizmo3d demo shader"),
                source: wgpu::ShaderSource::Wgsl(
                    r#"
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
"#
                    .into(),
                ),
            });

        let uniform = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gizmo3d demo view_proj uniform"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("gizmo3d demo bgl"),
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

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("gizmo3d demo bind group"),
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                }],
            });

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("gizmo3d demo pipeline layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    immediate_size: 0,
                });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4],
        };

        let line_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x3, // a
                1 => Float32x3, // b
                2 => Float32,   // t
                3 => Float32,   // side
                4 => Float32x4  // color
            ],
        };

        let depth_state = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24Plus,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

        let tri_pipeline = context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("gizmo3d demo tri pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main_tri"),
                    buffers: &[vertex_layout.clone()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: Some(depth_state.clone()),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        let thick_line_depth_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("gizmo3d demo thick line depth pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main_thick_line"),
                        buffers: &[line_vertex_layout.clone()],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Bgra8UnormSrgb,
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
                        ..depth_state.clone()
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview_mask: None,
                    cache: None,
                });

        let thick_line_always_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("gizmo3d demo thick line always pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main_thick_line"),
                        buffers: &[line_vertex_layout],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Bgra8UnormSrgb,
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
                        format: wgpu::TextureFormat::Depth24Plus,
                        depth_write_enabled: false,
                        depth_compare: wgpu::CompareFunction::Always,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview_mask: None,
                    cache: None,
                });

        let (cube_vertices, cube_indices) = cube_mesh();

        let cube_vb = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("gizmo3d demo cube vb"),
                contents: bytemuck::cast_slice(&cube_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let cube_ib = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("gizmo3d demo cube ib"),
                contents: bytemuck::cast_slice(&cube_indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        state.gpu = Some(Gizmo3dDemoGpu {
            uniform,
            bind_group,
            tri_pipeline,
            thick_line_depth_pipeline,
            thick_line_always_pipeline,
            cube_vb,
            cube_ib,
            cube_index_count: cube_indices.len().min(u32::MAX as usize) as u32,
        });
    }
}

fn cube_mesh() -> (Vec<Vertex>, Vec<u16>) {
    let c = [0.55, 0.58, 0.62, 1.0];
    let v = |x: f32, y: f32, z: f32| Vertex {
        pos: [x, y, z],
        color: c,
    };

    let verts = vec![
        v(-0.4, -0.4, 0.4),
        v(0.4, -0.4, 0.4),
        v(0.4, 0.4, 0.4),
        v(-0.4, 0.4, 0.4),
        v(-0.4, -0.4, -0.4),
        v(0.4, -0.4, -0.4),
        v(0.4, 0.4, -0.4),
        v(-0.4, 0.4, -0.4),
    ];

    let idx: Vec<u16> = vec![
        0, 1, 2, 0, 2, 3, // front
        1, 5, 6, 1, 6, 2, // right
        5, 4, 7, 5, 7, 6, // back
        4, 0, 3, 4, 3, 7, // left
        3, 2, 6, 3, 6, 7, // top
        4, 5, 1, 4, 1, 0, // bottom
    ];

    (verts, idx)
}

fn camera_view_projection(size: (u32, u32)) -> Mat4 {
    let (w, h) = size;
    let aspect = (w.max(1) as f32) / (h.max(1) as f32);
    let eye = Vec3::new(1.6, 1.2, 2.2);
    let center = Vec3::new(0.0, 0.0, 0.0);
    let up = Vec3::Y;
    let view = Mat4::look_at_rh(eye, center, up);
    let proj = Mat4::perspective_rh(55.0_f32.to_radians(), aspect, 0.05, 50.0);
    proj * view
}

impl WinitAppDriver for Gizmo3dDemoDriver {
    type WindowState = Gizmo3dDemoWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        let state = Self::build_ui(app, window);
        // Ensure we render at least one frame; otherwise the viewport surface can remain blank until
        // the first input event happens to request a redraw.
        app.request_redraw(window);
        state
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
            ..
        } = context;

        match event {
            Event::WindowCloseRequested => {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
            }
            Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                ..
            } => {
                let mut did_cancel = false;
                let _ = state.demo.update(app, |m, _cx| {
                    let is_dragging = m.input.dragging || m.gizmo.state.active.is_some();
                    if !is_dragging {
                        return;
                    }

                    let view_projection = camera_view_projection(m.viewport_px);
                    let viewport = ViewportRect::new(
                        Vec2::ZERO,
                        Vec2::new(m.viewport_px.0 as f32, m.viewport_px.1 as f32),
                    );

                    let mut input = m.input;
                    input.drag_started = false;
                    input.dragging = false;
                    input.cancel = true;

                    if let Some(update) = m.gizmo.update(
                        view_projection,
                        viewport,
                        input,
                        std::slice::from_ref(&m.target),
                    ) {
                        if update.phase == GizmoPhase::Cancel {
                            if let Some(start) = m.drag_start_target.take() {
                                m.target = start;
                            }
                            did_cancel = true;
                        }
                    }

                    m.input.cancel = false;
                    m.input.dragging = false;
                    m.input.drag_started = false;
                });

                if did_cancel {
                    app.request_redraw(window);
                } else {
                    app.push_effect(Effect::Window(WindowRequest::Close(window)));
                }
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyR,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    m.gizmo.config.mode = GizmoMode::Rotate;
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyS,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    m.gizmo.config.mode = GizmoMode::Scale;
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyT,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    m.gizmo.config.mode = GizmoMode::Translate;
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyU,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    m.gizmo.config.mode = GizmoMode::Universal;
                });
                app.request_redraw(window);
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyL,
                repeat: false,
                ..
            } => {
                let _ = state.demo.update(app, |m, _cx| {
                    m.gizmo.config.orientation = match m.gizmo.config.orientation {
                        GizmoOrientation::World => GizmoOrientation::Local,
                        GizmoOrientation::Local => GizmoOrientation::World,
                    };
                });
                app.request_redraw(window);
            }
            _ => {
                state.ui.dispatch_event(app, services, event);
            }
        }
    }

    fn viewport_input(&mut self, app: &mut App, event: ViewportInputEvent) {
        let model = app.with_global_mut(Gizmo3dDemoService::default, |svc, _app| {
            svc.per_window.get(&event.window).cloned()
        });
        let Some(model) = model else {
            return;
        };

        let _ = model.update(app, |m, _cx| {
            if m.viewport_target != event.target {
                return;
            }

            // Use UV instead of integer target pixels to avoid cursor quantization.
            let cursor_px = Vec2::new(
                event.uv.0 * m.viewport_px.0 as f32,
                event.uv.1 * m.viewport_px.1 as f32,
            );
            let (drag_started, dragging) = match event.kind {
                ViewportInputKind::PointerDown {
                    button: fret_core::MouseButton::Left,
                    ..
                } => (true, true),
                ViewportInputKind::PointerMove { buttons, .. } => {
                    // Some platforms can produce inconsistent "buttons" state for move events.
                    // Prefer to keep dragging latched until an explicit PointerUp arrives.
                    (false, m.input.dragging || buttons.left)
                }
                ViewportInputKind::PointerUp {
                    button: fret_core::MouseButton::Left,
                    ..
                } => (false, false),
                _ => (false, m.input.dragging),
            };

            let snap = match event.kind {
                ViewportInputKind::PointerMove { modifiers, .. } => modifiers.shift,
                ViewportInputKind::PointerDown { modifiers, .. } => modifiers.shift,
                ViewportInputKind::PointerUp { modifiers, .. } => modifiers.shift,
                ViewportInputKind::Wheel { modifiers, .. } => modifiers.shift,
            };

            m.input = GizmoInput {
                cursor_px,
                hovered: true,
                drag_started,
                dragging,
                snap,
                cancel: false,
            };

            let view_projection = camera_view_projection(m.viewport_px);
            let viewport = ViewportRect::new(
                Vec2::ZERO,
                Vec2::new(m.viewport_px.0 as f32, m.viewport_px.1 as f32),
            );
            if let Some(update) = m.gizmo.update(
                view_projection,
                viewport,
                m.input,
                std::slice::from_ref(&m.target),
            ) {
                match update.phase {
                    GizmoPhase::Begin => {
                        m.drag_start_target = Some(m.target);
                    }
                    GizmoPhase::Update => {
                        m.target = update.updated_targets[0];
                    }
                    GizmoPhase::Commit => {
                        m.drag_start_target = None;
                    }
                    GizmoPhase::Cancel => {
                        if let Some(start) = m.drag_start_target.take() {
                            m.target = start;
                        }
                    }
                }
            }
        });

        app.request_redraw(event.window);
    }

    fn record_engine_frame(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        state: &mut Self::WindowState,
        context: &WgpuContext,
        renderer: &mut Renderer,
        _scale_factor: f32,
        _tick_id: fret_runtime::TickId,
        frame_id: fret_runtime::FrameId,
    ) -> EngineFrameUpdate {
        let (_id, color_view, depth_view, size) =
            Self::ensure_target(app, window, state, context, renderer);
        Self::ensure_gpu(state, context);

        let gpu = state.gpu.as_ref().expect("gpu ensured");

        let view_proj = camera_view_projection(size);
        let uniforms = Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
            viewport_and_thickness: [size.0 as f32, size.1 as f32, 4.0, 0.0],
        };
        context
            .queue
            .write_buffer(&gpu.uniform, 0, bytemuck::bytes_of(&uniforms));

        let lines = state
            .demo
            .read(app, |_app, m| {
                m.gizmo.draw(
                    view_proj,
                    ViewportRect::new(Vec2::ZERO, Vec2::new(size.0 as f32, size.1 as f32)),
                    m.target,
                )
            })
            .unwrap_or_else(|_| Vec::new());

        let mut line_verts_depth: Vec<LineVertex> = Vec::new();
        let mut line_verts_always: Vec<LineVertex> = Vec::new();

        for line in lines {
            let a = line.a.to_array();
            let b = line.b.to_array();
            let color = [line.color.r, line.color.g, line.color.b, line.color.a];
            match line.depth {
                DepthMode::Test => {
                    push_thick_line_quad(&mut line_verts_depth, a, b, color);
                }
                DepthMode::Always => {
                    push_thick_line_quad(&mut line_verts_always, a, b, color);
                }
            }
        }

        let line_vb_depth = (!line_verts_depth.is_empty()).then(|| {
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("gizmo3d demo thick line vb (depth)"),
                    contents: bytemuck::cast_slice(&line_verts_depth),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });
        let line_vb_always = (!line_verts_always.is_empty()).then(|| {
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("gizmo3d demo thick line vb (always)"),
                    contents: bytemuck::cast_slice(&line_verts_always),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });

        let clear = wgpu::Color {
            r: 0.08,
            g: 0.08,
            b: 0.10,
            a: 1.0,
        };

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("gizmo3d demo encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("gizmo3d demo pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            pass.set_bind_group(0, &gpu.bind_group, &[]);

            pass.set_pipeline(&gpu.tri_pipeline);
            pass.set_vertex_buffer(0, gpu.cube_vb.slice(..));
            pass.set_index_buffer(gpu.cube_ib.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..gpu.cube_index_count, 0, 0..1);

            if let Some(line_vb_depth) = &line_vb_depth {
                pass.set_pipeline(&gpu.thick_line_depth_pipeline);
                pass.set_vertex_buffer(0, line_vb_depth.slice(..));
                pass.draw(
                    0..(line_verts_depth.len().min(u32::MAX as usize) as u32),
                    0..1,
                );
            }
            if let Some(line_vb_always) = &line_vb_always {
                pass.set_pipeline(&gpu.thick_line_always_pipeline);
                pass.set_vertex_buffer(0, line_vb_always.slice(..));
                pass.draw(
                    0..(line_verts_always.len().min(u32::MAX as usize) as u32),
                    0..1,
                );
            }

            let _ = frame_id;
        }

        EngineFrameUpdate {
            target_updates: Vec::new(),
            command_buffers: vec![encoder.finish()],
        }
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            scale_factor,
            scene,
        } = context;

        let root = state.root.get_or_insert_with(|| {
            let style = Plot3dStyle::default();
            let canvas = Plot3dCanvas::new(state.plot.clone()).style(style);
            let node = Plot3dCanvas::create_node(&mut state.ui, canvas);
            state.ui.set_root(node);
            node
        });

        state.ui.set_root(*root);
        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);

        scene.clear();
        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();
        frame.paint_all(scene);
    }
}

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo gizmo3d_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    Gizmo3dDemoDriver
}

pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let app = build_app();
    let config = build_runner_config();
    let driver = build_driver();

    crate::run_native_demo(config, app, driver).context("run gizmo3d_demo app")
}
