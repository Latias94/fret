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
// Params packing (EffectParamsV1 is 64 bytes):
// - vec4s[0].x: refraction_height_px
// - vec4s[0].y: refraction_amount_px
// - vec4s[0].z: depth_effect (0..1)
// - vec4s[0].w: chromatic_aberration (0..1)
// - vec4s[1]: corner_radii_px (tl, tr, br, bl)
// - vec4s[2].x: grain_strength (0..1)
// - vec4s[2].y: grain_scale (>= 0.1)

fn radius_at(centered: vec2<f32>, radii: vec4<f32>) -> f32 {
  if (centered.x >= 0.0) {
    if (centered.y <= 0.0) { return radii.y; } // top-right
    return radii.z; // bottom-right
  }
  if (centered.y <= 0.0) { return radii.x; } // top-left
  return radii.w; // bottom-left
}

fn sd_rounded_rect(centered: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
  let corner = abs(centered) - (half_size - vec2<f32>(radius));
  let outside = length(max(corner, vec2<f32>(0.0))) - radius;
  let inside = min(max(corner.x, corner.y), 0.0);
  return outside + inside;
}

fn grad_sd_rounded_rect(centered: vec2<f32>, half_size: vec2<f32>, radius: f32) -> vec2<f32> {
  let corner = abs(centered) - (half_size - vec2<f32>(radius));
  if (corner.x >= 0.0 || corner.y >= 0.0) {
    return sign(centered) * normalize(max(corner, vec2<f32>(0.0)));
  }
  let grad_x = select(0.0, 1.0, corner.y <= corner.x);
  return sign(centered) * vec2<f32>(grad_x, 1.0 - grad_x);
}

fn circle_map(x: f32) -> f32 {
  return 1.0 - sqrt(max(0.0, 1.0 - x * x));
}

fn clamp_pixel_pos(p: vec2<f32>) -> vec2<f32> {
  let dims_u = textureDimensions(src_texture);
  let dims = vec2<f32>(f32(dims_u.x), f32(dims_u.y));
  return clamp(p, vec2<f32>(0.5), dims - vec2<f32>(0.5));
}

fn sample_src_bilinear(pixel_pos: vec2<f32>) -> vec4<f32> {
  let dims_u = textureDimensions(src_texture);
  let px = clamp_pixel_pos(pixel_pos);

  let base = floor(px - vec2<f32>(0.5));
  let f = px - (base + vec2<f32>(0.5));

  let x0 = clamp(i32(base.x), 0, i32(dims_u.x) - 1);
  let y0 = clamp(i32(base.y), 0, i32(dims_u.y) - 1);
  let x1 = clamp(x0 + 1, 0, i32(dims_u.x) - 1);
  let y1 = clamp(y0 + 1, 0, i32(dims_u.y) - 1);

  let c00 = textureLoad(src_texture, vec2<i32>(x0, y0), 0);
  let c10 = textureLoad(src_texture, vec2<i32>(x1, y0), 0);
  let c01 = textureLoad(src_texture, vec2<i32>(x0, y1), 0);
  let c11 = textureLoad(src_texture, vec2<i32>(x1, y1), 0);

  let top = mix(c00, c10, f.x);
  let bot = mix(c01, c11, f.x);
  return mix(top, bot, f.y);
}

fn unpremul(c: vec4<f32>) -> vec3<f32> {
  if (c.a <= 1e-6) { return vec3<f32>(0.0); }
  return c.rgb / c.a;
}

