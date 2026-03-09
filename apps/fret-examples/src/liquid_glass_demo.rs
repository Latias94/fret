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
    BackdropWarpFieldV2, BackdropWarpKindV1, BackdropWarpV1, BackdropWarpV2,
    CustomEffectImageInputV1, CustomEffectPyramidRequestV1, CustomEffectSourcesV3, DitherMode,
    EffectChain, EffectMode, EffectParamsV1, EffectQuality, EffectStep, ImageSamplingHint, UvRect,
    WarpMapEncodingV1,
};
use fret_core::{Color, Corners, Edges, EffectId, ImageColorSpace, Px};
use fret_render::RendererCapabilities;
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::element::{
    ContainerProps, CrossAlign, EffectLayerProps, InsetStyle, LayoutStyle, Length, MainAlign,
    Overflow, PositionStyle, RowProps, SizeStyle, SpacerProps, SpacingLength, TextProps,
};
use fret_ui_assets::image_asset_cache::{ImageAssetCacheHostExt, ImageAssetKey};
use fret_ui_kit::Space;
use fret_ui_kit::custom_effects::{CustomEffectProgramV2, CustomEffectProgramV3};
use fret_ui_kit::ui;
use fret_ui_shadcn as shadcn;

use crate::custom_effect_v3_wgsl::CUSTOM_EFFECT_V3_LENS_WGSL;

mod act {
    fret::actions!([
        Reset = "liquid_glass_demo.reset.v1",
        ApplyCustomV3BevelPreset = "liquid_glass_demo.custom_v3_bevel_preset.v1",
        DisableCustomV3Bevel = "liquid_glass_demo.custom_v3_bevel_off.v1",
        ToggleInspector = "liquid_glass_demo.toggle_inspector.v1",
    ]);
}

const CUSTOM_WARP_V2_WGSL: &str = r#"
// Params packing (EffectParamsV1 is 64 bytes):
// - vec4s[0].x: strength_px
// - vec4s[0].y: scale_px
// - vec4s[0].z: phase
// - vec4s[0].w: chroma_px
// - vec4s[1].x: edge_falloff_px (>= 0)
// - vec4s[1].y: rim_strength (0..1)
// - vec4s[1].z: shadow_strength (0..1)
// - vec4s[2].x: grain_strength (0..1)
// - vec4s[2].y: grain_scale (>= 0.1)
// - vec4s[3]: corner_radii_px (tl, tr, br, bl)

fn radius_at(centered: vec2<f32>, radii: vec4<f32>) -> f32 {
  if (centered.x >= 0.0) {
    if (centered.y <= 0.0) { return radii.y; } // top-right
    return radii.z; // bottom-right
  }
  if (centered.y <= 0.0) { return radii.x; } // top-left
  return radii.w; // bottom-left
}

fn sd_rounded_rect(centered: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
  let r = clamp(radius, 0.0, min(half_size.x, half_size.y));
  let corner = abs(centered) - (half_size - vec2<f32>(r));
  let outside = length(max(corner, vec2<f32>(0.0))) - r;
  let inside = min(max(corner.x, corner.y), 0.0);
  return outside + inside;
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

fn fret_custom_effect(_src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, params: EffectParamsV1) -> vec4<f32> {
  let strength_px = max(0.0, params.vec4s[0].x);
  let scale_px = max(1.0, params.vec4s[0].y);
  let phase = params.vec4s[0].z;
  let chroma_px = max(0.0, params.vec4s[0].w);

  let edge_falloff_px = max(0.0, params.vec4s[1].x);
  let rim_strength = clamp(params.vec4s[1].y, 0.0, 1.0);
  let shadow_strength = clamp(params.vec4s[1].z, 0.0, 1.0);

  let grain_strength = max(0.0, params.vec4s[2].x);
  let grain_scale = max(0.1, params.vec4s[2].y);

  let local = fret_local_px(pos_px);
  let size = max(render_space.size_px, vec2<f32>(1.0));

  // Distance-to-edge inside a rounded rect. This keeps rim/shadow stable at corners even when the
  // effect is additionally clipped by a rounded container.
  let half_size = size * 0.5;
  let centered = local - half_size;
  let corner_radii = max(params.vec4s[3], vec4<f32>(0.0));
  let radius = radius_at(centered, corner_radii);
  let sd = sd_rounded_rect(centered, half_size, radius);
  let dist_in = max(0.0, -sd);
  var falloff = 1.0;
  if (edge_falloff_px > 0.0) {
    falloff = smoothstep(0.0, edge_falloff_px, dist_in);
  }

  // Sample a tiled displacement field from the user input (RGBA8 in our demo, linear color space).
  // Encoding matches `WarpMapEncodingV1::RgSigned`: RG store dx/dy in [-1, 1].
  let warp_uv = fract((local + vec2<f32>(phase * 17.0, phase * 11.0)) / scale_px);
  let warp = fret_sample_input(warp_uv);
  let disp = (warp.rg * 2.0 - vec2<f32>(1.0)) * (strength_px * falloff);

  let warped_pos = pos_px + disp;
  let base = sample_src_bilinear(warped_pos);

  var rgb = base.rgb;
  var a = base.a;

  // Simple chromatic aberration around the warped direction.
  if (chroma_px > 0.0) {
    let len = length(disp);
    let dir = select(vec2<f32>(1.0, 0.0), disp / len, len > 1e-3);
    let o = dir * chroma_px;
    let red = sample_src_bilinear(warped_pos + o);
    let blue = sample_src_bilinear(warped_pos - o);
    rgb = vec3<f32>(red.r, rgb.g, blue.b);
    a = max(a, max(red.a, blue.a));
  }

  // Rim + inner shadow: this is a recipe-only visual, not part of core semantics.
  let rim = smoothstep(1.5, 0.0, dist_in);
  var inner = 1.0;
  if (edge_falloff_px > 0.0) {
    inner = smoothstep(edge_falloff_px * 0.25, edge_falloff_px, dist_in);
  }
  rgb += vec3<f32>(1.0) * rim * (0.04 + 0.16 * rim_strength);
  rgb -= vec3<f32>(1.0) * (1.0 - inner) * (0.03 + 0.12 * shadow_strength);

  // Deterministic grain anchored to effect-local space.
  if (grain_strength > 0.0) {
    let n = fret_catalog_hash_noise01(local * grain_scale) - 0.5;
    rgb += vec3<f32>(n) * grain_strength;
  }

  return vec4<f32>(rgb, a);
}
"#;

fn srgb(r: u8, g: u8, b: u8, a: f32) -> Color {
    let mut c = fret_ui_kit::colors::linear_from_hex_rgb(
        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
    );
    c.a = a.clamp(0.0, 1.0);
    c
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
    radius: Px,
    mode: EffectMode,
    chain: EffectChain,
) -> AnyElement {
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
                move |_cx| vec![title],
            );

            vec![layer, pill]
        },
    )
}

