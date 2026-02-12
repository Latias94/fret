use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::scene::Paint;
use fret_core::{AppWindowId, Event, Px, Rect, UiServices};
use fret_launch::{
    EngineFrameUpdate, ImportedViewportRenderTarget, WinitAppDriver, WinitEventContext,
    WinitRenderContext, WinitRunnerConfig,
};
use fret_render::{RenderTargetColorSpace, Renderer, WgpuContext};
use fret_runtime::PlatformCapabilities;
use fret_ui::declarative;
use fret_ui::element::{
    ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, ViewportSurfaceProps,
};
use fret_ui::{Invalidation, Theme, UiTree};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CheckerUniforms {
    resolution: [f32; 2],
    t: f32,
    _pad: f32,
}

struct CheckerPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniforms: wgpu::Buffer,
}

impl CheckerPipeline {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("external texture imports checker shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
struct Uniforms {
  resolution: vec2<f32>,
  t: f32,
  _pad: f32,
};

@group(0) @binding(0)
var<uniform> u: Uniforms;

@vertex
fn vs(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0),
  );
  let p = pos[vi];
  return vec4<f32>(p, 0.0, 1.0);
}

fn checker(uv: vec2<f32>, cells: f32) -> f32 {
  let x = i32(floor(uv.x * cells));
  let y = i32(floor(uv.y * cells));
  return f32((x + y) & 1);
}

@fragment
fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let uv = pos.xy / max(u.resolution, vec2<f32>(1.0, 1.0));
  let phase = u.t * 0.25;
  let uv2 = vec2<f32>(uv.x + phase, uv.y - phase * 0.5);

  let c0 = vec3<f32>(0.08, 0.10, 0.14);
  let c1 = vec3<f32>(0.15, 0.18, 0.26);
  let v = checker(fract(uv2), 14.0);
  let base = mix(c0, c1, v);

  let ring = abs(sin((uv.x + uv.y + u.t * 0.15) * 6.28318));
  let tint = vec3<f32>(0.10, 0.18, 0.25) * ring * 0.25;

  return vec4<f32>(base + tint, 1.0);
}
"#
                .into(),
            ),
        });

        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("external texture imports checker uniforms"),
            size: std::mem::size_of::<CheckerUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("external texture imports checker bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("external texture imports checker bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("external texture imports checker pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("external texture imports checker pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
            uniforms,
        }
    }
}

struct ExternalTextureImportsWindowState {
    ui: UiTree<App>,
    show: fret_runtime::Model<bool>,

    target: ImportedViewportRenderTarget,
    target_px_size: (u32, u32),
    desired_target_px_size: (u32, u32),
    texture: Option<wgpu::Texture>,

    checker: Option<CheckerPipeline>,
}

#[derive(Default)]
struct ExternalTextureImportsDriver;

impl ExternalTextureImportsDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> ExternalTextureImportsWindowState {
        let show = app.models_mut().insert(true);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        ExternalTextureImportsWindowState {
            ui,
            show,
            target: ImportedViewportRenderTarget::new(
                wgpu::TextureFormat::Bgra8UnormSrgb,
                RenderTargetColorSpace::Srgb,
            ),
            target_px_size: (1, 1),
            desired_target_px_size: (1280, 720),
            texture: None,
            checker: None,
        }
    }

    fn ensure_target_registered(
        app: &mut App,
        window: AppWindowId,
        state: &mut ExternalTextureImportsWindowState,
        context: &WgpuContext,
        renderer: &mut Renderer,
    ) {
        let show = app.models().read(&state.show, |v| *v).unwrap_or(true);
        if !show {
            return;
        }

        if state.checker.is_none() {
            state.checker = Some(CheckerPipeline::new(&context.device, state.target.format()));
        }

        if state.target.is_registered() && state.texture.is_some() {
            return;
        }

        let size = state.desired_target_px_size;
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("external texture imports contract-path texture"),
            size: wgpu::Extent3d {
                width: size.0.max(1),
                height: size.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: state.target.format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let _id = state.target.ensure_registered(renderer, view, size);

        state.texture = Some(texture);
        state.target_px_size = size;
        app.request_redraw(window);
    }

    fn render_root(
        app: &mut App,
        ui: &mut UiTree<App>,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        show_model: fret_runtime::Model<bool>,
        show: bool,
        target: fret_core::RenderTargetId,
        target_px_size: (u32, u32),
    ) {
        let root = declarative::RenderRootContext::new(ui, app, services, window, bounds)
            .render_root("external-texture-imports", |cx| {
                cx.observe_model(&show_model, Invalidation::Layout);

                let theme = Theme::global(&*cx.app).snapshot();

                let mut fill = LayoutStyle::default();
                fill.size.width = Length::Fill;
                fill.size.height = Length::Fill;

                let mut panel_layout = LayoutStyle::default();
                panel_layout.size.width = Length::Px(Px(360.0));
                panel_layout.size.height = Length::Px(Px(240.0));

                let mut row = FlexProps {
                    layout: fill,
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(12.0),
                    padding: fret_core::Edges::all(Px(16.0)),
                    justify: MainAlign::Start,
                    align: CrossAlign::Start,
                    wrap: true,
                };
                row.layout.size.width = Length::Fill;
                row.layout.size.height = Length::Fill;

                let make_panel = |cx: &mut fret_ui::ElementContext<'_, App>,
                                  fit: fret_core::ViewportFit,
                                  test_id: &'static str|
                 -> fret_ui::element::AnyElement {
                    cx.container(
                        ContainerProps {
                            layout: panel_layout,
                            border: fret_core::Edges::all(Px(1.0)),
                            border_paint: Some(Paint::Solid(theme.color_required("border"))),
                            background: Some(theme.color_required("muted")),
                            corner_radii: fret_core::Corners::all(Px(10.0)),
                            ..Default::default()
                        },
                        |cx| {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            vec![
                                cx.viewport_surface_props(ViewportSurfaceProps {
                                    layout,
                                    target,
                                    target_px_size,
                                    fit,
                                    opacity: if show { 1.0 } else { 0.0 },
                                })
                                .test_id(test_id),
                            ]
                        },
                    )
                };

                vec![
                    cx.container(
                        ContainerProps {
                            layout: fill,
                            background: Some(theme.color_required("background")),
                            ..Default::default()
                        },
                        |cx| {
                            vec![cx.flex(row, |cx| {
                                vec![
                                    make_panel(
                                        cx,
                                        fret_core::ViewportFit::Contain,
                                        "ext-tex-fit-contain",
                                    ),
                                    make_panel(
                                        cx,
                                        fret_core::ViewportFit::Cover,
                                        "ext-tex-fit-cover",
                                    ),
                                    make_panel(
                                        cx,
                                        fret_core::ViewportFit::Stretch,
                                        "ext-tex-fit-stretch",
                                    ),
                                ]
                            })]
                        },
                    )
                    .test_id("external-texture-imports-root"),
                ]
            });

        ui.set_root(root);
    }
}

