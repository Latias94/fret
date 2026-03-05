use std::sync::Arc;

use fret::interop::embedded_viewport as embedded;
use fret::prelude::*;
use fret_core::ViewportFit;
use fret_launch::EngineFrameUpdate;
use fret_render::{RenderTargetColorSpace, Renderer, WgpuContext};
use fret_runtime::{FrameId, TickId};
use fret_ui::{ElementContext, UiTree};

const DEFAULT_VIEWPORT_PX_SIZE: (u32, u32) = (960, 540);

fn diag_enabled() -> bool {
    std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty() && v != "0")
}

mod act {
    fret::actions!([
        PickSize640 = "embedded_viewport_demo.pick_size.640.v1",
        PickSize960 = "embedded_viewport_demo.pick_size.960.v1",
        PickSize1280 = "embedded_viewport_demo.pick_size.1280.v1",
    ]);
}

struct EmbeddedViewportDemoView {
    embedded: embedded::EmbeddedViewportSurface,
    size_preset: Model<usize>,
}

impl View for EmbeddedViewportDemoView {
    fn init(app: &mut App, window: AppWindowId) -> Self {
        let models = embedded::ensure_models(app, window);
        let _ = app.models_mut().update(&models.last_input, |v| {
            *v = Arc::<str>::from("Click inside the viewport panel to see input forwarding.");
        });

        Self {
            embedded: embedded::EmbeddedViewportSurface::new(
                wgpu::TextureFormat::Bgra8UnormSrgb,
                RenderTargetColorSpace::Srgb,
                DEFAULT_VIEWPORT_PX_SIZE,
            ),
            size_preset: app.models_mut().insert(1usize),
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let window = cx.window;
        let theme = Theme::global(&*cx.app).snapshot();

        let models = embedded::models(&*cx.app, window)
            .unwrap_or_else(|| embedded::ensure_models(cx.app, window));

        let clicks = cx.watch_model(&models.clicks).paint().copied_or_default();
        let last_input: Arc<str> = cx
            .watch_model(&models.last_input)
            .paint()
            .cloned()
            .unwrap_or_else(|| Arc::from("<no input yet>"));

        let preset = cx
            .watch_model(&self.size_preset)
            .layout()
            .copied_or_default();
        let (target_px_size, preset_label): ((u32, u32), &'static str) = match preset {
            0 => ((640, 360), "640×360"),
            2 => ((1280, 720), "1280×720"),
            _ => (DEFAULT_VIEWPORT_PX_SIZE, "960×540"),
        };
        self.embedded.set_target_px_size(target_px_size);

        let header = ui::v_flex(|cx| {
            ui::children![
                cx;
                shadcn::CardTitle::new("Tier A interop: embedded viewport"),
                shadcn::CardDescription::new(
                    "Offscreen RenderTargetId + ViewportSurface + explicit input forwarding.",
                ),
            ]
        })
        .gap(Space::N1)
        .into_element(cx);

        let size_controls = ui::h_flex(|cx| {
            ui::children![
                cx;
                shadcn::Button::new("640×360")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .action(act::PickSize640)
                    .disabled(preset == 0),
                shadcn::Button::new("960×540")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .action(act::PickSize960)
                    .disabled(preset == 1),
                shadcn::Button::new("1280×720")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .action(act::PickSize1280)
                    .disabled(preset == 2),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let info = ui::v_flex(|cx| {
            ui::children![
                cx;
                ui::text(format!("Target: {preset_label}"))
                    .text_sm()
                    .into_element(cx),
                ui::text(format!("Clicks: {clicks}"))
                    .text_sm()
                    .into_element(cx),
                ui::text(format!("Last input: {last_input}"))
                    .text_sm()
                    .into_element(cx),
            ]
        })
        .gap(Space::N1)
        .into_element(cx);

        let viewport = self
            .embedded
            .panel(
                cx,
                embedded::EmbeddedViewportPanelProps {
                    fit: ViewportFit::Contain,
                    opacity: 1.0,
                    forward_input: true,
                },
            )
            .test_id("embedded-viewport-demo.surface");

        let viewport_card = shadcn::Card::new(ui::children![
            cx;
            shadcn::CardHeader::new(ui::children![cx; header, size_controls, info]),
            shadcn::CardContent::new(ui::children![cx;
                ui::container( |cx| ui::children![cx; viewport])
                    .bg(ColorRef::Color(theme.color_token("muted")))
                    .rounded(Radius::Md)
                    .border_1()
                    .border_color(ColorRef::Color(theme.color_token("border")))
                    .w_full()
                    .h_px(Px(420.0))
                    .into_element(cx),
            ]),
        ])
        .ui()
        .w_full()
        .max_w(Px(980.0))
        .into_element(cx);

        cx.on_action_notify::<act::PickSize640>({
            let size_preset = self.size_preset.clone();
            move |host, _acx| {
                let _ = host.models_mut().update(&size_preset, |v| *v = 0);
                true
            }
        });
        cx.on_action_notify::<act::PickSize960>({
            let size_preset = self.size_preset.clone();
            move |host, _acx| {
                let _ = host.models_mut().update(&size_preset, |v| *v = 1);
                true
            }
        });
        cx.on_action_notify::<act::PickSize1280>({
            let size_preset = self.size_preset.clone();
            move |host, _acx| {
                let _ = host.models_mut().update(&size_preset, |v| *v = 2);
                true
            }
        });

        let page = ui::container(|cx| {
            ui::children![
                cx;
                ui::v_flex( |cx| ui::children![cx; viewport_card])
                    .w_full()
                    .h_full()
                    .justify_center()
                    .items_center()
                    .into_element(cx),
            ]
        })
        .bg(ColorRef::Color(theme.color_token("background")))
        .p(Space::N6)
        .w_full()
        .h_full()
        .into_element(cx);

        if diag_enabled() {
            page.test_id("embedded-viewport-demo.root").into()
        } else {
            page.into()
        }
    }
}

struct EmbeddedViewportDemoWindowState {
    view: fret::view::ViewWindowState<EmbeddedViewportDemoView>,
}

impl embedded::EmbeddedViewportRecord for EmbeddedViewportDemoWindowState {
    fn embedded_viewport_surface(&mut self) -> &mut embedded::EmbeddedViewportSurface {
        &mut self.view.view.embedded
    }

    fn embedded_viewport_label(&self) -> Option<&'static str> {
        Some("embedded-viewport-demo")
    }

    fn record_embedded_viewport(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        _context: &WgpuContext,
        _renderer: &mut Renderer,
        _scale_factor: f32,
        _tick_id: TickId,
        frame_id: FrameId,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let clicks = embedded::models(app, window)
            .and_then(|m| app.models().read(&m.clicks, |v| *v).ok())
            .unwrap_or(0);

        // Visible feedback inside the viewport: change the clear color as clicks increase.
        let t = (frame_id.0 as f32 * 0.02).sin() * 0.5 + 0.5;
        let c = (clicks % 9) as f32 / 8.0;
        let clear = wgpu::Color {
            r: (0.06 + 0.70 * c) as f64,
            g: (0.08 + 0.25 * t) as f64,
            b: (0.10 + 0.35 * (1.0 - c)) as f64,
            a: 1.0,
        };
        embedded::clear_pass(encoder, view, Some("embedded viewport clear"), clear);
    }
}

fn init_window(app: &mut App, window: AppWindowId) -> EmbeddedViewportDemoWindowState {
    EmbeddedViewportDemoWindowState {
        view: fret::view::view_init_window::<EmbeddedViewportDemoView>(app, window),
    }
}

fn view(
    cx: &mut ElementContext<'_, App>,
    st: &mut EmbeddedViewportDemoWindowState,
) -> ViewElements {
    fret::view::view_view::<EmbeddedViewportDemoView>(cx, &mut st.view).into()
}

fn record_engine_frame(
    app: &mut App,
    window: AppWindowId,
    ui: &mut UiTree<App>,
    st: &mut EmbeddedViewportDemoWindowState,
    context: &WgpuContext,
    renderer: &mut Renderer,
    scale_factor: f32,
    tick_id: TickId,
    frame_id: FrameId,
) -> EngineFrameUpdate {
    // Preserve view runtime v1 behavior (view cache enablement) while also recording the embedded
    // viewport engine/offscreen pass.
    let _ = fret::view::view_record_engine_frame::<EmbeddedViewportDemoView>(
        app,
        window,
        ui,
        &mut st.view,
        context,
        renderer,
        scale_factor,
        tick_id,
        frame_id,
    );

    embedded::record_engine_frame(
        app,
        window,
        ui,
        st,
        context,
        renderer,
        scale_factor,
        tick_id,
        frame_id,
    )
}

pub fn run() -> anyhow::Result<()> {
    fret::app_with_hooks("embedded-viewport-demo", init_window, view, |d| {
        d.viewport_input(embedded::handle_viewport_input)
            .record_engine_frame(record_engine_frame)
    })?
    .with_main_window("embedded_viewport_demo", (1120.0, 720.0))
    .init_app(|app| {
        shadcn::shadcn_themes::apply_shadcn_new_york(
            app,
            shadcn::shadcn_themes::ShadcnBaseColor::Slate,
            shadcn::shadcn_themes::ShadcnColorScheme::Light,
        );
        fret_icons_lucide::install_app(app);
    })
    .run()?;
    Ok(())
}
