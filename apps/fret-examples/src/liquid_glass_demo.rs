//! Liquid glass demo (BackdropWarpV1 / BackdropWarpV2).
//!
//! This demo intentionally keeps the "stage" visible and places two small lenses on top:
//! - Fake glass: blur + color adjust
//! - True warp (v1): BackdropWarpV1 + blur + color adjust
//! - True warp (v2): BackdropWarpV2 (image warp field) + blur + color adjust

#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use fret::prelude::*;
use fret_core::scene::{
    BackdropWarpFieldV2, BackdropWarpKindV1, BackdropWarpV1, BackdropWarpV2, DitherMode,
    EffectChain, EffectMode, EffectQuality, EffectStep, ImageSamplingHint, UvRect,
    WarpMapEncodingV1,
};
use fret_core::{Color, Corners, Edges, ImageColorSpace, Px};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::element::{
    ContainerProps, CrossAlign, EffectLayerProps, InsetStyle, LayoutStyle, Length, MainAlign,
    Overflow, PositionStyle, RowProps, SizeStyle, SpacerProps, TextProps,
};
use fret_ui_assets::image_asset_cache::{ImageAssetCacheHostExt, ImageAssetKey};
use fret_ui_kit::Space;
use fret_ui_shadcn as shadcn;

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

fn watch_first_f32(cx: &mut ElementContext<'_, App>, model: &Model<Vec<f32>>, default: f32) -> f32 {
    cx.watch_model(model)
        .layout()
        .read_ref(|v| v.first().copied().unwrap_or(default))
        .ok()
        .unwrap_or(default)
}

fn build_chain(
    warp: Option<EffectStep>,
    blur_radius_px: f32,
    blur_downsample: u32,
    saturation: f32,
    brightness: f32,
    contrast: f32,
    dither: bool,
) -> EffectChain {
    let mut steps: Vec<EffectStep> = Vec::new();
    if let Some(step) = warp {
        steps.push(step);
    }
    if blur_radius_px > 0.0 && steps.len() < EffectChain::MAX_STEPS {
        steps.push(EffectStep::GaussianBlur {
            radius_px: Px(blur_radius_px.clamp(0.0, 64.0)),
            downsample: blur_downsample.clamp(1, 4),
        });
    }
    if steps.len() < EffectChain::MAX_STEPS {
        steps.push(EffectStep::ColorAdjust {
            saturation: saturation.clamp(0.0, 3.0),
            brightness: brightness.clamp(0.0, 3.0),
            contrast: contrast.clamp(0.0, 3.0),
        });
    }
    if dither && steps.len() < EffectChain::MAX_STEPS {
        steps.push(EffectStep::Dither {
            mode: DitherMode::Bayer4x4,
        });
    }
    EffectChain::from_steps(&steps).sanitize()
}

fn generate_warp_map_rg_signed(width: u32, height: u32) -> Vec<u8> {
    let w = width.max(1);
    let h = height.max(1);
    let mut out = vec![0u8; (w as usize) * (h as usize) * 4];

    for y in 0..h {
        for x in 0..w {
            let u = if w > 1 {
                (x as f32) / ((w - 1) as f32)
            } else {
                0.0
            };
            let v = if h > 1 {
                (y as f32) / ((h - 1) as f32)
            } else {
                0.0
            };

            let p = glam::Vec2::new(u - 0.5, v - 0.5);
            let r = p.length().min(1.0);
            let a = p.y.atan2(p.x);

            let amp = 0.22 * (1.0 - r).powf(0.6);
            let dx = (a * 4.0 + r * 18.0).sin() * amp;
            let dy = (a * 3.0 - r * 16.0).cos() * amp;

            let rr = ((dx * 0.5 + 0.5) * 255.0).round().clamp(0.0, 255.0) as u8;
            let gg = ((dy * 0.5 + 0.5) * 255.0).round().clamp(0.0, 255.0) as u8;

            let idx = ((y as usize) * (w as usize) + (x as usize)) * 4;
            out[idx] = rr;
            out[idx + 1] = gg;
            out[idx + 2] = 128;
            out[idx + 3] = 255;
        }
    }

    out
}

