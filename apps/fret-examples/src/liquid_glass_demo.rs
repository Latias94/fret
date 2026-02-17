use std::sync::Arc;

use fret::prelude::*;
use fret_core::scene::{
    ColorSpace, DitherMode, EffectChain, EffectMode, EffectQuality, EffectStep, GradientStop,
    LinearGradient, MAX_STOPS, Paint, TileMode,
};
use fret_core::{Color, Corners, Edges, Point, Px};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::element::{
    ColumnProps, ContainerProps, CrossAlign, EffectLayerProps, InsetStyle, LayoutStyle, Length,
    MainAlign, Overflow, PositionStyle, RowProps, SizeStyle, SpacerProps,
};

fn clamp01(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn linear_gradient(stops: &[(f32, Color)], start: Point, end: Point) -> Paint {
    let mut arr = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
    let mut count: u8 = 0;
    for (i, (offset, color)) in stops.iter().copied().enumerate().take(MAX_STOPS) {
        arr[i] = GradientStop::new(clamp01(offset), color);
        count = (i as u8) + 1;
    }

    Paint::LinearGradient(LinearGradient {
        start,
        end,
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count: count,
        stops: arr,
    })
}

fn srgb(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color {
        r: (r as f32) / 255.0,
        g: (g as f32) / 255.0,
        b: (b as f32) / 255.0,
        a: a.clamp(0.0, 1.0),
    }
}

fn rainbow_stripe(t: f32, a: f32) -> Color {
    let t = if t.is_finite() { t } else { 0.0 };
    let r = (t * std::f32::consts::TAU).sin() * 0.5 + 0.5;
    let g = ((t + 0.33) * std::f32::consts::TAU).sin() * 0.5 + 0.5;
    let b = ((t + 0.66) * std::f32::consts::TAU).sin() * 0.5 + 0.5;
    Color { r, g, b, a }
}

#[derive(Clone)]
struct LiquidGlassState {
    blur_radius_px: Model<Vec<f32>>,
    blur_downsample: Model<Vec<f32>>,
    saturation: Model<Vec<f32>>,
    brightness: Model<Vec<f32>>,
    contrast: Model<Vec<f32>>,
    use_backdrop: Model<bool>,
    use_dither: Model<bool>,
}

#[derive(Debug, Clone)]
enum Msg {
    Reset,
}

struct LiquidGlassProgram;

pub fn run() -> anyhow::Result<()> {
    fret::mvu::app::<LiquidGlassProgram>("liquid-glass-demo")?
        .with_main_window("liquid_glass_demo", (980.0, 700.0))
        .init_app(|app| {
            shadcn::shadcn_themes::apply_shadcn_new_york_v4(
                app,
                shadcn::shadcn_themes::ShadcnBaseColor::Slate,
                shadcn::shadcn_themes::ShadcnColorScheme::Dark,
            );
        })
        .run()?;
    Ok(())
}

impl MvuProgram for LiquidGlassProgram {
    type State = LiquidGlassState;
    type Message = Msg;

    fn init(app: &mut App, _window: AppWindowId) -> Self::State {
        Self::State {
            blur_radius_px: app.models_mut().insert(vec![18.0]),
            blur_downsample: app.models_mut().insert(vec![2.0]),
            saturation: app.models_mut().insert(vec![1.10]),
            brightness: app.models_mut().insert(vec![1.02]),
            contrast: app.models_mut().insert(vec![1.02]),
            use_backdrop: app.models_mut().insert(true),
            use_dither: app.models_mut().insert(true),
        }
    }

    fn update(app: &mut App, state: &mut Self::State, message: Self::Message) {
        match message {
            Msg::Reset => {
                let _ = app
                    .models_mut()
                    .update(&state.blur_radius_px, |v| *v = vec![18.0]);
                let _ = app
                    .models_mut()
                    .update(&state.blur_downsample, |v| *v = vec![2.0]);
                let _ = app
                    .models_mut()
                    .update(&state.saturation, |v| *v = vec![1.10]);
                let _ = app
                    .models_mut()
                    .update(&state.brightness, |v| *v = vec![1.02]);
                let _ = app
                    .models_mut()
                    .update(&state.contrast, |v| *v = vec![1.02]);
                let _ = app.models_mut().update(&state.use_backdrop, |v| *v = true);
                let _ = app.models_mut().update(&state.use_dither, |v| *v = true);
            }
        }
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        state: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        view(cx, state, msg)
    }
}

fn watch_first_f32(cx: &mut ElementContext<'_, App>, model: &Model<Vec<f32>>, default: f32) -> f32 {
    cx.watch_model(model)
        .layout()
        .read_ref(|v| v.first().copied().unwrap_or(default))
        .ok()
        .unwrap_or(default)
}

fn build_effect_chain(
    blur_radius_px: f32,
    blur_downsample: u32,
    saturation: f32,
    brightness: f32,
    contrast: f32,
    dither: bool,
) -> EffectChain {
    let blur_radius_px = if blur_radius_px.is_finite() {
        blur_radius_px.clamp(0.0, 64.0)
    } else {
        0.0
    };

    let blur_downsample = blur_downsample.clamp(1, 4);
    let saturation = if saturation.is_finite() {
        saturation.clamp(0.0, 3.0)
    } else {
        1.0
    };
    let brightness = if brightness.is_finite() {
        brightness.clamp(0.0, 3.0)
    } else {
        1.0
    };
    let contrast = if contrast.is_finite() {
        contrast.clamp(0.0, 3.0)
    } else {
        1.0
    };

    let mut steps: Vec<EffectStep> = Vec::new();
    if blur_radius_px > 0.0 {
        steps.push(EffectStep::GaussianBlur {
            radius_px: Px(blur_radius_px),
            downsample: blur_downsample,
        });
    }

    let needs_color_adjust = (saturation - 1.0).abs() > 1e-6
        || (brightness - 1.0).abs() > 1e-6
        || (contrast - 1.0).abs() > 1e-6;
    if needs_color_adjust {
        steps.push(EffectStep::ColorAdjust {
            saturation,
            brightness,
            contrast,
        });
    }

    if dither {
        steps.push(EffectStep::Dither {
            mode: DitherMode::Bayer4x4,
        });
    }

    EffectChain::from_steps(&steps)
}

fn glass_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutStyle,
    mode: EffectMode,
    chain: EffectChain,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    let _theme = Theme::global(&*cx.app).snapshot();
    let radius = Px(22.0);

    let outer = ContainerProps {
        layout: LayoutStyle {
            overflow: Overflow::Clip,
            ..layout
        },
        corner_radii: Corners::all(radius),
        ..Default::default()
    };

    cx.container(outer, move |cx| {
        let mut effect_layout = LayoutStyle::default();
        effect_layout.size.width = Length::Fill;
        effect_layout.size.height = Length::Fill;

        let highlight = linear_gradient(
            &[
                (0.0, srgb(255, 255, 255, 0.16)),
                (0.42, srgb(255, 255, 255, 0.03)),
                (1.0, srgb(255, 255, 255, 0.00)),
            ],
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(560.0), Px(360.0)),
        );

        let layer = cx.effect_layer_props(
            EffectLayerProps {
                layout: effect_layout,
                mode,
                chain,
                quality: EffectQuality::Auto,
            },
            move |cx| {
                let mut inner_layout = LayoutStyle::default();
                inner_layout.size.width = Length::Fill;
                inner_layout.size.height = Length::Fill;

                let inner = ContainerProps {
                    layout: inner_layout,
                    padding: Edges::all(Px(18.0)),
                    background: Some(srgb(20, 22, 28, 0.18)),
                    background_paint: Some(highlight),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(srgb(255, 255, 255, 0.16)),
                    corner_radii: Corners::all(radius),
                    ..Default::default()
                };

                vec![cx.container(inner, children)]
            },
        );

        vec![layer]
    })
}

