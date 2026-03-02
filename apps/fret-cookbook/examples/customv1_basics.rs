use fret::prelude::*;
use fret_app::{CommandMeta, CommandScope};
use fret_bootstrap::ui_app_driver::ViewElements;
use fret_core::scene::{EffectChain, EffectMode, EffectParamsV1, EffectQuality, EffectStep};
use fret_core::{AppWindowId, Color, EffectId, Px, SemanticsRole};
use fret_render::RendererCapabilities;
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, EffectLayerProps, LayoutStyle, Length, SemanticsDecoration, SpacerProps,
};
use fret_ui_kit::custom_effects::CustomEffectProgramV1;

const ROOT_NAME: &str = "cookbook.customv1_basics";

const TEST_ID_ROOT: &str = "cookbook.customv1_basics.root";
const TEST_ID_TOGGLE: &str = "cookbook.customv1_basics.toggle";
const TEST_ID_STRENGTH_LOW: &str = "cookbook.customv1_basics.strength_low";
const TEST_ID_STRENGTH_HIGH: &str = "cookbook.customv1_basics.strength_high";

const TEST_ID_SUPPORTED: &str = "cookbook.customv1_basics.supported";
const TEST_ID_REGISTERED: &str = "cookbook.customv1_basics.registered";
const TEST_ID_ENABLED: &str = "cookbook.customv1_basics.enabled";
const TEST_ID_STRENGTH: &str = "cookbook.customv1_basics.strength";

const CMD_TOGGLE: &str = "cookbook.customv1_basics.toggle";
const CMD_STRENGTH_LOW: &str = "cookbook.customv1_basics.strength_low";
const CMD_STRENGTH_HIGH: &str = "cookbook.customv1_basics.strength_high";

const WGSL: &str = r#"
fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, params: EffectParamsV1) -> vec4<f32> {
  let strength = clamp(params.vec4s[0].x, 0.0, 1.0);

  // Effect-local coordinates (stable contract): 0..size in px.
  let local = fret_local_px(pos_px);
  let size = max(render_space.size_px, vec2<f32>(1.0));
  let centered = (local / size) - vec2<f32>(0.5);

  // Simple vignette + deterministic grain, bounded to the effect region.
  let d = length(centered);
  let vignette = smoothstep(0.75, 0.10, d);
  let n = (fret_catalog_hash_noise01(local * 2.0) - 0.5) * 0.02;

  // `src` is linear premultiplied RGBA.
  let boost = 1.0 + 0.30 * strength * vignette;
  let tint = vec3<f32>(0.12, -0.05, 0.18) * (strength * vignette);

  let rgb = src.rgb * boost + src.a * tint + vec3<f32>(n) * strength;
  return vec4<f32>(rgb, src.a);
}
"#;

#[derive(Debug, Clone, Copy)]
struct CookbookCustomV1Effect(Option<EffectId>);

#[derive(Debug)]
struct CustomV1BasicsWindowState {
    enabled: Model<bool>,
    strength: Model<f32>,
}

fn install_commands(app: &mut App) {
    let scope = CommandScope::Widget;
    let category = "Custom effects";

    app.commands_mut().register(
        CommandId::from(CMD_TOGGLE),
        CommandMeta::new("Toggle CustomV1")
            .with_description("Enable/disable the CustomV1 filter panel.")
            .with_category(category)
            .with_scope(scope),
    );

    app.commands_mut().register(
        CommandId::from(CMD_STRENGTH_LOW),
        CommandMeta::new("Strength: low")
            .with_description("Set strength to a low preset.")
            .with_category(category)
            .with_scope(scope),
    );

    app.commands_mut().register(
        CommandId::from(CMD_STRENGTH_HIGH),
        CommandMeta::new("Strength: high")
            .with_description("Set strength to a high preset.")
            .with_category(category)
            .with_scope(scope),
    );
}

fn install_custom_effect(app: &mut App, effects: &mut dyn fret_core::CustomEffectService) {
    let mut program = CustomEffectProgramV1::wgsl_utf8(WGSL);
    let id = program.ensure_registered(effects).ok();
    app.set_global(CookbookCustomV1Effect(id));
}

fn init_window(app: &mut App, _window: AppWindowId) -> CustomV1BasicsWindowState {
    CustomV1BasicsWindowState {
        enabled: app.models_mut().insert(true),
        strength: app.models_mut().insert(0.35),
    }
}