fn lens_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    mode: EffectMode,
    chain: EffectChain,
) -> AnyElement {
    let radius = Px(20.0);
    let mut outer_layout = LayoutStyle::default();
    outer_layout.size.width = Length::Px(Px(320.0));
    outer_layout.size.height = Length::Px(Px(220.0));
    outer_layout.overflow = Overflow::Clip;

    cx.container(
        ContainerProps {
            layout: outer_layout,
            corner_radii: Corners::all(radius),
            border: Edges::all(Px(1.0)),
            border_color: Some(srgb(255, 255, 255, 0.24)),
            ..Default::default()
        },
        move |cx| {
            let mut effect_layout = LayoutStyle::default();
            effect_layout.size.width = Length::Fill;
            effect_layout.size.height = Length::Fill;

            let layer = cx.effect_layer_props(
                EffectLayerProps {
                    layout: effect_layout,
                    mode,
                    chain,
                    quality: EffectQuality::Auto,
                },
                move |_cx| {
                    // Do not draw any fill chrome inside the effect scope: any visible pixels
                    // should come from the backdrop sampling semantics (warp/blur/adjust).
                    Vec::<AnyElement>::new()
                },
            );

            let mut label_layout = LayoutStyle::default();
            label_layout.position = PositionStyle::Absolute;
            label_layout.inset.left = Some(Px(12.0));
            label_layout.inset.top = Some(Px(12.0));

            let title = cx.text_props(TextProps {
                layout: Default::default(),
                text: label.clone(),
                style: None,
                color: Some(srgb(255, 255, 255, 0.92)),
                align: fret_core::TextAlign::Start,
                wrap: fret_core::TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
            });

            let pill = cx.container(
                ContainerProps {
                    layout: label_layout,
                    padding: Edges {
                        left: Px(10.0),
                        right: Px(10.0),
                        top: Px(6.0),
                        bottom: Px(6.0),
                    },
                    background: Some(srgb(10, 12, 18, 0.32)),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(srgb(255, 255, 255, 0.18)),
                    corner_radii: Corners::all(Px(999.0)),
                    ..Default::default()
                },
                move |_cx| vec![title],
            );

            vec![layer, pill]
        },
    )
}

#[derive(Clone)]
struct LiquidGlassState {
    show_fake: Model<bool>,
    show_warp: Model<bool>,
    show_warp_v2: Model<bool>,
    show_inspector: Model<bool>,
    animate: Model<bool>,
    phase_speed: Model<Vec<f32>>,

    warp_map_size: (u32, u32),
    warp_map_key: ImageAssetKey,
    warp_map_rgba: Arc<Vec<u8>>,

    warp_strength_px: Model<Vec<f32>>,
    warp_scale_px: Model<Vec<f32>>,
    warp_phase: Model<Vec<f32>>,
    warp_chroma_px: Model<Vec<f32>>,

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
    ToggleInspector,
}

struct LiquidGlassProgram;

