#[cfg(not(target_arch = "wasm32"))]
use fret_app::{App, Effect, WindowRequest};
#[cfg(not(target_arch = "wasm32"))]
use fret_bootstrap::BootstrapBuilder;
#[cfg(not(target_arch = "wasm32"))]
use fret_core::{
    AppWindowId, Color, Corners, DrawOrder, Edges, Event, Paint, Px, Rect, SceneOp, Size,
};
#[cfg(not(target_arch = "wasm32"))]
use fret_launch::{WindowLogicalSize, WinitEventContext, WinitRenderContext};

#[cfg(not(target_arch = "wasm32"))]
fn create_window_state(_driver: &mut (), _app: &mut App, _window: AppWindowId) {}

#[cfg(not(target_arch = "wasm32"))]
fn handle_event(_driver: &mut (), context: WinitEventContext<'_, ()>, event: &Event) {
    let WinitEventContext { app, window, .. } = context;

    match event {
        Event::WindowCloseRequested
        | Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            ..
        } => {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
        }
        _ => {}
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn render(_driver: &mut (), context: WinitRenderContext<'_, ()>) {
    let WinitRenderContext { bounds, scene, .. } = context;

    scene.clear();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(
            bounds.origin,
            Size::new(
                Px(bounds.size.width.0.max(1.0)),
                Px(bounds.size.height.0.max(1.0)),
            ),
        ),
        background: Paint::Solid(Color {
            r: 0.10,
            g: 0.12,
            b: 0.16,
            a: 1.0,
        })
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let builder =
        BootstrapBuilder::new_fn(App::new(), (), create_window_state, handle_event, render)
            .configure(|config| {
                config.main_window_title = "fn_driver_escape_hatch".to_string();
                config.main_window_size = WindowLogicalSize::new(640.0, 400.0);
            });

    builder.run()?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
