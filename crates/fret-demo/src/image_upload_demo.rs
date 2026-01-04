use fret_app::{App, Effect, WindowRequest};
use fret_app_kit::image_asset_cache::{ImageAssetCacheHostExt, ImageAssetKey};
use fret_core::{
    AppWindowId, Color, Corners, DrawOrder, Edges, Event, ImageColorSpace, Px, Rect, SceneOp, Size,
};
use fret_runner_winit_wgpu::{
    WindowCreateSpec, WinitAppBuilder, WinitAppDriver, WinitEventContext, WinitRenderContext,
};

#[derive(Default)]
struct ImageUploadDemoDriver;

struct ImageUploadDemoWindowState {
    image_bytes: Vec<u8>,
    image_key: ImageAssetKey,
    image: Option<fret_core::ImageId>,
    image_size: (u32, u32),
    close_requested: bool,
}

impl ImageUploadDemoDriver {
    fn generate_rgba8_checkerboard(width: u32, height: u32) -> Vec<u8> {
        let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
        for y in 0..height {
            for x in 0..width {
                let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
                let cell = ((x / 16) ^ (y / 16)) & 1;
                let (r, g, b) = if cell == 0 {
                    (48u8, 56u8, 78u8)
                } else {
                    (92u8, 102u8, 128u8)
                };
                out[idx] = r;
                out[idx + 1] = g;
                out[idx + 2] = b;
                out[idx + 3] = 255;
            }
        }
        out
    }

    fn use_image_asset(
        app: &mut App,
        window: AppWindowId,
        key: ImageAssetKey,
        size: (u32, u32),
        bytes: &[u8],
    ) -> Option<fret_core::ImageId> {
        app.with_image_asset_cache(|cache, app| {
            cache.use_rgba8_keyed(
                app,
                window,
                key,
                size.0,
                size.1,
                bytes,
                ImageColorSpace::Srgb,
            )
        })
    }
}

impl WinitAppDriver for ImageUploadDemoDriver {
    type WindowState = ImageUploadDemoWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        let size = (256, 256);
        let bytes = Self::generate_rgba8_checkerboard(size.0, size.1);
        let key = ImageAssetKey::from_rgba8(size.0, size.1, ImageColorSpace::Srgb, &bytes);
        let image = Self::use_image_asset(app, window, key, size, &bytes);
        ImageUploadDemoWindowState {
            image_bytes: bytes,
            image_key: key,
            image,
            image_size: size,
            close_requested: false,
        }
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app, window, state, ..
        } = context;

        app.with_image_asset_cache(|cache, app| {
            cache.handle_event(app, window, event);
        });

        match event {
            Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                ..
            } => {
                if !state.close_requested {
                    state.close_requested = true;
                    app.push_effect(Effect::Window(WindowRequest::Close(window)));
                }
            }
            Event::KeyDown {
                key: fret_core::KeyCode::Space,
                ..
            } => {
                app.with_image_asset_cache(|cache, app| {
                    let _ = cache.evict(app, state.image_key);
                });
                state.image = None;
            }
            _ => {}
        }
    }

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

        state.image = Self::use_image_asset(
            app,
            window,
            state.image_key,
            state.image_size,
            &state.image_bytes,
        );

        if let Some(image) = state.image {
            scene.push(SceneOp::Image {
                order: DrawOrder(1),
                rect,
                image,
                opacity: 1.0,
            });
        } else {
            // Show a placeholder box while the upload is pending.
            scene.push(SceneOp::Quad {
                order: DrawOrder(1),
                rect,
                background: Color {
                    r: 0.18,
                    g: 0.18,
                    b: 0.20,
                    a: 1.0,
                },
                border: Edges::all(Px(1.0)),
                border_color: Color {
                    r: 0.35,
                    g: 0.35,
                    b: 0.40,
                    a: 1.0,
                },
                corner_radii: Corners::all(Px(6.0)),
            });
            app.push_effect(Effect::RequestAnimationFrame(window));
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
    WinitAppBuilder::new(App::new(), ImageUploadDemoDriver::default())
        .configure(|config| {
            config.main_window_title = "image_upload_demo".to_string();
            config.main_window_size = winit::dpi::LogicalSize::new(520.0, 380.0);
        })
        .run()
        .map_err(anyhow::Error::from)
}