fn fret_custom_effect(_src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, params: EffectParamsV1) -> vec4<f32> {
  let refraction_height_px = max(0.0, params.vec4s[0].x);
  let refraction_amount_px = max(0.0, params.vec4s[0].y);
  let depth_effect = clamp(params.vec4s[0].z, 0.0, 1.0);
  let chromatic = clamp(params.vec4s[0].w, 0.0, 1.0);
  let corner_radii = max(params.vec4s[1], vec4<f32>(0.0));
  let grain_strength = max(0.0, params.vec4s[2].x);
  let grain_scale = max(0.1, params.vec4s[2].y);

  if (refraction_height_px <= 0.0 || refraction_amount_px <= 0.0) {
    return sample_src_bilinear(pos_px);
  }

  let size = render_space.size_px;
  let half_size = size * 0.5;
  let coord = pos_px - render_space.origin_px;
  let centered = coord - half_size;
  let radius = radius_at(centered, corner_radii);

  var sd = sd_rounded_rect(centered, half_size, radius);
  if (-sd >= refraction_height_px) {
    return sample_src_bilinear(pos_px);
  }
  sd = min(sd, 0.0);

  let d = circle_map(1.0 - (-sd / refraction_height_px)) * refraction_amount_px;
  let grad_radius = min(radius * 1.5, min(half_size.x, half_size.y));
  let g0 = grad_sd_rounded_rect(centered, half_size, grad_radius);
  let g1 = select(vec2<f32>(0.0), normalize(centered), length(centered) > 1e-3);
  let grad = normalize(g0 + depth_effect * g1);

  let refracted = pos_px + d * grad;

  // Base refracted sample (premul linear).
  let base = sample_src_bilinear(refracted);
  var rgb_u = unpremul(base);
  var a = base.a;

  // Chromatic aberration: sample red/blue with bounded offsets, mix in unpremultiplied space,
  // then premultiply once. This avoids edge darkening due to premul mismatch.
  if (chromatic > 0.0) {
    let disp = chromatic * ((centered.x * centered.y) / max(1.0, half_size.x * half_size.y));
    let offset = d * grad * disp;

    let red = sample_src_bilinear(refracted + offset);
    let blue = sample_src_bilinear(refracted - offset);

    let red_u = unpremul(red);
    let blue_u = unpremul(blue);

    let aberr = vec3<f32>(red_u.r, rgb_u.g, blue_u.b);
    rgb_u = mix(rgb_u, aberr, chromatic);
    a = max(a, max(red.a, blue.a));
  }

  // Rim highlight + inner shadow to give the lens a "thicker" edge. This is purely a visual
  // recipe in the demo shader (not part of the core renderer semantics).
  let dist_in = max(0.0, -sd);
  let rim_px = 1.25;
  let shadow_px0 = 1.5;
  let shadow_px1 = 10.0;

  let rim = smoothstep(rim_px, 0.0, dist_in);
  let shadow_band = smoothstep(shadow_px0, shadow_px1, dist_in)
    * (1.0 - smoothstep(shadow_px1, shadow_px1 + 1.5, dist_in));

  let corner_boost = 1.0 + 1.2 * abs(g0.x * g0.y);
  let rim_strength = 0.08 + 0.10 * depth_effect;
  let shadow_strength = 0.06 + 0.06 * depth_effect;

  rgb_u += vec3<f32>(1.0) * rim * rim_strength * corner_boost;
  rgb_u -= vec3<f32>(1.0) * shadow_band * shadow_strength;

  // Subtle deterministic grain, anchored to the effect bounds (local space).
  if (grain_strength > 0.0) {
    let n = fret_catalog_hash_noise01(fret_local_px(pos_px) * grain_scale) - 0.5;
    rgb_u += vec3<f32>(n) * grain_strength;
  }
  rgb_u = clamp(rgb_u, vec3<f32>(0.0), vec3<f32>(4.0));

  return vec4<f32>(rgb_u * a, a);
}
"#;

#[derive(Debug, Clone, Copy)]
struct DemoEffect(EffectId);

