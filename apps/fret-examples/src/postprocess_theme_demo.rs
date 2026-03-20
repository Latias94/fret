//! Theme-like post-processing demo (CustomV1 + built-in effect steps).
//!
//! This demo exists to validate the “high ceiling” story without expanding the portable core
//! contract: authoring policy lives in app/ecosystem code, while the renderer stays bounded.

#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use fret::{FretApp, advanced::prelude::*, component::prelude::*, shadcn};
use fret_core::scene::{
    DitherMode, EffectChain, EffectMode, EffectParamsV1, EffectQuality, EffectStep,
};
use fret_core::{Color, Corners, Edges, EffectId, Px};
use fret_ui::element::{
    ContainerProps, EffectLayerProps, LayoutStyle, Length, Overflow, PositionStyle, SpacerProps,
    TextProps,
};
use fret_ui_kit::custom_effects::CustomEffectProgramV1;
use fret_ui_kit::ui;
use fret_ui_kit::{IntoUiElement, Space};

mod act {
    fret::actions!([Reset = "postprocess_theme_demo.reset.v1"]);
}

fn install_demo_theme(app: &mut KernelApp) {
    shadcn::themes::apply_shadcn_new_york(
        app,
        shadcn::themes::ShadcnBaseColor::Slate,
        shadcn::themes::ShadcnColorScheme::Dark,
    );
}

const WGSL: &str = r#"
// Params packing (EffectParamsV1 is 64 bytes):
// - vec4s[0].x: chromatic_offset_px
// - vec4s[0].y: scanline_strength (0..1)
// - vec4s[0].z: scanline_spacing_px (>= 1)
// - vec4s[0].w: vignette_strength (0..1)
// - vec4s[1].x: grain_strength (0..1)
// - vec4s[1].y: grain_scale (>= 0.1)

const TAU: f32 = 6.28318530718;

fn unpremul(c: vec4<f32>) -> vec3<f32> {
  if (c.a <= 1e-6) { return vec3<f32>(0.0); }
  return c.rgb / c.a;
}

fn clamp_i2(p: vec2<i32>, dims: vec2<i32>) -> vec2<i32> {
  return vec2<i32>(
    clamp(p.x, 0, dims.x - 1),
    clamp(p.y, 0, dims.y - 1),
  );
}

fn sample_src_nearest_i(p: vec2<i32>) -> vec4<f32> {
  return textureLoad(src_texture, p, 0);
}

fn smooth01(x: f32) -> f32 {
  // cubic Hermite smoothstep on [0,1]
  let t = clamp(x, 0.0, 1.0);
  return t * t * (3.0 - 2.0 * t);
}

fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, params: EffectParamsV1) -> vec4<f32> {
  let chromatic_offset_px = clamp(params.vec4s[0].x, 0.0, 6.0);
  let scanline_strength = clamp(params.vec4s[0].y, 0.0, 1.0);
  let scanline_spacing_px = max(1.0, params.vec4s[0].z);
  let vignette_strength = clamp(params.vec4s[0].w, 0.0, 1.0);

  let grain_strength = clamp(params.vec4s[1].x, 0.0, 0.25);
  let grain_scale = clamp(params.vec4s[1].y, 0.1, 8.0);

  let dims_u = textureDimensions(src_texture);
  let dims_i = vec2<i32>(i32(dims_u.x), i32(dims_u.y));
  let xi = clamp(i32(floor(pos_px.x)), 0, dims_i.x - 1);
  let yi = clamp(i32(floor(pos_px.y)), 0, dims_i.y - 1);

  var base_u = unpremul(src);
  var a = src.a;

  // Chromatic offset: sample red/blue with bounded integer offsets, then mix in unpremul space.
  if (chromatic_offset_px > 0.0) {
    let off = i32(round(chromatic_offset_px));
    let red = sample_src_nearest_i(clamp_i2(vec2<i32>(xi + off, yi), dims_i));
    let blue = sample_src_nearest_i(clamp_i2(vec2<i32>(xi - off, yi), dims_i));
    let red_u = unpremul(red);
    let blue_u = unpremul(blue);
    let aberr = vec3<f32>(red_u.r, base_u.g, blue_u.b);
    base_u = mix(base_u, aberr, min(1.0, chromatic_offset_px / 6.0));
    a = max(a, max(red.a, blue.a));
  }

  // Vignette: effect-local radial falloff from the render-space center.
  if (vignette_strength > 0.0) {
    let local = fret_local_px(pos_px);
    let size = max(render_space.size_px, vec2<f32>(1.0));
    let uv = (local / size) * 2.0 - vec2<f32>(1.0);
    let r = length(uv);
    let t = smooth01((r - 0.55) / 0.45);
    let vig = 1.0 - t;
    base_u *= mix(1.0, vig, vignette_strength);
  }

  // Scanlines: effect-local periodic modulation along Y.
  if (scanline_strength > 0.0) {
    let ly = fret_local_px(pos_px).y;
    let s = 0.5 + 0.5 * sin((ly / scanline_spacing_px) * TAU);
    let shade = 1.0 - scanline_strength * s;
    base_u *= shade;
  }

  // Deterministic grain anchored to effect bounds.
  if (grain_strength > 0.0) {
    let n = fret_catalog_hash_noise01(fret_local_px(pos_px) * grain_scale) - 0.5;
    base_u += vec3<f32>(n) * grain_strength;
  }

  base_u = clamp(base_u, vec3<f32>(0.0), vec3<f32>(4.0));
  return vec4<f32>(base_u * a, a);
}
"#;