fn on_command(
    app: &mut App,
    _services: &mut dyn fret_core::UiServices,
    _window: AppWindowId,
    _ui: &mut fret_ui::UiTree<App>,
    st: &mut CustomV1BasicsWindowState,
    command: &CommandId,
) {
    let cmd = command.as_str();

    if cmd == CMD_TOGGLE {
        let _ = app.models_mut().update(&st.enabled, |v| *v = !*v);
    } else if cmd == CMD_STRENGTH_LOW {
        let _ = app.models_mut().update(&st.strength, |v| *v = 0.20);
    } else if cmd == CMD_STRENGTH_HIGH {
        let _ = app.models_mut().update(&st.strength, |v| *v = 0.75);
    }
}

fn panel_shell(cx: &mut ElementContext<'_, App>, title: &str, body: AnyElement) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();

    let inner = ui::container(cx, |_cx| vec![body])
        .bg(ColorRef::Color(theme.color_token("muted")))
        .rounded(Radius::Md)
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .w_full()
        .h_px(Px(320.0))
        .into_element(cx);

    shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
            .into_element(cx),
        shadcn::CardContent::new(vec![inner]).into_element(cx),
    ])
    .ui()
    .w_full()
    .into_element(cx)
}

fn preview_content(cx: &mut ElementContext<'_, App>, label: &str) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();

    let swatch = |cx: &mut ElementContext<'_, App>, rgb: u32| -> AnyElement {
        ui::container(cx, |_cx| Vec::<AnyElement>::new())
            .bg(ColorRef::Color(Color::from_srgb_hex_rgb(rgb)))
            .rounded(Radius::Sm)
            .w_px(Px(44.0))
            .h_px(Px(44.0))
            .into_element(cx)
    };

    let title = shadcn::Badge::new(label)
        .variant(shadcn::BadgeVariant::Secondary)
        .into_element(cx);

    let row = ui::h_flex(cx, |cx| {
        [
            swatch(cx, 0x0EA5E9),
            swatch(cx, 0xA855F7),
            swatch(cx, 0xF97316),
            cx.spacer(SpacerProps::default()),
            title,
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    ui::v_flex(cx, |cx| {
        [
            row,
            ui::text(
                cx,
                "CustomV1 runs as a single-pass effect within EffectChain. Use params-only WGSL with bounded sampling.",
            )
            .text_sm()
            .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
            .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .p(Space::N4)
    .w_full()
    .h_full()
    .into_element(cx)
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut CustomV1BasicsWindowState) -> ViewElements {
    let theme = Theme::global(&*cx.app).snapshot();

    let caps_supported = cx
        .app
        .global::<RendererCapabilities>()
        .map(|c| c.custom_effect_v1)
        .unwrap_or(false);
    let supported_value = if caps_supported { 1.0 } else { 0.0 };

    let enabled = cx.watch_model(&st.enabled).paint().copied_or_default();
    let strength = cx.watch_model(&st.strength).paint().copied_or_default();

    let effect_id = cx.app.global::<CookbookCustomV1Effect>().and_then(|v| v.0);

    let registered_value = if effect_id.is_some() { 1.0 } else { 0.0 };
    let enabled_value = if enabled { 1.0 } else { 0.0 };

    let supported_badge = shadcn::Badge::new(if caps_supported {
        "CustomV1: supported"
    } else {
        "CustomV1: unavailable"
    })
    .variant(shadcn::BadgeVariant::Secondary)
    .into_element(cx)
    .attach_semantics(
        SemanticsDecoration::default()
            .role(SemanticsRole::Meter)
            .test_id(TEST_ID_SUPPORTED)
            .numeric_value(supported_value),
    );

    let enabled_badge = shadcn::Badge::new(if enabled {
        "Enabled: true"
    } else {
        "Enabled: false"
    })
    .variant(shadcn::BadgeVariant::Secondary)
    .into_element(cx)
    .attach_semantics(
        SemanticsDecoration::default()
            .role(SemanticsRole::Meter)
            .test_id(TEST_ID_ENABLED)
            .numeric_value(enabled_value),
    );

    let registered_badge = shadcn::Badge::new(if effect_id.is_some() {
        "Registered: true"
    } else {
        "Registered: false"
    })
    .variant(shadcn::BadgeVariant::Secondary)
    .into_element(cx)
    .attach_semantics(
        SemanticsDecoration::default()
            .role(SemanticsRole::Meter)
            .test_id(TEST_ID_REGISTERED)
            .numeric_value(registered_value),
    );

    let strength_badge = shadcn::Badge::new(format!("Strength: {:.2}", strength.clamp(0.0, 1.0)))
        .variant(shadcn::BadgeVariant::Secondary)
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Meter)
                .test_id(TEST_ID_STRENGTH)
                .numeric_value(strength as f64)
                .numeric_range(0.0, 1.0),
        );

    let toggle_label = if enabled { "Disable" } else { "Enable" };
    let toolbar = ui::h_flex(cx, |cx| {
        [
            shadcn::Button::new(toggle_label)
                .variant(shadcn::ButtonVariant::Outline)
                .on_click(CMD_TOGGLE)
                .test_id(TEST_ID_TOGGLE)
                .disabled(!caps_supported || effect_id.is_none())
                .into_element(cx),
            shadcn::Button::new("Strength: low")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_STRENGTH_LOW)
                .test_id(TEST_ID_STRENGTH_LOW)
                .disabled(!enabled)
                .into_element(cx),
            shadcn::Button::new("Strength: high")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_STRENGTH_HIGH)
                .test_id(TEST_ID_STRENGTH_HIGH)
                .disabled(!enabled)
                .into_element(cx),
            supported_badge,
            registered_badge,
            enabled_badge,
            strength_badge,
        ]
    })
    .gap(Space::N2)
    .items_center()
    .wrap()
    .into_element(cx);

    let plain_body = preview_content(cx, "plain");
    let plain = panel_shell(cx, "Plain", plain_body);

    let custom_panel = if enabled && caps_supported {
        if let Some(effect) = effect_id {
            let params = EffectParamsV1 {
                vec4s: [
                    [strength.clamp(0.0, 1.0), 0.0, 0.0, 0.0],
                    [0.0; 4],
                    [0.0; 4],
                    [0.0; 4],
                ],
            };

            let chain = EffectChain::from_steps(&[EffectStep::CustomV1 {
                id: effect,
                params,
                max_sample_offset_px: Px(0.0),
            }])
            .sanitize();

            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.height = Length::Fill;

            let layer = cx.effect_layer_props(
                EffectLayerProps {
                    layout,
                    mode: EffectMode::FilterContent,
                    chain,
                    quality: EffectQuality::Auto,
                },
                |cx| vec![preview_content(cx, "customv1")],
            );

            panel_shell(cx, "CustomV1 (FilterContent)", layer)
        } else {
            let alert = shadcn::Alert::new([
                shadcn::AlertTitle::new("Effect not registered").into_element(cx),
                shadcn::AlertDescription::new(
                    "The WGSL program did not register (or GPU services are not ready).",
                )
                .into_element(cx),
            ])
            .into_element(cx);
            panel_shell(cx, "CustomV1 (FilterContent)", alert)
        }
    } else {
        let disabled_body = preview_content(cx, "disabled");
        panel_shell(cx, "CustomV1 (FilterContent)", disabled_body)
    };

    let content = ui::v_flex(cx, |cx| {
        [
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("CustomV1 basics").into_element(cx),
                shadcn::CardDescription::new(
                    "Registers a bounded WGSL snippet at on_gpu_ready and applies EffectStep::CustomV1 (single pass).",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![
                toolbar,
                ui::h_flex(cx, |_cx| [plain, custom_panel])
                    .gap(Space::N3)
                    .items_stretch()
                    .into_element(cx),
            ])
            .into_element(cx),
        ]
    })
    .into_element(cx);

    let card = shadcn::Card::new(vec![content])
        .ui()
        .w_full()
        .h_full()
        .max_w(Px(1180.0))
        .into_element(cx);

    let root = ui::container(cx, |cx| {
        vec![ui::v_flex(cx, |_cx| vec![card])
            .items_center()
            .justify_center()
            .size_full()
            .into_element(cx)]
    })
    .bg(ColorRef::Color(theme.color_token("background")))
    .p(Space::N6)
    .size_full()
    .into_element(cx)
    .test_id(TEST_ID_ROOT);

    vec![root].into()
}

fn configure_driver(
    driver: fret_bootstrap::ui_app_driver::UiAppDriver<CustomV1BasicsWindowState>,
) -> fret_bootstrap::ui_app_driver::UiAppDriver<CustomV1BasicsWindowState> {
    driver.on_command(on_command)
}

fn main() -> anyhow::Result<()> {
    let builder = fret_bootstrap::ui_app_with_hooks(ROOT_NAME, init_window, view, configure_driver)
        .with_main_window("cookbook-customv1-basics", (1180.0, 820.0))
        .with_command_default_keybindings()
        .install_app(install_commands)
        .install_app(shadcn::install_app)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .install_custom_effects(install_custom_effect)
        .with_ui_assets_budgets(64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096)
        .with_lucide_icons()
        .with_default_diagnostics();

    builder.run().map_err(anyhow::Error::from)
}
