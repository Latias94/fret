use fret_app::{App, Effect, WindowRequest};
use fret_bootstrap::BootstrapBuilder;
use fret_core::{
    AlphaMode, AppWindowId, ChromaSiting, Color, ColorPrimaries, ColorRange, Corners, DrawOrder,
    Edges, Event, ImageColorInfo, ImageEncoding, ImageId, Px, Rect, SceneOp, Size,
    TransferFunction, YuvMatrix,
};
use fret_launch::{FnDriver, WinitEventContext, WinitRenderContext};

struct StreamingI420DemoState {
    image: Option<ImageId>,
    pending_token: Option<fret_core::ImageUploadToken>,
    image_size: (u32, u32),
    frame: u64,
    close_requested: bool,
    perf_every: u64,
    auto_exit_frames: Option<u64>,
}

fn env_u64(name: &str) -> Option<u64> {
    std::env::var(name).ok()?.parse().ok()
}

fn generate_solid_rgba8(width: u32, height: u32, rgba: (u8, u8, u8, u8)) -> Vec<u8> {
    let (r, g, b, a) = rgba;
    let mut out = vec![
        0u8;
        (width as usize)
            .saturating_mul(height as usize)
            .saturating_mul(4)
    ];
    for px in out.chunks_exact_mut(4) {
        px[0] = r;
        px[1] = g;
        px[2] = b;
        px[3] = a;
    }
    out
}

fn yuv_info() -> ImageColorInfo {
    ImageColorInfo {
        encoding: ImageEncoding::Srgb,
        range: ColorRange::Limited,
        matrix: YuvMatrix::Bt709,
        primaries: ColorPrimaries::Bt709,
        transfer: TransferFunction::Bt709,
        chroma_siting: Some(ChromaSiting::Center),
    }
}

fn fill_i420_frame(size: (u32, u32), frame: u64) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let (width, height) = size;
    let (cw, ch) = ((width + 1) / 2, (height + 1) / 2);

    let mut y_plane = vec![0u8; (width as usize).saturating_mul(height as usize)];
    let mut u_plane = vec![0u8; (cw as usize).saturating_mul(ch as usize)];
    let mut v_plane = vec![0u8; (cw as usize).saturating_mul(ch as usize)];

    for y in 0..height {
        let row = (y as usize).saturating_mul(width as usize);
        for x in 0..width {
            let v = ((x as u64).saturating_add(frame * 2) ^ (y as u64 * 11)) & 0xFF;
            let yv = 16u8.saturating_add(((v as u32) % 220) as u8);
            y_plane[row + x as usize] = yv;
        }
    }

    let t = (frame % 180) as i32;
    let u_base = (128 + (t - 90) / 2).clamp(16, 240) as u8;
    let v_base = (128 - (t - 90) / 3).clamp(16, 240) as u8;

    for y in 0..ch {
        let row = (y as usize).saturating_mul(cw as usize);
        for x in 0..cw {
            let idx = row + x as usize;
            u_plane[idx] = u_base;
            v_plane[idx] = v_base;
        }
    }

    (y_plane, u_plane, v_plane)
}

fn create_window_state(
    _driver: &mut (),
    app: &mut App,
    window: AppWindowId,
) -> StreamingI420DemoState {
    let image_size = (256, 144);
    let bytes = generate_solid_rgba8(image_size.0, image_size.1, (0, 0, 0, 255));
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

    StreamingI420DemoState {
        image: None,
        pending_token: Some(token),
        image_size,
        frame: 0,
        close_requested: false,
        perf_every: env_u64("FRET_DEMO_STREAMING_PERF_EVERY")
            .unwrap_or(60)
            .max(1),
        auto_exit_frames: env_u64("FRET_DEMO_AUTO_EXIT_FRAMES"),
    }
}

fn handle_event(
    _driver: &mut (),
    context: WinitEventContext<'_, StreamingI420DemoState>,
    event: &Event,
) {
    let WinitEventContext {
        app, window, state, ..
    } = context;

    match event {
        Event::ImageRegistered { token, image, .. } => {
            if state.pending_token == Some(*token) {
                state.pending_token = None;
                state.image = Some(*image);
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

fn render(_driver: &mut (), context: WinitRenderContext<'_, StreamingI420DemoState>) {
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
            r: 0.10,
            g: 0.11,
            b: 0.13,
            a: 1.0,
        },
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });

    let rect = Rect::new(
        fret_core::Point::new(Px(24.0), Px(24.0)),
        Size::new(Px(state.image_size.0 as f32), Px(state.image_size.1 as f32)),
    );

    if let Some(image) = state.image {
        if state.frame % state.perf_every == 0 {
            if let Some(snapshot) = app.global::<fret_core::StreamingUploadPerfSnapshot>() {
                println!(
                    "streaming_perf frame={} seen={} applied={} upload_budgeted={} upload_applied={} pending={} yuv_us={} yuv_out_bytes={}",
                    state.frame,
                    snapshot.update_effects_seen,
                    snapshot.update_effects_applied,
                    snapshot.upload_bytes_budgeted,
                    snapshot.upload_bytes_applied,
                    snapshot.pending_updates,
                    snapshot.yuv_convert_us,
                    snapshot.yuv_convert_output_bytes
                );
            }
        }

        if let Some(limit) = state.auto_exit_frames
            && state.frame >= limit
            && !state.close_requested
        {
            state.close_requested = true;
            if let Some(image) = state.image.take() {
                app.push_effect(Effect::ImageUnregister { image });
            }
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        scene.push(SceneOp::Image {
            order: DrawOrder(1),
            rect,
            image,
            fit: fret_core::ViewportFit::Stretch,
            opacity: 1.0,
        });

        let (y_plane, u_plane, v_plane) = fill_i420_frame(state.image_size, state.frame);
        let cw = (state.image_size.0 + 1) / 2;

        app.push_effect(Effect::ImageUpdateI420 {
            window: Some(window),
            token: fret_runtime::ImageUpdateToken(state.frame),
            image,
            stream_generation: 0,
            width: state.image_size.0,
            height: state.image_size.1,
            update_rect_px: None,
            y_bytes_per_row: state.image_size.0,
            y_plane,
            u_bytes_per_row: cw,
            u_plane,
            v_bytes_per_row: cw,
            v_plane,
            color_info: yuv_info(),
            alpha_mode: AlphaMode::Opaque,
        });

        app.push_effect(Effect::RequestAnimationFrame(window));
        state.frame = state.frame.saturating_add(1);
    } else {
        app.push_effect(Effect::RequestAnimationFrame(window));
    }
}

pub fn run() -> anyhow::Result<()> {
    let driver = FnDriver::new((), create_window_state, handle_event, render);

    let builder = BootstrapBuilder::new(App::new(), driver).configure(|config| {
        config.main_window_title = "streaming_i420_demo".to_string();
        config.main_window_size = winit::dpi::LogicalSize::new(720.0, 480.0);
        config.streaming_nv12_gpu_convert_enabled = true;
    });

    let builder = builder.with_default_config_files()?;
    builder.run().map_err(anyhow::Error::from)
}