#[derive(Debug, Clone, Copy)]
struct DemoEffect(EffectId);

struct ThemePostprocessState {
    enabled: LocalState<bool>,
    compare: LocalState<bool>,

    theme: LocalState<Option<Arc<str>>>,
    theme_open: LocalState<bool>,

    chromatic_offset_px: LocalState<Vec<f32>>,
    scanline_strength: LocalState<Vec<f32>>,
    scanline_spacing_px: LocalState<Vec<f32>>,
    vignette_strength: LocalState<Vec<f32>>,
    grain_strength: LocalState<Vec<f32>>,
    grain_scale: LocalState<Vec<f32>>,

    retro_pixel_scale: LocalState<Vec<f32>>,
    retro_dither: LocalState<bool>,
}

struct ThemePostprocessView;

#[derive(Clone)]
struct ThemePostprocessViewSettings {
    enabled: bool,
    compare: bool,
    theme: Option<Arc<str>>,
    retro_dither: bool,
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("postprocess-theme-demo")
        .window("postprocess-theme-demo", (1200.0, 760.0))
        .setup(install_demo_theme)
        .view::<ThemePostprocessView>()?
        .install_custom_effects(install_custom_effect)
        .run()
        .map_err(anyhow::Error::from)
}

fn install_custom_effect(app: &mut KernelApp, effects: &mut dyn fret_core::CustomEffectService) {
    let mut program = CustomEffectProgramV1::wgsl_utf8(WGSL);
    let id = program
        .ensure_registered(effects)
        .expect("custom effect registration must succeed on wgpu backends");
    app.set_global(DemoEffect(id));
}

impl ThemePostprocessState {
    fn new(cx: &mut AppUi<'_, '_>) -> Self {
        Self {
            enabled: cx.state().local_init(|| true),
            compare: cx.state().local_init(|| true),
            theme: cx
                .state()
                .local_init(|| Option::<Arc<str>>::Some(Arc::from("cyberpunk"))),
            theme_open: cx.state().local_init(|| false),
            chromatic_offset_px: cx.state().local_init(|| vec![4.0]),
            scanline_strength: cx.state().local_init(|| vec![0.32]),
            scanline_spacing_px: cx.state().local_init(|| vec![3.0]),
            vignette_strength: cx.state().local_init(|| vec![0.6]),
            grain_strength: cx.state().local_init(|| vec![0.12]),
            grain_scale: cx.state().local_init(|| vec![1.5]),
            retro_pixel_scale: cx.state().local_init(|| vec![10.0]),
            retro_dither: cx.state().local_init(|| true),
        }
    }
}

