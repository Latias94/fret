//! Custom effect demo (CustomV3).
//!
//! Native (desktop) authoring demo for the CustomV3 "lens" recipe. This intentionally uses the
//! `fret::mvu` path so it participates in the UI diagnostics + scripted testing pipeline
//! (`fretboard diag run`).

#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use fret::prelude::*;
use fret_core::scene::{
    CustomEffectPyramidRequestV1, CustomEffectSourcesV3, EffectChain, EffectMode, EffectParamsV1,
    EffectQuality, EffectStep,
};
use fret_core::{Color, Corners, Edges, EffectId, Px};
use fret_render::RendererCapabilities;
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, EffectLayerProps, LayoutStyle, Length, MainAlign,
    Overflow, PositionStyle, RowProps, SpacingLength, TextProps,
};
use fret_ui_kit::custom_effects::CustomEffectProgramV3;
use fret_ui_shadcn as shadcn;

use crate::custom_effect_v3_wgsl::CUSTOM_EFFECT_V3_LENS_WGSL;

#[derive(Debug)]
struct DemoGlobals {
    program: CustomEffectProgramV3,
}

impl DemoGlobals {
    fn new() -> Self {
        Self {
            program: CustomEffectProgramV3::wgsl_utf8(CUSTOM_EFFECT_V3_LENS_WGSL),
        }
    }
}

#[derive(Debug)]
struct State {
    enabled: Model<bool>,
}

struct Program;

pub fn run() -> anyhow::Result<()> {
    let builder = fret::mvu::app::<Program>("custom-effect-v3-demo")?
        .with_main_window("custom_effect_v3_demo", (1100.0, 720.0))
        .init_app(|app| {
            shadcn::shadcn_themes::apply_shadcn_new_york_v4(
                app,
                shadcn::shadcn_themes::ShadcnBaseColor::Slate,
                shadcn::shadcn_themes::ShadcnColorScheme::Dark,
            );
        });

    install_into(builder).run()?;
    Ok(())
}

/// Example of a “one line install” entrypoint for consumers on the native desktop builder path.
///
/// This is the intended pattern for third-party effect libraries:
/// - keep `EffectId` renderer-scoped and runtime-assigned,
/// - register lazily and cache the returned `EffectId`,
/// - keep the authoring demo small and diagnostics-friendly.
fn install_into<S: 'static>(builder: fret::UiAppBuilder<S>) -> fret::UiAppBuilder<S> {
    builder
        .install_app(install_app_globals)
        .install_custom_effects(register_custom_effect_v3)
}

fn install_app_globals(app: &mut App) {
    app.set_global(DemoGlobals::new());
}

fn register_custom_effect_v3(app: &mut App, effects: &mut dyn fret_core::CustomEffectService) {
    app.with_global_mut(DemoGlobals::new, |g, _app| {
        if let Err(err) = g.program.ensure_registered(effects) {
            tracing::warn!(?err, "custom effect v3 registration failed");
        }
    });
}

impl MvuProgram for Program {
    type State = State;
    type Message = ();

    fn init(app: &mut App, _window: AppWindowId) -> Self::State {
        Self::State {
            enabled: app.models_mut().insert(true),
        }
    }

    fn update(_app: &mut App, _st: &mut Self::State, _msg: Self::Message) {}