impl WinitAppDriver for ExternalTextureImportsDriver {
    type WindowState = ExternalTextureImportsWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn hot_reload_window(
        &mut self,
        app: &mut App,
        _services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
    ) {
        crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
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
            Event::WindowCloseRequested
            | Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                ..
            } => {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyV,
                ..
            } => {
                let _ = app.models_mut().update(&state.show, |v| *v = !*v);
                app.request_redraw(window);
            }
            _ => {
                state.ui.dispatch_event(app, services, event);
            }
        }
    }

    fn gpu_frame_prepare(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        state: &mut Self::WindowState,
        context: &WgpuContext,
        renderer: &mut Renderer,
        _scale_factor: f32,
    ) {
        Self::ensure_target_registered(app, window, state, context, renderer);
    }

    fn record_engine_frame(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        state: &mut Self::WindowState,
        context: &WgpuContext,
        _renderer: &mut Renderer,
        _scale_factor: f32,
        _tick_id: fret_runtime::TickId,
        frame_id: fret_runtime::FrameId,
    ) -> EngineFrameUpdate {
        let show = app.models().read(&state.show, |v| *v).unwrap_or(true);

        let mut update = EngineFrameUpdate::default();

        if !show {
            state.target.push_unregister(&mut update);
            state.texture = None;
            state.target_px_size = (1, 1);
            return update;
        }

        let desired = state.desired_target_px_size;
        if state.target_px_size != desired {
            let new_texture = context.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("external texture imports contract-path texture"),
                size: wgpu::Extent3d {
                    width: desired.0.max(1),
                    height: desired.1.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: state.target.format(),
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            state.texture = Some(new_texture);
            state.target_px_size = desired;
        }

        let texture = state
            .texture
            .as_ref()
            .expect("texture must exist after resize");
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        state
            .target
            .push_update(&mut update, view.clone(), state.target_px_size);

        if let Some(checker) = state.checker.as_ref() {
            let t = frame_id.0 as f32 * (1.0 / 60.0);
            let uniforms = CheckerUniforms {
                resolution: [state.target_px_size.0 as f32, state.target_px_size.1 as f32],
                t,
                _pad: 0.0,
            };
            context
                .queue
                .write_buffer(&checker.uniforms, 0, bytemuck::bytes_of(&uniforms));

            let mut encoder =
                context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("external texture imports contract-path encoder"),
                    });
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("external texture imports contract-path pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(&checker.pipeline);
                pass.set_bind_group(0, &checker.bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
            update.push_command_buffer(encoder.finish());
        }

        app.push_effect(Effect::RequestAnimationFrame(window));
        update
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

        let w_px = (bounds.size.width.0.max(1.0) * scale_factor).round() as u32;
        let h_px = (bounds.size.height.0.max(1.0) * scale_factor).round() as u32;
        state.desired_target_px_size = (w_px.max(1).min(4096), h_px.max(1).min(4096));

        let show_model = state.show.clone();
        let show = app.models().read(&show_model, |v| *v).unwrap_or(true);
        let target = state.target.id();
        let target_px_size = state.target_px_size;

        Self::render_root(
            app,
            &mut state.ui,
            services,
            window,
            bounds,
            show_model,
            show,
            target,
            target_px_size,
        );

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
        main_window_title: "fret-demo external_texture_imports_demo (V toggles target)".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    ExternalTextureImportsDriver::default()
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

    crate::run_native_demo(config, app, driver).context("run external_texture_imports_demo app")
}
