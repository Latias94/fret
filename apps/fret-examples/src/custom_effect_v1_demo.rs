//! Custom effect demo (CustomV1).
//!
//! Demonstrates the "escape hatch with a ceiling": app/ecosystem code registers a small WGSL
//! snippet on GPU readiness and uses the resulting `EffectId` in an `EffectChain`.

#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use fret::prelude::*;
use fret_core::scene::{EffectChain, EffectMode, EffectParamsV1, EffectQuality, EffectStep};
use fret_core::{Color, Corners, Edges, EffectId, Px};
use fret_runtime::Model;
use fret_ui::element::{
    ContainerProps, EffectLayerProps, LayoutStyle, Length, Overflow, PositionStyle, SpacerProps,
    TextProps,
};
use fret_ui_kit::custom_effects::CustomEffectProgramV1;
use fret_ui_kit::{Space, UiIntoElement};

const WGSL: &str = r#"
fn fret_custom_effect(src: vec4<f32>, uv: vec2<f32>, _pos_px: vec2<f32>, params: EffectParamsV1) -> vec4<f32> {
  let strength = clamp(params.vec4s[0].x, 0.0, 1.0);
  let r = distance(uv, vec2<f32>(0.5, 0.5));
  let vignette = 1.0 - smoothstep(0.2, 0.85, r);
  let inv = vec3<f32>(1.0, 1.0, 1.0) - src.rgb;
  let rgb = mix(src.rgb, inv, strength * vignette);
  return vec4<f32>(rgb, src.a);
}
"#;

#[derive(Debug, Clone, Copy)]
struct DemoEffect(EffectId);

#[derive(Debug)]
struct CustomEffectV1State {
    enabled: Model<bool>,
    strength: Model<Vec<f32>>,
}

struct CustomEffectV1Program;

#[derive(Debug, Clone)]
enum Msg {
    Reset,
}

pub fn run() -> anyhow::Result<()> {
    fret::mvu::app::<CustomEffectV1Program>("custom-effect-v1-demo")?
        .with_main_window("custom_effect_v1_demo", (1100.0, 720.0))
        .init_app(|app| {
            shadcn::shadcn_themes::apply_shadcn_new_york_v4(
                app,
                shadcn::shadcn_themes::ShadcnBaseColor::Slate,
                shadcn::shadcn_themes::ShadcnColorScheme::Dark,
            );
        })
        .install_custom_effects(install_custom_effect)
        .run()?;
    Ok(())
}

fn install_custom_effect(app: &mut App, effects: &mut dyn fret_core::CustomEffectService) {
    let mut program = CustomEffectProgramV1::wgsl_utf8(WGSL);
    let id = program
        .ensure_registered(effects)
        .expect("custom effect registration must succeed on wgpu backends");
    app.set_global(DemoEffect(id));
}

impl MvuProgram for CustomEffectV1Program {
    type State = CustomEffectV1State;
    type Message = Msg;

    fn init(app: &mut App, _window: AppWindowId) -> Self::State {
        Self::State {
            enabled: app.models_mut().insert(true),
            strength: app.models_mut().insert(vec![0.85]),
        }
    }

    fn update(app: &mut App, st: &mut Self::State, message: Self::Message) {
        if matches!(message, Msg::Reset) {
            let _ = app.models_mut().update(&st.enabled, |v| *v = true);
            let _ = app.models_mut().update(&st.strength, |v| *v = vec![0.85]);
        }
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        st: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        view(cx, st, msg)
    }
}

fn srgb(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color {
        r: (r as f32) / 255.0,
        g: (g as f32) / 255.0,
        b: (b as f32) / 255.0,
        a: a.clamp(0.0, 1.0),
    }
}

fn watch_first_f32(cx: &mut ElementContext<'_, App>, model: &Model<Vec<f32>>, default: f32) -> f32 {
    cx.watch_model(model)
        .layout()
        .read_ref(|v| v.first().copied().unwrap_or(default))
        .ok()
        .unwrap_or(default)
}

fn view(
    cx: &mut ElementContext<'_, App>,
    st: &mut CustomEffectV1State,
    msg: &mut MessageRouter<Msg>,
) -> Elements {
    let Some(effect) = cx.app.global::<DemoEffect>().map(|v| v.0) else {
        return vec![shadcn::typography::h3(cx, "Custom effects unavailable")].into();
    };

    let enabled = cx.watch_model(&st.enabled).layout().copied_or(true);
    let strength = watch_first_f32(cx, &st.strength, 0.85);

    let inspector = inspector(cx, st, strength, msg);
    let stage = stage(cx, enabled, effect, strength);

    let root = shadcn::stack::hstack(
        cx,
        shadcn::stack::HStackProps::default()
            .layout(LayoutRefinement::default().size_full())
            .items_stretch()
            .gap(Space::N0),
        move |_cx| vec![inspector, stage],
    );

    vec![root].into()
}

