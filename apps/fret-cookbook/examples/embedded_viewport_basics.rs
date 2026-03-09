use std::collections::HashMap;

use fret::interop::embedded_viewport as embedded;
use fret::prelude::*;
use fret_app::{CommandMeta, CommandScope};
use fret_core::{AppWindowId, RenderTargetId, ViewportFit, ViewportInputEvent, ViewportInputKind};
use fret_render::{RenderTargetColorSpace, Renderer, WgpuContext};
use fret_runtime::{CommandId, FrameId, Model, TickId};
use fret_ui::element::SemanticsDecoration;

const ROOT_NAME: &str = "cookbook.embedded_viewport_basics";

const TEST_ID_ROOT: &str = "cookbook.embedded_viewport_basics.root";
const TEST_ID_SURFACE: &str = "cookbook.embedded_viewport_basics.surface";

const TEST_ID_SIZE_640: &str = "cookbook.embedded_viewport_basics.size_640";
const TEST_ID_SIZE_960: &str = "cookbook.embedded_viewport_basics.size_960";
const TEST_ID_SIZE_1280: &str = "cookbook.embedded_viewport_basics.size_1280";

const TEST_ID_FIT_CONTAIN: &str = "cookbook.embedded_viewport_basics.fit_contain";
const TEST_ID_FIT_COVER: &str = "cookbook.embedded_viewport_basics.fit_cover";
const TEST_ID_FIT_STRETCH: &str = "cookbook.embedded_viewport_basics.fit_stretch";

const TEST_ID_CLICKS: &str = "cookbook.embedded_viewport_basics.clicks";
const TEST_ID_UV_X: &str = "cookbook.embedded_viewport_basics.uv_x";
const TEST_ID_UV_Y: &str = "cookbook.embedded_viewport_basics.uv_y";
const TEST_ID_TARGET_W: &str = "cookbook.embedded_viewport_basics.target_w";
const TEST_ID_TARGET_H: &str = "cookbook.embedded_viewport_basics.target_h";
const TEST_ID_KIND: &str = "cookbook.embedded_viewport_basics.kind";

const DEFAULT_VIEWPORT_PX_SIZE: (u32, u32) = (960, 540);

const CMD_SIZE_640: &str = "cookbook.embedded_viewport_basics.size_640";
const CMD_SIZE_960: &str = "cookbook.embedded_viewport_basics.size_960";
const CMD_SIZE_1280: &str = "cookbook.embedded_viewport_basics.size_1280";
const CMD_FIT_CONTAIN: &str = "cookbook.embedded_viewport_basics.fit_contain";
const CMD_FIT_COVER: &str = "cookbook.embedded_viewport_basics.fit_cover";
const CMD_FIT_STRETCH: &str = "cookbook.embedded_viewport_basics.fit_stretch";

#[derive(Debug, Clone)]
struct EmbeddedViewportBasicsDiagModels {
    pub uv_x: Model<f64>,
    pub uv_y: Model<f64>,
    pub target_w: Model<f64>,
    pub target_h: Model<f64>,
    pub kind: Model<f64>,
}

#[derive(Default)]
struct EmbeddedViewportBasicsDiagService {
    by_window: HashMap<AppWindowId, EmbeddedViewportBasicsDiagModels>,
}

impl EmbeddedViewportBasicsDiagService {
    fn ensure_window(
        &mut self,
        app: &mut App,
        window: AppWindowId,
    ) -> EmbeddedViewportBasicsDiagModels {
        if let Some(existing) = self.by_window.get(&window) {
            return existing.clone();
        }

        let uv_x = app.models_mut().insert(0.5f64);
        let uv_y = app.models_mut().insert(0.5f64);
        let target_w = app.models_mut().insert(DEFAULT_VIEWPORT_PX_SIZE.0 as f64);
        let target_h = app.models_mut().insert(DEFAULT_VIEWPORT_PX_SIZE.1 as f64);
        let kind = app.models_mut().insert(0.0f64);

        let models = EmbeddedViewportBasicsDiagModels {
            uv_x,
            uv_y,
            target_w,
            target_h,
            kind,
        };
        self.by_window.insert(window, models.clone());
        models
    }
}