impl View for ThemePostprocessView {
    fn init(_app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let Some(effect) = cx.app.global::<DemoEffect>().map(|v| v.0) else {
            return vec![
                shadcn::raw::typography::h3("Custom effects unavailable").into_element(cx),
            ]
            .into();
        };

        let mut st = ThemePostprocessState::new(cx);
        let view_settings: ThemePostprocessViewSettings = cx.data().selector_layout(
            (&st.enabled, &st.compare, &st.theme, &st.retro_dither),
            |(enabled, compare, theme, retro_dither)| ThemePostprocessViewSettings {
                enabled,
                compare,
                theme,
                retro_dither,
            },
        );

        let chromatic_offset_px = watch_first_f32(cx, &st.chromatic_offset_px, 2.0);
        let scanline_strength = watch_first_f32(cx, &st.scanline_strength, 0.18);
        let scanline_spacing_px = watch_first_f32(cx, &st.scanline_spacing_px, 3.0);
        let vignette_strength = watch_first_f32(cx, &st.vignette_strength, 0.35);
        let grain_strength = watch_first_f32(cx, &st.grain_strength, 0.06);
        let grain_scale = watch_first_f32(cx, &st.grain_scale, 1.5);

        let retro_pixel_scale = watch_first_f32(cx, &st.retro_pixel_scale, 10.0);

        let inspector = inspector(
            cx,
            &mut st,
            view_settings.theme.as_deref().unwrap_or("cyberpunk"),
            chromatic_offset_px,
            scanline_strength,
            scanline_spacing_px,
            vignette_strength,
            grain_strength,
            grain_scale,
            retro_pixel_scale,
            view_settings.retro_dither,
        );

        let stage = stage(
            cx,
            view_settings.enabled,
            view_settings.compare,
            view_settings.theme.as_deref().unwrap_or("cyberpunk"),
            effect,
            chromatic_offset_px,
            scanline_strength,
            scanline_spacing_px,
            vignette_strength,
            grain_strength,
            grain_scale,
            retro_pixel_scale,
            view_settings.retro_dither,
        );

        cx.actions().local(&st.enabled).set::<act::Reset>(true);
        cx.actions().local(&st.compare).set::<act::Reset>(true);
        cx.actions()
            .local(&st.theme)
            .set::<act::Reset>(Some(Arc::<str>::from("cyberpunk")));
        cx.actions()
            .local(&st.chromatic_offset_px)
            .set::<act::Reset>(vec![4.0]);
        cx.actions()
            .local(&st.scanline_strength)
            .set::<act::Reset>(vec![0.32]);
        cx.actions()
            .local(&st.scanline_spacing_px)
            .set::<act::Reset>(vec![3.0]);
        cx.actions()
            .local(&st.vignette_strength)
            .set::<act::Reset>(vec![0.6]);
        cx.actions()
            .local(&st.grain_strength)
            .set::<act::Reset>(vec![0.12]);
        cx.actions()
            .local(&st.grain_scale)
            .set::<act::Reset>(vec![1.5]);
        cx.actions()
            .local(&st.retro_pixel_scale)
            .set::<act::Reset>(vec![10.0]);
        cx.actions().local(&st.retro_dither).set::<act::Reset>(true);

        let root = ui::h_flex(move |cx| {
            let inspector = inspector.into_element(cx);
            let stage = stage.into_element(cx);
            [inspector, stage]
        })
        .layout(LayoutRefinement::default().size_full())
        .items_stretch()
        .gap(Space::N0)
        .into_element(cx);

        vec![root].into()
    }
}

fn srgb(r: u8, g: u8, b: u8, a: f32) -> Color {
    let mut c = fret_ui_kit::colors::linear_from_hex_rgb(
        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
    );
    c.a = a.clamp(0.0, 1.0);
    c
}

fn watch_first_f32(cx: &mut UiCx<'_>, model: &LocalState<Vec<f32>>, default: f32) -> f32 {
    model.layout_read_ref_in(cx, |v| v.first().copied().unwrap_or(default))
}