#[derive(Debug)]
struct CustomEffectV1State {
    enabled: Model<bool>,
    blur_radius_px: Model<Vec<f32>>,
    blur_downsample: Model<Vec<f32>>,
    refraction_height_px: Model<Vec<f32>>,
    refraction_amount_px: Model<Vec<f32>>,
    depth_effect: Model<Vec<f32>>,
    chromatic_aberration: Model<Vec<f32>>,
    corner_radius_px: Model<Vec<f32>>,
    grain_strength: Model<Vec<f32>>,
    grain_scale: Model<Vec<f32>>,
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
            blur_radius_px: app.models_mut().insert(vec![14.0]),
            blur_downsample: app.models_mut().insert(vec![2.0]),
            refraction_height_px: app.models_mut().insert(vec![20.0]),
            refraction_amount_px: app.models_mut().insert(vec![12.0]),
            depth_effect: app.models_mut().insert(vec![0.35]),
            chromatic_aberration: app.models_mut().insert(vec![0.75]),
            corner_radius_px: app.models_mut().insert(vec![20.0]),
            grain_strength: app.models_mut().insert(vec![0.06]),
            grain_scale: app.models_mut().insert(vec![1.0]),
        }
    }

    fn update(app: &mut App, st: &mut Self::State, message: Self::Message) {
        if matches!(message, Msg::Reset) {
            let _ = app.models_mut().update(&st.enabled, |v| *v = true);
            let _ = app
                .models_mut()
                .update(&st.blur_radius_px, |v| *v = vec![14.0]);
            let _ = app
                .models_mut()
                .update(&st.blur_downsample, |v| *v = vec![2.0]);
            let _ = app
                .models_mut()
                .update(&st.refraction_height_px, |v| *v = vec![20.0]);
            let _ = app
                .models_mut()
                .update(&st.refraction_amount_px, |v| *v = vec![12.0]);
            let _ = app
                .models_mut()
                .update(&st.depth_effect, |v| *v = vec![0.35]);
            let _ = app
                .models_mut()
                .update(&st.chromatic_aberration, |v| *v = vec![0.75]);
            let _ = app
                .models_mut()
                .update(&st.corner_radius_px, |v| *v = vec![20.0]);
            let _ = app
                .models_mut()
                .update(&st.grain_strength, |v| *v = vec![0.06]);
            let _ = app.models_mut().update(&st.grain_scale, |v| *v = vec![1.0]);
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
    let blur_radius_px = watch_first_f32(cx, &st.blur_radius_px, 14.0);
    let blur_downsample = watch_first_f32(cx, &st.blur_downsample, 2.0);
    let refraction_height_px = watch_first_f32(cx, &st.refraction_height_px, 20.0);
    let refraction_amount_px = watch_first_f32(cx, &st.refraction_amount_px, 12.0);
    let depth_effect = watch_first_f32(cx, &st.depth_effect, 0.35);
    let chromatic_aberration = watch_first_f32(cx, &st.chromatic_aberration, 0.75);
    let corner_radius_px = watch_first_f32(cx, &st.corner_radius_px, 20.0);
    let grain_strength = watch_first_f32(cx, &st.grain_strength, 0.06);
    let grain_scale = watch_first_f32(cx, &st.grain_scale, 1.0);

    let inspector = inspector(
        cx,
        st,
        blur_radius_px,
        blur_downsample,
        refraction_height_px,
        refraction_amount_px,
        depth_effect,
        chromatic_aberration,
        corner_radius_px,
        grain_strength,
        grain_scale,
        msg,
    );
    let stage = stage(
        cx,
        enabled,
        effect,
        blur_radius_px,
        blur_downsample,
        refraction_height_px,
        refraction_amount_px,
        depth_effect,
        chromatic_aberration,
        corner_radius_px,
        grain_strength,
        grain_scale,
    );

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
    blur_radius_px: f32,
    blur_downsample: f32,
    refraction_height_px: f32,
    refraction_amount_px: f32,
    depth_effect: f32,
    chromatic_aberration: f32,
    corner_radius_px: f32,
    grain_strength: f32,
    grain_scale: f32,
) -> AnyElement {
    let lenses = lens_row(
        cx,
        enabled,
        effect,
        blur_radius_px,
        blur_downsample,
        refraction_height_px,
        refraction_amount_px,
        depth_effect,
        chromatic_aberration,
        corner_radius_px,
        grain_strength,
        grain_scale,
    );

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

    let mut stripes_layer_layout = LayoutStyle::default();
    stripes_layer_layout.position = PositionStyle::Absolute;
    stripes_layer_layout.inset.left = Some(Px(0.0)).into();
    stripes_layer_layout.inset.right = Some(Px(0.0)).into();
    stripes_layer_layout.inset.top = Some(Px(0.0)).into();
    stripes_layer_layout.inset.bottom = Some(Px(0.0)).into();

    let mut stage_layout = LayoutStyle::default();
    stage_layout.size.width = Length::Fill;
    stage_layout.size.height = Length::Fill;
    stage_layout.flex.grow = 1.0;

    cx.container(
        ContainerProps {
            layout: stage_layout,
            ..Default::default()
        },
        move |cx| {
            let stripes = cx.container(
                ContainerProps {
                    layout: stripes_layer_layout,
                    ..Default::default()
                },
                move |_cx| vec![stripes],
            );

            let mut header_layout = LayoutStyle::default();
            header_layout.size.width = Length::Fill;

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
                            .gap(Space::N4)
                            .items_start(),
                        move |_cx| vec![header, lenses],
                    )]
                },
            );

            vec![stripes, content]
        },
    )
}