fn stage(
    cx: &mut ElementContext<'_, App>,
    enabled: bool,
    effect: EffectId,
    strength: f32,
) -> AnyElement {
    let lenses = lens_row(cx, enabled, effect, strength);

    let title = shadcn::typography::h3(cx, "Custom Effect V1 (CustomV1)");
    let subtitle = shadcn::typography::muted(
        cx,
        "The lens on the right runs a custom WGSL function and is clipped/scissored.",
    );

    let stripes = shadcn::stack::hstack(
        cx,
        shadcn::stack::HStackProps::default()
            .layout(LayoutRefinement::default().size_full())
            .gap(Space::N0)
            .items_stretch(),
        |cx| {
            (0..10)
                .map(|i| {
                    let t = (i as f32) / 9.0;
                    let c = Color {
                        r: (t * std::f32::consts::TAU).sin() * 0.5 + 0.5,
                        g: ((t + 0.33) * std::f32::consts::TAU).sin() * 0.5 + 0.5,
                        b: ((t + 0.66) * std::f32::consts::TAU).sin() * 0.5 + 0.5,
                        a: 1.0,
                    };

                    let mut stripe_layout = LayoutStyle::default();
                    stripe_layout.flex.grow = 1.0;
                    stripe_layout.size.height = Length::Fill;

                    cx.container(
                        ContainerProps {
                            layout: stripe_layout,
                            background: Some(c),
                            ..Default::default()
                        },
                        |_cx| Vec::<AnyElement>::new(),
                    )
                })
                .collect::<Vec<_>>()
        },
    );

    let mut stage_layout = LayoutStyle::default();
    stage_layout.size.width = Length::Fill;
    stage_layout.size.height = Length::Fill;
    stage_layout.flex.grow = 1.0;

    cx.container(
        ContainerProps {
            layout: stage_layout,
            background: Some(srgb(9, 12, 18, 1.0)),
            ..Default::default()
        },
        move |cx| {
            let stripes = stripes;

            let mut header_layout = LayoutStyle::default();
            header_layout.position = PositionStyle::Absolute;
            header_layout.inset.left = Some(Px(24.0)).into();
            header_layout.inset.top = Some(Px(20.0)).into();

            let header = cx.container(
                ContainerProps {
                    layout: header_layout,
                    padding: Edges::all(Px(12.0)).into(),
                    background: Some(srgb(0, 0, 0, 0.38)),
                    corner_radii: Corners::all(Px(12.0)),
                    ..Default::default()
                },
                move |cx| {
                    vec![shadcn::stack::vstack(
                        cx,
                        shadcn::stack::VStackProps::default().gap(Space::N1),
                        |_cx| vec![title, subtitle],
                    )]
                },
            );

            let mut lenses_layout = LayoutStyle::default();
            lenses_layout.position = PositionStyle::Absolute;
            lenses_layout.inset.left = Some(Px(24.0)).into();
            lenses_layout.inset.top = Some(Px(120.0)).into();

            let lenses = cx.container(
                ContainerProps {
                    layout: lenses_layout,
                    ..Default::default()
                },
                move |_cx| vec![lenses],
            );

            vec![stripes, header, lenses]
        },
    )
}

fn lens_row(
    cx: &mut ElementContext<'_, App>,
    enabled: bool,
    effect: EffectId,
    strength: f32,
) -> AnyElement {
    shadcn::stack::hstack(
        cx,
        shadcn::stack::HStackProps::default()
            .gap(Space::N3)
            .items_start(),
        move |cx| {
            vec![
                plain_lens(cx, "Plain (no effect)"),
                if enabled {
                    custom_effect_lens(cx, "CustomV1 lens", effect, strength)
                } else {
                    plain_lens(cx, "CustomV1 lens (disabled)")
                },
            ]
        },
    )
}

fn lens_shell(cx: &mut ElementContext<'_, App>, label: Arc<str>, body: AnyElement) -> AnyElement {
    let radius = Px(20.0);

    let mut outer_layout = LayoutStyle::default();
    outer_layout.size.width = Length::Px(Px(380.0));
    outer_layout.size.height = Length::Px(Px(240.0));
    outer_layout.overflow = Overflow::Clip;

    cx.container(
        ContainerProps {
            layout: outer_layout,
            corner_radii: Corners::all(radius),
            border: Edges::all(Px(1.0)),
            border_color: Some(srgb(255, 255, 255, 0.22)),
            ..Default::default()
        },
        move |cx| {
            let mut label_layout = LayoutStyle::default();
            label_layout.position = PositionStyle::Absolute;
            label_layout.inset.left = Some(Px(12.0)).into();
            label_layout.inset.top = Some(Px(12.0)).into();

            let title = cx.text_props(TextProps {
                layout: Default::default(),
                text: label.clone(),
                style: None,
                color: Some(srgb(255, 255, 255, 0.92)),
                align: fret_core::TextAlign::Start,
                wrap: fret_core::TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
                ink_overflow: Default::default(),
            });

            let pill = cx.container(
                ContainerProps {
                    layout: label_layout,
                    padding: Edges {
                        left: Px(10.0),
                        right: Px(10.0),
                        top: Px(6.0),
                        bottom: Px(6.0),
                    }
                    .into(),
                    background: Some(srgb(0, 0, 0, 0.45)),
                    corner_radii: Corners::all(Px(999.0)),
                    ..Default::default()
                },
                move |cx| vec![title.into_element(cx)],
            );

            vec![body, pill]
        },
    )
}

