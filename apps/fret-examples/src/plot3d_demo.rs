use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event, RenderTargetId};
use fret_launch::{
    EngineFrameUpdate, FnDriver, ViewportRenderTarget, WinitAppDriver, WinitEventContext,
    WinitHotReloadContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_plot3d::retained::{Plot3dCanvas, Plot3dModel, Plot3dStyle, Plot3dViewport};
use fret_render::{RenderTargetColorSpace, Renderer, WgpuContext};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;

struct Plot3dDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: fret_runtime::Model<Plot3dModel>,
    target: ViewportRenderTarget,
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
            target: ViewportRenderTarget::new(
                wgpu::TextureFormat::Bgra8UnormSrgb,
                RenderTargetColorSpace::Srgb,
            ),
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

        let prev_id = state.target.id();
        let prev_size = state.target.size();
        let (id, view) = {
            let (id, view_ref) = state.target.ensure_size(
                context,
                renderer,
                desired_size,
                Some("plot3d demo target"),
            );
            (id, view_ref.clone())
        };
        let new_size = state.target.size();

        if prev_id != id || prev_size != new_size {
            let _ = state.plot.update(app, |m, _cx| {
                m.viewport.target = id;
                m.viewport.target_px_size = new_size;
            });
            app.request_redraw(window);
        }

        (id, view)
    }
}

fn create_window_state(
    _driver: &mut Plot3dDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> Plot3dDemoWindowState {
    Plot3dDemoDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut Plot3dDemoDriver,
    context: WinitHotReloadContext<'_, Plot3dDemoWindowState>,
) {
    let WinitHotReloadContext {
        app, window, state, ..
    } = context;

    crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
    state.root = None;
}

fn handle_event(
    _driver: &mut Plot3dDemoDriver,
    context: WinitEventContext<'_, Plot3dDemoWindowState>,
    event: &Event,
) {
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
    _driver: &mut Plot3dDemoDriver,
    app: &mut App,
    window: AppWindowId,
    state: &mut Plot3dDemoWindowState,
    context: &WgpuContext,
    renderer: &mut Renderer,
    _scale_factor: f32,
    _tick_id: fret_runtime::TickId,
    frame_id: fret_runtime::FrameId,
) -> EngineFrameUpdate {
    let (_id, view) = Plot3dDemoDriver::ensure_target(app, window, state, context, renderer);

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
        keepalive: Vec::new(),
    }
}

fn render(_driver: &mut Plot3dDemoDriver, context: WinitRenderContext<'_, Plot3dDemoWindowState>) {
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

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo plot3d_demo".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    FnDriver::new(Plot3dDemoDriver, create_window_state, handle_event, render).with_hooks(|hooks| {
        hooks.hot_reload_window = Some(hot_reload_window);
        hooks.record_engine_frame = Some(record_engine_frame);
    })
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

    crate::run_native_with_compat_driver(config, app, driver).context("run plot3d_demo app")
}