fn lens_row(
    cx: &mut ElementContext<'_, App>,
    enabled: bool,
    effect: EffectId,
    blur_radius_px: f32,
    blur_downsample: f32,
    refraction_height_px: f32,
    refraction_amount_px: f32,
    depth_effect: f32,
    chromatic_aberration: f32,
    corner_radius_px: f32,
    grain_strength: f32,
    grain_scale: f32,
) -> AnyElement {
    let radius = Px(corner_radius_px.clamp(0.0, 64.0));
    shadcn::stack::hstack(
        cx,
        shadcn::stack::HStackProps::default()
            .gap(Space::N3)
            .items_start(),
        move |cx| {
            vec![
                plain_lens(cx, "Plain (no effect)", radius),
                if enabled {
                    custom_effect_lens(
                        cx,
                        "CustomV1 lens",
                        effect,
                        blur_radius_px,
                        blur_downsample,
                        refraction_height_px,
                        refraction_amount_px,
                        depth_effect,
                        chromatic_aberration,
                        corner_radius_px,
                        grain_strength,
                        grain_scale,
                    )
                } else {
                    plain_lens(cx, "CustomV1 lens (disabled)", radius)
                },
            ]
        },
    )
}

fn lens_shell(
    cx: &mut ElementContext<'_, App>,
    label: Arc<str>,
    radius: Px,
    body: AnyElement,
) -> AnyElement {
    let mut outer_layout = LayoutStyle::default();
    outer_layout.size.width = Length::Px(Px(380.0));
    outer_layout.size.height = Length::Px(Px(240.0));
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
                    background: Some(srgb(10, 12, 18, 0.32)),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(srgb(255, 255, 255, 0.18)),
                    corner_radii: Corners::all(Px(999.0)),
                    ..Default::default()
                },
                move |cx| vec![title.into_element(cx)],
            );

            vec![body, pill]
        },
    )
}

fn plain_lens(
    cx: &mut ElementContext<'_, App>,
    label: impl Into<Arc<str>>,
    radius: Px,
) -> AnyElement {
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

    lens_shell(cx, label.into(), radius, body)
}

