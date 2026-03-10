use fret::{FretApp, advanced::prelude::*, shadcn};
use fret_core::scene::{EffectChain, EffectMode, EffectParamsV1, EffectQuality, EffectStep};
use fret_core::{AppWindowId, Color, EffectId, Px, SemanticsRole};
use fret_render::RendererCapabilities;
use fret_ui::element::{
    AnyElement, EffectLayerProps, LayoutStyle, Length, SemanticsDecoration, SpacerProps,
};
use fret_ui_kit::custom_effects::CustomEffectProgramV1;

mod act {
    fret::actions!([
        ToggleEnabled = "cookbook.customv1_basics.toggle_enabled.v1",
        SetStrengthLow = "cookbook.customv1_basics.set_strength_low.v1",
        SetStrengthHigh = "cookbook.customv1_basics.set_strength_high.v1"
    ]);
}

const ROOT_NAME: &str = "cookbook.customv1_basics";

const TEST_ID_ROOT: &str = "cookbook.customv1_basics.root";
const TEST_ID_TOGGLE: &str = "cookbook.customv1_basics.toggle";
const TEST_ID_STRENGTH_LOW: &str = "cookbook.customv1_basics.strength_low";
const TEST_ID_STRENGTH_HIGH: &str = "cookbook.customv1_basics.strength_high";

const TEST_ID_SUPPORTED: &str = "cookbook.customv1_basics.supported";
const TEST_ID_REGISTERED: &str = "cookbook.customv1_basics.registered";
const TEST_ID_ENABLED: &str = "cookbook.customv1_basics.enabled";
const TEST_ID_STRENGTH: &str = "cookbook.customv1_basics.strength";

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

struct CustomV1BasicsView;

fn install_custom_effect(app: &mut KernelApp, effects: &mut dyn fret_core::CustomEffectService) {
    let mut program = CustomEffectProgramV1::wgsl_utf8(WGSL);
    let id = program.ensure_registered(effects).ok();
    app.set_global(CookbookCustomV1Effect(id));
}

fn panel_shell<I: UiChildIntoElement<KernelApp>>(
    cx: &mut ElementContext<'_, KernelApp>,
    title: &str,
    body: I,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();
    let body = body.into_child_element(cx);

    shadcn::Card::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::CardHeader::build(|cx, out| {
                out.push_ui(cx, shadcn::CardTitle::new(title));
            }),
        );
        out.push_ui(
            cx,
            shadcn::CardContent::build(|cx, out| {
                out.push_ui(
                    cx,
                    ui::container(|_cx| vec![body])
                        .bg(ColorRef::Color(theme.color_token("muted")))
                        .rounded(Radius::Md)
                        .border_1()
                        .border_color(ColorRef::Color(theme.color_token("border")))
                        .w_full()
                        .h_px(Px(320.0)),
                );
            }),
        );
    })
    .ui()
    .w_full()
    .into_element(cx)
}