fn ensure_diag_models(app: &mut App, window: AppWindowId) -> EmbeddedViewportBasicsDiagModels {
    app.with_global_mut(EmbeddedViewportBasicsDiagService::default, |svc, app| {
        svc.ensure_window(app, window)
    })
}

fn diag_models(app: &App, window: AppWindowId) -> Option<EmbeddedViewportBasicsDiagModels> {
    app.global::<EmbeddedViewportBasicsDiagService>()
        .and_then(|svc| svc.by_window.get(&window).cloned())
}

fn viewport_kind_code(kind: ViewportInputKind) -> f64 {
    match kind {
        ViewportInputKind::PointerMove { .. } => 1.0,
        ViewportInputKind::PointerDown { .. } => 2.0,
        ViewportInputKind::PointerUp { .. } => 3.0,
        ViewportInputKind::PointerCancel { .. } => 4.0,
        ViewportInputKind::Wheel { .. } => 5.0,
    }
}

fn on_viewport_input(app: &mut App, event: ViewportInputEvent) {
    let Some(models) = embedded::models(app, event.window) else {
        embedded::handle_viewport_input(app, event);
        return;
    };

    let expected_target: RenderTargetId = app
        .models()
        .read(&models.target, |v| *v)
        .unwrap_or_default();

    if expected_target != RenderTargetId::default() && event.target == expected_target {
        let diag = ensure_diag_models(app, event.window);
        let _ = app
            .models_mut()
            .update(&diag.uv_x, |v| *v = event.uv.0 as f64);
        let _ = app
            .models_mut()
            .update(&diag.uv_y, |v| *v = event.uv.1 as f64);
        let _ = app.models_mut().update(&diag.target_w, |v| {
            *v = event.geometry.target_px_size.0 as f64
        });
        let _ = app.models_mut().update(&diag.target_h, |v| {
            *v = event.geometry.target_px_size.1 as f64
        });
        let _ = app
            .models_mut()
            .update(&diag.kind, |v| *v = viewport_kind_code(event.kind));
    }

    embedded::handle_viewport_input(app, event);
}

#[derive(Debug)]
struct EmbeddedViewportBasicsWindowState {
    embedded: embedded::EmbeddedViewportSurface,
    size_preset: Model<usize>,
    fit: Model<ViewportFit>,
}

fn install_commands(app: &mut App) {
    let scope = CommandScope::Widget;

    app.commands_mut().register(
        CommandId::from(CMD_SIZE_640),
        CommandMeta::new("Viewport size: 640×360")
            .with_description("Set the embedded viewport render target size preset.")
            .with_category("Embedded viewport")
            .with_scope(scope),
    );
    app.commands_mut().register(
        CommandId::from(CMD_SIZE_960),
        CommandMeta::new("Viewport size: 960×540")
            .with_description("Set the embedded viewport render target size preset.")
            .with_category("Embedded viewport")
            .with_scope(scope),
    );
    app.commands_mut().register(
        CommandId::from(CMD_SIZE_1280),
        CommandMeta::new("Viewport size: 1280×720")
            .with_description("Set the embedded viewport render target size preset.")
            .with_category("Embedded viewport")
            .with_scope(scope),
    );

    app.commands_mut().register(
        CommandId::from(CMD_FIT_CONTAIN),
        CommandMeta::new("Viewport fit: Contain")
            .with_description("Set the viewport surface fit to Contain.")
            .with_category("Embedded viewport")
            .with_scope(scope),
    );
    app.commands_mut().register(
        CommandId::from(CMD_FIT_COVER),
        CommandMeta::new("Viewport fit: Cover")
            .with_description("Set the viewport surface fit to Cover.")
            .with_category("Embedded viewport")
            .with_scope(scope),
    );
    app.commands_mut().register(
        CommandId::from(CMD_FIT_STRETCH),
        CommandMeta::new("Viewport fit: Stretch")
            .with_description("Set the viewport surface fit to Stretch.")
            .with_category("Embedded viewport")
            .with_scope(scope),
    );
}