fn custom_effect_lens(
    cx: &mut ElementContext<'_, App>,
    label: impl Into<Arc<str>>,
    effect: EffectId,
    blur_radius_px: f32,
    blur_downsample: f32,
    refraction_height_px: f32,
    refraction_amount_px: f32,
    depth_effect: f32,
    chromatic_aberration: f32,
    corner_radius_px: f32,
    grain_strength: f32,
    grain_scale: f32,
) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;

    let blur_radius_px = blur_radius_px.clamp(0.0, 64.0);
    let blur_downsample = blur_downsample.round().clamp(1.0, 4.0) as u32;

    let refraction_height_px = refraction_height_px.clamp(0.0, 64.0);
    let refraction_amount_px = refraction_amount_px.clamp(0.0, 32.0);
    let depth_effect = depth_effect.clamp(0.0, 1.0);
    let chromatic_aberration = chromatic_aberration.clamp(0.0, 1.0);
    let radius = corner_radius_px.clamp(0.0, 64.0);
    let grain_strength = grain_strength.clamp(0.0, 0.25);
    let grain_scale = grain_scale.clamp(0.1, 8.0);

    let params = EffectParamsV1 {
        vec4s: [
            [
                refraction_height_px,
                refraction_amount_px,
                depth_effect,
                chromatic_aberration,
            ],
            [radius, radius, radius, radius],
            [grain_strength, grain_scale, 0.0, 0.0],
            [0.0; 4],
        ],
    };

    let mut steps = Vec::new();
    if blur_radius_px > 0.0 {
        steps.push(EffectStep::GaussianBlur {
            radius_px: Px(blur_radius_px),
            downsample: blur_downsample,
        });
    }
    let max_sample_offset_px = Px(refraction_amount_px + 8.0);
    steps.push(EffectStep::CustomV1 {
        id: effect,
        params,
        max_sample_offset_px,
    });
    let chain = EffectChain::from_steps(&steps).sanitize();

    let layer = cx.effect_layer_props(
        EffectLayerProps {
            layout,
            mode: EffectMode::Backdrop,
            chain,
            quality: EffectQuality::Auto,
        },
        |_cx| Vec::<AnyElement>::new(),
    );

    lens_shell(cx, label.into(), Px(radius), layer)
}