fn preview_content(cx: &mut ElementContext<'_, KernelApp>, label: &str) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();

    let swatch = |_cx: &mut ElementContext<'_, KernelApp>, rgb: u32| {
        ui::container(|_cx| Vec::<AnyElement>::new())
            .bg(ColorRef::Color(Color::from_srgb_hex_rgb(rgb)))
            .rounded(Radius::Sm)
            .w_px(Px(44.0))
            .h_px(Px(44.0))
    };

    let title = shadcn::Badge::new(label).variant(shadcn::BadgeVariant::Secondary);

    let row = ui::h_flex(|cx| {
        ui::children![cx;
            swatch(cx, 0x0EA5E9),
            swatch(cx, 0xA855F7),
            swatch(cx, 0xF97316),
            cx.spacer(SpacerProps::default()),
            title,
        ]
    })
    .gap(Space::N2)
    .items_center();

    ui::v_flex(|cx| {
        ui::children![cx;
            row,
            ui::text("")
                .text_sm()
                .text_color(ColorRef::Color(theme.color_token("muted-foreground"))),
        ]
    })
    .gap(Space::N2)
    .p(Space::N4)
    .w_full()
    .h_full()
    .into_element(cx)
}
impl View for CustomV1BasicsView {
    fn init(_app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, KernelApp>) -> Elements {
        let enabled_state = cx.use_local_with(|| true);
        let strength_state = cx.use_local_with(|| 0.35f32);

        cx.on_action_notify_toggle_local_bool::<act::ToggleEnabled>(&enabled_state);
        cx.on_action_notify_local_set::<act::SetStrengthLow, f32>(&strength_state, 0.20f32);
        cx.on_action_notify_local_set::<act::SetStrengthHigh, f32>(&strength_state, 0.75f32);

        let caps_supported = cx
            .app
            .global::<RendererCapabilities>()
            .map(|caps| caps.custom_effect_v1)
            .unwrap_or(false);
        let supported_value = if caps_supported { 1.0 } else { 0.0 };

        let enabled = enabled_state.paint(cx).value_or(true);
        let strength = strength_state.paint(cx).value_or(0.35f32);

        let effect_id = cx
            .app
            .global::<CookbookCustomV1Effect>()
            .and_then(|value| value.0);
        let registered_value = if effect_id.is_some() { 1.0 } else { 0.0 };
        let enabled_value = if enabled { 1.0 } else { 0.0 };

        let supported_badge = shadcn::Badge::new(if caps_supported {
            "CustomV1: supported"
        } else {
            "CustomV1: unavailable"
        })
        .variant(shadcn::BadgeVariant::Secondary)
        .a11y(
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
        .a11y(
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
        .a11y(
            SemanticsDecoration::default()
                .role(SemanticsRole::Meter)
                .test_id(TEST_ID_REGISTERED)
                .numeric_value(registered_value),
        );

        let strength_badge =
            shadcn::Badge::new(format!("Strength: {:.2}", strength.clamp(0.0, 1.0)))
                .variant(shadcn::BadgeVariant::Secondary)
                .a11y(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Meter)
                        .test_id(TEST_ID_STRENGTH)
                        .numeric_value(strength as f64)
                        .numeric_range(0.0, 1.0),
                );

        let strength_low = shadcn::Button::new("Strength: low")
            .variant(shadcn::ButtonVariant::Secondary)
            .action(act::SetStrengthLow)
            .disabled(!enabled)
            .a11y(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Button)
                    .test_id(TEST_ID_STRENGTH_LOW)
                    .label("Strength: low"),
            );

        let strength_high = shadcn::Button::new("Strength: high")
            .variant(shadcn::ButtonVariant::Secondary)
            .action(act::SetStrengthHigh)
            .disabled(!enabled)
            .a11y(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Button)
                    .test_id(TEST_ID_STRENGTH_HIGH)
                    .label("Strength: high"),
            );

        let toggle_label = if enabled { "Disable" } else { "Enable" };
        let toolbar = ui::h_flex(|cx| {
            ui::children![cx;
                shadcn::Button::new(toggle_label)
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::ToggleEnabled)
                    .test_id(TEST_ID_TOGGLE)
                    .disabled(!caps_supported || effect_id.is_none()),
                strength_low,
                strength_high,
                supported_badge,
                registered_badge,
                enabled_badge,
                strength_badge,
            ]
        })
        .gap(Space::N2)
        .items_center()
        .wrap();

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
                let alert = shadcn::Alert::new(ui::children![cx;
                    shadcn::AlertTitle::new("Effect not registered"),
                    shadcn::AlertDescription::new(
                        "The WGSL program did not register (or GPU services are not ready).",
                    ),
                ]);
                panel_shell(cx, "CustomV1 (FilterContent)", alert)
            }
        } else {
            let disabled_body = preview_content(cx, "disabled");
            panel_shell(cx, "CustomV1 (FilterContent)", disabled_body)
        };

        let panels = ui::h_flex(|cx| ui::children![cx; plain, custom_panel])
            .gap(Space::N3)
            .items_stretch();

        let body = ui::v_flex(|cx| ui::children![cx; toolbar, panels])
            .gap(Space::N4)
            .w_full();

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardHeader::build(|cx, out| {
                    out.push_ui(cx, shadcn::CardTitle::new("CustomV1 basics"));
                    out.push_ui(
                        cx,
                        shadcn::CardDescription::new(
                            "Registers a bounded WGSL snippet at on_gpu_ready and applies EffectStep::CustomV1 (single pass).",
                        ),
                    );
                }),
            );
            out.push_ui(
                cx,
                shadcn::CardContent::build(|cx, out| {
                    out.push_ui(cx, body);
                }),
            );
        })
        .ui()
        .w_full()
        .h_full()
        .max_w(Px(1180.0));

        fret_cookbook::scaffold::centered_page_background_ui(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new(ROOT_NAME)
        .window("cookbook-customv1-basics", (1180.0, 820.0))
        .config_files(false)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .view::<CustomV1BasicsView>()?
        .install_custom_effects(install_custom_effect)
        .run()
        .map_err(anyhow::Error::from)
}
