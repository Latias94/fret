use fret_app::{App, Effect, WindowRequest};
use fret_bootstrap::BootstrapBuilder;
use fret_core::{
    AlphaMode, AppWindowId, Color, Corners, DrawOrder, Edges, Event, ImageColorInfo, Px, Rect,
    RectPx, SceneOp, Size,
};
use fret_launch::{FnDriver, WinitEventContext, WinitRenderContext};

struct StreamingImageDemoState {
    image: Option<fret_core::ImageId>,
    pending_token: Option<fret_core::ImageUploadToken>,
    image_size: (u32, u32),
    frame: u64,
    close_requested: bool,
}

fn generate_rgba8_checkerboard(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let cell = ((x / 16) ^ (y / 16)) & 1;
            let (r, g, b) = if cell == 0 {
                (20u8, 30u8, 60u8)
            } else {
                (40u8, 60u8, 110u8)
            };
            out[idx] = r;
            out[idx + 1] = g;
            out[idx + 2] = b;
            out[idx + 3] = 255;
        }
    }
    out
}

fn generate_solid_rgba8(width: u32, height: u32, rgba: (u8, u8, u8, u8)) -> Vec<u8> {
    let (r, g, b, a) = rgba;
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    for px in out.chunks_exact_mut(4) {
        px[0] = r;
        px[1] = g;
        px[2] = b;
        px[3] = a;
    }
    out
}

fn create_window_state(
    _driver: &mut (),
    app: &mut App,
    window: AppWindowId,
) -> StreamingImageDemoState {
    let image_size = (320, 200);
    let bytes = generate_rgba8_checkerboard(image_size.0, image_size.1);
    let token = app.next_image_upload_token();
    app.push_effect(Effect::ImageRegisterRgba8 {
        window,
        token,
        width: image_size.0,
        height: image_size.1,
        bytes,
        color_info: ImageColorInfo::srgb_rgba(),
        alpha_mode: AlphaMode::Opaque,
    });

    StreamingImageDemoState {
        image: None,
        pending_token: Some(token),
        image_size,
        frame: 0,
        close_requested: false,
    }
}

fn handle_event(
    _driver: &mut (),
    context: WinitEventContext<'_, StreamingImageDemoState>,
    event: &Event,
) {
    let WinitEventContext {
        app, window, state, ..
    } = context;

    match event {
        Event::ImageRegistered {
            token,
            image,
            width,
            height,
        } => {
            if state.pending_token == Some(*token) {
                state.pending_token = None;
                state.image = Some(*image);
                state.image_size = (*width, *height);
            }
        }
        Event::ImageRegisterFailed { token, message } => {
            if state.pending_token == Some(*token) {
                state.pending_token = None;
                tracing::error!(message, "image register failed");
                state.close_requested = true;
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
            }
        }
        Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            ..
        } => {
            if !state.close_requested {
                state.close_requested = true;
                if let Some(image) = state.image.take() {
                    app.push_effect(Effect::ImageUnregister { image });
                }
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
            }
        }
        _ => {}
    }
}

fn render(_driver: &mut (), context: WinitRenderContext<'_, StreamingImageDemoState>) {
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
        order: DrawOrder(0),
        rect: Rect::new(
            bounds.origin,
            Size::new(
                Px(bounds.size.width.0.max(1.0)),
                Px(bounds.size.height.0.max(1.0)),
            ),
        ),
        background: fret_core::Paint::Solid(Color {
            r: 0.10,
            g: 0.11,
            b: 0.13,
            a: 1.0,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT,

        corner_radii: Corners::all(Px(0.0)),
    });

    let rect = Rect::new(
        fret_core::Point::new(Px(24.0), Px(24.0)),
        Size::new(Px(state.image_size.0 as f32), Px(state.image_size.1 as f32)),
    );

    if let Some(image) = state.image {
        scene.push(SceneOp::Image {
            order: DrawOrder(1),
            rect,
            image,
            fit: fret_core::ViewportFit::Stretch,
            opacity: 1.0,
        });

        let bar_w = 24u32;
        let max_x = state.image_size.0.saturating_sub(bar_w).max(1);
        let bar_x = (state.frame as u32) % max_x;

        let bytes = generate_solid_rgba8(bar_w, state.image_size.1, (240, 90, 80, 255));
        app.push_effect(Effect::ImageUpdateRgba8 {
            window: Some(window),
            token: fret_runtime::ImageUpdateToken(state.frame),
            image,
            stream_generation: 0,
            width: state.image_size.0,
            height: state.image_size.1,
            update_rect_px: Some(RectPx::new(bar_x, 0, bar_w, state.image_size.1)),
            bytes_per_row: bar_w * 4,
            bytes,
            color_info: ImageColorInfo::srgb_rgba(),
            alpha_mode: AlphaMode::Opaque,
        });

        app.push_effect(Effect::RequestAnimationFrame(window));
        state.frame = state.frame.saturating_add(1);
    } else {
        scene.push(SceneOp::Quad {
            order: DrawOrder(1),
            rect,
            background: fret_core::Paint::Solid(Color {
                r: 0.18,
                g: 0.18,
                b: 0.20,
                a: 1.0,
            }),
            border: Edges::all(Px(1.0)),
            border_paint: fret_core::Paint::Solid(Color {
                r: 0.35,
                g: 0.35,
                b: 0.40,
                a: 1.0,
            }),
            corner_radii: Corners::all(Px(6.0)),
        });
        app.push_effect(Effect::RequestAnimationFrame(window));
    }
}

pub fn run() -> anyhow::Result<()> {
    let driver = FnDriver::new((), create_window_state, handle_event, render);

    let builder = BootstrapBuilder::new(App::new(), driver).configure(|config| {
        config.main_window_title = "streaming_image_demo".to_string();
        config.main_window_size = winit::dpi::LogicalSize::new(720.0, 480.0);
    });

    let builder = builder.with_default_config_files()?;
    builder.run().map_err(anyhow::Error::from)
}