fn on_command(
    app: &mut App,
    _services: &mut dyn fret_core::UiServices,
    _window: AppWindowId,
    _ui: &mut fret_ui::UiTree<App>,
    st: &mut EmbeddedViewportBasicsWindowState,
    command: &CommandId,
) {
    let cmd = command.as_str();

    if cmd == CMD_SIZE_640 {
        let _ = app.models_mut().update(&st.size_preset, |v| *v = 0);
    } else if cmd == CMD_SIZE_960 {
        let _ = app.models_mut().update(&st.size_preset, |v| *v = 1);
    } else if cmd == CMD_SIZE_1280 {
        let _ = app.models_mut().update(&st.size_preset, |v| *v = 2);
    } else if cmd == CMD_FIT_CONTAIN {
        let _ = app
            .models_mut()
            .update(&st.fit, |v| *v = ViewportFit::Contain);
    } else if cmd == CMD_FIT_COVER {
        let _ = app
            .models_mut()
            .update(&st.fit, |v| *v = ViewportFit::Cover);
    } else if cmd == CMD_FIT_STRETCH {
        let _ = app
            .models_mut()
            .update(&st.fit, |v| *v = ViewportFit::Stretch);
    }
}

impl embedded::EmbeddedViewportRecord for EmbeddedViewportBasicsWindowState {
    fn embedded_viewport_surface(&mut self) -> &mut embedded::EmbeddedViewportSurface {
        &mut self.embedded
    }

    fn embedded_viewport_label(&self) -> Option<&'static str> {
        Some("cookbook-embedded-viewport-basics")
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

        let (uv_x, uv_y) = diag_models(app, window)
            .and_then(|m| {
                let x = app.models().read(&m.uv_x, |v| *v).ok()?;
                let y = app.models().read(&m.uv_y, |v| *v).ok()?;
                Some((x, y))
            })
            .unwrap_or((0.5, 0.5));

        let t = (frame_id.0 as f32 * 0.02).sin() * 0.5 + 0.5;
        let c = (clicks % 9) as f32 / 8.0;
        let clear = wgpu::Color {
            r: (0.08 + 0.70 * (uv_x as f32).clamp(0.0, 1.0)) as f64,
            g: (0.06 + 0.65 * (uv_y as f32).clamp(0.0, 1.0)) as f64,
            b: (0.10 + 0.25 * t + 0.30 * (1.0 - c)) as f64,
            a: 1.0,
        };
        embedded::clear_pass(encoder, view, Some("embedded viewport clear"), clear);
    }
}

fn init_window(app: &mut App, window: AppWindowId) -> EmbeddedViewportBasicsWindowState {
    let _ = embedded::ensure_models(app, window);
    let _ = ensure_diag_models(app, window);

    EmbeddedViewportBasicsWindowState {
        embedded: embedded::EmbeddedViewportSurface::new(
            wgpu::TextureFormat::Bgra8UnormSrgb,
            RenderTargetColorSpace::Srgb,
            DEFAULT_VIEWPORT_PX_SIZE,
        ),
        size_preset: app.models_mut().insert(1usize),
        fit: app.models_mut().insert(ViewportFit::Contain),
    }
}

