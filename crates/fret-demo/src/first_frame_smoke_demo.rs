use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Corners, Edges, Px, Rect, Scene, SceneOp, Size, UiServices};
use fret_runner_winit_wgpu::{
    RunnerUserEvent, WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext,
    WinitRunner, WinitRunnerConfig,
};
use winit::event_loop::EventLoop;

#[derive(Default)]
struct FirstFrameSmokeDriver;

struct FirstFrameSmokeWindowState {
    frames_drawn: u32,
    close_requested: bool,
}

impl WinitAppDriver for FirstFrameSmokeDriver {
    type WindowState = FirstFrameSmokeWindowState;

    fn create_window_state(&mut self, _app: &mut App, _window: AppWindowId) -> Self::WindowState {
        FirstFrameSmokeWindowState {
            frames_drawn: 0,
            close_requested: false,
        }
    }

    fn handle_event(&mut self, _context: WinitEventContext<'_, Self::WindowState>, _event: &fret_core::Event) {}

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
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
            background: fret_core::Color {
                r: 0.12,
                g: 0.15,
                b: 0.22,
                a: 1.0,
            },
            border: Edges::all(Px(0.0)),
            border_color: fret_core::Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });

        state.frames_drawn = state.frames_drawn.saturating_add(1);

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
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
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

pub fn run() -> anyhow::Result<()> {
    let event_loop = EventLoop::<RunnerUserEvent>::with_user_event().build()?;

    let config = WinitRunnerConfig {
        main_window_title: "first_frame_smoke_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(520.0, 200.0),
        ..WinitRunnerConfig::default()
    };

    let app = App::new();
    let driver = FirstFrameSmokeDriver::default();
    let mut runner = WinitRunner::new_app(config, app, driver);

    event_loop
        .run_app(&mut runner)
        .context("winit run_app failed")?;

    Ok(())
}
