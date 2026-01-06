use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event, RenderTargetId};
use fret_launch::{
    EngineFrameUpdate, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
    run_app,
};
use fret_plot3d::retained::{Plot3dCanvas, Plot3dModel, Plot3dStyle, Plot3dViewport};
use fret_render::{RenderTargetColorSpace, RenderTargetDescriptor, Renderer, WgpuContext};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;

struct Plot3dDemoTarget {
    id: RenderTargetId,
    size: (u32, u32),
    texture: wgpu::Texture,
}

struct Plot3dDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: fret_runtime::Model<Plot3dModel>,
    target: Option<Plot3dDemoTarget>,
}

#[derive(Default)]
struct Plot3dDemoDriver;

impl Plot3dDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> Plot3dDemoWindowState {
        let plot = app.models_mut().insert(Plot3dModel {
            viewport: Plot3dViewport {
                target: RenderTargetId::default(),
                target_px_size: (960, 540),
                fit: fret_core::ViewportFit::Contain,
                opacity: 1.0,
            },
        });

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Plot3dDemoWindowState {
            ui,
            root: None,
            plot,
            target: None,
        }
    }

    fn ensure_target(
        app: &mut App,
        window: AppWindowId,
        state: &mut Plot3dDemoWindowState,
        context: &WgpuContext,
        renderer: &mut Renderer,
    ) -> (RenderTargetId, wgpu::TextureView) {
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

            let texture = context.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("plot3d demo target"),
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

            let view_for_registry = texture.create_view(&wgpu::TextureViewDescriptor::default());

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

            state.target = Some(Plot3dDemoTarget { id, size, texture });

            let _ = state.plot.update(app, |m, _cx| {
                m.viewport.target = id;
                m.viewport.target_px_size = size;
            });

            app.request_redraw(window);
        }

        let target = state.target.as_ref().expect("target ensured");
        let view = target
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        (target.id, view)
    }
}

impl WinitAppDriver for Plot3dDemoDriver {
    type WindowState = Plot3dDemoWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
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
            _ => {
                state.ui.dispatch_event(app, services, event);
            }
        }
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
        let (_id, view) = Self::ensure_target(app, window, state, context, renderer);

        // Simple animated clear to prove end-to-end viewport surfaces + engine hook work.
        let t = (frame_id.0 as f32 * 0.02).sin() * 0.5 + 0.5;
        let clear = wgpu::Color {
            r: (0.08 + 0.25 * t) as f64,
            g: (0.08 + 0.18 * (1.0 - t)) as f64,
            b: (0.12 + 0.25 * (0.5 - (t - 0.5).abs())) as f64,
            a: 1.0,
        };

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("plot3d demo encoder"),
            });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("plot3d demo clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
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

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
    ) -> Option<fret_launch::WindowCreateSpec> {
        None
    }

    fn window_created(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
        _new_window: AppWindowId,
    ) {
    }
}

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo plot3d_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    Plot3dDemoDriver
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

    run_app(config, app, driver).context("run plot3d_demo app")
}