#[derive(Clone, Copy)]
struct LiquidGlassCustomV2Effect(Option<EffectId>);

#[derive(Clone, Copy)]
struct LiquidGlassCustomV3Effect(Option<EffectId>);

#[derive(Clone)]
struct LiquidGlassState {
    show_fake: Model<bool>,
    show_warp: Model<bool>,
    show_warp_v2: Model<bool>,
    show_custom_v2: Model<bool>,
    show_custom_v3: Model<bool>,
    custom_v3_pair: Model<bool>,
    custom_v3_source_group: Model<bool>,
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

    lens_radius_px: Model<Vec<f32>>,

    custom_edge_falloff_px: Model<Vec<f32>>,
    custom_rim_strength: Model<Vec<f32>>,
    custom_shadow_strength: Model<Vec<f32>>,
    custom_grain_strength: Model<Vec<f32>>,
    custom_grain_scale: Model<Vec<f32>>,

    custom_v3_dispersion: Model<Vec<f32>>,
    custom_v3_bevel_strength: Model<Vec<f32>>,
    custom_v3_bevel_angle_deg: Model<Vec<f32>>,
    custom_v3_bevel_secondary: Model<Vec<f32>>,

    blur_radius_px: Model<Vec<f32>>,
    blur_downsample: Model<Vec<f32>>,
    saturation: Model<Vec<f32>>,
    brightness: Model<Vec<f32>>,
    contrast: Model<Vec<f32>>,

    use_backdrop: Model<bool>,
    use_dither: Model<bool>,
}

struct LiquidGlassView {
    st: LiquidGlassState,
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("liquid-glass-demo")
        .window("liquid-glass-demo", (1280.0, 720.0))
        .install_app(|app| {
            shadcn::shadcn_themes::apply_shadcn_new_york(
                app,
                shadcn::shadcn_themes::ShadcnBaseColor::Slate,
                shadcn::shadcn_themes::ShadcnColorScheme::Dark,
            );
        })
        .view::<LiquidGlassView>()?
        .install_custom_effects(install_custom_effects)
        .run()
        .map_err(anyhow::Error::from)
}

fn install_custom_effects(app: &mut App, effects: &mut dyn fret_core::CustomEffectService) {
    let mut program_v2 = CustomEffectProgramV2::wgsl_utf8(CUSTOM_WARP_V2_WGSL);
    let v2 = match program_v2.ensure_registered(effects) {
        Ok(id) => Some(id),
        Err(err) => {
            tracing::warn!(?err, "liquid-glass custom effect v2 registration failed");
            None
        }
    };
    app.set_global(LiquidGlassCustomV2Effect(v2));

    let mut program_v3 = CustomEffectProgramV3::wgsl_utf8(CUSTOM_EFFECT_V3_LENS_WGSL);
    let v3 = match program_v3.ensure_registered(effects) {
        Ok(id) => Some(id),
        Err(err) => {
            tracing::warn!(?err, "liquid-glass custom effect v3 registration failed");
            None
        }
    };
    app.set_global(LiquidGlassCustomV3Effect(v3));
}

impl LiquidGlassState {
    fn reset(models: &mut fret_runtime::ModelStore, st: &LiquidGlassState) {
        let _ = models.update(&st.show_fake, |v| *v = true);
        let _ = models.update(&st.show_warp, |v| *v = true);
        let _ = models.update(&st.show_warp_v2, |v| *v = false);
        let _ = models.update(&st.show_custom_v2, |v| *v = false);
        let _ = models.update(&st.show_custom_v3, |v| *v = false);
        let _ = models.update(&st.custom_v3_pair, |v| *v = false);
        let _ = models.update(&st.custom_v3_source_group, |v| *v = false);
        let _ = models.update(&st.show_inspector, |v| *v = false);
        let _ = models.update(&st.animate, |v| *v = true);
        let _ = models.update(&st.phase_speed, |v| *v = vec![0.65]);
        let _ = models.update(&st.warp_strength_px, |v| *v = vec![10.0]);
        let _ = models.update(&st.warp_scale_px, |v| *v = vec![72.0]);
        let _ = models.update(&st.warp_phase, |v| *v = vec![0.0]);
        let _ = models.update(&st.warp_chroma_px, |v| *v = vec![2.0]);
        let _ = models.update(&st.lens_radius_px, |v| *v = vec![20.0]);
        let _ = models.update(&st.custom_edge_falloff_px, |v| *v = vec![18.0]);
        let _ = models.update(&st.custom_rim_strength, |v| *v = vec![0.65]);
        let _ = models.update(&st.custom_shadow_strength, |v| *v = vec![0.55]);
        let _ = models.update(&st.custom_grain_strength, |v| *v = vec![0.06]);
        let _ = models.update(&st.custom_grain_scale, |v| *v = vec![1.0]);
        let _ = models.update(&st.custom_v3_dispersion, |v| *v = vec![0.55]);
        let _ = models.update(&st.custom_v3_bevel_strength, |v| *v = vec![1.0]);
        let _ = models.update(&st.custom_v3_bevel_angle_deg, |v| *v = vec![45.0]);
        let _ = models.update(&st.custom_v3_bevel_secondary, |v| *v = vec![1.0]);
        let _ = models.update(&st.blur_radius_px, |v| *v = vec![16.0]);
        let _ = models.update(&st.blur_downsample, |v| *v = vec![2.0]);
        let _ = models.update(&st.saturation, |v| *v = vec![1.10]);
        let _ = models.update(&st.brightness, |v| *v = vec![1.02]);
        let _ = models.update(&st.contrast, |v| *v = vec![1.02]);
        let _ = models.update(&st.use_backdrop, |v| *v = true);
        let _ = models.update(&st.use_dither, |v| *v = true);
    }

