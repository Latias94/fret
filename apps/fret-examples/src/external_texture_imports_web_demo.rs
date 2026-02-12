//! Web-only external texture imports demo.
//!
//! This is a staged "real backend path" for ADR 0234 on wasm32:
//! - It uses browser-provided surfaces (HTML canvas + image decode) as an external image source.
//! - Each frame, it copies that external source into a renderer-owned `wgpu::Texture`.
//! - The UI embeds the result via `RenderTargetId` + `ViewportSurface`.
//!
//! Notes:
//! - This is a GPU copy path (not a zero-copy `ExternalTexture` import yet).
//! - The `ImportedViewportRenderTarget` helper records updates as runner deltas.

use anyhow::Context as _;
use fret_app::{App, Effect};
use fret_core::scene::Paint;
use fret_core::{AppWindowId, Event, KeyCode, Px};
use fret_launch::{
    EngineFrameUpdate, ImportedViewportRenderTarget, WinitAppDriver, WinitEventContext,
    WinitRenderContext, WinitRunnerConfig,
};
use fret_render::{RenderTargetColorSpace, Renderer, WgpuContext};
use fret_runtime::PlatformCapabilities;
use fret_ui::declarative;
use fret_ui::element::{
    ContainerProps, CrossAlign, Elements, FlexProps, LayoutStyle, Length, MainAlign,
    ViewportSurfaceProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiTree};

#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::JsCast as _;
#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

#[cfg(target_arch = "wasm32")]
const DEMO_IMAGE_DATA_URL: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMB/6XGDp8AAAAASUVORK5CYII=";

#[cfg(target_arch = "wasm32")]
struct WebExternalCanvasSource {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    image: HtmlImageElement,
    image_loaded: std::rc::Rc<std::cell::Cell<bool>>,
    _onload: Closure<dyn FnMut()>,
}

#[cfg(target_arch = "wasm32")]
impl WebExternalCanvasSource {
    fn new(px_size: (u32, u32)) -> anyhow::Result<Self> {
        let Some(window) = web_sys::window() else {
            anyhow::bail!("missing web window");
        };
        let document = window.document().context("missing document")?;

        let canvas: HtmlCanvasElement = document
            .create_element("canvas")
            .map_err(|e| anyhow::anyhow!("create canvas element failed: {e:?}"))?
            .dyn_into()
            .map_err(|e| anyhow::anyhow!("canvas dyn_into failed: {e:?}"))?;
        canvas.set_width(px_size.0.max(1));
        canvas.set_height(px_size.1.max(1));

        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .map_err(|e| anyhow::anyhow!("canvas get_context failed: {e:?}"))?
            .context("missing 2d context")?
            .dyn_into()
            .map_err(|e| anyhow::anyhow!("2d ctx dyn_into failed: {e:?}"))?;

        let image: HtmlImageElement = document
            .create_element("img")
            .map_err(|e| anyhow::anyhow!("create img element failed: {e:?}"))?
            .dyn_into()
            .map_err(|e| anyhow::anyhow!("img dyn_into failed: {e:?}"))?;
        image.set_src(DEMO_IMAGE_DATA_URL);

        let image_loaded = std::rc::Rc::new(std::cell::Cell::new(false));
        let loaded = image_loaded.clone();
        let onload = Closure::wrap(Box::new(move || {
            loaded.set(true);
        }) as Box<dyn FnMut()>);
        image.set_onload(Some(onload.as_ref().unchecked_ref()));

        Ok(Self {
            canvas,
            ctx,
            image,
            image_loaded,
            _onload: onload,
        })
    }

    fn draw(&self, secs: f64) {
        let w = self.canvas.width() as f64;
        let h = self.canvas.height() as f64;

        self.ctx.set_fill_style_str("#0b1020");
        self.ctx.fill_rect(0.0, 0.0, w, h);

        if self.image_loaded.get() {
            let _ = self.ctx.draw_image_with_html_image_element_and_dw_and_dh(
                &self.image,
                0.0,
                0.0,
                w,
                h,
            );
        }

        let cx = w * 0.5 + (secs * 0.9).cos() * (w * 0.25);
        let cy = h * 0.5 + (secs * 0.7).sin() * (h * 0.25);
        let r = (w.min(h) * 0.12).max(12.0);

        self.ctx.begin_path();
        let _ = self.ctx.arc(cx, cy, r, 0.0, std::f64::consts::TAU);
        self.ctx.set_fill_style_str("rgba(120, 220, 255, 0.35)");
        let _ = self.ctx.fill();
    }
}

struct ExternalTextureImportsWebWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,

    show: fret_runtime::Model<bool>,

    target: ImportedViewportRenderTarget,
    target_px_size: (u32, u32),

    texture: Option<wgpu::Texture>,
    view: Option<wgpu::TextureView>,

    #[cfg(target_arch = "wasm32")]
    external: Option<WebExternalCanvasSource>,
}

#[derive(Default)]
struct ExternalTextureImportsWebDriver;

impl ExternalTextureImportsWebDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> ExternalTextureImportsWebWindowState {
        let show = app.models_mut().insert(true);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        ExternalTextureImportsWebWindowState {
            ui,
            root: None,
            show,
            target: ImportedViewportRenderTarget::new(
                wgpu::TextureFormat::Bgra8UnormSrgb,
                RenderTargetColorSpace::Srgb,
            ),
            target_px_size: (1280, 720),
            texture: None,
            view: None,
            #[cfg(target_arch = "wasm32")]
            external: None,
        }
    }

    fn ensure_target_registered(
        state: &mut ExternalTextureImportsWebWindowState,
        context: &WgpuContext,
        renderer: &mut Renderer,
    ) {
        if state.texture.is_none() {
            let px_size = state.target_px_size;
            let texture = context.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("external texture imports web demo texture"),
                size: wgpu::Extent3d {
                    width: px_size.0.max(1),
                    height: px_size.1.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: state.target.format(),
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            state.texture = Some(texture);
            state.view = Some(view);

            #[cfg(target_arch = "wasm32")]
            if let Ok(ext) = WebExternalCanvasSource::new(px_size) {
                state.external = Some(ext);
            }
        }

        let Some(view) = state.view.as_ref() else {
            return;
        };
        let _ = state
            .target
            .ensure_registered(renderer, view.clone(), state.target_px_size);
    }

    fn record_external_source_into_texture(
        state: &mut ExternalTextureImportsWebWindowState,
        context: &WgpuContext,
        secs: f64,
    ) {
        #[cfg(target_arch = "wasm32")]
        {
            let (Some(ext), Some(texture)) = (state.external.as_ref(), state.texture.as_ref())
            else {
                return;
            };

            ext.draw(secs);

            let extent = wgpu::Extent3d {
                width: state.target_px_size.0.max(1),
                height: state.target_px_size.1.max(1),
                depth_or_array_layers: 1,
            };

            let src = wgpu::CopyExternalImageSourceInfo {
                source: wgpu::ExternalImageSource::HTMLCanvasElement(ext.canvas.clone()),
                origin: wgpu::Origin2d { x: 0, y: 0 },
                flip_y: false,
            };

            let dst = wgpu::CopyExternalImageDestInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
                color_space: wgpu::PredefinedColorSpace::Srgb,
                premultiplied_alpha: true,
            };

            context
                .queue
                .copy_external_image_to_texture(&src, dst, extent);
        }
    }

    fn render_root(
        cx: &mut ElementContext<'_, App>,
        show_model: fret_runtime::Model<bool>,
        target: fret_core::RenderTargetId,
        target_px_size: (u32, u32),
    ) -> Elements {
        cx.observe_model(&show_model, Invalidation::Layout);

        let show = cx.app.models().read(&show_model, |v| *v).unwrap_or(true);
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

        let make_panel = |cx: &mut ElementContext<'_, App>,
                          fit: fret_core::ViewportFit,
                          test_id: &'static str| {
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
                            make_panel(cx, fret_core::ViewportFit::Contain, "ext-tex-web-contain"),
                            make_panel(cx, fret_core::ViewportFit::Cover, "ext-tex-web-cover"),
                            make_panel(cx, fret_core::ViewportFit::Stretch, "ext-tex-web-stretch"),
                        ]
                    })]
                },
            )
            .test_id("external-texture-imports-web-root"),
        ]
        .into()
    }
}

impl WinitAppDriver for ExternalTextureImportsWebDriver {
    type WindowState = ExternalTextureImportsWebWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn gpu_ready(&mut self, app: &mut App, _context: &WgpuContext, _renderer: &mut Renderer) {
        app.set_global(PlatformCapabilities::default());
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
        _frame_id: fret_runtime::FrameId,
    ) -> EngineFrameUpdate {
        let show = app.models().read(&state.show, |v| *v).unwrap_or(true);
        let mut update = EngineFrameUpdate::default();

        if !show {
            state.target.push_unregister(&mut update);
            return update;
        }

        Self::ensure_target_registered(state, context, renderer);

        let secs = web_time_seconds();
        Self::record_external_source_into_texture(state, context, secs);

        if let Some(view) = state.view.as_ref() {
            state
                .target
                .push_update(&mut update, view.clone(), state.target_px_size);
        }

        app.push_effect(Effect::RequestAnimationFrame(window));
        update
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app, window, state, ..
        } = context;

        if let Event::KeyDown { key, .. } = event
            && *key == KeyCode::KeyV
        {
            let _ = app.models_mut().update(&state.show, |v| *v = !*v);
            app.request_redraw(window);
        }
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            ..
        } = context;

        let show = state.show.clone();
        let target = state.target.id();
        let target_px_size = state.target_px_size;

        let root =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("external-texture-imports-web", |cx| {
                    Self::render_root(cx, show.clone(), target, target_px_size)
                });

        state.ui.set_root(root);
        state.root = Some(root);
    }
}

fn web_time_seconds() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now() * 0.001)
            .unwrap_or(0.0)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0.0
    }
}

pub fn build_app() -> App {
    App::new()
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo external_texture_imports_web_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    ExternalTextureImportsWebDriver::default()
}
