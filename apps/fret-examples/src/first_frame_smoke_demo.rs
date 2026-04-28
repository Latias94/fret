use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Corners, Edges, Px, Rect, SceneOp, Size};
use fret_launch::{
    FnDriver, WindowCreateSpec, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
};

#[derive(Default)]
pub struct FirstFrameSmokeDriver;

pub struct FirstFrameSmokeWindowState {
    frames_drawn: u32,
    close_requested: bool,
}

fn create_window_state(
    _driver: &mut FirstFrameSmokeDriver,
    _app: &mut App,
    _window: AppWindowId,
) -> FirstFrameSmokeWindowState {
    if std::env::var_os("FRET_FIRST_FRAME_SMOKE_LOG").is_some() {
        eprintln!("first_frame_smoke_demo: create_window_state window={_window:?}");
    }
    FirstFrameSmokeWindowState {
        frames_drawn: 0,
        close_requested: false,
    }
}

fn handle_event(
    _driver: &mut FirstFrameSmokeDriver,
    context: WinitEventContext<'_, FirstFrameSmokeWindowState>,
    event: &fret_core::Event,
) {
    if std::env::var_os("FRET_FIRST_FRAME_SMOKE_LOG").is_some() {
        eprintln!(
            "first_frame_smoke_demo: event window={:?} frames_drawn={} event={event:?}",
            context.window, context.state.frames_drawn
        );
    }
}

fn render(
    _driver: &mut FirstFrameSmokeDriver,
    context: WinitRenderContext<'_, FirstFrameSmokeWindowState>,
) {
    let WinitRenderContext {
        app,
        window,
        state,
        bounds,
        scene,
        ..
    } = context;
    scene.clear();
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(0),
        rect: Rect::new(
            bounds.origin,
            Size::new(
                Px(bounds.size.width.0.max(1.0)),
                Px(bounds.size.height.0.max(1.0)),
            ),
        ),
        background: fret_core::Paint::Solid(fret_core::Color {
            a: 1.0,
            ..fret_core::Color::from_srgb_hex_rgb(0x1f_26_38)
        })
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    });

    state.frames_drawn = state.frames_drawn.saturating_add(1);
    if std::env::var_os("FRET_FIRST_FRAME_SMOKE_LOG").is_some() {
        eprintln!(
            "first_frame_smoke_demo: render window={window:?} frame={}",
            state.frames_drawn
        );
    }

    if state.frames_drawn < 3 {
        app.push_effect(Effect::RequestAnimationFrame(window));
        return;
    }

    if !state.close_requested {
        state.close_requested = true;
        app.push_effect(Effect::Window(WindowRequest::Close(window)));
    }
}

fn window_create_spec(
    _driver: &mut FirstFrameSmokeDriver,
    _app: &mut App,
    _request: &fret_app::CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    None
}

fn configure_fn_driver_hooks(
    hooks: &mut fret_launch::FnDriverHooks<FirstFrameSmokeDriver, FirstFrameSmokeWindowState>,
) {
    hooks.window_create_spec = Some(window_create_spec);
}

pub fn build_app() -> App {
    App::new()
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "first_frame_smoke_demo".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(520.0, 200.0),
        ..Default::default()
    }
}

pub fn build_fn_driver() -> FnDriver<FirstFrameSmokeDriver, FirstFrameSmokeWindowState> {
    FnDriver::new(
        FirstFrameSmokeDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
    .with_hooks(configure_fn_driver_hooks)
}

pub fn run() -> anyhow::Result<()> {
    let app = build_app();
    let config = build_runner_config();
    crate::run_native_with_fn_driver_with_hooks(
        config,
        app,
        FirstFrameSmokeDriver::default(),
        create_window_state,
        handle_event,
        render,
        configure_fn_driver_hooks,
    )
}