fn plain_lens(cx: &mut ElementContext<'_, App>, label: impl Into<Arc<str>>) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;

    let body = cx.container(
        ContainerProps {
            layout,
            background: Some(srgb(15, 23, 42, 0.35)),
            ..Default::default()
        },
        |_cx| Vec::<AnyElement>::new(),
    );

    lens_shell(cx, label.into(), body)
}

fn custom_effect_lens(
    cx: &mut ElementContext<'_, App>,
    label: impl Into<Arc<str>>,
    effect: EffectId,
    strength: f32,
) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;

    let params = EffectParamsV1 {
        vec4s: [
            [strength.clamp(0.0, 1.0), 0.0, 0.0, 0.0],
            [0.0; 4],
            [0.0; 4],
            [0.0; 4],
        ],
    };

    let chain = EffectChain::from_steps(&[EffectStep::CustomV1 { id: effect, params }]).sanitize();

    let layer = cx.effect_layer_props(
        EffectLayerProps {
            layout,
            mode: EffectMode::Backdrop,
            chain,
            quality: EffectQuality::Auto,
        },
        |_cx| Vec::<AnyElement>::new(),
    );

    lens_shell(cx, label.into(), layer)
}

fn inspector(
    cx: &mut ElementContext<'_, App>,
    st: &mut CustomEffectV1State,
    strength: f32,
    msg: &mut MessageRouter<Msg>,
) -> AnyElement {
    let reset_cmd = msg.cmd(Msg::Reset);
    let enabled_model = st.enabled.clone();
    let strength_model = st.strength.clone();

    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Px(Px(360.0));
    layout.size.height = Length::Fill;
    layout.flex.shrink = 0.0;

    cx.container(
        ContainerProps {
            layout,
            padding: Edges::all(Px(16.0)).into(),
            background: Some(srgb(17, 19, 24, 0.92)),
            ..Default::default()
        },
        move |cx| {
            vec![shadcn::stack::vstack(
                cx,
                shadcn::stack::VStackProps::default()
                    .gap(Space::N3)
                    .items_stretch(),
                move |cx| {
                    let strength_row = shadcn::stack::vstack(
                        cx,
                        shadcn::stack::VStackProps::default().gap(Space::N2),
                        move |cx| {
                            vec![
                                shadcn::stack::hstack(
                                    cx,
                                    shadcn::stack::HStackProps::default()
                                        .gap(Space::N2)
                                        .items_center(),
                                    move |cx| {
                                        vec![
                                            shadcn::Label::new("Strength").into_element(cx),
                                            cx.spacer(SpacerProps::default()),
                                            shadcn::Badge::new(format!("{strength:.2}"))
                                                .variant(shadcn::BadgeVariant::Secondary)
                                                .into_element(cx),
                                        ]
                                    },
                                ),
                                shadcn::Slider::new(strength_model.clone())
                                    .range(0.0, 1.0)
                                    .step(0.01)
                                    .into_element(cx),
                            ]
                        },
                    );

                    vec![
                        shadcn::typography::h4(cx, "Custom Effect V1"),
                        shadcn::typography::muted(
                            cx,
                            "Registers WGSL at on_gpu_ready and applies EffectStep::CustomV1.",
                        ),
                        shadcn::Separator::new().into_element(cx),
                        shadcn::stack::hstack(
                            cx,
                            shadcn::stack::HStackProps::default()
                                .gap(Space::N2)
                                .items_center(),
                            |cx| {
                                vec![
                                    shadcn::Switch::new(enabled_model.clone())
                                        .a11y_label("Enable custom effect")
                                        .test_id("custom-effect-v1.enabled")
                                        .into_element(cx),
                                    shadcn::Label::new("Enable").into_element(cx),
                                ]
                            },
                        ),
                        strength_row,
                        shadcn::Button::new("Reset")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .on_click(reset_cmd.clone())
                            .test_id("custom-effect-v1.reset")
                            .into_element(cx),
                    ]
                },
            )]
        },
    )
}