    fn view(
        cx: &mut ElementContext<'_, App>,
        st: &mut Self::State,
        _msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        view(cx, st)
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut State) -> Elements {
    // Animations make refraction far easier to see than static gradients.
    // Hold a continuous-frames lease so the backdrop moves without user input.
    let _frames = cx.begin_continuous_frames();

    let globals = cx.app.global::<DemoGlobals>();
    let effect = globals.and_then(|g| g.program.id());
    let supported = cx
        .app
        .global::<RendererCapabilities>()
        .is_some_and(|caps| caps.custom_effect_v3);
    let Some(effect) = effect else {
        let msg = if supported {
            "CustomV3 is unavailable (registration failed)"
        } else {
            "CustomV3 is unsupported on this backend"
        };
        return vec![shadcn::typography::h3(cx, msg)].into();
    };

    let enabled = cx.watch_model(&st.enabled).layout().copied_or(true);

    let stage = stage(cx, enabled, effect);

    let mut root_layout = LayoutStyle::default();
    root_layout.size.width = Length::Fill;
    root_layout.size.height = Length::Fill;

    let root = cx.container(
        ContainerProps {
            layout: root_layout,
            ..Default::default()
        },
        move |_cx| vec![stage],
    );

    vec![root].into()
}

fn stage(cx: &mut ElementContext<'_, App>, enabled: bool, effect: EffectId) -> AnyElement {
    let backdrop = animated_backdrop(cx);
    let lenses = lens_row(cx, enabled, effect);

    let title = shadcn::typography::h3(cx, "Custom Effect V3 (CustomV3)");
    let subtitle = shadcn::typography::muted(
        cx,
        "V3 can request renderer sources: src_raw + an optional bounded pyramid (for liquid glass ceilings).",
    );

    let mut header_layout = LayoutStyle::default();
    header_layout.size.width = Length::Fill;

    let header = cx.container(
        ContainerProps {
            layout: header_layout,
            padding: Edges::all(Px(12.0)).into(),
            background: Some(Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.38,
            }),
            corner_radii: Corners::all(Px(12.0)),
            ..Default::default()
        },
        move |cx| {
            vec![shadcn::stack::vstack(
                cx,
                shadcn::stack::VStackProps::default().gap(fret_ui_kit::Space::N1),
                |_cx| vec![title, subtitle],
            )]
        },
    );

    let mut content_layout = LayoutStyle::default();
    content_layout.size.width = Length::Fill;
    content_layout.size.height = Length::Fill;

    let content = cx.container(
        ContainerProps {
            layout: content_layout,
            padding: Edges {
                left: Px(24.0),
                right: Px(24.0),
                top: Px(20.0),
                bottom: Px(24.0),
            }
            .into(),
            ..Default::default()
        },
        move |cx| {
            vec![shadcn::stack::vstack(
                cx,
                shadcn::stack::VStackProps::default()
                    .gap(fret_ui_kit::Space::N4)
                    .items_start(),
                move |_cx| vec![header, lenses],
            )]
        },
    );

    let mut stage_layout = LayoutStyle::default();
    stage_layout.size.width = Length::Fill;
    stage_layout.size.height = Length::Fill;

    cx.container(
        ContainerProps {
            layout: stage_layout,
            ..Default::default()
        },
        move |_cx| vec![backdrop, content],
    )
}