    fn apply_custom_v3_bevel_preset(models: &mut fret_runtime::ModelStore, st: &LiquidGlassState) {
        let _ = models.update(&st.custom_v3_bevel_strength, |v| *v = vec![1.0]);
        let _ = models.update(&st.custom_v3_bevel_angle_deg, |v| *v = vec![45.0]);
        let _ = models.update(&st.custom_v3_bevel_secondary, |v| *v = vec![1.0]);
    }

    fn disable_custom_v3_bevel(models: &mut fret_runtime::ModelStore, st: &LiquidGlassState) {
        let _ = models.update(&st.custom_v3_bevel_strength, |v| *v = vec![0.0]);
    }

    fn toggle_inspector(models: &mut fret_runtime::ModelStore, st: &LiquidGlassState) {
        let _ = models.update(&st.show_inspector, |v| *v = !*v);
    }
}

impl View for LiquidGlassView {
    fn init(app: &mut App, _window: AppWindowId) -> Self {
        let warp_map_size = (128u32, 128u32);
        let warp_map_rgba = generate_warp_map_rg_signed(warp_map_size.0, warp_map_size.1);
        let warp_map_key = ImageAssetKey::from_rgba8(
            warp_map_size.0,
            warp_map_size.1,
            ImageColorSpace::Linear,
            &warp_map_rgba,
        );
        let warp_map_rgba = Arc::new(warp_map_rgba);

        Self {
            st: LiquidGlassState {
                // Important: keep these defaults stable because perf scripts/baselines assume them.
                // - v1 baseline expects fake + v1 visible by default.
                // - v2 script toggles fake/v1 off and v2 on deterministically.
                show_fake: app.models_mut().insert(true),
                show_warp: app.models_mut().insert(true),
                show_warp_v2: app.models_mut().insert(false),
                show_custom_v2: app.models_mut().insert(false),
                show_custom_v3: app.models_mut().insert(false),
                custom_v3_pair: app.models_mut().insert(false),
                custom_v3_source_group: app.models_mut().insert(false),
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

                lens_radius_px: app.models_mut().insert(vec![20.0]),

                custom_edge_falloff_px: app.models_mut().insert(vec![18.0]),
                custom_rim_strength: app.models_mut().insert(vec![0.65]),
                custom_shadow_strength: app.models_mut().insert(vec![0.55]),
                custom_grain_strength: app.models_mut().insert(vec![0.06]),
                custom_grain_scale: app.models_mut().insert(vec![1.0]),

                custom_v3_dispersion: app.models_mut().insert(vec![0.55]),
                custom_v3_bevel_strength: app.models_mut().insert(vec![1.0]),
                custom_v3_bevel_angle_deg: app.models_mut().insert(vec![45.0]),
                custom_v3_bevel_secondary: app.models_mut().insert(vec![1.0]),

                // Keep defaults stable: perf scripts/baselines assume a visible blur chain.
                blur_radius_px: app.models_mut().insert(vec![16.0]),
                blur_downsample: app.models_mut().insert(vec![2.0]),
                saturation: app.models_mut().insert(vec![1.10]),
                brightness: app.models_mut().insert(vec![1.02]),
                contrast: app.models_mut().insert(vec![1.02]),

                use_backdrop: app.models_mut().insert(true),
                use_dither: app.models_mut().insert(true),
            },
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        cx.on_action_notify_models::<act::Reset>({
            let st = self.st.clone();
            move |models| {
                LiquidGlassState::reset(models, &st);
                true
            }
        });
        cx.on_action_notify_models::<act::ApplyCustomV3BevelPreset>({
            let st = self.st.clone();
            move |models| {
                LiquidGlassState::apply_custom_v3_bevel_preset(models, &st);
                true
            }
        });
        cx.on_action_notify_models::<act::DisableCustomV3Bevel>({
            let st = self.st.clone();
            move |models| {
                LiquidGlassState::disable_custom_v3_bevel(models, &st);
                true
            }
        });
        cx.on_action_notify_models::<act::ToggleInspector>({
            let st = self.st.clone();
            move |models| {
                LiquidGlassState::toggle_inspector(models, &st);
                true
            }
        });

        view(cx, &mut self.st)
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut LiquidGlassState) -> Elements {
    let theme = Theme::global(&*cx.app).snapshot();
    let theme_stage = theme.clone();
    let viewport = cx.environment_viewport_bounds(Invalidation::Layout);
    let left = Px(24.0);
    let top = Px(24.0);
    let right = Px(24.0);
    let bottom = Px(24.0);

    let show_fake_model = st.show_fake.clone();
    let show_warp_model = st.show_warp.clone();
    let show_warp_v2_model = st.show_warp_v2.clone();
    let show_custom_v2_model = st.show_custom_v2.clone();
    let show_custom_v3_model = st.show_custom_v3.clone();
    let custom_v3_pair_model = st.custom_v3_pair.clone();
    let custom_v3_source_group_model = st.custom_v3_source_group.clone();
    let custom_v3_dispersion_model = st.custom_v3_dispersion.clone();
    let animate_model = st.animate.clone();
    let phase_speed_model = st.phase_speed.clone();
    let show_inspector_model = st.show_inspector.clone();

    let warp_strength_model = st.warp_strength_px.clone();
    let warp_scale_model = st.warp_scale_px.clone();
    let warp_phase_model = st.warp_phase.clone();
    let warp_chroma_model = st.warp_chroma_px.clone();

    let lens_radius_model = st.lens_radius_px.clone();

    let custom_edge_model = st.custom_edge_falloff_px.clone();
    let custom_rim_model = st.custom_rim_strength.clone();
    let custom_shadow_model = st.custom_shadow_strength.clone();
    let custom_grain_model = st.custom_grain_strength.clone();
    let custom_grain_scale_model = st.custom_grain_scale.clone();

    let custom_v3_bevel_strength_model = st.custom_v3_bevel_strength.clone();
    let custom_v3_bevel_angle_model = st.custom_v3_bevel_angle_deg.clone();
    let custom_v3_bevel_secondary_model = st.custom_v3_bevel_secondary.clone();

    let blur_radius_model = st.blur_radius_px.clone();
    let blur_downsample_model = st.blur_downsample.clone();
    let saturation_model = st.saturation.clone();
    let brightness_model = st.brightness.clone();
    let contrast_model = st.contrast.clone();

    let use_backdrop_model = st.use_backdrop.clone();
    let use_dither_model = st.use_dither.clone();

    let show_fake = cx.watch_model(&st.show_fake).layout().value_or(true);
    let show_warp = cx.watch_model(&st.show_warp).layout().value_or(true);
    let show_warp_v2 = cx.watch_model(&st.show_warp_v2).layout().value_or(false);
    let show_custom_v2 = cx.watch_model(&st.show_custom_v2).layout().value_or(false);
    let show_custom_v3 = cx.watch_model(&st.show_custom_v3).layout().value_or(false);
    let custom_v3_pair = cx.watch_model(&st.custom_v3_pair).layout().value_or(false);
    let custom_v3_source_group = cx
        .watch_model(&st.custom_v3_source_group)
        .layout()
        .value_or(false);
    let show_inspector = cx.watch_model(&st.show_inspector).layout().value_or(true);
    let animate = cx.watch_model(&st.animate).layout().value_or(true);
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

    let lens_radius_px = watch_first_f32(cx, &st.lens_radius_px, 20.0).clamp(0.0, 64.0);
    let lens_radius = Px(lens_radius_px);

    let custom_edge_falloff_px = watch_first_f32(cx, &st.custom_edge_falloff_px, 18.0);
    let custom_rim_strength = watch_first_f32(cx, &st.custom_rim_strength, 0.65);
    let custom_shadow_strength = watch_first_f32(cx, &st.custom_shadow_strength, 0.55);
    let custom_grain_strength = watch_first_f32(cx, &st.custom_grain_strength, 0.06);
    let custom_grain_scale = watch_first_f32(cx, &st.custom_grain_scale, 1.0);

    let custom_v3_dispersion = watch_first_f32(cx, &st.custom_v3_dispersion, 0.55);
    let custom_v3_bevel_strength = watch_first_f32(cx, &st.custom_v3_bevel_strength, 0.0);
    let custom_v3_bevel_angle_deg = watch_first_f32(cx, &st.custom_v3_bevel_angle_deg, 45.0);
    let custom_v3_bevel_secondary = watch_first_f32(cx, &st.custom_v3_bevel_secondary, 1.0);

    let use_backdrop = cx.watch_model(&st.use_backdrop).layout().value_or(true);
    let use_dither = cx.watch_model(&st.use_dither).layout().value_or(true);
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

    let renderer_caps = cx.app.global::<RendererCapabilities>().cloned();
    let custom_v2_capable = renderer_caps
        .as_ref()
        .map(|c| c.custom_effect_v2_user_image)
        .unwrap_or(false);
    let custom_v3_capable = renderer_caps
        .as_ref()
        .map(|c| c.custom_effect_v3)
        .unwrap_or(false);

    let custom_v2_effect = cx
        .app
        .global::<LiquidGlassCustomV2Effect>()
        .and_then(|v| v.0);
    let custom_v2_supported = custom_v2_effect.is_some();
    let custom_v3_effect = cx
        .app
        .global::<LiquidGlassCustomV3Effect>()
        .and_then(|v| v.0);
    let custom_v3_supported = custom_v3_effect.is_some();

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

    let custom_v2_chain = {
        let step = match (custom_v2_effect, warp_image) {
            (Some(effect), Some(image)) => Some(EffectStep::CustomV2 {
                id: effect,
                params: EffectParamsV1 {
                    vec4s: [
                        [
                            warp_base.strength_px.0,
                            warp_base.scale_px.0,
                            warp_base.phase,
                            warp_base.chromatic_aberration_px.0,
                        ],
                        [
                            custom_edge_falloff_px.clamp(0.0, 64.0),
                            custom_rim_strength.clamp(0.0, 1.0),
                            custom_shadow_strength.clamp(0.0, 1.0),
                            0.0,
                        ],
                        [
                            custom_grain_strength.clamp(0.0, 0.25),
                            custom_grain_scale.clamp(0.1, 8.0),
                            0.0,
                            0.0,
                        ],
                        [
                            lens_radius_px,
                            lens_radius_px,
                            lens_radius_px,
                            lens_radius_px,
                        ],
                    ],
                },
                max_sample_offset_px:
                    crate::effect_authoring::custom_effect_warp_max_sample_offset_px(
                        warp_base.strength_px.0,
                        warp_base.chromatic_aberration_px.0,
                    ),
                input_image: Some(CustomEffectImageInputV1 {
                    image,
                    uv: UvRect::FULL,
                    sampling: ImageSamplingHint::Linear,
                }),
            }),
            // Keep the lens deterministic while the warp input image is still loading: the backend
            // binds a renderer-owned fallback input texture for `input_image: None`.
            (Some(effect), None) => Some(EffectStep::CustomV2 {
                id: effect,
                params: EffectParamsV1 {
                    vec4s: [
                        [
                            warp_base.strength_px.0,
                            warp_base.scale_px.0,
                            warp_base.phase,
                            warp_base.chromatic_aberration_px.0,
                        ],
                        [
                            custom_edge_falloff_px.clamp(0.0, 64.0),
                            custom_rim_strength.clamp(0.0, 1.0),
                            custom_shadow_strength.clamp(0.0, 1.0),
                            0.0,
                        ],
                        [
                            custom_grain_strength.clamp(0.0, 0.25),
                            custom_grain_scale.clamp(0.1, 8.0),
                            0.0,
                            0.0,
                        ],
                        [
                            lens_radius_px,
                            lens_radius_px,
                            lens_radius_px,
                            lens_radius_px,
                        ],
                    ],
                },
                max_sample_offset_px:
                    crate::effect_authoring::custom_effect_warp_max_sample_offset_px(
                        warp_base.strength_px.0,
                        warp_base.chromatic_aberration_px.0,
                    ),
                input_image: None,
            }),
            _ => None,
        };

        // Reuse the standard post chain (blur + adjust + optional dither).
        build_chain(
            step,
            blur_radius_px,
            blur_downsample,
            saturation,
            brightness,
            contrast,
            use_dither,
        )
    };

    let custom_v3_chain = custom_v3_effect.map(|id| {
        let sf = cx.environment_scale_factor(Invalidation::Paint).max(1.0e-6);
        let refraction_height_px = custom_edge_falloff_px.clamp(0.0, 64.0);
        // Map the demo's warp strength to a more noticeable refraction amount.
        let refraction_amount_px = (warp_strength_px * 3.2 + 8.0).clamp(0.0, 96.0);
        let dispersion = custom_v3_dispersion.clamp(0.0, 1.0);
        let noise_alpha = (custom_grain_strength * 0.2).clamp(0.0, 0.1);
        let max_sample_offset_px =
            crate::effect_authoring::custom_effect_v3_lens_max_sample_offset_px(
                refraction_amount_px,
                dispersion,
            );

        let mut steps: Vec<EffectStep> = Vec::new();
        if blur_radius_px > 0.0 && steps.len() < EffectChain::MAX_STEPS {
            steps.push(EffectStep::GaussianBlur {
                radius_px: Px(blur_radius_px.clamp(0.0, 64.0)),
                downsample: blur_downsample.clamp(1, 4),
            });
        }
        if steps.len() < EffectChain::MAX_STEPS {
            steps.push(EffectStep::CustomV3 {
                id,
                params: EffectParamsV1 {
                    vec4s: [
                        // (refraction_height_px, refraction_amount_px, pyramid_level, frost_mix)
                        [
                            refraction_height_px * sf,
                            refraction_amount_px * sf,
                            3.0,
                            0.75,
                        ],
                        // (corner_radius_px, depth_effect, dispersion, dispersion_quality)
                        [lens_radius_px * sf, 0.18, dispersion, 1.0],
                        // (noise_alpha, bevel_strength, bevel_light_angle_deg, bevel_secondary_strength)
                        [
                            noise_alpha,
                            custom_v3_bevel_strength.clamp(0.0, 1.0),
                            custom_v3_bevel_angle_deg,
                            custom_v3_bevel_secondary.clamp(0.0, 1.0),
                        ],
                        // tint (rgb + alpha)
                        [1.0, 1.0, 1.0, 0.08],
                    ],
                },
                max_sample_offset_px,
                user0: None,
                user1: None,
                sources: CustomEffectSourcesV3 {
                    want_raw: true,
                    pyramid: Some(CustomEffectPyramidRequestV1 {
                        max_levels: 6,
                        max_radius_px: Px(32.0),
                    }),
                },
            });
        }
        if steps.len() < EffectChain::MAX_STEPS {
            steps.push(EffectStep::ColorAdjust {
                saturation: saturation.clamp(0.0, 3.0),
                brightness: brightness.clamp(0.0, 3.0),
                contrast: contrast.clamp(0.0, 3.0),
            });
        }
        if use_dither && steps.len() < EffectChain::MAX_STEPS {
            steps.push(EffectStep::Dither {
                mode: DitherMode::Bayer4x4,
            });
        }
        EffectChain::from_steps(&steps).sanitize()
    });

    let mut root_layout = LayoutStyle::default();
    root_layout.size = SizeStyle {
        width: Length::Fill,
        height: Length::Fill,
        ..Default::default()
    };
    root_layout.position = PositionStyle::Relative;

    let bg = srgb(10, 12, 18, 1.0);

    let reset_stage: fret_runtime::CommandId = act::Reset.into();
    let reset_inspector: fret_runtime::CommandId = act::Reset.into();
    let toggle_inspector: fret_runtime::CommandId = act::ToggleInspector.into();
    let bevel_preset: fret_runtime::CommandId = act::ApplyCustomV3BevelPreset.into();
    let bevel_off: fret_runtime::CommandId = act::DisableCustomV3Bevel.into();

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
                                top: Some(Px(0.0)).into(),
                                right: Some(Px(0.0)).into(),
                                bottom: Some(Px(0.0)).into(),
                                left: Some(Px(0.0)).into(),
                            };

                            let stripe_w = Px(18.0);
                            let stripe_count =
                                ((viewport.size.width.0 / stripe_w.0).ceil() as usize).max(1) + 1;
                            let stripes = cx.row(
                                RowProps {
                                    layout: stripes_layout,
                                    gap: SpacingLength::Px(Px(0.0)),
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
                            blob_layout.inset.left = Some(left).into();
                            blob_layout.inset.top = Some(top).into();
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
                            blob2_layout.inset.right = Some(right).into();
                            blob2_layout.inset.top = Some(top).into();
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
                            cards_layout.inset.left = Some(left).into();
                            cards_layout.inset.top = Some(top).into();
                            cards_layout.size.width = Length::Px(Px(760.0));
                            cards_layout.size.height = Length::Px(Px(120.0));

                            let cards = cx.row(
                                RowProps {
                                    layout: cards_layout,
                                    gap: SpacingLength::Px(Px(12.0)),
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
                                            ink_overflow: Default::default(),
                                        });

                                        cx.container(
                                            ContainerProps {
                                                layout,
                                                padding: Edges::all(Px(14.0)).into(),
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
                            hud_layout.inset.top = Some(top).into();
                            hud_layout.inset.left = Some(left).into();
                            hud_layout.overflow = Overflow::Clip;

                            let mut hud_bg = theme_stage.color_token("card");
                            hud_bg.a = (hud_bg.a * 0.92).clamp(0.0, 1.0);
                            let hud = cx.container(
                                ContainerProps {
                                    layout: hud_layout,
                                    padding: Edges::all(Px(12.0)).into(),
                                    background: Some(hud_bg),
                                    border: Edges::all(Px(1.0)),
                                    border_color: Some(theme_stage.color_token("border")),
                                    corner_radii: Corners::all(Px(12.0)),
                                    ..Default::default()
                                },
                                move |cx| {
                                    vec![ui::v_flex(|cx| {
                                            vec![
                                                shadcn::typography::h4(cx, "Liquid glass"),
                                                shadcn::typography::muted(
                                                    cx,
                                                    "BackdropWarpV2 (bounded), WebGPU-safe.",
                                                ),
                                                shadcn::Separator::new().into_element(cx),
                                                ui::h_row(|cx| {
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
                                                    })
                                                    .gap(Space::N2)
                                                    .items_center()
                                                    .into_element(cx),
                                                ui::h_row(|cx| {
                                                        vec![
                                                            shadcn::Switch::new(
                                                                show_custom_v2_model.clone(),
                                                            )
                                                            .a11y_label("Show custom v2 lens")
                                                            .test_id(
                                                                "liquid-glass-switch-show-custom-v2",
                                                            )
                                                            .into_element(cx),
                                                            shadcn::Label::new("Custom v2")
                                                                .into_element(cx),
                                                            shadcn::Switch::new(
                                                                show_custom_v3_model.clone(),
                                                            )
                                                            .a11y_label("Show custom v3 lens")
                                                            .test_id(
                                                                "liquid-glass-switch-show-custom-v3",
                                                            )
                                                            .into_element(cx),
                                                            shadcn::Label::new("Custom v3")
                                                                .into_element(cx),
                                                        ]
                                                    })
                                                    .gap(Space::N2)
                                                    .items_center()
                                                    .into_element(cx),
                                                ui::h_row(|cx| {
                                                        vec![
                                                            shadcn::Switch::new(
                                                                custom_v3_pair_model.clone(),
                                                            )
                                                            .a11y_label(
                                                                "Show two custom v3 lenses",
                                                            )
                                                            .test_id(
                                                                "liquid-glass-switch-custom-v3-pair",
                                                            )
                                                            .into_element(cx),
                                                            shadcn::Label::new("V3 pair")
                                                                .into_element(cx),
                                                            shadcn::Switch::new(
                                                                custom_v3_source_group_model
                                                                    .clone(),
                                                            )
                                                            .a11y_label(
                                                                "Use custom v3 backdrop source group",
                                                            )
                                                            .test_id(
                                                                "liquid-glass-switch-custom-v3-source-group",
                                                            )
                                                            .into_element(cx),
                                                            shadcn::Label::new("V3 group")
                                                                .into_element(cx),
                                                        ]
                                                    })
                                                    .gap(Space::N2)
                                                    .items_center()
                                                    .into_element(cx),
                                                ui::h_row(|cx| {
                                                        vec![
                                                            shadcn::Switch::new(
                                                                use_backdrop_model.clone(),
                                                            )
                                                            .a11y_label("Backdrop mode")
                                                            .test_id(
                                                                "liquid-glass-switch-use-backdrop",
                                                            )
                                                            .into_element(cx),
                                                            shadcn::Label::new("Backdrop")
                                                                .into_element(cx),
                                                            shadcn::Switch::new(
                                                                use_dither_model.clone(),
                                                            )
                                                            .a11y_label("Dither")
                                                            .test_id(
                                                                "liquid-glass-switch-use-dither",
                                                            )
                                                            .into_element(cx),
                                                            shadcn::Label::new("Dither")
                                                                .into_element(cx),
                                                            shadcn::Switch::new(
                                                                animate_model.clone(),
                                                            )
                                                            .a11y_label("Animate phase")
                                                            .test_id("liquid-glass-switch-animate")
                                                            .into_element(cx),
                                                            shadcn::Label::new("Animate")
                                                                .into_element(cx),
                                                        ]
                                                    })
                                                    .gap(Space::N2)
                                                    .items_center()
                                                    .into_element(cx),
                                                ui::h_row(|cx| {
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
                                                                .test_id(
                                                                    "liquid-glass-button-reset-stage",
                                                                )
                                                                .into_element(cx),
                                                        ]
                                                    })
                                                    .gap(Space::N2)
                                                    .items_center()
                                                    .into_element(cx),
                                            ]
                                        })
                                        .gap(Space::N2)
                                        .items_stretch()
                                        .into_element(cx)]
                                },
                            );

                            // Lenses (bottom-left).
                            let mut lenses_layout = LayoutStyle::default();
                            lenses_layout.position = PositionStyle::Absolute;
                            lenses_layout.inset.left = Some(left).into();
                            lenses_layout.inset.bottom = Some(bottom).into();
                            let lenses = cx.row(
                                RowProps {
                                    layout: lenses_layout,
                                    gap: SpacingLength::Px(Px(14.0)),
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
                                                lens_radius,
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
                                                lens_radius,
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
                                                lens_radius,
                                                mode,
                                                warp_v2_chain,
                                            )
                                            .test_id("liquid-glass-lens-warp-v2"),
                                        );
                                    }
                                    if show_custom_v2 {
                                        let label = if !custom_v2_supported {
                                            if !custom_v2_capable {
                                                Arc::from("CustomV2 (unsupported backend)")
                                            } else {
                                                Arc::from("CustomV2 (registration failed)")
                                            }
                                        } else if warp_map_loaded {
                                            Arc::from("CustomV2 (image warp + rim/grain)")
                                        } else {
                                            Arc::from("CustomV2 (loading input)")
                                        };
                                        out.push(
                                            lens_panel(cx, label, lens_radius, mode, custom_v2_chain)
                                                .test_id("liquid-glass-lens-custom-v2"),
                                        );
                                    }
                                    if show_custom_v3 {
                                        let label: Arc<str> = if !custom_v3_supported {
                                            if !custom_v3_capable {
                                                Arc::from("CustomV3 (unsupported backend)")
                                            } else {
                                                Arc::from("CustomV3 (registration failed)")
                                            }
                                        } else {
                                            Arc::from("CustomV3 (lens refraction; raw+pyramid)")
                                        };
                                        let chain = custom_v3_chain.unwrap_or(fake_chain.clone());
                                        if custom_v3_pair {
                                            let lens_a = lens_panel(
                                                cx,
                                                label.clone(),
                                                lens_radius,
                                                mode,
                                                chain.clone(),
                                            )
                                                    .test_id("liquid-glass-lens-custom-v3");
                                            let lens_b = lens_panel(
                                                cx,
                                                Arc::from("CustomV3 (lens B; ordering drift)"),
                                                lens_radius,
                                                mode,
                                                chain,
                                            )
                                            .test_id("liquid-glass-lens-custom-v3-b");

                                            let pair = cx.row(
                                                RowProps {
                                                    layout: LayoutStyle::default(),
                                                    gap: SpacingLength::Px(Px(14.0)),
                                                    justify: MainAlign::Start,
                                                    align: CrossAlign::Start,
                                                    ..Default::default()
                                                },
                                                move |_cx| vec![lens_a, lens_b],
                                            );

                                            let wants_group = custom_v3_source_group
                                                && custom_v3_supported
                                                && mode == EffectMode::Backdrop;
                                            if wants_group {
                                                out.push(cx.backdrop_source_group_v1(
                                                    Some(CustomEffectPyramidRequestV1 {
                                                        max_levels: 6,
                                                        max_radius_px: Px(32.0),
                                                    }),
                                                    EffectQuality::Auto,
                                                    move |_cx| vec![pair],
                                                ));
                                            } else {
                                                out.push(pair);
                                            }
                                        } else {
                                            out.push(
                                                lens_panel(cx, label, lens_radius, mode, chain)
                                                    .test_id("liquid-glass-lens-custom-v3"),
                                            );
                                        }
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
                        layout.inset.top = Some(top).into();
                        layout.inset.right = Some(right).into();
                        layout.inset.bottom = Some(bottom).into();
                        layout.size.width = Length::Px(Px(380.0));
                        layout.size.height = Length::Fill;
                        layout.overflow = Overflow::Clip;

                        cx.container(
                            ContainerProps {
                                layout,
                                padding: Edges::all(Px(16.0)).into(),
                                background: Some(theme.color_token("card")),
                                border: Edges::all(Px(1.0)),
                                border_color: Some(theme.color_token("border")),
                                ..Default::default()
                            },
                            move |cx| {
                                let header = shadcn::CardHeader::new([
                                    shadcn::CardTitle::new("Inspector").into_element(cx),
                                    shadcn::CardDescription::new(format!(
                                        "mode={:?} steps(fake={}, v1={}, v2={}, custom_v3={}) warp_map_loaded={}",
                                        mode,
                                        fake_chain.iter().count(),
                                        warp_chain.iter().count(),
                                        warp_v2_chain.iter().count(),
                                        custom_v3_chain.as_ref().map_or(0, |c| c.iter().count()),
                                        warp_map_loaded
                                    ))
                                    .into_element(cx),
                                ]);

                                let label_row =
                                    |cx: &mut ElementContext<'_, App>,
                                     label: &str,
                                     value: String| {
                                        ui::h_row(move |cx| {
                                            vec![
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

                                let lens_radius_row = ui::v_flex(|cx| {
                                    vec![
                                        label_row(
                                            cx,
                                            "Lens radius (px)",
                                            format!("{lens_radius_px:.1}"),
                                        ),
                                        shadcn::Slider::new(lens_radius_model.clone())
                                            .range(0.0, 64.0)
                                            .step(0.5)
                                            .into_element(cx),
                                    ]
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let warp_strength_row = ui::v_flex(|cx| {
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
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let warp_scale_row = ui::v_flex(|cx| {
                                    vec![
                                        label_row(cx, "Warp scale (px)", format!("{warp_scale_px:.0}")),
                                        shadcn::Slider::new(warp_scale_model.clone())
                                            .range(BackdropWarpV1::MIN_SCALE_PX.0, 256.0)
                                            .step(1.0)
                                            .into_element(cx),
                                    ]
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let chroma_row = ui::v_flex(|cx| {
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
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let custom_edge_row = ui::v_flex(|cx| {
                                    vec![
                                        label_row(
                                            cx,
                                            "CustomV2 edge falloff (px)",
                                            format!("{custom_edge_falloff_px:.1}"),
                                        ),
                                        shadcn::Slider::new(custom_edge_model.clone())
                                            .range(0.0, 64.0)
                                            .step(0.25)
                                            .into_element(cx),
                                    ]
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let custom_rim_row = ui::v_flex(|cx| {
                                    vec![
                                        label_row(
                                            cx,
                                            "CustomV2 rim strength",
                                            format!("{custom_rim_strength:.2}"),
                                        ),
                                        shadcn::Slider::new(custom_rim_model.clone())
                                            .range(0.0, 1.0)
                                            .step(0.01)
                                            .into_element(cx),
                                    ]
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let custom_shadow_row = ui::v_flex(|cx| {
                                    vec![
                                        label_row(
                                            cx,
                                            "CustomV2 shadow strength",
                                            format!("{custom_shadow_strength:.2}"),
                                        ),
                                        shadcn::Slider::new(custom_shadow_model.clone())
                                            .range(0.0, 1.0)
                                            .step(0.01)
                                            .into_element(cx),
                                    ]
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let custom_grain_row = ui::v_flex(|cx| {
                                    vec![
                                        label_row(
                                            cx,
                                            "CustomV2 grain strength",
                                            format!("{custom_grain_strength:.2}"),
                                        ),
                                        shadcn::Slider::new(custom_grain_model.clone())
                                            .range(0.0, 0.25)
                                            .step(0.01)
                                            .into_element(cx),
                                    ]
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let custom_grain_scale_row = ui::v_flex(|cx| {
                                    vec![
                                        label_row(
                                            cx,
                                            "CustomV2 grain scale",
                                            format!("{custom_grain_scale:.2}"),
                                        ),
                                        shadcn::Slider::new(custom_grain_scale_model.clone())
                                            .range(0.25, 6.0)
                                            .step(0.05)
                                            .into_element(cx),
                                    ]
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let custom_v3_bevel_strength_row = ui::v_flex(|cx| {
                                    vec![
                                        label_row(
                                            cx,
                                            "CustomV3 bevel strength",
                                            format!("{custom_v3_bevel_strength:.2}"),
                                        ),
                                        shadcn::Slider::new(custom_v3_bevel_strength_model.clone())
                                            .range(0.0, 1.0)
                                            .step(0.01)
                                            .into_element(cx)
                                            .test_id(
                                                "liquid-glass-slider-custom-v3-bevel-strength",
                                            ),
                                    ]
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let custom_v3_bevel_angle_row = ui::v_flex(|cx| {
                                    vec![
                                        label_row(
                                            cx,
                                            "CustomV3 bevel light angle (deg)",
                                            format!("{custom_v3_bevel_angle_deg:.0}"),
                                        ),
                                        shadcn::Slider::new(custom_v3_bevel_angle_model.clone())
                                            .range(0.0, 360.0)
                                            .step(1.0)
                                            .into_element(cx)
                                            .test_id(
                                                "liquid-glass-slider-custom-v3-bevel-angle-deg",
                                            ),
                                    ]
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let custom_v3_bevel_secondary_row = ui::v_flex(|cx| {
                                    vec![
                                        label_row(
                                            cx,
                                            "CustomV3 bevel secondary strength",
                                            format!("{custom_v3_bevel_secondary:.2}"),
                                        ),
                                        shadcn::Slider::new(
                                            custom_v3_bevel_secondary_model.clone(),
                                        )
                                        .range(0.0, 1.0)
                                        .step(0.01)
                                        .into_element(cx)
                                        .test_id(
                                            "liquid-glass-slider-custom-v3-bevel-secondary",
                                        ),
                                    ]
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let custom_v3_dispersion_row = ui::v_flex(|cx| {
                                    vec![
                                        label_row(
                                            cx,
                                            "CustomV3 dispersion",
                                            format!("{custom_v3_dispersion:.2}"),
                                        ),
                                        shadcn::Slider::new(custom_v3_dispersion_model.clone())
                                            .range(0.0, 1.0)
                                            .step(0.01)
                                            .into_element(cx)
                                            .test_id(
                                                "liquid-glass-slider-custom-v3-dispersion",
                                            ),
                                    ]
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let phase_row = ui::v_flex(|cx| {
                                    vec![
                                        label_row(cx, "Phase", format!("{phase:.2}")),
                                        shadcn::Slider::new(warp_phase_model.clone())
                                            .range(0.0, 12.0)
                                            .step(0.01)
                                            .into_element(cx),
                                    ]
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let speed_row = ui::v_flex(|cx| {
                                    vec![
                                        label_row(cx, "Phase speed", format!("{phase_speed:.2}")),
                                        shadcn::Slider::new(phase_speed_model.clone())
                                            .range(0.0, 2.0)
                                            .step(0.01)
                                            .into_element(cx),
                                    ]
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let blur_row = ui::v_flex(|cx| {
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
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let downsample_row = ui::v_flex(|cx| {
                                    vec![
                                        label_row(cx, "Blur downsample", format!("{blur_downsample}x")),
                                        shadcn::Slider::new(blur_downsample_model.clone())
                                            .range(1.0, 4.0)
                                            .step(1.0)
                                            .into_element(cx),
                                    ]
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let sat_row = ui::v_flex(|cx| {
                                    vec![
                                        label_row(cx, "Saturation", format!("{saturation:.2}")),
                                        shadcn::Slider::new(saturation_model.clone())
                                            .range(0.6, 1.8)
                                            .step(0.01)
                                            .into_element(cx),
                                    ]
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let bright_row = ui::v_flex(|cx| {
                                    vec![
                                        label_row(cx, "Brightness", format!("{brightness:.2}")),
                                        shadcn::Slider::new(brightness_model.clone())
                                            .range(0.8, 1.3)
                                            .step(0.01)
                                            .into_element(cx),
                                    ]
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let contrast_row = ui::v_flex(|cx| {
                                    vec![
                                        label_row(cx, "Contrast", format!("{contrast:.2}")),
                                        shadcn::Slider::new(contrast_model.clone())
                                            .range(0.8, 1.3)
                                            .step(0.01)
                                            .into_element(cx),
                                    ]
                                })
                                .gap(Space::N2)
                                .into_element(cx);

                                let footer = ui::h_row(|cx| {
                                    vec![
                                        shadcn::Button::new("Bevel preset")
                                            .variant(shadcn::ButtonVariant::Secondary)
                                            .size(shadcn::ButtonSize::Sm)
                                            .on_click(bevel_preset)
                                            .test_id(
                                                "liquid-glass-button-custom-v3-bevel-preset",
                                            )
                                            .into_element(cx),
                                        shadcn::Button::new("Bevel off")
                                            .variant(shadcn::ButtonVariant::Secondary)
                                            .size(shadcn::ButtonSize::Sm)
                                            .on_click(bevel_off)
                                            .test_id("liquid-glass-button-custom-v3-bevel-off")
                                            .into_element(cx),
                                        cx.spacer(SpacerProps::default()),
                                        shadcn::Button::new("Reset")
                                            .variant(shadcn::ButtonVariant::Secondary)
                                            .size(shadcn::ButtonSize::Sm)
                                            .on_click(reset_inspector)
                                            .test_id("liquid-glass-button-reset-inspector")
                                            .into_element(cx),
                                    ]
                                })
                                .gap(Space::N2)
                                .items_center()
                                .into_element(cx);

                                let body = ui::v_flex(|cx| {
                                    vec![
                                        header.into_element(cx),
                                        shadcn::Separator::new().into_element(cx),
                                        lens_radius_row,
                                        shadcn::Separator::new().into_element(cx),
                                        warp_strength_row,
                                        warp_scale_row,
                                        chroma_row,
                                        phase_row,
                                        speed_row,
                                        shadcn::Separator::new().into_element(cx),
                                        custom_edge_row,
                                        custom_rim_row,
                                        custom_shadow_row,
                                        custom_grain_row,
                                        custom_grain_scale_row,
                                        custom_v3_bevel_strength_row,
                                        custom_v3_bevel_angle_row,
                                        custom_v3_bevel_secondary_row,
                                        custom_v3_dispersion_row,
                                        shadcn::Separator::new().into_element(cx),
                                        blur_row,
                                        downsample_row,
                                        sat_row,
                                        bright_row,
                                        contrast_row,
                                        shadcn::Separator::new().into_element(cx),
                                        footer,
                                    ]
                                })
                                .gap(Space::N4)
                                .items_stretch()
                                .into_element(cx);

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