pub fn run() -> anyhow::Result<()> {
    fret::mvu::app::<LiquidGlassProgram>("liquid-glass-demo")?
        .with_main_window("liquid_glass_demo", (1280.0, 720.0))
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
        let warp_map_size = (128u32, 128u32);
        let warp_map_rgba = generate_warp_map_rg_signed(warp_map_size.0, warp_map_size.1);
        let warp_map_key = ImageAssetKey::from_rgba8(
            warp_map_size.0,
            warp_map_size.1,
            ImageColorSpace::Linear,
            &warp_map_rgba,
        );
        let warp_map_rgba = Arc::new(warp_map_rgba);

        Self::State {
            // Important: keep these defaults stable because perf scripts/baselines assume them.
            // - v1 baseline expects fake + v1 visible by default.
            // - v2 script toggles fake/v1 off and v2 on deterministically.
            show_fake: app.models_mut().insert(true),
            show_warp: app.models_mut().insert(true),
            show_warp_v2: app.models_mut().insert(false),
            show_inspector: app.models_mut().insert(false),
            animate: app.models_mut().insert(true),
            phase_speed: app.models_mut().insert(vec![0.65]),

            warp_map_size,
            warp_map_key,
            warp_map_rgba,

            warp_strength_px: app.models_mut().insert(vec![10.0]),
            warp_scale_px: app.models_mut().insert(vec![72.0]),
            warp_phase: app.models_mut().insert(vec![0.0]),
            warp_chroma_px: app.models_mut().insert(vec![2.0]),

            // Keep defaults stable: perf scripts/baselines assume a visible blur chain.
            blur_radius_px: app.models_mut().insert(vec![16.0]),
            blur_downsample: app.models_mut().insert(vec![2.0]),
            saturation: app.models_mut().insert(vec![1.10]),
            brightness: app.models_mut().insert(vec![1.02]),
            contrast: app.models_mut().insert(vec![1.02]),

            use_backdrop: app.models_mut().insert(true),
            use_dither: app.models_mut().insert(true),
        }
    }

    fn update(app: &mut App, st: &mut Self::State, message: Self::Message) {
        if matches!(message, Msg::Reset) {
            let _ = app.models_mut().update(&st.show_fake, |v| *v = true);
            let _ = app.models_mut().update(&st.show_warp, |v| *v = true);
            let _ = app.models_mut().update(&st.show_warp_v2, |v| *v = false);
            let _ = app.models_mut().update(&st.show_inspector, |v| *v = false);
            let _ = app.models_mut().update(&st.animate, |v| *v = true);
            let _ = app
                .models_mut()
                .update(&st.phase_speed, |v| *v = vec![0.65]);
            let _ = app
                .models_mut()
                .update(&st.warp_strength_px, |v| *v = vec![10.0]);
            let _ = app
                .models_mut()
                .update(&st.warp_scale_px, |v| *v = vec![72.0]);
            let _ = app.models_mut().update(&st.warp_phase, |v| *v = vec![0.0]);
            let _ = app
                .models_mut()
                .update(&st.warp_chroma_px, |v| *v = vec![2.0]);
            let _ = app
                .models_mut()
                .update(&st.blur_radius_px, |v| *v = vec![16.0]);
            let _ = app
                .models_mut()
                .update(&st.blur_downsample, |v| *v = vec![2.0]);
            let _ = app.models_mut().update(&st.saturation, |v| *v = vec![1.10]);
            let _ = app.models_mut().update(&st.brightness, |v| *v = vec![1.02]);
            let _ = app.models_mut().update(&st.contrast, |v| *v = vec![1.02]);
            let _ = app.models_mut().update(&st.use_backdrop, |v| *v = true);
            let _ = app.models_mut().update(&st.use_dither, |v| *v = true);
        } else if matches!(message, Msg::ToggleInspector) {
            let _ = app.models_mut().update(&st.show_inspector, |v| *v = !*v);
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

fn view(
    cx: &mut ElementContext<'_, App>,
    st: &mut LiquidGlassState,
    msg: &mut MessageRouter<Msg>,
) -> Elements {
    let theme = Theme::global(&*cx.app).snapshot();
    let theme_stage = theme.clone();
    let viewport = cx.environment_viewport_bounds(Invalidation::Layout);

    let show_fake_model = st.show_fake.clone();
    let show_warp_model = st.show_warp.clone();
    let show_warp_v2_model = st.show_warp_v2.clone();
    let animate_model = st.animate.clone();
    let phase_speed_model = st.phase_speed.clone();
    let show_inspector_model = st.show_inspector.clone();

    let warp_strength_model = st.warp_strength_px.clone();
    let warp_scale_model = st.warp_scale_px.clone();
    let warp_phase_model = st.warp_phase.clone();
    let warp_chroma_model = st.warp_chroma_px.clone();

    let blur_radius_model = st.blur_radius_px.clone();
    let blur_downsample_model = st.blur_downsample.clone();
    let saturation_model = st.saturation.clone();
    let brightness_model = st.brightness.clone();
    let contrast_model = st.contrast.clone();

    let use_backdrop_model = st.use_backdrop.clone();
    let use_dither_model = st.use_dither.clone();

    let show_fake = cx.watch_model(&st.show_fake).layout().copied_or(true);
    let show_warp = cx.watch_model(&st.show_warp).layout().copied_or(true);
    let show_warp_v2 = cx.watch_model(&st.show_warp_v2).layout().copied_or(false);
    let show_inspector = cx.watch_model(&st.show_inspector).layout().copied_or(true);
    let animate = cx.watch_model(&st.animate).layout().copied_or(true);
    let phase_speed = watch_first_f32(cx, &st.phase_speed, 0.65);

    let blur_radius_px = watch_first_f32(cx, &st.blur_radius_px, 16.0);
    let blur_downsample_raw = watch_first_f32(cx, &st.blur_downsample, 2.0);
    let blur_downsample = blur_downsample_raw.round().clamp(1.0, 4.0) as u32;
    let saturation = watch_first_f32(cx, &st.saturation, 1.1);
    let brightness = watch_first_f32(cx, &st.brightness, 1.02);
    let contrast = watch_first_f32(cx, &st.contrast, 1.02);

    let warp_strength_px = watch_first_f32(cx, &st.warp_strength_px, 10.0);
    let warp_scale_px = watch_first_f32(cx, &st.warp_scale_px, 72.0);
    let warp_phase = watch_first_f32(cx, &st.warp_phase, 0.0);
    let warp_chroma_px = watch_first_f32(cx, &st.warp_chroma_px, 2.0);

    let use_backdrop = cx.watch_model(&st.use_backdrop).layout().copied_or(true);
    let use_dither = cx.watch_model(&st.use_dither).layout().copied_or(true);
    let mode = if use_backdrop {
        EffectMode::Backdrop
    } else {
        EffectMode::FilterContent
    };

    let frame = cx.app.frame_id().0 as f32;
    let t = frame / 60.0;
    let phase = if animate { t * phase_speed } else { warp_phase };
    if animate {
        cx.request_animation_frame();
    }

    let warp_image = cx.app.with_image_asset_cache(|cache, app| {
        cache.use_rgba8_keyed(
            app,
            cx.window,
            st.warp_map_key,
            st.warp_map_size.0,
            st.warp_map_size.1,
            st.warp_map_rgba.as_ref().as_slice(),
            ImageColorSpace::Linear,
        )
    });
    let warp_map_loaded = warp_image.is_some();

    let warp_base = BackdropWarpV1 {
        strength_px: Px(warp_strength_px),
        scale_px: Px(warp_scale_px.max(1.0)),
        phase,
        chromatic_aberration_px: Px(warp_chroma_px),
        kind: BackdropWarpKindV1::LensReserved,
    };
    let warp_base = warp_base.sanitize();

    let fake_chain = build_chain(
        None,
        blur_radius_px,
        blur_downsample,
        saturation,
        brightness,
        contrast,
        use_dither,
    );
    let warp_chain = build_chain(
        Some(EffectStep::BackdropWarpV1(warp_base)),
        blur_radius_px,
        blur_downsample,
        saturation,
        brightness,
        contrast,
        use_dither,
    );

    let warp_v2_field = match warp_image {
        Some(image) => BackdropWarpFieldV2::ImageDisplacementMap {
            image,
            uv: UvRect::FULL,
            sampling: ImageSamplingHint::Linear,
            encoding: WarpMapEncodingV1::RgSigned,
        },
        None => BackdropWarpFieldV2::Procedural,
    };
    let warp_v2_chain = build_chain(
        Some(EffectStep::BackdropWarpV2(BackdropWarpV2 {
            base: warp_base,
            field: warp_v2_field,
        })),
        blur_radius_px,
        blur_downsample,
        saturation,
        brightness,
        contrast,
        use_dither,
    );

    let mut root_layout = LayoutStyle::default();
    root_layout.size = SizeStyle {
        width: Length::Fill,
        height: Length::Fill,
        ..Default::default()
    };
    root_layout.position = PositionStyle::Relative;

    let bg = srgb(10, 12, 18, 1.0);

    let reset_stage = msg.cmd(Msg::Reset);
    let reset_inspector = msg.cmd(Msg::Reset);
    let toggle_inspector = msg.cmd(Msg::ToggleInspector);

    let root = cx
        .container(
            ContainerProps {
                layout: root_layout,
                background: Some(bg),
                ..Default::default()
            },
            move |cx| {
                let stage = cx.keyed("liquid_glass.stage", |cx| {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout.position = PositionStyle::Relative;
                    layout.overflow = Overflow::Clip;

                    cx.container(
                        ContainerProps {
                            layout,
                            ..Default::default()
                        },
                        move |cx| {
                            // Stage stripes.
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

                            let stripe_w = Px(18.0);
                            let stripe_count =
                                ((viewport.size.width.0 / stripe_w.0).ceil() as usize).max(1) + 1;
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
                                        let tt = if stripe_count > 1 {
                                            (i as f32) / ((stripe_count - 1) as f32)
                                        } else {
                                            0.0
                                        };
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(stripe_w);
                                        layout.size.height = Length::Fill;
                                        out.push(cx.container(
                                            ContainerProps {
                                                layout,
                                                background: Some(rainbow_stripe(tt, 0.10)),
                                                ..Default::default()
                                            },
                                            |_cx| Vec::<AnyElement>::new(),
                                        ));
                                    }
                                    out
                                },
                            );

                            // Moving blobs (helps make displacement obvious).
                            let mut blob_layout = LayoutStyle::default();
                            blob_layout.position = PositionStyle::Absolute;
                            blob_layout.size.width = Length::Px(Px(140.0));
                            blob_layout.size.height = Length::Px(Px(140.0));
                            blob_layout.inset.left = Some(Px(110.0 + (t * 0.9).cos() * 120.0));
                            blob_layout.inset.top = Some(Px(110.0 + (t * 0.7).sin() * 90.0));
                            let blob = cx.container(
                                ContainerProps {
                                    layout: blob_layout,
                                    background: Some(srgb(120, 220, 255, 0.22)),
                                    border: Edges::all(Px(1.0)),
                                    border_color: Some(srgb(200, 240, 255, 0.35)),
                                    corner_radii: Corners::all(Px(999.0)),
                                    ..Default::default()
                                },
                                |_cx| Vec::<AnyElement>::new(),
                            );

                            let mut blob2_layout = LayoutStyle::default();
                            blob2_layout.position = PositionStyle::Absolute;
                            blob2_layout.size.width = Length::Px(Px(220.0));
                            blob2_layout.size.height = Length::Px(Px(180.0));
                            blob2_layout.inset.right = Some(Px(140.0 + (t * 0.55).sin() * 90.0));
                            blob2_layout.inset.top = Some(Px(140.0 + (t * 0.65).cos() * 70.0));
                            let blob2 = cx.container(
                                ContainerProps {
                                    layout: blob2_layout,
                                    background: Some(srgb(255, 140, 80, 0.16)),
                                    border: Edges::all(Px(1.0)),
                                    border_color: Some(srgb(255, 200, 160, 0.24)),
                                    corner_radii: Corners::all(Px(999.0)),
                                    ..Default::default()
                                },
                                |_cx| Vec::<AnyElement>::new(),
                            );

                            // A few sharp, high-contrast cards (helps differentiate refraction from blur).
                            let mut cards_layout = LayoutStyle::default();
                            cards_layout.position = PositionStyle::Absolute;
                            cards_layout.inset.left = Some(Px(240.0));
                            cards_layout.inset.top = Some(Px(420.0));
                            cards_layout.size.width = Length::Px(Px(760.0));
                            cards_layout.size.height = Length::Px(Px(120.0));

                            let cards = cx.row(
                                RowProps {
                                    layout: cards_layout,
                                    gap: Px(12.0),
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Stretch,
                                    ..Default::default()
                                },
                                move |cx| {
                                    let mk_card = |cx: &mut ElementContext<'_, App>,
                                                   title: &'static str,
                                                   bg: Color,
                                                   border: Color| {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Fill;
                                        layout.size.height = Length::Fill;

                                        let title = cx.text_props(TextProps {
                                            layout: Default::default(),
                                            text: Arc::from(title),
                                            style: None,
                                            color: Some(srgb(255, 255, 255, 0.92)),
                                            align: fret_core::TextAlign::Start,
                                            wrap: fret_core::TextWrap::None,
                                            overflow: fret_core::TextOverflow::Clip,
                                        });

                                        cx.container(
                                            ContainerProps {
                                                layout,
                                                padding: Edges::all(Px(14.0)),
                                                background: Some(bg),
                                                border: Edges::all(Px(1.0)),
                                                border_color: Some(border),
                                                corner_radii: Corners::all(Px(16.0)),
                                                ..Default::default()
                                            },
                                            move |_cx| vec![title],
                                        )
                                    };

                                    vec![
                                        mk_card(
                                            cx,
                                            "RGB bars + sharp edges",
                                            srgb(220, 80, 92, 0.22),
                                            srgb(255, 200, 205, 0.24),
                                        ),
                                        mk_card(
                                            cx,
                                            "Text / glyphs behind lens",
                                            srgb(80, 210, 170, 0.18),
                                            srgb(180, 255, 235, 0.22),
                                        ),
                                        mk_card(
                                            cx,
                                            "Motion makes refraction obvious",
                                            srgb(90, 130, 255, 0.18),
                                            srgb(190, 210, 255, 0.22),
                                        ),
                                    ]
                                },
                            );

                            // Stage HUD (always present so perf scripts can target stable `test_id`s
                            // without depending on the inspector panel state).
                            let mut hud_layout = LayoutStyle::default();
                            hud_layout.position = PositionStyle::Absolute;
                            hud_layout.inset.top = Some(Px(16.0));
                            hud_layout.inset.left = Some(Px(16.0));
                            hud_layout.overflow = Overflow::Clip;

                            let mut hud_bg = theme_stage.color_token("card");
                            hud_bg.a = (hud_bg.a * 0.92).clamp(0.0, 1.0);
                            let hud = cx.container(
                                ContainerProps {
                                    layout: hud_layout,
                                    padding: Edges::all(Px(12.0)),
                                    background: Some(hud_bg),
                                    border: Edges::all(Px(1.0)),
                                    border_color: Some(theme_stage.color_token("border")),
                                    corner_radii: Corners::all(Px(12.0)),
                                    ..Default::default()
                                },
                                move |cx| {
                                    vec![shadcn::stack::vstack(
                                        cx,
                                        shadcn::stack::VStackProps::default()
                                            .gap(Space::N2)
                                            .items_stretch(),
                                        |cx| {
                                            vec![
                                                shadcn::typography::h4(cx, "Liquid glass"),
                                                shadcn::typography::muted(
                                                    cx,
                                                    "BackdropWarpV2 (bounded), WebGPU-safe.",
                                                ),
                                                shadcn::Separator::new().into_element(cx),
                                                shadcn::stack::hstack(
                                                    cx,
                                                    shadcn::stack::HStackProps::default()
                                                        .gap(Space::N2)
                                                        .items_center(),
                                                    |cx| {
                                                        vec![
                                                            shadcn::Switch::new(
                                                                show_fake_model.clone(),
                                                            )
                                                            .a11y_label("Show fake lens")
                                                            .test_id("liquid-glass-switch-show-fake")
                                                            .into_element(cx),
                                                            shadcn::Label::new("Fake")
                                                                .into_element(cx),
                                                            shadcn::Switch::new(
                                                                show_warp_model.clone(),
                                                            )
                                                            .a11y_label("Show warp v1 lens")
                                                            .test_id(
                                                                "liquid-glass-switch-show-warp-v1",
                                                            )
                                                            .into_element(cx),
                                                            shadcn::Label::new("Warp v1")
                                                                .into_element(cx),
                                                            shadcn::Switch::new(
                                                                show_warp_v2_model.clone(),
                                                            )
                                                            .a11y_label("Show warp v2 lens")
                                                            .test_id(
                                                                "liquid-glass-switch-show-warp-v2",
                                                            )
                                                            .into_element(cx),
                                                            shadcn::Label::new("Warp v2")
                                                                .into_element(cx),
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
                                                            shadcn::Switch::new(
                                                                use_backdrop_model.clone(),
                                                            )
                                                            .a11y_label("Backdrop mode")
                                                            .into_element(cx),
                                                            shadcn::Label::new("Backdrop")
                                                                .into_element(cx),
                                                            shadcn::Switch::new(
                                                                use_dither_model.clone(),
                                                            )
                                                            .a11y_label("Dither")
                                                            .into_element(cx),
                                                            shadcn::Label::new("Dither")
                                                                .into_element(cx),
                                                            shadcn::Switch::new(
                                                                animate_model.clone(),
                                                            )
                                                            .a11y_label("Animate phase")
                                                            .into_element(cx),
                                                            shadcn::Label::new("Animate")
                                                                .into_element(cx),
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
                                                            shadcn::Switch::new(
                                                                show_inspector_model.clone(),
                                                            )
                                                            .a11y_label("Show inspector")
                                                            .test_id(
                                                                "liquid-glass-switch-show-inspector",
                                                            )
                                                            .into_element(cx),
                                                            shadcn::Label::new("Inspector")
                                                                .into_element(cx),
                                                            cx.spacer(SpacerProps::default()),
                                                            shadcn::Button::new(if show_inspector {
                                                                "Hide"
                                                            } else {
                                                                "Show"
                                                            })
                                                            .variant(shadcn::ButtonVariant::Secondary)
                                                            .size(shadcn::ButtonSize::Sm)
                                                            .on_click(toggle_inspector)
                                                            .test_id("liquid-glass-toggle-inspector")
                                                            .into_element(cx),
                                                            shadcn::Button::new("Reset")
                                                                .variant(
                                                                    shadcn::ButtonVariant::Secondary,
                                                                )
                                                                .size(shadcn::ButtonSize::Sm)
                                                                .on_click(reset_stage)
                                                                .into_element(cx),
                                                        ]
                                                    },
                                                ),
                                            ]
                                        },
                                    )]
                                },
                            );

                            // Lenses (bottom-left).
                            let mut lenses_layout = LayoutStyle::default();
                            lenses_layout.position = PositionStyle::Absolute;
                            lenses_layout.inset.left = Some(Px(24.0));
                            lenses_layout.inset.bottom = Some(Px(24.0));
                            let lenses = cx.row(
                                RowProps {
                                    layout: lenses_layout,
                                    gap: Px(14.0),
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Start,
                                    ..Default::default()
                                },
                                move |cx| {
                                    let mut out: Vec<AnyElement> = Vec::new();
                                    if show_fake {
                                        out.push(
                                            lens_panel(
                                                cx,
                                                Arc::from("Fake glass (blur + adjust)"),
                                                mode,
                                                fake_chain,
                                            )
                                            .test_id("liquid-glass-lens-fake"),
                                        );
                                    }
                                    if show_warp {
                                        out.push(
                                            lens_panel(
                                                cx,
                                                Arc::from("Warp v1 (procedural)"),
                                                mode,
                                                warp_chain,
                                            )
                                            .test_id("liquid-glass-lens-warp-v1"),
                                        );
                                    }
                                    if show_warp_v2 {
                                        out.push(
                                            lens_panel(
                                                cx,
                                                Arc::from("Warp v2 (image field)"),
                                                mode,
                                                warp_v2_chain,
                                            )
                                            .test_id("liquid-glass-lens-warp-v2"),
                                        );
                                    }
                                    out
                                },
                            );

                            vec![stripes, blob, blob2, cards, hud, lenses]
                        },
                    )
                });

                let inspector = show_inspector.then(|| {
                    cx.keyed("liquid_glass.inspector", |cx| {
                        let mut layout = LayoutStyle::default();
                        layout.position = PositionStyle::Absolute;
                        layout.inset.top = Some(Px(0.0));
                        layout.inset.right = Some(Px(0.0));
                        layout.inset.bottom = Some(Px(0.0));
                        layout.size.width = Length::Px(Px(380.0));
                        layout.size.height = Length::Fill;
                        layout.overflow = Overflow::Clip;

                        cx.container(
                            ContainerProps {
                                layout,
                                padding: Edges::all(Px(16.0)),
                                background: Some(theme.color_token("card")),
                                border: Edges::all(Px(1.0)),
                                border_color: Some(theme.color_token("border")),
                                ..Default::default()
                            },
                            move |cx| {
                                let header = shadcn::CardHeader::new([
                                    shadcn::CardTitle::new("Inspector").into_element(cx),
                                    shadcn::CardDescription::new(format!(
                                        "mode={:?} steps(fake={}, v1={}, v2={}) warp_map_loaded={}",
                                        mode,
                                        fake_chain.iter().count(),
                                        warp_chain.iter().count(),
                                        warp_v2_chain.iter().count(),
                                        warp_map_loaded
                                    ))
                                    .into_element(cx),
                                ]);

                                let label_row =
                                    |cx: &mut ElementContext<'_, App>,
                                     label: &str,
                                     value: String| {
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

                                let warp_strength_row = shadcn::stack::vstack(
                                    cx,
                                    shadcn::stack::VStackProps::default().gap(Space::N2),
                                    |cx| {
                                        vec![
                                            label_row(
                                                cx,
                                                "Warp strength (px)",
                                                format!("{warp_strength_px:.1}"),
                                            ),
                                            shadcn::Slider::new(warp_strength_model.clone())
                                                .range(0.0, BackdropWarpV1::MAX_STRENGTH_PX.0)
                                                .step(0.25)
                                                .into_element(cx),
                                        ]
                                    },
                                );

                                let warp_scale_row = shadcn::stack::vstack(
                                    cx,
                                    shadcn::stack::VStackProps::default().gap(Space::N2),
                                    |cx| {
                                        vec![
                                            label_row(
                                                cx,
                                                "Warp scale (px)",
                                                format!("{warp_scale_px:.0}"),
                                            ),
                                            shadcn::Slider::new(warp_scale_model.clone())
                                                .range(BackdropWarpV1::MIN_SCALE_PX.0, 256.0)
                                                .step(1.0)
                                                .into_element(cx),
                                        ]
                                    },
                                );

                                let chroma_row = shadcn::stack::vstack(
                                    cx,
                                    shadcn::stack::VStackProps::default().gap(Space::N2),
                                    |cx| {
                                        vec![
                                            label_row(
                                                cx,
                                                "Chromatic aberration (px)",
                                                format!("{warp_chroma_px:.2}"),
                                            ),
                                            shadcn::Slider::new(warp_chroma_model.clone())
                                                .range(
                                                    0.0,
                                                    BackdropWarpV1::MAX_CHROMATIC_ABERRATION_PX.0,
                                                )
                                                .step(0.05)
                                                .into_element(cx),
                                        ]
                                    },
                                );

                                let phase_row = shadcn::stack::vstack(
                                    cx,
                                    shadcn::stack::VStackProps::default().gap(Space::N2),
                                    |cx| {
                                        vec![
                                            label_row(cx, "Phase", format!("{phase:.2}")),
                                            shadcn::Slider::new(warp_phase_model.clone())
                                                .range(0.0, 12.0)
                                                .step(0.01)
                                                .into_element(cx),
                                        ]
                                    },
                                );

                                let speed_row = shadcn::stack::vstack(
                                    cx,
                                    shadcn::stack::VStackProps::default().gap(Space::N2),
                                    |cx| {
                                        vec![
                                            label_row(
                                                cx,
                                                "Phase speed",
                                                format!("{phase_speed:.2}"),
                                            ),
                                            shadcn::Slider::new(phase_speed_model.clone())
                                                .range(0.0, 2.0)
                                                .step(0.01)
                                                .into_element(cx),
                                        ]
                                    },
                                );

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
                                            label_row(
                                                cx,
                                                "Blur downsample",
                                                format!("{blur_downsample}x"),
                                            ),
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
                                            label_row(cx, "Saturation", format!("{saturation:.2}")),
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
                                            label_row(cx, "Brightness", format!("{brightness:.2}")),
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
                                            label_row(cx, "Contrast", format!("{contrast:.2}")),
                                            shadcn::Slider::new(contrast_model.clone())
                                                .range(0.8, 1.3)
                                                .step(0.01)
                                                .into_element(cx),
                                        ]
                                    },
                                );

                                let footer = shadcn::stack::hstack(
                                    cx,
                                    shadcn::stack::HStackProps::default()
                                        .gap(Space::N2)
                                        .items_center(),
                                    |cx| {
                                        vec![
                                            cx.spacer(SpacerProps::default()),
                                            shadcn::Button::new("Reset")
                                                .variant(shadcn::ButtonVariant::Secondary)
                                                .size(shadcn::ButtonSize::Sm)
                                                .on_click(reset_inspector)
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
                                            warp_strength_row,
                                            warp_scale_row,
                                            chroma_row,
                                            phase_row,
                                            speed_row,
                                            shadcn::Separator::new().into_element(cx),
                                            blur_row,
                                            downsample_row,
                                            sat_row,
                                            bright_row,
                                            contrast_row,
                                            shadcn::Separator::new().into_element(cx),
                                            footer,
                                        ]
                                    },
                                );

                                vec![body]
                            },
                        )
                    })
                });

                let mut out = Vec::with_capacity(if show_inspector { 2 } else { 1 });
                out.push(stage);
                if let Some(inspector) = inspector {
                    out.push(inspector);
                }
                out
            },
        )
        .test_id("liquid-glass-root");

    vec![root].into()
}