fn view(
    cx: &mut ElementContext<'_, App>,
    st: &mut LiquidGlassState,
    msg: &mut MessageRouter<Msg>,
) -> Elements {
    let theme = Theme::global(&*cx.app).snapshot();
    let viewport = cx.environment_viewport_bounds(Invalidation::Layout);

    let blur_radius_model = st.blur_radius_px.clone();
    let blur_downsample_model = st.blur_downsample.clone();
    let saturation_model = st.saturation.clone();
    let brightness_model = st.brightness.clone();
    let contrast_model = st.contrast.clone();
    let use_backdrop_model = st.use_backdrop.clone();
    let use_dither_model = st.use_dither.clone();

    let blur_radius_px = watch_first_f32(cx, &blur_radius_model, 18.0);
    let blur_downsample_raw = watch_first_f32(cx, &blur_downsample_model, 2.0);
    let saturation = watch_first_f32(cx, &saturation_model, 1.1);
    let brightness = watch_first_f32(cx, &brightness_model, 1.02);
    let contrast = watch_first_f32(cx, &contrast_model, 1.02);
    let use_backdrop = cx.watch_model(&use_backdrop_model).layout().copied_or(true);
    let use_dither = cx.watch_model(&use_dither_model).layout().copied_or(true);

    let blur_downsample = blur_downsample_raw.round().clamp(1.0, 4.0) as u32;

    let chain = build_effect_chain(
        blur_radius_px,
        blur_downsample,
        saturation,
        brightness,
        contrast,
        use_dither,
    );

    let mode = if use_backdrop {
        EffectMode::Backdrop
    } else {
        EffectMode::FilterContent
    };

    let mut root_layout = LayoutStyle::default();
    root_layout.size = SizeStyle {
        width: Length::Fill,
        height: Length::Fill,
        ..Default::default()
    };
    root_layout.position = PositionStyle::Relative;

    let bg = linear_gradient(
        &[
            (0.0, srgb(10, 12, 18, 1.0)),
            (0.38, srgb(21, 16, 46, 1.0)),
            (0.70, srgb(6, 40, 44, 1.0)),
            (1.0, srgb(10, 12, 18, 1.0)),
        ],
        viewport.origin,
        Point::new(
            Px(viewport.origin.x.0 + viewport.size.width.0),
            Px(viewport.origin.y.0 + viewport.size.height.0),
        ),
    );

    let root = cx.container(
        ContainerProps {
            layout: root_layout,
            background_paint: Some(bg),
            ..Default::default()
        },
        move |cx| {
            // Background stripes (high-frequency detail helps reveal blur/refinement changes).
            let mut stripes_layout = LayoutStyle::default();
            stripes_layout.size.width = Length::Fill;
            stripes_layout.size.height = Length::Fill;
            stripes_layout.position = PositionStyle::Absolute;
            stripes_layout.inset = InsetStyle {
                top: Some(Px(0.0)),
                right: Some(Px(0.0)),
                bottom: Some(Px(0.0)),
                left: Some(Px(0.0)),
            };

            let stripe_w = Px(26.0);
            let stripe_count = ((viewport.size.width.0 / stripe_w.0).ceil() as usize).max(1) + 1;
            let stripes = cx.row(
                RowProps {
                    layout: stripes_layout,
                    gap: Px(0.0),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    ..Default::default()
                },
                move |cx| {
                    let mut out = Vec::with_capacity(stripe_count);
                    for i in 0..stripe_count {
                        let t = if stripe_count > 1 {
                            (i as f32) / ((stripe_count - 1) as f32)
                        } else {
                            0.0
                        };
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(stripe_w);
                        layout.size.height = Length::Fill;

                        let stripe = cx.container(
                            ContainerProps {
                                layout,
                                background: Some(rainbow_stripe(t, 0.11)),
                                ..Default::default()
                            },
                            |_cx| Vec::<AnyElement>::new(),
                        );
                        out.push(stripe);
                    }
                    out
                },
            );

            // Foreground: centered glass panel with controls.
            let mut overlay_layout = LayoutStyle::default();
            overlay_layout.size.width = Length::Fill;
            overlay_layout.size.height = Length::Fill;
            overlay_layout.position = PositionStyle::Absolute;
            overlay_layout.inset = InsetStyle {
                top: Some(Px(0.0)),
                right: Some(Px(0.0)),
                bottom: Some(Px(0.0)),
                left: Some(Px(0.0)),
            };

            let reset = msg.cmd(Msg::Reset);
            let title: Arc<str> = Arc::from("Liquid Glass (Backdrop Blur) Demo");
            let subtitle: Arc<str> = Arc::from(
                "This is a \"fake\" liquid glass: backdrop blur + color adjust + subtle dither. \
True refraction/distortion would need a dedicated effect step/material.",
            );

            let mut panel_layout = LayoutStyle::default();
            panel_layout.size.width = Length::Px(Px(660.0));
            panel_layout.size.height = Length::Auto;

            let panel = glass_panel(cx, panel_layout, mode, chain, move |cx| {
                let header = shadcn::CardHeader::new([
                    shadcn::CardTitle::new(title.clone()).into_element(cx),
                    shadcn::CardDescription::new(subtitle.clone()).into_element(cx),
                ]);

                let label_row = |cx: &mut ElementContext<'_, App>, label: &str, value: String| {
                    shadcn::stack::hstack(
                        cx,
                        shadcn::stack::HStackProps::default()
                            .gap(Space::N2)
                            .items_center(),
                        move |cx| {
                            vec![
                                shadcn::Label::new(label).into_element(cx),
                                cx.spacer(SpacerProps::default()),
                                shadcn::Badge::new(value)
                                    .variant(shadcn::BadgeVariant::Secondary)
                                    .into_element(cx),
                            ]
                        },
                    )
                };

                let blur_row = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default().gap(Space::N2),
                    |cx| {
                        vec![
                            label_row(
                                cx,
                                "Blur radius (px)",
                                format!("{:.1}", blur_radius_px.clamp(0.0, 64.0)),
                            ),
                            shadcn::Slider::new(blur_radius_model.clone())
                                .range(0.0, 48.0)
                                .step(0.5)
                                .into_element(cx),
                        ]
                    },
                );

                let downsample_row = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default().gap(Space::N2),
                    |cx| {
                        vec![
                            label_row(cx, "Blur downsample", format!("{blur_downsample}x")),
                            shadcn::Slider::new(blur_downsample_model.clone())
                                .range(1.0, 4.0)
                                .step(1.0)
                                .into_element(cx),
                        ]
                    },
                );

                let sat_row = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default().gap(Space::N2),
                    |cx| {
                        vec![
                            label_row(cx, "Saturation", format!("{:.2}", saturation)),
                            shadcn::Slider::new(saturation_model.clone())
                                .range(0.6, 1.8)
                                .step(0.01)
                                .into_element(cx),
                        ]
                    },
                );

                let bright_row = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default().gap(Space::N2),
                    |cx| {
                        vec![
                            label_row(cx, "Brightness", format!("{:.2}", brightness)),
                            shadcn::Slider::new(brightness_model.clone())
                                .range(0.8, 1.3)
                                .step(0.01)
                                .into_element(cx),
                        ]
                    },
                );

                let contrast_row = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default().gap(Space::N2),
                    |cx| {
                        vec![
                            label_row(cx, "Contrast", format!("{:.2}", contrast)),
                            shadcn::Slider::new(contrast_model.clone())
                                .range(0.8, 1.3)
                                .step(0.01)
                                .into_element(cx),
                        ]
                    },
                );

                let toggles = shadcn::stack::hstack(
                    cx,
                    shadcn::stack::HStackProps::default()
                        .gap(Space::N4)
                        .items_center(),
                    |cx| {
                        vec![
                            shadcn::stack::hstack(
                                cx,
                                shadcn::stack::HStackProps::default()
                                    .gap(Space::N2)
                                    .items_center(),
                                |cx| {
                                    vec![
                                        shadcn::Switch::new(use_backdrop_model.clone())
                                            .a11y_label("Use backdrop mode")
                                            .into_element(cx),
                                        shadcn::Label::new("Backdrop").into_element(cx),
                                    ]
                                },
                            ),
                            shadcn::stack::hstack(
                                cx,
                                shadcn::stack::HStackProps::default()
                                    .gap(Space::N2)
                                    .items_center(),
                                |cx| {
                                    vec![
                                        shadcn::Switch::new(use_dither_model.clone())
                                            .a11y_label("Use dither")
                                            .into_element(cx),
                                        shadcn::Label::new("Dither").into_element(cx),
                                    ]
                                },
                            ),
                            cx.spacer(SpacerProps::default()),
                            shadcn::Button::new("Reset")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .size(shadcn::ButtonSize::Sm)
                                .on_click(reset.clone())
                                .into_element(cx),
                        ]
                    },
                );

                let body = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default()
                        .gap(Space::N4)
                        .items_stretch(),
                    |cx| {
                        vec![
                            header.into_element(cx),
                            shadcn::Separator::new().into_element(cx),
                            blur_row,
                            downsample_row,
                            sat_row,
                            bright_row,
                            contrast_row,
                            shadcn::Separator::new().into_element(cx),
                            toggles,
                            cx.text_props(TextProps {
                                layout: Default::default(),
                                text: Arc::from(format!(
                                    "Mode: {:?} | Steps: {}",
                                    mode,
                                    chain.iter().count()
                                )),
                                style: None,
                                color: Some(theme.color_token("muted-foreground")),
                                align: fret_core::TextAlign::Start,
                                wrap: fret_core::TextWrap::None,
                                overflow: fret_core::TextOverflow::Clip,
                            }),
                        ]
                    },
                );

                vec![body]
            });

            let overlay = cx.column(
                ColumnProps {
                    layout: overlay_layout,
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    ..Default::default()
                },
                move |_cx| vec![panel],
            );

            vec![stripes, overlay]
        },
    );

    vec![root].into()
}
