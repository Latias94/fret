use fret_app::{App, Effect};
use fret_core::{
    AlphaMode, AppWindowId, Color, Corners, DrawOrder, Edges, Event, ImageColorInfo, ImageId, Px,
    Rect, SceneOp, Size,
};
use fret_launch::{FnDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig};

struct AlphaModeDemoState {
    pending_straight: Option<fret_core::ImageUploadToken>,
    pending_premul: Option<fret_core::ImageUploadToken>,
    straight_image: Option<ImageId>,
    premul_image: Option<ImageId>,
    image_size: (u32, u32),
    close_requested: bool,
}

fn generate_straight_alpha_gradient_rgba8(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![
        0u8;
        (width as usize)
            .saturating_mul(height as usize)
            .saturating_mul(4)
    ];
    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let a = ((x.saturating_mul(255)).saturating_div(width.saturating_sub(1).max(1))) as u8;
            out[idx] = 255;
            out[idx + 1] = 60;
            out[idx + 2] = 40;
            out[idx + 3] = a;
        }
    }
    out
}

fn premultiply_rgba8_in_place(bytes: &mut [u8]) {
    for px in bytes.chunks_exact_mut(4) {
        let a = px[3] as u32;
        px[0] = ((px[0] as u32 * a + 127) / 255) as u8;
        px[1] = ((px[1] as u32 * a + 127) / 255) as u8;
        px[2] = ((px[2] as u32 * a + 127) / 255) as u8;
    }
}

fn create_window_state(_driver: &mut (), app: &mut App, window: AppWindowId) -> AlphaModeDemoState {
    let image_size = (320, 160);

    let straight_bytes = generate_straight_alpha_gradient_rgba8(image_size.0, image_size.1);
    let mut premul_bytes = straight_bytes.clone();
    premultiply_rgba8_in_place(&mut premul_bytes);

    let straight_token = app.next_image_upload_token();
    app.push_effect(Effect::ImageRegisterRgba8 {
        window,
        token: straight_token,
        width: image_size.0,
        height: image_size.1,
        bytes: straight_bytes,
        color_info: ImageColorInfo::srgb_rgba(),
        alpha_mode: AlphaMode::Straight,
    });

    let premul_token = app.next_image_upload_token();
    app.push_effect(Effect::ImageRegisterRgba8 {
        window,
        token: premul_token,
        width: image_size.0,
        height: image_size.1,
        bytes: premul_bytes,
        color_info: ImageColorInfo::srgb_rgba(),
        alpha_mode: AlphaMode::Premultiplied,
    });

    AlphaModeDemoState {
        pending_straight: Some(straight_token),
        pending_premul: Some(premul_token),
        straight_image: None,
        premul_image: None,
        image_size,
        close_requested: false,
    }
}

fn handle_event(
    _driver: &mut (),
    context: WinitEventContext<'_, AlphaModeDemoState>,
    event: &Event,
) {
    let WinitEventContext {
        app, window, state, ..
    } = context;

    match event {
        Event::ImageRegistered { token, image, .. } => {
            if state.pending_straight == Some(*token) {
                state.pending_straight = None;
                state.straight_image = Some(*image);
            }
            if state.pending_premul == Some(*token) {
                state.pending_premul = None;
                state.premul_image = Some(*image);
            }
        }
        Event::ImageRegisterFailed { token, message } => {
            if state.pending_straight == Some(*token) || state.pending_premul == Some(*token) {
                state.pending_straight = None;
                state.pending_premul = None;
                tracing::error!(message, "image register failed");
                state.close_requested = true;
                app.push_effect(fret_runtime::Effect::Window(
                    fret_runtime::WindowRequest::Close(window),
                ));
            }
        }
        Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            ..
        } => {
            if !state.close_requested {
                state.close_requested = true;
                if let Some(image) = state.straight_image.take() {
                    app.push_effect(Effect::ImageUnregister { image });
                }
                if let Some(image) = state.premul_image.take() {
                    app.push_effect(Effect::ImageUnregister { image });
                }
                app.push_effect(fret_runtime::Effect::Window(
                    fret_runtime::WindowRequest::Close(window),
                ));
            }
        }
        _ => {}
    }
}

fn render(_driver: &mut (), context: WinitRenderContext<'_, AlphaModeDemoState>) {
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
        background: Color {
            r: 0.08,
            g: 0.09,
            b: 0.10,
            a: 1.0,
        },
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });

    let card_size = Size::new(
        Px(state.image_size.0 as f32 + 32.0),
        Px(state.image_size.1 as f32 + 32.0),
    );
    let left = Rect::new(fret_core::Point::new(Px(24.0), Px(24.0)), card_size);
    let right = Rect::new(
        fret_core::Point::new(Px(24.0 + card_size.width.0 + 16.0), Px(24.0)),
        card_size,
    );

    for (order, rect) in [(DrawOrder(1), left), (DrawOrder(1), right)] {
        scene.push(SceneOp::Quad {
            order,
            rect,
            background: Color {
                r: 0.16,
                g: 0.16,
                b: 0.18,
                a: 1.0,
            },
            border: Edges::all(Px(1.0)),
            border_color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.10,
            },
            corner_radii: Corners::all(Px(12.0)),
        });
    }

    let image_rect = |card: Rect| {
        Rect::new(
            fret_core::Point::new(Px(card.origin.x.0 + 16.0), Px(card.origin.y.0 + 16.0)),
            Size::new(Px(state.image_size.0 as f32), Px(state.image_size.1 as f32)),
        )
    };

    if let Some(image) = state.straight_image {
        scene.push(SceneOp::Image {
            order: DrawOrder(2),
            rect: image_rect(left),
            image,
            fit: fret_core::ViewportFit::Stretch,
            opacity: 1.0,
        });
    }
    if let Some(image) = state.premul_image {
        scene.push(SceneOp::Image {
            order: DrawOrder(2),
            rect: image_rect(right),
            image,
            fit: fret_core::ViewportFit::Stretch,
            opacity: 1.0,
        });
    }

    // The two cards should look visually identical; if premultiplied sources are double-multiplied,
    // the right card will appear darker/washed out.
    app.push_effect(Effect::RequestAnimationFrame(window));
}

pub fn run() -> anyhow::Result<()> {
    let driver = FnDriver::new((), create_window_state, handle_event, render);
    fret_kit::run_native_demo(
        WinitRunnerConfig {
            main_window_title: "alpha_mode_demo".to_string(),
            main_window_size: winit::dpi::LogicalSize::new(920.0, 320.0),
            ..Default::default()
        },
        App::new(),
        driver,
    )
    .map_err(anyhow::Error::from)
}