fn animated_backdrop(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let viewport = cx.environment_viewport_bounds(Invalidation::Paint);
    let w = viewport.size.width.0.max(1.0);
    let h = viewport.size.height.0.max(1.0);

    // Use frame-driven motion so the demo stays deterministic under `diag` scripts.
    let t = (cx.frame_id.0 as f32) * (1.0 / 60.0);

    let cols = 18u32;
    let rows = 10u32;
    let tile_w = (w / cols as f32).max(1.0);
    let tile_h = (h / rows as f32).max(1.0);

    let mut layout = LayoutStyle::default();
    layout.position = PositionStyle::Absolute;
    layout.inset.left = Some(Px(0.0)).into();
    layout.inset.right = Some(Px(0.0)).into();
    layout.inset.top = Some(Px(0.0)).into();
    layout.inset.bottom = Some(Px(0.0)).into();

    cx.container(
        ContainerProps {
            layout,
            ..Default::default()
        },
        move |cx| {
            let mut out: Vec<AnyElement> = Vec::new();

            // High-frequency, slowly varying tile colors give the lens something to refract.
            for iy in 0..rows {
                for ix in 0..cols {
                    let x = ix as f32 * tile_w;
                    let y = iy as f32 * tile_h;
                    let phase = t * 0.75 + (ix as f32) * 0.27 + (iy as f32) * 0.19;
                    let c = Color {
                        r: 0.14 + 0.12 * phase.sin().abs(),
                        g: 0.12 + 0.14 * (phase * 1.3).cos().abs(),
                        b: 0.18 + 0.12 * (phase * 0.9).sin().abs(),
                        a: 1.0,
                    };

                    let mut tile_layout = LayoutStyle::default();
                    tile_layout.position = PositionStyle::Absolute;
                    tile_layout.inset.left = Some(Px(x)).into();
                    tile_layout.inset.top = Some(Px(y)).into();
                    tile_layout.size.width = Length::Px(Px(tile_w + 1.0));
                    tile_layout.size.height = Length::Px(Px(tile_h + 1.0));

                    out.push(
                        cx.container(
                            ContainerProps {
                                layout: tile_layout,
                                background: Some(c),
                                ..Default::default()
                            },
                            |_cx| Vec::<AnyElement>::new(),
                        )
                        .into(),
                    );
                }
            }

            // Moving bars create crisp edges; refraction becomes obvious even in stills.
            let bar_w = 56.0;
            let stride = 120.0;
            let speed = 140.0;
            let count = ((w / stride).ceil() as u32).max(10) + 2;
            for i in 0..count {
                let x = ((i as f32) * stride + t * speed).rem_euclid(w + stride) - stride;

                let mut bar_layout = LayoutStyle::default();
                bar_layout.position = PositionStyle::Absolute;
                bar_layout.inset.left = Some(Px(x)).into();
                bar_layout.inset.top = Some(Px(0.0)).into();
                bar_layout.size.width = Length::Px(Px(bar_w));
                bar_layout.size.height = Length::Fill;

                out.push(
                    cx.container(
                        ContainerProps {
                            layout: bar_layout,
                            background: Some(Color {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                                a: 0.04,
                            }),
                            ..Default::default()
                        },
                        |_cx| Vec::<AnyElement>::new(),
                    )
                    .into(),
                );
            }

            // A soft moving blob provides local contrast and lets you judge dispersion.
            let blob_r = 140.0;
            let blob_x = (w * 0.65 + (t * 0.7).sin() * (w * 0.18)).clamp(0.0, w);
            let blob_y = (h * 0.28 + (t * 0.9).cos() * (h * 0.12)).clamp(0.0, h);

            let mut blob_layout = LayoutStyle::default();
            blob_layout.position = PositionStyle::Absolute;
            blob_layout.inset.left = Some(Px(blob_x - blob_r)).into();
            blob_layout.inset.top = Some(Px(blob_y - blob_r)).into();
            blob_layout.size.width = Length::Px(Px(blob_r * 2.0));
            blob_layout.size.height = Length::Px(Px(blob_r * 2.0));

            out.push(
                cx.container(
                    ContainerProps {
                        layout: blob_layout,
                        background: Some(Color {
                            r: 0.96,
                            g: 0.92,
                            b: 0.25,
                            a: 0.20,
                        }),
                        corner_radii: Corners::all(Px(blob_r)),
                        ..Default::default()
                    },
                    |_cx| Vec::<AnyElement>::new(),
                )
                .into(),
            );

            out
        },
    )
}

fn lens_row(cx: &mut ElementContext<'_, App>, enabled: bool, effect: EffectId) -> AnyElement {
    let radius = Px(24.0);
    let lens_w = Px(360.0);
    let lens_h = Px(260.0);

    let mut row_layout = LayoutStyle::default();
    row_layout.size.width = Length::Fill;

    cx.row(
        RowProps {
            layout: row_layout,
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            gap: SpacingLength::Px(Px(36.0)),
            ..Default::default()
        },
        move |cx| {
            vec![
                plain_lens(cx, "Plain (no effect)", radius, lens_w, lens_h)
                    .test_id("custom-effect-v3-demo.lens_left"),
                if enabled {
                    custom_effect_lens(cx, "CustomV3 lens", effect, radius, lens_w, lens_h)
                        .test_id("custom-effect-v3-demo.lens_right")
                } else {
                    plain_lens(cx, "CustomV3 lens (disabled)", radius, lens_w, lens_h)
                        .test_id("custom-effect-v3-demo.lens_right")
                },
            ]
        },
    )
}