fn inspector(
    cx: &mut UiCx<'_>,
    st: &mut ThemePostprocessState,
    theme: &str,
    chromatic_offset_px: f32,
    scanline_strength: f32,
    scanline_spacing_px: f32,
    vignette_strength: f32,
    grain_strength: f32,
    grain_scale: f32,
    retro_pixel_scale: f32,
    retro_dither: bool,
) -> impl IntoUiElement<KernelApp> + use<> {
    let theme_snapshot = Theme::global(&*cx.app).snapshot();

    let enabled_model = st.enabled.clone_model();
    let compare_model = st.compare.clone_model();
    let theme_model = st.theme.clone_model();
    let theme_open_model = st.theme_open.clone_model();

    let chromatic_state = st.chromatic_offset_px.clone();
    let scanline_strength_state = st.scanline_strength.clone();
    let scanline_spacing_state = st.scanline_spacing_px.clone();
    let vignette_state = st.vignette_strength.clone();
    let grain_strength_state = st.grain_strength.clone();
    let grain_scale_state = st.grain_scale.clone();
    let retro_pixel_scale_state = st.retro_pixel_scale.clone();
    let retro_dither_model = st.retro_dither.clone_model();

    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Px(Px(380.0));
    layout.size.height = Length::Fill;
    layout.flex.shrink = 0.0;

    cx.container(
        ContainerProps {
            layout,
            padding: Edges::all(Px(16.0)).into(),
            background: Some(theme_snapshot.color_token("background")),
            border: Edges {
                left: Px(0.0),
                right: Px(1.0),
                top: Px(0.0),
                bottom: Px(0.0),
            },
            border_color: Some(theme_snapshot.color_token("border")),
            ..Default::default()
        },
        move |cx| {
            let label_row = |cx: &mut UiCx<'_>, label: &str, value: String| {
                ui::h_row(|cx| {
                    [
                        shadcn::Label::new(label).into_element(cx),
                        cx.spacer(SpacerProps::default()),
                        shadcn::Badge::new(value)
                            .variant(shadcn::BadgeVariant::Secondary)
                            .into_element(cx),
                    ]
                })
                .gap(Space::N2)
                .items_center()
                .into_element(cx)
            };

            let header = shadcn::CardHeader::new([
                shadcn::CardTitle::new("Theme Postprocess").into_element(cx),
                shadcn::CardDescription::new(
                    "Built-in steps (Pixelate/Dither/ColorAdjust) + CustomV1 (scanlines/vignette/chromatic/grain).",
                )
                .into_element(cx),
            ])
            .into_element(cx);

            let theme_row = ui::v_flex(move |cx| {
                vec![
                    label_row(cx, "Theme", theme.to_string()),
                    shadcn::Select::new(theme_model.clone(), theme_open_model.clone())
                        .value(shadcn::SelectValue::new().placeholder("Pick a theme"))
                        .items([
                            shadcn::SelectItem::new("none", "None"),
                            shadcn::SelectItem::new("cyberpunk", "Cyberpunk"),
                            shadcn::SelectItem::new("retro", "Retro"),
                        ])
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .into_element(cx);

            let chromatic_row = label_row(
                cx,
                "Chromatic offset",
                format!("{chromatic_offset_px:.2}px"),
            );
            let vignette_row =
                label_row(cx, "Vignette strength", format!("{vignette_strength:.2}"));
            let scanline_strength_row =
                label_row(cx, "Scanline strength", format!("{scanline_strength:.2}"));
            let scanline_spacing_row =
                label_row(cx, "Scanline spacing", format!("{scanline_spacing_px:.2}px"));

            let grain_strength_row = label_row(cx, "Grain strength", format!("{grain_strength:.2}"));
            let grain_scale_row = label_row(cx, "Grain scale", format!("{grain_scale:.2}"));

            let retro_pixel_row =
                label_row(cx, "Retro pixel scale", format!("{retro_pixel_scale:.2}"));
            let retro_dither_row = ui::h_row(move |cx| {
                [
                    shadcn::Switch::new(retro_dither_model.clone())
                        .a11y_label("Enable retro dither")
                        .test_id("postprocess.retro.dither")
                        .into_element(cx),
                    shadcn::Label::new(format!("Retro dither ({retro_dither})")).into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

            let content = shadcn::CardContent::new([ui::v_flex(move |cx| {
                vec![
                    ui::h_row(|cx| {
                        [
                            shadcn::Switch::new(enabled_model.clone())
                                .a11y_label("Enable postprocess")
                                .test_id("postprocess.enabled")
                                .into_element(cx),
                            shadcn::Label::new("Enable").into_element(cx),
                        ]
                    })
                    .gap(Space::N2)
                    .items_center()
                    .into_element(cx),
                    ui::h_row(|cx| {
                        [
                            shadcn::Switch::new(compare_model.clone())
                                .a11y_label("Compare raw vs processed")
                                .test_id("postprocess.compare")
                                .into_element(cx),
                            shadcn::Label::new("Compare (Raw vs Processed)")
                                .into_element(cx),
                        ]
                    })
                    .gap(Space::N2)
                    .items_center()
                    .into_element(cx),
                        theme_row,
                        shadcn::Separator::new().into_element(cx),
                        chromatic_row,
                        vignette_row,
                        scanline_strength_row,
                        scanline_spacing_row,
                        shadcn::Separator::new().into_element(cx),
                        grain_strength_row,
                        grain_scale_row,
                        shadcn::Separator::new().into_element(cx),
                        retro_pixel_row,
                        retro_dither_row,
                        shadcn::Slider::new(chromatic_state.clone())
                            .range(0.0, 6.0)
                            .step(0.25)
                            .into_element(cx),
                        shadcn::Slider::new(vignette_state.clone())
                            .range(0.0, 0.9)
                            .step(0.01)
                            .into_element(cx),
                        shadcn::Slider::new(scanline_strength_state.clone())
                            .range(0.0, 0.5)
                            .step(0.01)
                            .into_element(cx),
                        shadcn::Slider::new(scanline_spacing_state.clone())
                            .range(1.0, 10.0)
                            .step(0.25)
                            .into_element(cx),
                        shadcn::Slider::new(grain_strength_state.clone())
                            .range(0.0, 0.2)
                            .step(0.01)
                            .into_element(cx),
                        shadcn::Slider::new(grain_scale_state.clone())
                            .range(0.25, 6.0)
                            .step(0.05)
                            .into_element(cx),
                        shadcn::Slider::new(retro_pixel_scale_state.clone())
                            .range(2.0, 24.0)
                            .step(1.0)
                            .into_element(cx),
                        shadcn::Button::new("Reset")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .action(act::Reset)
                            .test_id("postprocess.reset")
                            .into_element(cx),
                ]
            })
            .gap(Space::N3)
            .items_stretch()
            .into_element(cx)])
            .into_element(cx);

            vec![
                shadcn::Card::new([header, content])
                    .ui()
                    .w_full()
                    .into_element(cx),
            ]
        },
    )
}

fn stage(
    cx: &mut UiCx<'_>,
    enabled: bool,
    compare: bool,
    theme: &str,
    effect: EffectId,
    chromatic_offset_px: f32,
    scanline_strength: f32,
    scanline_spacing_px: f32,
    vignette_strength: f32,
    grain_strength: f32,
    grain_scale: f32,
    retro_pixel_scale: f32,
    retro_dither: bool,
) -> impl IntoUiElement<KernelApp> + use<> {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout.flex.grow = 1.0;

    let theme = theme.to_string();
    let enabled = enabled && theme != "none";
    let compare = compare && enabled;

    let params = EffectParamsV1 {
        vec4s: [
            [
                chromatic_offset_px.clamp(0.0, 6.0),
                scanline_strength.clamp(0.0, 1.0),
                scanline_spacing_px.clamp(1.0, 12.0),
                vignette_strength.clamp(0.0, 1.0),
            ],
            [
                grain_strength.clamp(0.0, 0.25),
                grain_scale.clamp(0.1, 8.0),
                0.0,
                0.0,
            ],
            [0.0; 4],
            [0.0; 4],
        ],
    };

    let pixel_scale_u32 = retro_pixel_scale.round().clamp(2.0, 24.0) as u32;
    let max_sample_offset_px = Px(chromatic_offset_px.clamp(0.0, 6.0));

    let mut steps = Vec::new();
    if enabled {
        match theme.as_str() {
            "retro" => {
                steps.push(EffectStep::Pixelate {
                    scale: pixel_scale_u32,
                });
                if retro_dither {
                    steps.push(EffectStep::Dither {
                        mode: DitherMode::Bayer4x4,
                    });
                }
            }
            "cyberpunk" => {
                steps.push(EffectStep::ColorAdjust {
                    saturation: 1.15,
                    brightness: 1.05,
                    contrast: 1.2,
                });
            }
            _ => {}
        }
        steps.push(EffectStep::CustomV1 {
            id: effect,
            params,
            max_sample_offset_px,
        });
    }

    let chain = EffectChain::from_steps(&steps).sanitize();

    let raw_body = stage_body(cx, false, "Raw (unprocessed)");
    let raw_body = raw_body.into_element(cx);
    let processed_body = stage_body(cx, true, "Postprocess (filtered)");
    let processed_body = processed_body.into_element(cx);

    if !enabled {
        return raw_body;
    }

    let processed = cx.effect_layer_props(
        EffectLayerProps {
            layout,
            mode: EffectMode::FilterContent,
            chain,
            quality: EffectQuality::Auto,
        },
        move |_cx| vec![processed_body],
    );

    if !compare {
        return processed;
    }

    ui::h_flex(move |cx| {
        let mut cell_layout = LayoutStyle::default();
        cell_layout.size.width = Length::Fill;
        cell_layout.size.height = Length::Fill;
        cell_layout.flex.grow = 1.0;

        vec![
            cx.container(
                ContainerProps {
                    layout: cell_layout,
                    ..Default::default()
                },
                move |_cx| vec![raw_body],
            ),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .flex_stretch_cross_axis(true)
                .into_element(cx),
            cx.container(
                ContainerProps {
                    layout: cell_layout,
                    ..Default::default()
                },
                move |_cx| vec![processed],
            ),
        ]
    })
    .layout(LayoutRefinement::default().size_full())
    .items_stretch()
    .gap(Space::N0)
    .into_element(cx)
}

fn stage_body(
    cx: &mut UiCx<'_>,
    postprocess_applied: bool,
    label: &str,
) -> impl IntoUiElement<KernelApp> + use<> {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout.overflow = Overflow::Clip;

    cx.container(
        ContainerProps {
            layout,
            background: Some(srgb(6, 8, 14, 1.0)),
            ..Default::default()
        },
        move |cx| {
            let enabled_badge = if postprocess_applied {
                shadcn::Badge::new("ON")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx)
            } else {
                shadcn::Badge::new("OFF")
                    .variant(shadcn::BadgeVariant::Outline)
                    .into_element(cx)
            };

            let mut header_layout = LayoutStyle::default();
            header_layout.position = PositionStyle::Absolute;
            header_layout.inset.left = Some(Px(20.0)).into();
            header_layout.inset.top = Some(Px(18.0)).into();

            let title = cx.text_props(TextProps {
                layout: Default::default(),
                text: Arc::<str>::from("Theme-like Postprocess (CustomV1)"),
                style: None,
                color: Some(srgb(255, 255, 255, 0.92)),
                align: fret_core::TextAlign::Start,
                wrap: fret_core::TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
                ink_overflow: Default::default(),
            });

            let subtitle = cx.text_props(TextProps {
                layout: Default::default(),
                text: Arc::<str>::from(
                    "Scanlines + vignette + chromatic + grain (bounded, deterministic).",
                ),
                style: None,
                color: Some(srgb(255, 255, 255, 0.68)),
                align: fret_core::TextAlign::Start,
                wrap: fret_core::TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
                ink_overflow: Default::default(),
            });

            let header = cx.container(
                ContainerProps {
                    layout: header_layout,
                    padding: Edges {
                        left: Px(14.0),
                        right: Px(14.0),
                        top: Px(12.0),
                        bottom: Px(12.0),
                    }
                    .into(),
                    background: Some(srgb(0, 0, 0, 0.28)),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(srgb(255, 255, 255, 0.12)),
                    corner_radii: Corners::all(Px(14.0)),
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        ui::h_row(move |_cx| [title, enabled_badge])
                            .gap(Space::N2)
                            .items_center()
                            .into_element(cx),
                        subtitle,
                        shadcn::raw::typography::muted(label).into_element(cx),
                    ]
                },
            );

            let body = stage_cards(cx);
            let body = body.into_element(cx);
            vec![header, body]
        },
    )
}

fn stage_cards(cx: &mut UiCx<'_>) -> impl IntoUiElement<KernelApp> + use<> {
    let theme_snapshot = Theme::global(&*cx.app).snapshot();

    let card = |cx: &mut UiCx<'_>, title: &str, subtitle: &str| {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Px(Px(320.0));
        layout.size.height = Length::Px(Px(220.0));
        layout.overflow = Overflow::Clip;

        cx.container(
            ContainerProps {
                layout,
                padding: Edges::all(Px(14.0)).into(),
                background: Some(theme_snapshot.color_token("card")),
                border: Edges::all(Px(1.0)),
                border_color: Some(theme_snapshot.color_token("border")),
                corner_radii: Corners::all(Px(16.0)),
                ..Default::default()
            },
            move |cx| {
                vec![
                    shadcn::raw::typography::large(title).into_element(cx),
                    shadcn::raw::typography::muted(subtitle).into_element(cx),
                    cx.spacer(SpacerProps::default()),
                    shadcn::Button::new("Primary")
                        .variant(shadcn::ButtonVariant::Default)
                        .into_element(cx),
                    shadcn::Button::new("Secondary")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .into_element(cx),
                ]
            },
        )
    };

    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;

    cx.container(
        ContainerProps {
            layout,
            padding: Edges::all(Px(28.0)).into(),
            ..Default::default()
        },
        move |cx| {
            let row = ui::h_flex(move |cx| {
                vec![
                    card(cx, "UI sample", "Buttons + text to reveal postprocess."),
                    card(
                        cx,
                        "Small details",
                        "Scanlines + dither make edges obvious.",
                    ),
                ]
            })
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full())
            .into_element(cx);

            vec![row]
        },
    )
}