fn inspector(
    cx: &mut ElementContext<'_, App>,
    st: &mut CustomEffectV1State,
    blur_radius_px: f32,
    blur_downsample: f32,
    refraction_height_px: f32,
    refraction_amount_px: f32,
    depth_effect: f32,
    chromatic_aberration: f32,
    corner_radius_px: f32,
    grain_strength: f32,
    grain_scale: f32,
    msg: &mut MessageRouter<Msg>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();

    let reset_cmd = msg.cmd(Msg::Reset);
    let enabled_model = st.enabled.clone();
    let blur_radius_model = st.blur_radius_px.clone();
    let blur_downsample_model = st.blur_downsample.clone();
    let refraction_height_model = st.refraction_height_px.clone();
    let refraction_amount_model = st.refraction_amount_px.clone();
    let depth_effect_model = st.depth_effect.clone();
    let chromatic_model = st.chromatic_aberration.clone();
    let corner_radius_model = st.corner_radius_px.clone();
    let grain_strength_model = st.grain_strength.clone();
    let grain_scale_model = st.grain_scale.clone();

    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Px(Px(360.0));
    layout.size.height = Length::Fill;
    layout.flex.shrink = 0.0;

    cx.container(
        ContainerProps {
            layout,
            padding: Edges::all(Px(16.0)).into(),
            background: Some(theme.color_token("background")),
            border: Edges {
                left: Px(0.0),
                right: Px(1.0),
                top: Px(0.0),
                bottom: Px(0.0),
            },
            border_color: Some(theme.color_token("border")),
            ..Default::default()
        },
        move |cx| {
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

            let header = shadcn::CardHeader::new([
                shadcn::CardTitle::new("Custom Effect V1").into_element(cx),
                shadcn::CardDescription::new(
                    "Registers WGSL at on_gpu_ready and applies EffectStep::CustomV1.",
                )
                .into_element(cx),
            ])
            .into_element(cx);

            let content = shadcn::CardContent::new([shadcn::stack::vstack(
                cx,
                shadcn::stack::VStackProps::default()
                    .gap(Space::N3)
                    .items_stretch(),
                move |cx| {
                    let blur_radius_row = shadcn::stack::vstack(
                        cx,
                        shadcn::stack::VStackProps::default().gap(Space::N2),
                        move |cx| {
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

                    let blur_downsample_row = shadcn::stack::vstack(
                        cx,
                        shadcn::stack::VStackProps::default().gap(Space::N2),
                        move |cx| {
                            let v = blur_downsample.round().clamp(1.0, 4.0) as u32;
                            vec![
                                label_row(cx, "Blur downsample", format!("{v}x")),
                                shadcn::Slider::new(blur_downsample_model.clone())
                                    .range(1.0, 4.0)
                                    .step(1.0)
                                    .into_element(cx),
                            ]
                        },
                    );

                    let refraction_height_row = shadcn::stack::vstack(
                        cx,
                        shadcn::stack::VStackProps::default().gap(Space::N2),
                        move |cx| {
                            vec![
                                label_row(
                                    cx,
                                    "Refraction height (px)",
                                    format!("{:.1}", refraction_height_px.clamp(0.0, 64.0)),
                                ),
                                shadcn::Slider::new(refraction_height_model.clone())
                                    .range(0.0, 64.0)
                                    .step(0.5)
                                    .into_element(cx),
                            ]
                        },
                    );

                    let refraction_amount_row = shadcn::stack::vstack(
                        cx,
                        shadcn::stack::VStackProps::default().gap(Space::N2),
                        move |cx| {
                            vec![
                                label_row(
                                    cx,
                                    "Refraction amount (px)",
                                    format!("{:.1}", refraction_amount_px.clamp(0.0, 32.0)),
                                ),
                                shadcn::Slider::new(refraction_amount_model.clone())
                                    .range(0.0, 24.0)
                                    .step(0.25)
                                    .into_element(cx),
                            ]
                        },
                    );

                    let depth_effect_row = shadcn::stack::vstack(
                        cx,
                        shadcn::stack::VStackProps::default().gap(Space::N2),
                        move |cx| {
                            vec![
                                label_row(cx, "Depth effect", format!("{depth_effect:.2}")),
                                shadcn::Slider::new(depth_effect_model.clone())
                                    .range(0.0, 1.0)
                                    .step(0.01)
                                    .into_element(cx),
                            ]
                        },
                    );

                    let chromatic_row = shadcn::stack::vstack(
                        cx,
                        shadcn::stack::VStackProps::default().gap(Space::N2),
                        move |cx| {
                            vec![
                                label_row(
                                    cx,
                                    "Chromatic aberration",
                                    format!("{chromatic_aberration:.2}"),
                                ),
                                shadcn::Slider::new(chromatic_model.clone())
                                    .range(0.0, 1.0)
                                    .step(0.01)
                                    .into_element(cx),
                            ]
                        },
                    );

                    let corner_radius_row = shadcn::stack::vstack(
                        cx,
                        shadcn::stack::VStackProps::default().gap(Space::N2),
                        move |cx| {
                            vec![
                                label_row(
                                    cx,
                                    "Corner radius (px)",
                                    format!("{:.1}", corner_radius_px.clamp(0.0, 64.0)),
                                ),
                                shadcn::Slider::new(corner_radius_model.clone())
                                    .range(0.0, 48.0)
                                    .step(0.5)
                                    .into_element(cx),
                            ]
                        },
                    );

                    let grain_strength_row = shadcn::stack::vstack(
                        cx,
                        shadcn::stack::VStackProps::default().gap(Space::N2),
                        move |cx| {
                            vec![
                                label_row(cx, "Grain strength", format!("{grain_strength:.2}")),
                                shadcn::Slider::new(grain_strength_model.clone())
                                    .range(0.0, 0.2)
                                    .step(0.01)
                                    .into_element(cx),
                            ]
                        },
                    );

                    let grain_scale_row = shadcn::stack::vstack(
                        cx,
                        shadcn::stack::VStackProps::default().gap(Space::N2),
                        move |cx| {
                            vec![
                                label_row(cx, "Grain scale", format!("{grain_scale:.2}")),
                                shadcn::Slider::new(grain_scale_model.clone())
                                    .range(0.25, 6.0)
                                    .step(0.05)
                                    .into_element(cx),
                            ]
                        },
                    );

                    vec![
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
                        blur_radius_row,
                        blur_downsample_row,
                        shadcn::Separator::new().into_element(cx),
                        refraction_height_row,
                        refraction_amount_row,
                        depth_effect_row,
                        chromatic_row,
                        corner_radius_row,
                        shadcn::Separator::new().into_element(cx),
                        grain_strength_row,
                        grain_scale_row,
                        shadcn::Button::new("Reset")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .on_click(reset_cmd.clone())
                            .test_id("custom-effect-v1.reset")
                            .into_element(cx),
                    ]
                },
            )])
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