fn view(
    cx: &mut ElementContext<'_, App>,
    st: &mut EmbeddedViewportBasicsWindowState,
) -> ViewElements {
    let window = cx.window;
    let theme = Theme::global(&*cx.app).snapshot();

    let embedded_models = embedded::models(&*cx.app, window)
        .unwrap_or_else(|| embedded::ensure_models(cx.app, window));
    let diag = diag_models(&*cx.app, window).unwrap_or_else(|| ensure_diag_models(cx.app, window));

    let clicks = cx
        .watch_model(&embedded_models.clicks)
        .paint()
        .value_or_default();
    let uv_x = cx.watch_model(&diag.uv_x).paint().value_or_default();
    let uv_y = cx.watch_model(&diag.uv_y).paint().value_or_default();
    let target_w = cx.watch_model(&diag.target_w).paint().value_or_default();
    let target_h = cx.watch_model(&diag.target_h).paint().value_or_default();
    let kind = cx.watch_model(&diag.kind).paint().value_or_default();

    let preset = cx.watch_model(&st.size_preset).paint().value_or_default();
    let (target_px_size, preset_label): ((u32, u32), &'static str) = match preset {
        0 => ((640, 360), "640×360"),
        2 => ((1280, 720), "1280×720"),
        _ => (DEFAULT_VIEWPORT_PX_SIZE, "960×540"),
    };
    st.embedded.set_target_px_size(target_px_size);

    let fit = cx
        .watch_model(&st.fit)
        .paint()
        .value_or(ViewportFit::Contain);

    let header = ui::v_flex(|cx| {
        ui::children![
            cx;
            shadcn::CardTitle::new("Tier A interop: embedded viewport (basics)"),
            shadcn::CardDescription::new(
                "Offscreen RenderTargetId + ViewportSurface + explicit input forwarding (ViewportInputEvent).",
            ),
        ]
    })
    .gap(Space::N1);

    let size_controls = ui::h_flex(|cx| {
        ui::children![
            cx;
            shadcn::Button::new("640?360")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_SIZE_640)
                .disabled(preset == 0)
                .test_id(TEST_ID_SIZE_640),
            shadcn::Button::new("960?540")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_SIZE_960)
                .disabled(preset == 1)
                .test_id(TEST_ID_SIZE_960),
            shadcn::Button::new("1280?720")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_SIZE_1280)
                .disabled(preset == 2)
                .test_id(TEST_ID_SIZE_1280),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical),
            shadcn::Button::new("Fit: Contain")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_FIT_CONTAIN)
                .disabled(fit == ViewportFit::Contain)
                .test_id(TEST_ID_FIT_CONTAIN),
            shadcn::Button::new("Cover")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_FIT_COVER)
                .disabled(fit == ViewportFit::Cover)
                .test_id(TEST_ID_FIT_COVER),
            shadcn::Button::new("Stretch")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_FIT_STRETCH)
                .disabled(fit == ViewportFit::Stretch)
                .test_id(TEST_ID_FIT_STRETCH),
        ]
    })
    .gap(Space::N2)
    .items_center();

    let clicks_badge = shadcn::Badge::new(format!("Clicks: {clicks}"))
        .variant(shadcn::BadgeVariant::Secondary)
        .a11y(
            SemanticsDecoration::default()
                .role(SemanticsRole::Meter)
                .test_id(TEST_ID_CLICKS)
                .numeric_value(clicks as f64),
        );

    let uv_x_badge = shadcn::Badge::new(format!("uv.x: {:.3}", uv_x))
        .variant(shadcn::BadgeVariant::Secondary)
        .a11y(
            SemanticsDecoration::default()
                .role(SemanticsRole::Meter)
                .test_id(TEST_ID_UV_X)
                .numeric_value(uv_x)
                .numeric_range(0.0, 1.0),
        );
    let uv_y_badge = shadcn::Badge::new(format!("uv.y: {:.3}", uv_y))
        .variant(shadcn::BadgeVariant::Secondary)
        .a11y(
            SemanticsDecoration::default()
                .role(SemanticsRole::Meter)
                .test_id(TEST_ID_UV_Y)
                .numeric_value(uv_y)
                .numeric_range(0.0, 1.0),
        );

    let target_w_badge = shadcn::Badge::new(format!("target.w: {:.0}", target_w))
        .variant(shadcn::BadgeVariant::Secondary)
        .a11y(
            SemanticsDecoration::default()
                .role(SemanticsRole::Meter)
                .test_id(TEST_ID_TARGET_W)
                .numeric_value(target_w),
        );
    let target_h_badge = shadcn::Badge::new(format!("target.h: {:.0}", target_h))
        .variant(shadcn::BadgeVariant::Secondary)
        .a11y(
            SemanticsDecoration::default()
                .role(SemanticsRole::Meter)
                .test_id(TEST_ID_TARGET_H)
                .numeric_value(target_h),
        );

    let kind_badge = shadcn::Badge::new(format!("kind: {kind:.0}"))
        .variant(shadcn::BadgeVariant::Secondary)
        .a11y(
            SemanticsDecoration::default()
                .role(SemanticsRole::Meter)
                .test_id(TEST_ID_KIND)
                .numeric_value(kind),
        );

    let info = ui::h_flex(|cx| {
        ui::children![
            cx;
            ui::text(format!("target_px_size preset: {preset_label}")).text_xs(),
            clicks_badge,
            uv_x_badge,
            uv_y_badge,
            target_w_badge,
            target_h_badge,
            kind_badge,
        ]
    })
    .gap(Space::N2)
    .items_center()
    .wrap();

    let hint = shadcn::Alert::new(ui::children![
        cx;
        shadcn::AlertTitle::new("What this teaches"),
        shadcn::AlertDescription::new(
            "This panel renders into an offscreen texture and is presented via a ViewportSurface element. Input does not arrive as normal UI pointer events; it is forwarded as ViewportInputEvent at the app level.",
        ),
    ]);

    let viewport = st
        .embedded
        .panel(
            cx,
            embedded::EmbeddedViewportPanelProps {
                fit,
                opacity: 1.0,
                forward_input: true,
            },
        )
        .test_id(TEST_ID_SURFACE);

    let viewport_panel = ui::container(|_cx| vec![viewport])
        .bg(ColorRef::Color(theme.color_token("muted")))
        .rounded(Radius::Md)
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .w_full()
        .h_px(Px(420.0));

    let content = ui::v_flex(|cx| ui::children![cx; hint, viewport_panel]).gap(Space::N3);

    let card = shadcn::Card::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::CardHeader::build(|cx, out| {
                out.push_ui(cx, header);
                out.push_ui(cx, size_controls);
                out.push_ui(cx, info);
            }),
        );
        out.push_ui(
            cx,
            shadcn::CardContent::build(|cx, out| {
                out.push_ui(cx, content);
            }),
        );
    })
    .ui()
    .w_full()
    .max_w(Px(1100.0));

    let root = fret_cookbook::scaffold::centered_page_background_ui(cx, TEST_ID_ROOT, card);

    vec![root].into()
}

fn configure_driver(
    driver: fret_bootstrap::ui_app_driver::UiAppDriver<EmbeddedViewportBasicsWindowState>,
) -> fret_bootstrap::ui_app_driver::UiAppDriver<EmbeddedViewportBasicsWindowState> {
    driver
        .on_command(on_command)
        .viewport_input(on_viewport_input)
        .record_engine_frame(embedded::record_engine_frame::<EmbeddedViewportBasicsWindowState>)
}

fn main() -> anyhow::Result<()> {
    let builder = fret_bootstrap::ui_app_with_hooks(ROOT_NAME, init_window, view, configure_driver)
        .with_main_window("cookbook-embedded-viewport-basics", (1120.0, 780.0))
        .install_app(install_commands)
        .install_app(shadcn::install_app)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .with_ui_assets_budgets(64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096)
        .with_lucide_icons();

    #[cfg(feature = "cookbook-diag")]
    let builder = builder.with_default_diagnostics();

    builder.run().map_err(anyhow::Error::from)
}
