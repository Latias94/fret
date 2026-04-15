use std::sync::Arc;

use fret::advanced::interop::embedded_viewport as embedded;
use fret::{FretApp, advanced::prelude::*, component::prelude::*, shadcn};
use fret_core::ViewportFit;
use fret_render::{RenderTargetColorSpace, Renderer, WgpuContext};
use fret_runtime::{FrameId, TickId};

const DEFAULT_VIEWPORT_PX_SIZE: (u32, u32) = (960, 540);
const SIZE_PRESET_640: &str = "640x360";
const SIZE_PRESET_960: &str = "960x540";
const SIZE_PRESET_1280: &str = "1280x720";

fn diag_enabled() -> bool {
    std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty() && v != "0")
}

fn selected_target_px_size(value: Option<&str>) -> ((u32, u32), &'static str) {
    match value {
        Some(SIZE_PRESET_640) => ((640, 360), "640×360"),
        Some(SIZE_PRESET_1280) => ((1280, 720), "1280×720"),
        _ => (DEFAULT_VIEWPORT_PX_SIZE, "960×540"),
    }
}

struct EmbeddedViewportDemoView {
    embedded: embedded::EmbeddedViewportSurface,
}

impl View for EmbeddedViewportDemoView {
    fn init(app: &mut KernelApp, window: AppWindowId) -> Self {
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
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let window = cx.window_id();
        let theme = cx.theme_snapshot();

        let models = embedded::models(cx.app(), window)
            .unwrap_or_else(|| embedded::ensure_models(cx.app_mut(), window));

        let clicks = models.clicks.paint(cx).value_or_default();
        let last_input: Arc<str> = models
            .last_input
            .paint(cx)
            .value_or_else(|| Arc::from("<no input yet>"));

        let size_preset_state = cx
            .state()
            .local_init(|| Some(Arc::<str>::from(SIZE_PRESET_960)));
        let preset = size_preset_state.layout_value(cx);
        let (target_px_size, preset_label) = selected_target_px_size(preset.as_deref());
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

        let size_controls = shadcn::ToggleGroup::single(&size_preset_state)
            .deselectable(false)
            .variant(shadcn::ToggleVariant::Outline)
            .spacing(Space::N2)
            .items([
                shadcn::ToggleGroupItem::new(SIZE_PRESET_640, [cx.text("640×360")])
                    .a11y_label("Viewport size 640 by 360"),
                shadcn::ToggleGroupItem::new(SIZE_PRESET_960, [cx.text("960×540")])
                    .a11y_label("Viewport size 960 by 540"),
                shadcn::ToggleGroupItem::new(SIZE_PRESET_1280, [cx.text("1280×720")])
                    .a11y_label("Viewport size 1280 by 720"),
            ])
            .refine_layout(LayoutRefinement::default().flex_none())
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
                ui::container(|cx| ui::children![cx; viewport])
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

        embedded_viewport_page(cx, theme, viewport_card, diag_enabled())
    }
}

fn embedded_viewport_page<'a, Cx, C>(
    cx: &mut Cx,
    theme: ThemeSnapshot,
    viewport_card: C,
    diag: bool,
) -> Ui
where
    Cx: fret::app::ElementContextAccess<'a, KernelApp>,
    C: IntoUiElement<KernelApp>,
{
    let cx = cx.elements();
    let page = ui::container(move |cx| {
        ui::single(
            cx,
            ui::v_flex(move |cx| ui::single(cx, viewport_card))
                .w_full()
                .h_full()
                .justify_center()
                .items_center(),
        )
    })
    .bg(ColorRef::Color(theme.color_token("background")))
    .p(Space::N6)
    .w_full()
    .h_full()
    .into_element(cx);

    if diag {
        page.test_id("embedded-viewport-demo.root").into()
    } else {
        page.into()
    }
}

impl embedded::EmbeddedViewportView for EmbeddedViewportDemoView {
    fn embedded_viewport_surface(&mut self) -> &mut embedded::EmbeddedViewportSurface {
        &mut self.embedded
    }

    fn embedded_viewport_label(&self) -> Option<&'static str> {
        Some("embedded-viewport-demo")
    }

    fn record_embedded_viewport(
        &mut self,
        app: &mut KernelApp,
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

pub fn run() -> anyhow::Result<()> {
    FretApp::new("embedded-viewport-demo")
        .window("embedded_viewport_demo", (1120.0, 720.0))
        .view_with_hooks::<EmbeddedViewportDemoView>(|d| d.drive_embedded_viewport())?
        .setup((install_demo_theme, fret_icons_lucide::app::install))
        .run()?;
    Ok(())
}

fn install_demo_theme(app: &mut KernelApp) {
    shadcn::themes::apply_shadcn_new_york(
        app,
        shadcn::themes::ShadcnBaseColor::Slate,
        shadcn::themes::ShadcnColorScheme::Light,
    );
}