fn lens_shell(
    cx: &mut ElementContext<'_, App>,
    title: &'static str,
    radius: Px,
    lens_w: Px,
    lens_h: Px,
    with_effect: Option<EffectChain>,
) -> AnyElement {
    let mut lens_layout = LayoutStyle::default();
    lens_layout.size.width = Length::Px(lens_w);
    lens_layout.size.height = Length::Px(lens_h);
    lens_layout.overflow = Overflow::Clip;

    let mut fill_layout = LayoutStyle::default();
    fill_layout.size.width = Length::Fill;
    fill_layout.size.height = Length::Fill;

    let mut chrome_layout = fill_layout;
    chrome_layout.position = PositionStyle::Absolute;
    chrome_layout.inset.left = Some(Px(0.0)).into();
    chrome_layout.inset.right = Some(Px(0.0)).into();
    chrome_layout.inset.top = Some(Px(0.0)).into();
    chrome_layout.inset.bottom = Some(Px(0.0)).into();

    let mut label_layout = LayoutStyle::default();
    label_layout.position = PositionStyle::Absolute;
    label_layout.inset.left = Some(Px(14.0)).into();
    label_layout.inset.top = Some(Px(12.0)).into();

    cx.container(
        ContainerProps {
            layout: lens_layout,
            corner_radii: Corners::all(radius),
            ..Default::default()
        },
        move |cx| {
            let effect_layer = with_effect.map(|chain| {
                cx.effect_layer_props(
                    EffectLayerProps {
                        layout: fill_layout,
                        mode: EffectMode::Backdrop,
                        chain,
                        quality: EffectQuality::Auto,
                    },
                    |_cx| Vec::<AnyElement>::new(),
                )
            });

            let chrome = cx.container(
                ContainerProps {
                    layout: chrome_layout,
                    background: Some(Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.08,
                    }),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.12,
                    }),
                    corner_radii: Corners::all(radius),
                    ..Default::default()
                },
                |_cx| Vec::<AnyElement>::new(),
            );

            let label = cx.text_props(TextProps {
                layout: label_layout,
                text: Arc::from(title),
                style: None,
                color: Some(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.92,
                }),
                align: fret_core::TextAlign::Start,
                wrap: fret_core::TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
                ink_overflow: Default::default(),
            });

            let mut out = Vec::new();
            if let Some(layer) = effect_layer {
                out.push(layer);
            }
            out.push(chrome);
            out.push(label);
            out
        },
    )
}

fn plain_lens(
    cx: &mut ElementContext<'_, App>,
    title: &'static str,
    radius: Px,
    lens_w: Px,
    lens_h: Px,
) -> AnyElement {
    lens_shell(cx, title, radius, lens_w, lens_h, None)
}

fn custom_effect_lens(
    cx: &mut ElementContext<'_, App>,
    title: &'static str,
    effect: EffectId,
    radius: Px,
    lens_w: Px,
    lens_h: Px,
) -> AnyElement {
    let sf = cx.environment_scale_factor(Invalidation::Paint).max(1.0e-6);
    let params = EffectParamsV1 {
        vec4s: [
            // (refraction_height_px, refraction_amount_px, pyramid_level, frost_mix)
            [22.0 * sf, 34.0 * sf, 3.0, 0.75],
            // (corner_radius_px, depth_effect, dispersion, dispersion_quality)
            // - dispersion_quality: 0 = 3-tap, 1 = 7-tap Android-like.
            [radius.0 * sf, 0.18, 0.55, 1.0],
            // (noise_alpha, reserved, reserved, reserved)
            [0.012, 0.0, 0.0, 0.0],
            // tint (rgb + alpha)
            [1.0, 1.0, 1.0, 0.08],
        ],
    };

    let chain = EffectChain::from_steps(&[
        EffectStep::GaussianBlur {
            radius_px: Px(18.0),
            downsample: 2,
        },
        EffectStep::CustomV3 {
            id: effect,
            params,
            // Refraction + dispersion can reach beyond 40px at the rim; keep the bound generous.
            max_sample_offset_px: Px(96.0),
            user0: None,
            user1: None,
            sources: CustomEffectSourcesV3 {
                want_raw: true,
                pyramid: Some(CustomEffectPyramidRequestV1 {
                    max_levels: 6,
                    max_radius_px: Px(32.0),
                }),
            },
        },
    ])
    .sanitize();

    lens_shell(cx, title, radius, lens_w, lens_h, Some(chain))
}
