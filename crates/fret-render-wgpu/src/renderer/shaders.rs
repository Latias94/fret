const CLIP_SDF_CORE_WGSL: &str = include_str!("pipelines/wgsl/clip_sdf_core.wgsl");

const QUAD_SHADER_PART_A: &str = include_str!("pipelines/wgsl/quad_part_a.wgsl");

const QUAD_SHADER_PART_B: &str = include_str!("pipelines/wgsl/quad_part_b.wgsl");

pub(super) fn quad_shader_source() -> String {
    format!("{QUAD_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{QUAD_SHADER_PART_B}")
}

pub(super) const VIEWPORT_SHADER: &str = include_str!("pipelines/wgsl/viewport.wgsl");

// Large effect shaders live as external WGSL files for reviewable diffs and lower merge conflict risk.
pub(super) const BLIT_SHADER: &str = include_str!("pipelines/wgsl/blit.wgsl");

pub(super) const BLIT_SRGB_ENCODE_SHADER: &str =
    include_str!("pipelines/wgsl/blit_srgb_encode.wgsl");

pub(super) const MIP_DOWNSAMPLE_BOX_2X2_SHADER: &str =
    include_str!("pipelines/wgsl/mip_downsample_box_2x2.wgsl");

pub(super) const DROP_SHADOW_SHADER: &str = include_str!("pipelines/wgsl/drop_shadow.wgsl");

const DROP_SHADOW_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/drop_shadow_masked_part_a.wgsl");

const DROP_SHADOW_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/drop_shadow_masked_part_b.wgsl");

pub(super) fn drop_shadow_masked_shader_source() -> String {
    format!(
        "{DROP_SHADOW_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{DROP_SHADOW_MASKED_SHADER_PART_B}"
    )
}

pub(super) const DROP_SHADOW_MASK_SHADER: &str =
    include_str!("pipelines/wgsl/drop_shadow_mask.wgsl");

pub(super) const DOWNSAMPLE_NEAREST_SHADER: &str =
    include_str!("pipelines/wgsl/downsample_nearest.wgsl");

pub(super) const UPSCALE_NEAREST_SHADER: &str = include_str!("pipelines/wgsl/upscale_nearest.wgsl");

const UPSCALE_NEAREST_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/upscale_nearest_masked_part_a.wgsl");

const UPSCALE_NEAREST_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/upscale_nearest_masked_part_b.wgsl");

pub(super) fn upscale_nearest_masked_shader_source() -> String {
    format!(
        "{UPSCALE_NEAREST_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{UPSCALE_NEAREST_MASKED_SHADER_PART_B}"
    )
}

pub(super) const UPSCALE_NEAREST_MASK_SHADER: &str =
    include_str!("pipelines/wgsl/upscale_nearest_mask.wgsl");

const CLIP_MASK_SHADER_PART_A: &str = include_str!("pipelines/wgsl/clip_mask_part_a.wgsl");

const CLIP_MASK_SHADER_PART_B: &str = include_str!("pipelines/wgsl/clip_mask_part_b.wgsl");

pub(super) fn clip_mask_shader_source() -> String {
    format!("{CLIP_MASK_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{CLIP_MASK_SHADER_PART_B}")
}

pub(super) const BACKDROP_WARP_SHADER: &str = include_str!("pipelines/wgsl/backdrop_warp.wgsl");

pub(super) const BACKDROP_WARP_IMAGE_SHADER: &str =
    include_str!("pipelines/wgsl/backdrop_warp_image.wgsl");

const BACKDROP_WARP_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/backdrop_warp_masked_part_a.wgsl");

const BACKDROP_WARP_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/backdrop_warp_masked_part_b.wgsl");

pub(super) fn backdrop_warp_masked_shader_source() -> String {
    format!(
        "{BACKDROP_WARP_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{BACKDROP_WARP_MASKED_SHADER_PART_B}"
    )
}

const BACKDROP_WARP_IMAGE_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/backdrop_warp_image_masked_part_a.wgsl");

const BACKDROP_WARP_IMAGE_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/backdrop_warp_image_masked_part_b.wgsl");

pub(super) fn backdrop_warp_image_masked_shader_source() -> String {
    format!(
        "{BACKDROP_WARP_IMAGE_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{BACKDROP_WARP_IMAGE_MASKED_SHADER_PART_B}"
    )
}

pub(super) const BACKDROP_WARP_IMAGE_MASK_SHADER: &str =
    include_str!("pipelines/wgsl/backdrop_warp_image_mask.wgsl");

pub(super) const BACKDROP_WARP_MASK_SHADER: &str =
    include_str!("pipelines/wgsl/backdrop_warp_mask.wgsl");

pub(super) const COLOR_ADJUST_SHADER: &str = include_str!("pipelines/wgsl/color_adjust.wgsl");

const COLOR_ADJUST_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/color_adjust_masked_part_a.wgsl");

const COLOR_ADJUST_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/color_adjust_masked_part_b.wgsl");

pub(super) fn color_adjust_masked_shader_source() -> String {
    format!(
        "{COLOR_ADJUST_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{COLOR_ADJUST_MASKED_SHADER_PART_B}"
    )
}

pub(super) const COLOR_ADJUST_MASK_SHADER: &str =
    include_str!("pipelines/wgsl/color_adjust_mask.wgsl");

pub(super) const COLOR_MATRIX_SHADER: &str = include_str!("pipelines/wgsl/color_matrix.wgsl");

const COLOR_MATRIX_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/color_matrix_masked_part_a.wgsl");

const COLOR_MATRIX_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/color_matrix_masked_part_b.wgsl");

pub(super) fn color_matrix_masked_shader_source() -> String {
    format!(
        "{COLOR_MATRIX_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{COLOR_MATRIX_MASKED_SHADER_PART_B}"
    )
}

pub(super) const COLOR_MATRIX_MASK_SHADER: &str =
    include_str!("pipelines/wgsl/color_matrix_mask.wgsl");

pub(super) const ALPHA_THRESHOLD_SHADER: &str = include_str!("pipelines/wgsl/alpha_threshold.wgsl");

const ALPHA_THRESHOLD_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/alpha_threshold_masked_part_a.wgsl");

const ALPHA_THRESHOLD_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/alpha_threshold_masked_part_b.wgsl");

pub(super) fn alpha_threshold_masked_shader_source() -> String {
    format!(
        "{ALPHA_THRESHOLD_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{ALPHA_THRESHOLD_MASKED_SHADER_PART_B}"
    )
}

pub(super) const ALPHA_THRESHOLD_MASK_SHADER: &str =
    include_str!("pipelines/wgsl/alpha_threshold_mask.wgsl");

pub(super) const DITHER_SHADER: &str = include_str!("pipelines/wgsl/dither.wgsl");

const DITHER_MASKED_SHADER_PART_A: &str = include_str!("pipelines/wgsl/dither_masked_part_a.wgsl");

const DITHER_MASKED_SHADER_PART_B: &str = include_str!("pipelines/wgsl/dither_masked_part_b.wgsl");

pub(super) fn dither_masked_shader_source() -> String {
    format!("{DITHER_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{DITHER_MASKED_SHADER_PART_B}")
}

pub(super) const DITHER_MASK_SHADER: &str = include_str!("pipelines/wgsl/dither_mask.wgsl");

const CUSTOM_EFFECT_UNMASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_unmasked_part_a.wgsl");
const CUSTOM_EFFECT_UNMASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_unmasked_part_b.wgsl");

const CUSTOM_EFFECT_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_masked_part_a.wgsl");
const CUSTOM_EFFECT_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_masked_part_b.wgsl");

const CUSTOM_EFFECT_MASK_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_mask_part_a.wgsl");
const CUSTOM_EFFECT_MASK_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_mask_part_b.wgsl");

pub(super) fn custom_effect_unmasked_shader_source(user_source: &str) -> String {
    format!(
        "{CUSTOM_EFFECT_UNMASKED_SHADER_PART_A}{user_source}\n{CUSTOM_EFFECT_UNMASKED_SHADER_PART_B}"
    )
}

pub(super) fn custom_effect_masked_shader_source(user_source: &str) -> String {
    format!(
        "{CUSTOM_EFFECT_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{user_source}\n{CUSTOM_EFFECT_MASKED_SHADER_PART_B}"
    )
}

pub(super) fn custom_effect_mask_shader_source(user_source: &str) -> String {
    format!("{CUSTOM_EFFECT_MASK_SHADER_PART_A}{user_source}\n{CUSTOM_EFFECT_MASK_SHADER_PART_B}")
}

const CUSTOM_EFFECT_V2_UNMASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_v2_unmasked_part_a.wgsl");
const CUSTOM_EFFECT_V2_UNMASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_v2_unmasked_part_b.wgsl");

const CUSTOM_EFFECT_V2_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_v2_masked_part_a.wgsl");
const CUSTOM_EFFECT_V2_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_v2_masked_part_b.wgsl");

const CUSTOM_EFFECT_V2_MASK_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_v2_mask_part_a.wgsl");
const CUSTOM_EFFECT_V2_MASK_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_v2_mask_part_b.wgsl");

pub(super) fn custom_effect_v2_unmasked_shader_source(user_source: &str) -> String {
    format!(
        "{CUSTOM_EFFECT_V2_UNMASKED_SHADER_PART_A}{user_source}\n{CUSTOM_EFFECT_V2_UNMASKED_SHADER_PART_B}"
    )
}

pub(super) fn custom_effect_v2_masked_shader_source(user_source: &str) -> String {
    format!(
        "{CUSTOM_EFFECT_V2_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{user_source}\n{CUSTOM_EFFECT_V2_MASKED_SHADER_PART_B}"
    )
}

pub(super) fn custom_effect_v2_mask_shader_source(user_source: &str) -> String {
    format!(
        "{CUSTOM_EFFECT_V2_MASK_SHADER_PART_A}{user_source}\n{CUSTOM_EFFECT_V2_MASK_SHADER_PART_B}"
    )
}

const CUSTOM_EFFECT_V3_UNMASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_v3_unmasked_part_a.wgsl");
const CUSTOM_EFFECT_V3_UNMASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_v3_unmasked_part_b.wgsl");

const CUSTOM_EFFECT_V3_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_v3_masked_part_a.wgsl");
const CUSTOM_EFFECT_V3_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_v3_masked_part_b.wgsl");

const CUSTOM_EFFECT_V3_MASK_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_v3_mask_part_a.wgsl");
const CUSTOM_EFFECT_V3_MASK_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_v3_mask_part_b.wgsl");

pub(super) fn custom_effect_v3_unmasked_shader_source(user_source: &str) -> String {
    format!(
        "{CUSTOM_EFFECT_V3_UNMASKED_SHADER_PART_A}{user_source}\n{CUSTOM_EFFECT_V3_UNMASKED_SHADER_PART_B}"
    )
}

pub(super) fn custom_effect_v3_masked_shader_source(user_source: &str) -> String {
    format!(
        "{CUSTOM_EFFECT_V3_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{user_source}\n{CUSTOM_EFFECT_V3_MASKED_SHADER_PART_B}"
    )
}

pub(super) fn custom_effect_v3_mask_shader_source(user_source: &str) -> String {
    format!(
        "{CUSTOM_EFFECT_V3_MASK_SHADER_PART_A}{user_source}\n{CUSTOM_EFFECT_V3_MASK_SHADER_PART_B}"
    )
}

pub(super) const BLUR_H_SHADER: &str = include_str!("pipelines/wgsl/blur_h.wgsl");

pub(super) const BLUR_V_SHADER: &str = include_str!("pipelines/wgsl/blur_v.wgsl");

const BLUR_H_MASKED_SHADER_PART_A: &str = include_str!("pipelines/wgsl/blur_h_masked_part_a.wgsl");

const BLUR_H_MASKED_SHADER_PART_B: &str = include_str!("pipelines/wgsl/blur_h_masked_part_b.wgsl");

pub(super) fn blur_h_masked_shader_source() -> String {
    format!("{BLUR_H_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{BLUR_H_MASKED_SHADER_PART_B}")
}

const BLUR_V_MASKED_SHADER_PART_A: &str = include_str!("pipelines/wgsl/blur_v_masked_part_a.wgsl");

const BLUR_V_MASKED_SHADER_PART_B: &str = include_str!("pipelines/wgsl/blur_v_masked_part_b.wgsl");

pub(super) fn blur_v_masked_shader_source() -> String {
    format!("{BLUR_V_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{BLUR_V_MASKED_SHADER_PART_B}")
}

pub(super) const BLUR_H_MASK_SHADER: &str = include_str!("pipelines/wgsl/blur_h_mask.wgsl");

pub(super) const BLUR_V_MASK_SHADER: &str = include_str!("pipelines/wgsl/blur_v_mask.wgsl");

pub(super) const COMPOSITE_PREMUL_SHADER: &str =
    include_str!("pipelines/wgsl/composite_premul.wgsl");

pub(super) const COMPOSITE_PREMUL_MASK_SHADER: &str =
    include_str!("pipelines/wgsl/composite_premul_mask.wgsl");

pub(super) const PATH_CLIP_MASK_SHADER: &str = include_str!("pipelines/wgsl/path_clip_mask.wgsl");

pub(super) const PATH_SHADER: &str = r#"
struct ClipRRect {
  rect: vec4<f32>,
  corner_radii: vec4<f32>,
  inv0: vec4<f32>,
  inv1: vec4<f32>,
};

struct Viewport {
  viewport_size: vec2<f32>,
  clip_head: u32,
  clip_count: u32,
  mask_head: u32,
  mask_count: u32,
  mask_scope_head: u32,
  mask_scope_count: u32,
  output_is_srgb: u32,
  _pad0: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
  text_gamma_ratios: vec4<f32>,
  text_grayscale_enhanced_contrast: f32,
  text_subpixel_enhanced_contrast: f32,
  _pad_text_quality0: u32,
  _pad_text_quality1: u32,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;

struct RenderSpace {
  origin_px: vec2<f32>,
  size_px: vec2<f32>,
};

@group(0) @binding(5) var<uniform> render_space: RenderSpace;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

struct MaskGradient {
  bounds: vec4<f32>,
  kind: u32,
  tile_mode: u32,
  stop_count: u32,
  _pad0: u32,
  params0: vec4<f32>,
  inv0: vec4<f32>,
  inv1: vec4<f32>,
  stop_alphas0: vec4<f32>,
  stop_alphas1: vec4<f32>,
  stop_offsets0: vec4<f32>,
  stop_offsets1: vec4<f32>,
};

struct MaskStack {
  masks: array<MaskGradient>,
};

@group(0) @binding(2) var<storage, read> mask_stack: MaskStack;

@group(0) @binding(6) var mask_image_sampler: sampler;
@group(0) @binding(7) var mask_image_texture: texture_2d<f32>;

@group(0) @binding(3) var material_catalog_texture: texture_2d_array<f32>;
@group(0) @binding(4) var material_catalog_sampler: sampler;

const MAX_STOPS: u32 = 8u;

struct Paint {
  kind: u32,
  tile_mode: u32,
  color_space: u32,
  stop_count: u32,
  eval_space: u32,
  _pad_eval0: u32,
  _pad_eval1: u32,
  _pad_eval2: u32,
  params0: vec4<f32>,
  params1: vec4<f32>,
  params2: vec4<f32>,
  params3: vec4<f32>,
  stop_colors: array<vec4<f32>, 8>,
  stop_offsets0: vec4<f32>,
  stop_offsets1: vec4<f32>,
};

struct PathPaints {
  paints: array<Paint>,
};

@group(1) @binding(0) var<storage, read> path_paints: PathPaints;

fn paint_stop_offset(p: Paint, i: u32) -> f32 {
  if (i < 4u) { return p.stop_offsets0[i]; }
  return p.stop_offsets1[i - 4u];
}

fn paint_unpremul_rgb(c: vec4<f32>) -> vec3<f32> {
  let a = max(c.a, 1e-6);
  return c.rgb / a;
}

fn linear_rgb_to_oklab(rgb: vec3<f32>) -> vec3<f32> {
  let c = max(rgb, vec3<f32>(0.0));
  let lms = vec3<f32>(
    0.4122214708 * c.x + 0.5363325363 * c.y + 0.0514459929 * c.z,
    0.2119034982 * c.x + 0.6806995451 * c.y + 0.1073969566 * c.z,
    0.0883024619 * c.x + 0.2817188376 * c.y + 0.6299787005 * c.z
  );
  let lms_cbrt = pow(max(lms, vec3<f32>(0.0)), vec3<f32>(1.0 / 3.0));
  return vec3<f32>(
    0.2104542553 * lms_cbrt.x + 0.7936177850 * lms_cbrt.y - 0.0040720468 * lms_cbrt.z,
    1.9779984951 * lms_cbrt.x - 2.4285922050 * lms_cbrt.y + 0.4505937099 * lms_cbrt.z,
    0.0259040371 * lms_cbrt.x + 0.7827717662 * lms_cbrt.y - 0.8086757660 * lms_cbrt.z
  );
}

fn oklab_to_linear_rgb(lab: vec3<f32>) -> vec3<f32> {
  let lms_cbrt = vec3<f32>(
    lab.x + 0.3963377774 * lab.y + 0.2158037573 * lab.z,
    lab.x - 0.1055613458 * lab.y - 0.0638541728 * lab.z,
    lab.x - 0.0894841775 * lab.y - 1.2914855480 * lab.z
  );
  let lms = lms_cbrt * lms_cbrt * lms_cbrt;
  return vec3<f32>(
    4.0767416621 * lms.x - 3.3077115913 * lms.y + 0.2309699292 * lms.z,
    -1.2684380046 * lms.x + 2.6097574011 * lms.y - 0.3413193965 * lms.z,
    -0.0041960863 * lms.x - 0.7034186147 * lms.y + 1.7076147010 * lms.z
  );
}

fn paint_mix_colorspace(p: Paint, a: vec4<f32>, b: vec4<f32>, u: f32) -> vec4<f32> {
  if (p.color_space == 1u) {
    let a0 = clamp(a.a, 0.0, 1.0);
    let a1 = clamp(b.a, 0.0, 1.0);
    let alpha = clamp(mix(a0, a1, u), 0.0, 1.0);

    let rgb0 = clamp(paint_unpremul_rgb(a), vec3<f32>(0.0), vec3<f32>(1.0));
    let rgb1 = clamp(paint_unpremul_rgb(b), vec3<f32>(0.0), vec3<f32>(1.0));
    let lab0 = linear_rgb_to_oklab(rgb0);
    let lab1 = linear_rgb_to_oklab(rgb1);
    let lab = mix(lab0, lab1, u);
    let rgb = clamp(oklab_to_linear_rgb(lab), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(rgb * alpha, alpha);
  }
  return mix(a, b, u);
}

fn paint_sample_stops(p: Paint, t: f32) -> vec4<f32> {
  let n = min(p.stop_count, MAX_STOPS);
  if (n == 0u) {
    return vec4<f32>(0.0);
  }
  if (n == 1u) {
    return p.stop_colors[0u];
  }

  var prev_offset = paint_stop_offset(p, 0u);
  var prev_color = p.stop_colors[0u];
  if (t <= prev_offset) {
    return prev_color;
  }
  for (var i = 1u; i < 8u; i = i + 1u) {
    if (i >= n) {
      break;
    }
    let off = paint_stop_offset(p, i);
    let c = p.stop_colors[i];
    if (t <= off) {
      let denom = max(off - prev_offset, 1e-6);
      let u = saturate((t - prev_offset) / denom);
      return paint_mix_colorspace(p, prev_color, c, u);
    }
    prev_offset = off;
    prev_color = c;
  }
  return prev_color;
}

fn mat_hash_u32(x: u32) -> u32 {
  var v = x;
  v = v ^ (v >> 16u);
  v = v * 0x7feb352du;
  v = v ^ (v >> 15u);
  v = v * 0x846ca68bu;
  v = v ^ (v >> 16u);
  return v;
}

fn mat_hash2(p: vec2<u32>, seed: u32) -> u32 {
  let h = p.x ^ (p.y * 0x9e3779b9u) ^ (seed * 0x85ebca6bu);
  return mat_hash_u32(h);
}

fn mat_rand01(p: vec2<u32>, seed: u32) -> f32 {
  let h = mat_hash2(p, seed);
  return f32(h) * (1.0 / 4294967295.0);
}

fn mat_rot(v: vec2<f32>, a: f32) -> vec2<f32> {
  let s = sin(a);
  let c = cos(a);
  return vec2<f32>(c * v.x - s * v.y, s * v.x + c * v.y);
}

fn material_eval(p: Paint, local_pos: vec2<f32>, sample_catalog: bool) -> vec4<f32> {
  let base = p.params0;
  let fg = p.params1;
  let pos = local_pos + p.params3.zw;

  // params2: primary (x/y), thickness/radius (z), seed (w)
  // params3: time/phase (x), angle/softness (y), offset (z/w)
  let spacing = max(p.params2.x, 1.0);
  let spacing_y = max(p.params2.y, 1.0);
  let thickness = max(p.params2.z, 0.0);
  let seed = u32(max(p.params2.w, 0.0));
  let time = p.params3.x;
  let angle = p.params3.y;

  let tm0 = p.tile_mode == 0u;
  let tm1 = p.tile_mode == 1u;
  let tm2 = p.tile_mode == 2u;
  let tm3 = p.tile_mode == 3u;
  let tm4 = p.tile_mode == 4u;
  let tm5 = p.tile_mode == 5u;
  let tm6 = p.tile_mode == 6u;
  let tm7 = p.tile_mode == 7u;

  // 0 DotGrid
  let dot_cell = pos / spacing;
  let dot_frac = fract(dot_cell) - vec2<f32>(0.5);
  let dot_r = select(spacing * 0.12, thickness, thickness > 0.0);
  let dot_d = length(dot_frac) * spacing;
  let dot_aa = max(fwidth(dot_d), 1e-4);
  let dot_cov = 1.0 - smoothstep(dot_r, dot_r + dot_aa, dot_d);
  let mat0 = base * (1.0 - dot_cov) + fg * dot_cov;

  // 1 Grid
  let grid_cell = pos / vec2<f32>(spacing, spacing_y);
  let grid_frac = abs(fract(grid_cell) - vec2<f32>(0.5));
  let grid_dx = grid_frac.x * spacing;
  let grid_dy = grid_frac.y * spacing_y;
  let grid_w = select(1.0, thickness, thickness > 0.0);
  let grid_aa_x = max(fwidth(grid_dx), 1e-4);
  let grid_aa_y = max(fwidth(grid_dy), 1e-4);
  let grid_cov_x = 1.0 - smoothstep(grid_w * 0.5, grid_w * 0.5 + grid_aa_x, grid_dx);
  let grid_cov_y = 1.0 - smoothstep(grid_w * 0.5, grid_w * 0.5 + grid_aa_y, grid_dy);
  let grid_cov = max(grid_cov_x, grid_cov_y);
  let mat1 = base * (1.0 - grid_cov) + fg * grid_cov;

  // 2 Checkerboard
  let chk_cell = vec2<u32>(
    u32(floor(pos.x / spacing)),
    u32(floor(pos.y / spacing_y))
  );
  let chk_parity = (chk_cell.x + chk_cell.y) & 1u;
  let mat2 = select(base, fg, chk_parity == 1u);

  // 3 Stripe
  let stripe_p2 = mat_rot(pos, angle);
  let stripe_u = stripe_p2.x / spacing;
  let stripe_du = abs(fract(stripe_u) - 0.5) * spacing;
  let stripe_w = select(spacing * 0.25, thickness, thickness > 0.0);
  let stripe_aa = max(fwidth(stripe_du), 1e-4);
  let stripe_cov = 1.0 - smoothstep(stripe_w * 0.5, stripe_w * 0.5 + stripe_aa, stripe_du);
  let mat3 = base * (1.0 - stripe_cov) + fg * stripe_cov;

  // 4 Noise (deterministic cell noise; optionally sampled from a renderer-owned catalog texture)
  let noise_scale = spacing;
  let noise_cell = vec2<u32>(
    u32(floor(pos.x / noise_scale + 0.5)),
    u32(floor(pos.y / noise_scale + 0.5))
  );
  let noise_r0 = mat_rand01(noise_cell, seed);
  var noise_r = noise_r0;
  if (sample_catalog) {
    let noise_xi = i32(noise_cell.x & 63u);
    let noise_yi = i32(noise_cell.y & 63u);
    let noise_layer = clamp(i32(p.color_space), 0, 1);
    noise_r = textureLoad(
      material_catalog_texture,
      vec2<i32>(noise_xi, noise_yi),
      noise_layer,
      0
    ).r;
  }
  let noise_intensity = clamp(p.params2.y, 0.0, 1.0);
  let noise_cov = noise_intensity * noise_r;
  let mat4 = base * (1.0 - noise_cov) + fg * noise_cov;

  // 5 Beam (caller-driven phase via `time`)
  let beam_p2 = mat_rot(pos, angle);
  let beam_u = beam_p2.x;
  let beam_center = time;
  let beam_width = max(p.params2.x, 1.0);
  let beam_soft = max(p.params2.y, 0.0);
  let beam_d = abs(beam_u - beam_center);
  let beam_aa = max(fwidth(beam_d), 1e-4);
  let beam_cov = 1.0 - smoothstep(beam_width * 0.5, beam_width * 0.5 + beam_soft + beam_aa, beam_d);
  let mat5 = base * (1.0 - beam_cov) + fg * beam_cov;

  // 6 Sparkle (cell-based, explicit `time`, explicit `seed`)
  let sp_cell_size = max(p.params2.x, 1.0);
  let sp_cell = vec2<u32>(
    u32(floor(pos.x / sp_cell_size)),
    u32(floor(pos.y / sp_cell_size))
  );
  let sp_r0 = mat_rand01(sp_cell, seed);
  let sp_density = clamp(p.params2.y, 0.0, 1.0);
  let sp_enabled = sp_r0 <= sp_density;
  let sp_rx = mat_rand01(sp_cell, seed ^ 0x68bc21ebu);
  let sp_ry = mat_rand01(sp_cell, seed ^ 0x02e5be93u);
  let sp_phase = mat_rand01(sp_cell, seed ^ 0xa1b3c5d7u) * 6.2831853;
  let sp_p_cell = (fract(pos / sp_cell_size) - vec2<f32>(sp_rx, sp_ry)) * sp_cell_size;
  let sp_radius = select(sp_cell_size * 0.08, thickness, thickness > 0.0);
  let sp_d = length(sp_p_cell);
  let sp_aa = max(fwidth(sp_d), 1e-4);
  let sp_cov = 1.0 - smoothstep(sp_radius, sp_radius + sp_aa, sp_d);
  let sp_twinkle = 0.5 + 0.5 * sin(time * 2.0 + sp_phase);
  let sp_k = sp_cov * sp_twinkle;
  let sp_out = base * (1.0 - sp_k) + fg * sp_k;
  let mat6 = select(base, sp_out, sp_enabled);

  // 7 ConicSweep (center in params2.xy, width in params2.z (turns), phase in params3.x (turns))
  let con_center = p.params2.xy;
  let con_v = local_pos - con_center;
  let con_a = atan2(con_v.y, con_v.x);
  let con_turns = fract(con_a * (1.0 / 6.2831853) + fract(p.params3.x));
  let con_d = abs(fract(con_turns + 0.5) - 0.5);
  let con_w = clamp(p.params2.z, 0.0, 0.5);
  let con_soft = max(p.params3.y, 0.0);
  let con_aa = max(fwidth(con_d), 1e-4);
  let con_cov = 1.0 - smoothstep(con_w, con_w + con_soft + con_aa, con_d);
  let mat7 = base * (1.0 - con_cov) + fg * con_cov;

  var material = base;
  material = select(material, mat0, tm0);
  material = select(material, mat1, tm1);
  material = select(material, mat2, tm2);
  material = select(material, mat3, tm3);
  material = select(material, mat4, tm4);
  material = select(material, mat5, tm5);
  material = select(material, mat6, tm6);
  material = select(material, mat7, tm7);
  return material;
}

fn paint_eval(p: Paint, local_pos: vec2<f32>, pixel_pos: vec2<f32>) -> vec4<f32> {
  // WebGPU/WGSL constraint: derivative ops (fwidth/dpdx/dpdy) must only be used from uniform control flow.
  // Because `p.kind` is per-instance (not uniform), we avoid control-flow branching on it here and instead
  // compute candidate fills eagerly and select the final result.
  let pos = select(local_pos, pixel_pos, p.eval_space == 1u);
  let is_solid = p.kind == 0u;
  let is_linear = p.kind == 1u;
  let is_radial = p.kind == 2u;
  let is_material = p.kind == 3u;
  let is_conic = p.kind == 4u;

  let solid = p.params0;

  let start = p.params0.xy;
  let end = p.params0.zw;
  let dir = end - start;
  let len2 = dot(dir, dir);
  let lin_denom = max(len2, 1e-6);
  let lin_t0 = dot(pos - start, dir) / lin_denom;
  let lin_t = select(0.0, lin_t0, len2 > 1e-6);
  let linear = paint_sample_stops(p, gradient_tile_mode_apply(lin_t, p.tile_mode));

  let radial_center = p.params0.xy;
  let radial_radius = max(p.params0.zw, vec2<f32>(1e-6));
  let radial_d = (pos - radial_center) / radial_radius;
  let radial_t = length(radial_d);
  let radial = paint_sample_stops(p, gradient_tile_mode_apply(radial_t, p.tile_mode));

  let conic_center = p.params0.xy;
  let conic_start = p.params0.z;
  let conic_span = max(p.params0.w, 1e-6);
  let conic_v = pos - conic_center;
  let conic_a = atan2(conic_v.y, conic_v.x);
  let conic_turns = fract(conic_a * (1.0 / 6.2831853) + 1.0);
  let conic_rel = fract(conic_turns - fract(conic_start) + 1.0);
  let conic_t = conic_rel / conic_span;
  let conic = paint_sample_stops(p, gradient_tile_mode_apply(conic_t, p.tile_mode));

  let material_sampled = is_material && (p.stop_count != 0u);
  let material = material_eval(p, pos, material_sampled);

  var out = vec4<f32>(0.0);
  out = select(out, solid, is_solid);
  out = select(out, linear, is_linear);
  out = select(out, radial, is_radial);
  out = select(out, material, is_material);
  out = select(out, conic, is_conic);
  return out;
}

struct VsIn {
  @location(0) pos_px: vec2<f32>,
  @location(1) local_pos_px: vec2<f32>,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) pixel_pos: vec2<f32>,
  @location(1) local_pos_px: vec2<f32>,
  @location(2) @interpolate(flat) paint_index: u32,
};

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let local = pixel_pos - render_space.origin_px;
  let ndc_x = (local.x / render_space.size_px.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (local.y / render_space.size_px.y) * 2.0;
  return vec2<f32>(ndc_x, ndc_y);
}

fn pick_corner_radius(center_to_point: vec2<f32>, radii: vec4<f32>) -> f32 {
  if (center_to_point.x < 0.0) {
    if (center_to_point.y < 0.0) { return radii.x; }
    return radii.w;
  }
  if (center_to_point.y < 0.0) { return radii.y; }
  return radii.z;
}

fn quad_sdf_impl(corner_center_to_point: vec2<f32>, corner_radius: f32) -> f32 {
  if (corner_radius == 0.0) {
    return max(corner_center_to_point.x, corner_center_to_point.y);
  }
  let signed_distance_to_inset_quad =
    length(max(vec2<f32>(0.0), corner_center_to_point)) +
    min(0.0, max(corner_center_to_point.x, corner_center_to_point.y));
  return signed_distance_to_inset_quad - corner_radius;
}

fn quad_sdf(point: vec2<f32>, rect_origin: vec2<f32>, rect_size: vec2<f32>, corner_radii: vec4<f32>) -> f32 {
  let center = rect_origin + rect_size * 0.5;
  let center_to_point = point - center;
  let half_size = rect_size * 0.5;
  let corner_radius = pick_corner_radius(center_to_point, corner_radii);
  let corner_to_point = abs(center_to_point) - half_size;
  let corner_center_to_point = corner_to_point + corner_radius;
  return quad_sdf_impl(corner_center_to_point, corner_radius);
}

fn clip_alpha(pixel_pos: vec2<f32>) -> f32 {
  var alpha = 1.0;
  var idx = viewport.clip_head;
  for (var i = 0u; i < 64u; i = i + 1u) {
    if (i >= viewport.clip_count) {
      break;
    }
    if (idx == 0xffffffffu) {
      break;
    }
    let clip = clip_stack.clips[idx];
    idx = bitcast<u32>(clip.inv0.w);
    let clip_local = vec2<f32>(
      dot(clip.inv0.xy, pixel_pos) + clip.inv0.z,
      dot(clip.inv1.xy, pixel_pos) + clip.inv1.z
    );
    let sdf = quad_sdf(clip_local, clip.rect.xy, clip.rect.zw, clip.corner_radii);
    let aa = max(fwidth(sdf), 1e-4);
    let a = 1.0 - smoothstep(-aa, aa, sdf);
    alpha = alpha * a;
  }
  return alpha;
}

fn saturate(x: f32) -> f32 {
  return clamp(x, 0.0, 1.0);
}

fn gradient_tile_mode_apply(t: f32, tile_mode: u32) -> f32 {
  if (tile_mode == 1u) {
    return fract(t);
  }
  if (tile_mode == 2u) {
    let seg = floor(t);
    let r = fract(t);
    let odd = (i32(seg) & 1) != 0;
    return select(r, 1.0 - r, odd);
  }
  return clamp(t, 0.0, 1.0);
}

fn mask_stop_offset(m: MaskGradient, i: u32) -> f32 {
  if (i < 4u) { return m.stop_offsets0[i]; }
  return m.stop_offsets1[i - 4u];
}

fn mask_stop_alpha(m: MaskGradient, i: u32) -> f32 {
  if (i < 4u) { return m.stop_alphas0[i]; }
  return m.stop_alphas1[i - 4u];
}

fn mask_sample_stops(m: MaskGradient, t: f32) -> f32 {
  let n = min(m.stop_count, 8u);
  if (n == 0u) { return 1.0; }

  var prev_offset = mask_stop_offset(m, 0u);
  var prev_alpha = mask_stop_alpha(m, 0u);
  if (n == 1u || t <= prev_offset) {
    return prev_alpha;
  }

  for (var i = 1u; i < 8u; i = i + 1u) {
    if (i >= n) {
      break;
    }
    let off = mask_stop_offset(m, i);
    let a = mask_stop_alpha(m, i);
    if (t <= off) {
      let denom = max(off - prev_offset, 1e-6);
      let u = saturate((t - prev_offset) / denom);
      return mix(prev_alpha, a, u);
    }
    prev_offset = off;
    prev_alpha = a;
  }
  return prev_alpha;
}

fn mask_image_sample_bilinear_clamp(uv: vec2<f32>) -> vec4<f32> {
  let dims_u = textureDimensions(mask_image_texture);
  if (dims_u.x == 0u || dims_u.y == 0u) {
    return vec4<f32>(0.0);
  }

  let dims = vec2<f32>(f32(dims_u.x), f32(dims_u.y));
  let p_px = uv * dims;

  // Manual bilinear sampling avoids WGSL uniformity restrictions on WebGPU.
  let max_p = vec2<f32>(dims.x - 0.5, dims.y - 0.5);
  let p = clamp(p_px, vec2<f32>(0.5), max_p);

  let t = p - vec2<f32>(0.5);
  let base_f = floor(t);
  let f = fract(t);

  let x0 = clamp(i32(base_f.x), 0, i32(dims_u.x) - 1);
  let y0 = clamp(i32(base_f.y), 0, i32(dims_u.y) - 1);
  let x1 = min(x0 + 1, i32(dims_u.x) - 1);
  let y1 = min(y0 + 1, i32(dims_u.y) - 1);

  let c00 = textureLoad(mask_image_texture, vec2<i32>(x0, y0), 0);
  let c10 = textureLoad(mask_image_texture, vec2<i32>(x1, y0), 0);
  let c01 = textureLoad(mask_image_texture, vec2<i32>(x0, y1), 0);
  let c11 = textureLoad(mask_image_texture, vec2<i32>(x1, y1), 0);

  let cx0 = mix(c00, c10, f.x);
  let cx1 = mix(c01, c11, f.x);
  return mix(cx0, cx1, f.y);
}

fn mask_eval(m: MaskGradient, pixel_pos: vec2<f32>) -> f32 {
  let local_pos = vec2<f32>(
    dot(m.inv0.xy, pixel_pos) + m.inv0.z,
    dot(m.inv1.xy, pixel_pos) + m.inv1.z
  );

  let p = local_pos - m.bounds.xy;
  let in_bounds = p.x >= 0.0 && p.y >= 0.0 && p.x <= m.bounds.z && p.y <= m.bounds.w;

  if (m.kind == 1u) {
    let start = m.params0.xy;
    let end = m.params0.zw;
    let dir = end - start;
    let len2 = dot(dir, dir);
    let t = select(0.0, dot(local_pos - start, dir) / len2, len2 > 1e-6);
    let tt = gradient_tile_mode_apply(t, m.tile_mode);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 2u) {
    let center = m.params0.xy;
    let radius = max(m.params0.zw, vec2<f32>(1e-6));
    let d = (local_pos - center) / radius;
    let t = length(d);
    let tt = gradient_tile_mode_apply(t, m.tile_mode);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 3u) {
    let uv0 = m.params0.xy;
    let uv1 = m.params0.zw;
    let denom = max(m.bounds.zw, vec2<f32>(1e-6));
    let t = clamp(p / denom, vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = mix(uv0, uv1, t);
    let s = mask_image_sample_bilinear_clamp(uv);
    let cov = select(s.r, s.a, m.tile_mode == 1u);
    return select(1.0, clamp(cov, 0.0, 1.0), in_bounds);
  }

  return 1.0;
}

fn mask_alpha(pixel_pos: vec2<f32>) -> f32 {
  var alpha = 1.0;
  var idx = viewport.mask_head;
  for (var i = 0u; i < 64u; i = i + 1u) {
    if (i >= viewport.mask_count) {
      break;
    }
    if (viewport.mask_scope_count != 0u && idx == viewport.mask_scope_head) {
      break;
    }
    if (idx == 0xffffffffu) {
      break;
    }
    let m = mask_stack.masks[idx];
    idx = bitcast<u32>(m.inv0.w);
    alpha = alpha * clamp(mask_eval(m, pixel_pos), 0.0, 1.0);
  }
  return alpha;
}

fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
  let a = 0.055;
  let lo = rgb * 12.92;
  let hi = (1.0 + a) * pow(rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(a);
  return select(hi, lo, rgb <= vec3<f32>(0.0031308));
}

fn encode_output_premul(c: vec4<f32>) -> vec4<f32> {
  return c;
}

@vertex
fn vs_main(input: VsIn, @builtin(instance_index) instance_index: u32) -> VsOut {
  var out: VsOut;
  let clip_xy = to_clip_space(input.pos_px);
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.pixel_pos = input.pos_px;
  out.local_pos_px = input.local_pos_px;
  out.paint_index = instance_index;
  return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let clip = clip_alpha(input.pixel_pos);
  let mask = mask_alpha(input.pixel_pos);
  let paint = path_paints.paints[input.paint_index];
  let fill = paint_eval(paint, input.local_pos_px, input.pixel_pos);
  let out = fill * clip * mask;
  return encode_output_premul(out);
}
"#;

pub(super) const TEXT_SHADER: &str = r#"
struct ClipRRect {
  rect: vec4<f32>,
  corner_radii: vec4<f32>,
  inv0: vec4<f32>,
  inv1: vec4<f32>,
};

struct Viewport {
  viewport_size: vec2<f32>,
  clip_head: u32,
  clip_count: u32,
  mask_head: u32,
  mask_count: u32,
  mask_scope_head: u32,
  mask_scope_count: u32,
  output_is_srgb: u32,
  _pad0: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
  text_gamma_ratios: vec4<f32>,
  text_grayscale_enhanced_contrast: f32,
  text_subpixel_enhanced_contrast: f32,
  _pad_text_quality0: u32,
  _pad_text_quality1: u32,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;

struct RenderSpace {
  origin_px: vec2<f32>,
  size_px: vec2<f32>,
};

@group(0) @binding(5) var<uniform> render_space: RenderSpace;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

struct MaskGradient {
  bounds: vec4<f32>,
  kind: u32,
  tile_mode: u32,
  stop_count: u32,
  _pad0: u32,
  params0: vec4<f32>,
  inv0: vec4<f32>,
  inv1: vec4<f32>,
  stop_alphas0: vec4<f32>,
  stop_alphas1: vec4<f32>,
  stop_offsets0: vec4<f32>,
  stop_offsets1: vec4<f32>,
};

struct MaskStack {
  masks: array<MaskGradient>,
};

@group(0) @binding(2) var<storage, read> mask_stack: MaskStack;

@group(0) @binding(6) var mask_image_sampler: sampler;
@group(0) @binding(7) var mask_image_texture: texture_2d<f32>;

@group(1) @binding(0) var glyph_sampler: sampler;
@group(1) @binding(1) var glyph_atlas: texture_2d<f32>;

const MAX_STOPS: u32 = 8u;

const FRET_TEXT_OUTLINE_PRESENT: u32 = 0u;

struct Paint {
  kind: u32,
  tile_mode: u32,
  color_space: u32,
  stop_count: u32,
  eval_space: u32,
  _pad_eval0: u32,
  _pad_eval1: u32,
  _pad_eval2: u32,
  params0: vec4<f32>,
  params1: vec4<f32>,
  params2: vec4<f32>,
  params3: vec4<f32>,
  stop_colors: array<vec4<f32>, 8>,
  stop_offsets0: vec4<f32>,
  stop_offsets1: vec4<f32>,
};

struct TextPaints {
  paints: array<Paint>,
};

@group(2) @binding(0) var<storage, read> text_paints: TextPaints;

struct VsIn {
  @location(0) pos_px: vec2<f32>,
  @location(1) local_pos_px: vec2<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) outline_params: u32,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) pixel_pos: vec2<f32>,
  @location(3) local_pos_px: vec2<f32>,
  @location(4) @interpolate(flat) paint_index: u32,
  @location(5) @interpolate(flat) outline_params: u32,
};

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let local = pixel_pos - render_space.origin_px;
  let ndc_x = (local.x / render_space.size_px.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (local.y / render_space.size_px.y) * 2.0;
  return vec2<f32>(ndc_x, ndc_y);
}

fn pick_corner_radius(center_to_point: vec2<f32>, radii: vec4<f32>) -> f32 {
  if (center_to_point.x < 0.0) {
    if (center_to_point.y < 0.0) { return radii.x; }
    return radii.w;
  }
  if (center_to_point.y < 0.0) { return radii.y; }
  return radii.z;
}

fn quad_sdf_impl(corner_center_to_point: vec2<f32>, corner_radius: f32) -> f32 {
  if (corner_radius == 0.0) {
    return max(corner_center_to_point.x, corner_center_to_point.y);
  }
  let signed_distance_to_inset_quad =
    length(max(vec2<f32>(0.0), corner_center_to_point)) +
    min(0.0, max(corner_center_to_point.x, corner_center_to_point.y));
  return signed_distance_to_inset_quad - corner_radius;
}

fn quad_sdf(point: vec2<f32>, rect_origin: vec2<f32>, rect_size: vec2<f32>, corner_radii: vec4<f32>) -> f32 {
  let center = rect_origin + rect_size * 0.5;
  let center_to_point = point - center;
  let half_size = rect_size * 0.5;
  let corner_radius = pick_corner_radius(center_to_point, corner_radii);
  let corner_to_point = abs(center_to_point) - half_size;
  let corner_center_to_point = corner_to_point + corner_radius;
  return quad_sdf_impl(corner_center_to_point, corner_radius);
}

fn clip_alpha(pixel_pos: vec2<f32>) -> f32 {
  var alpha = 1.0;
  var idx = viewport.clip_head;
  for (var i = 0u; i < 64u; i = i + 1u) {
    if (i >= viewport.clip_count) {
      break;
    }
    if (idx == 0xffffffffu) {
      break;
    }
    let clip = clip_stack.clips[idx];
    idx = bitcast<u32>(clip.inv0.w);
    let clip_local = vec2<f32>(
      dot(clip.inv0.xy, pixel_pos) + clip.inv0.z,
      dot(clip.inv1.xy, pixel_pos) + clip.inv1.z
    );
    let sdf = quad_sdf(clip_local, clip.rect.xy, clip.rect.zw, clip.corner_radii);
    let aa = max(fwidth(sdf), 1e-4);
    let a = 1.0 - smoothstep(-aa, aa, sdf);
    alpha = alpha * a;
  }
  return alpha;
}

fn saturate(x: f32) -> f32 {
  return clamp(x, 0.0, 1.0);
}

fn gradient_tile_mode_apply(t: f32, tile_mode: u32) -> f32 {
  if (tile_mode == 1u) {
    return fract(t);
  }
  if (tile_mode == 2u) {
    let seg = floor(t);
    let r = fract(t);
    let odd = (i32(seg) & 1) != 0;
    return select(r, 1.0 - r, odd);
  }
  return clamp(t, 0.0, 1.0);
}

fn paint_stop_offset(p: Paint, i: u32) -> f32 {
  if (i < 4u) {
    return p.stop_offsets0[i];
  }
  return p.stop_offsets1[i - 4u];
}

fn paint_unpremul_rgb(c: vec4<f32>) -> vec3<f32> {
  let a = max(c.a, 1e-6);
  return c.rgb / a;
}

fn linear_rgb_to_oklab(rgb: vec3<f32>) -> vec3<f32> {
  let c = max(rgb, vec3<f32>(0.0));
  let lms = vec3<f32>(
    0.4122214708 * c.x + 0.5363325363 * c.y + 0.0514459929 * c.z,
    0.2119034982 * c.x + 0.6806995451 * c.y + 0.1073969566 * c.z,
    0.0883024619 * c.x + 0.2817188376 * c.y + 0.6299787005 * c.z
  );
  let lms_cbrt = pow(max(lms, vec3<f32>(0.0)), vec3<f32>(1.0 / 3.0));
  return vec3<f32>(
    0.2104542553 * lms_cbrt.x + 0.7936177850 * lms_cbrt.y - 0.0040720468 * lms_cbrt.z,
    1.9779984951 * lms_cbrt.x - 2.4285922050 * lms_cbrt.y + 0.4505937099 * lms_cbrt.z,
    0.0259040371 * lms_cbrt.x + 0.7827717662 * lms_cbrt.y - 0.8086757660 * lms_cbrt.z
  );
}

fn oklab_to_linear_rgb(lab: vec3<f32>) -> vec3<f32> {
  let lms_cbrt = vec3<f32>(
    lab.x + 0.3963377774 * lab.y + 0.2158037573 * lab.z,
    lab.x - 0.1055613458 * lab.y - 0.0638541728 * lab.z,
    lab.x - 0.0894841775 * lab.y - 1.2914855480 * lab.z
  );
  let lms = lms_cbrt * lms_cbrt * lms_cbrt;
  return vec3<f32>(
    4.0767416621 * lms.x - 3.3077115913 * lms.y + 0.2309699292 * lms.z,
    -1.2684380046 * lms.x + 2.6097574011 * lms.y - 0.3413193965 * lms.z,
    -0.0041960863 * lms.x - 0.7034186147 * lms.y + 1.7076147010 * lms.z
  );
}

fn paint_mix_colorspace(p: Paint, a: vec4<f32>, b: vec4<f32>, u: f32) -> vec4<f32> {
  if (p.color_space == 1u) {
    let a0 = clamp(a.a, 0.0, 1.0);
    let a1 = clamp(b.a, 0.0, 1.0);
    let alpha = clamp(mix(a0, a1, u), 0.0, 1.0);

    let rgb0 = clamp(paint_unpremul_rgb(a), vec3<f32>(0.0), vec3<f32>(1.0));
    let rgb1 = clamp(paint_unpremul_rgb(b), vec3<f32>(0.0), vec3<f32>(1.0));
    let lab0 = linear_rgb_to_oklab(rgb0);
    let lab1 = linear_rgb_to_oklab(rgb1);
    let lab = mix(lab0, lab1, u);
    let rgb = clamp(oklab_to_linear_rgb(lab), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(rgb * alpha, alpha);
  }
  return mix(a, b, u);
}

fn paint_sample_stops(p: Paint, t: f32) -> vec4<f32> {
  let n = min(p.stop_count, MAX_STOPS);
  if (n == 0u) {
    return vec4<f32>(0.0);
  }

  var prev_offset = paint_stop_offset(p, 0u);
  var prev_color = p.stop_colors[0u];
  if (n == 1u || t <= prev_offset) {
    return prev_color;
  }

  for (var i = 1u; i < 8u; i = i + 1u) {
    if (i >= n) {
      break;
    }
    let off = paint_stop_offset(p, i);
    let col = p.stop_colors[i];
    if (t <= off) {
      let denom = max(off - prev_offset, 1e-6);
      let u = saturate((t - prev_offset) / denom);
      return paint_mix_colorspace(p, prev_color, col, u);
    }
    prev_offset = off;
    prev_color = col;
  }
  return prev_color;
}

fn paint_eval(p: Paint, local_pos: vec2<f32>, pixel_pos: vec2<f32>) -> vec4<f32> {
  let pos = select(local_pos, pixel_pos, p.eval_space == 1u);
  if (p.kind == 0u) {
    return p.params0;
  }
  if (p.kind == 1u) {
    let start = p.params0.xy;
    let end = p.params0.zw;
    let dir = end - start;
    let len2 = dot(dir, dir);
    let t = select(0.0, dot(pos - start, dir) / len2, len2 > 1e-6);
    let tt = gradient_tile_mode_apply(t, p.tile_mode);
    return paint_sample_stops(p, tt);
  }
  if (p.kind == 2u) {
    let center = p.params0.xy;
    let radius = max(p.params0.zw, vec2<f32>(1e-6));
    let d = (pos - center) / radius;
    let t = length(d);
    let tt = gradient_tile_mode_apply(t, p.tile_mode);
    return paint_sample_stops(p, tt);
  }
  if (p.kind == 4u) {
    let center = p.params0.xy;
    let start = p.params0.z;
    let span = max(p.params0.w, 1e-6);
    let v = pos - center;
    let a = atan2(v.y, v.x);
    let turns = fract(a * (1.0 / 6.2831853) + 1.0);
    let rel = fract(turns - fract(start) + 1.0);
    let t = rel / span;
    let tt = gradient_tile_mode_apply(t, p.tile_mode);
    return paint_sample_stops(p, tt);
  }
  return vec4<f32>(0.0);
}

fn mask_stop_offset(m: MaskGradient, i: u32) -> f32 {
  if (i < 4u) { return m.stop_offsets0[i]; }
  return m.stop_offsets1[i - 4u];
}

fn mask_stop_alpha(m: MaskGradient, i: u32) -> f32 {
  if (i < 4u) { return m.stop_alphas0[i]; }
  return m.stop_alphas1[i - 4u];
}

fn mask_sample_stops(m: MaskGradient, t: f32) -> f32 {
  let n = min(m.stop_count, 8u);
  if (n == 0u) { return 1.0; }

  var prev_offset = mask_stop_offset(m, 0u);
  var prev_alpha = mask_stop_alpha(m, 0u);
  if (n == 1u || t <= prev_offset) {
    return prev_alpha;
  }

  for (var i = 1u; i < 8u; i = i + 1u) {
    if (i >= n) {
      break;
    }
    let off = mask_stop_offset(m, i);
    let a = mask_stop_alpha(m, i);
    if (t <= off) {
      let denom = max(off - prev_offset, 1e-6);
      let u = saturate((t - prev_offset) / denom);
      return mix(prev_alpha, a, u);
    }
    prev_offset = off;
    prev_alpha = a;
  }
  return prev_alpha;
}

fn mask_image_sample_bilinear_clamp(uv: vec2<f32>) -> vec4<f32> {
  let dims_u = textureDimensions(mask_image_texture);
  if (dims_u.x == 0u || dims_u.y == 0u) {
    return vec4<f32>(0.0);
  }

  let dims = vec2<f32>(f32(dims_u.x), f32(dims_u.y));
  let p_px = uv * dims;

  // Manual bilinear sampling avoids WGSL uniformity restrictions on WebGPU.
  let max_p = vec2<f32>(dims.x - 0.5, dims.y - 0.5);
  let p = clamp(p_px, vec2<f32>(0.5), max_p);

  let t = p - vec2<f32>(0.5);
  let base_f = floor(t);
  let f = fract(t);

  let x0 = clamp(i32(base_f.x), 0, i32(dims_u.x) - 1);
  let y0 = clamp(i32(base_f.y), 0, i32(dims_u.y) - 1);
  let x1 = min(x0 + 1, i32(dims_u.x) - 1);
  let y1 = min(y0 + 1, i32(dims_u.y) - 1);

  let c00 = textureLoad(mask_image_texture, vec2<i32>(x0, y0), 0);
  let c10 = textureLoad(mask_image_texture, vec2<i32>(x1, y0), 0);
  let c01 = textureLoad(mask_image_texture, vec2<i32>(x0, y1), 0);
  let c11 = textureLoad(mask_image_texture, vec2<i32>(x1, y1), 0);

  let cx0 = mix(c00, c10, f.x);
  let cx1 = mix(c01, c11, f.x);
  return mix(cx0, cx1, f.y);
}

fn mask_eval(m: MaskGradient, pixel_pos: vec2<f32>) -> f32 {
  let local_pos = vec2<f32>(
    dot(m.inv0.xy, pixel_pos) + m.inv0.z,
    dot(m.inv1.xy, pixel_pos) + m.inv1.z
  );

  let p = local_pos - m.bounds.xy;
  let in_bounds = p.x >= 0.0 && p.y >= 0.0 && p.x <= m.bounds.z && p.y <= m.bounds.w;

  if (m.kind == 1u) {
    let start = m.params0.xy;
    let end = m.params0.zw;
    let dir = end - start;
    let len2 = dot(dir, dir);
    let t = select(0.0, dot(local_pos - start, dir) / len2, len2 > 1e-6);
    let tt = gradient_tile_mode_apply(t, m.tile_mode);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 2u) {
    let center = m.params0.xy;
    let radius = max(m.params0.zw, vec2<f32>(1e-6));
    let d = (local_pos - center) / radius;
    let t = length(d);
    let tt = gradient_tile_mode_apply(t, m.tile_mode);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 3u) {
    let uv0 = m.params0.xy;
    let uv1 = m.params0.zw;
    let denom = max(m.bounds.zw, vec2<f32>(1e-6));
    let t = clamp(p / denom, vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = mix(uv0, uv1, t);
    let s = mask_image_sample_bilinear_clamp(uv);
    let cov = select(s.r, s.a, m.tile_mode == 1u);
    return select(1.0, clamp(cov, 0.0, 1.0), in_bounds);
  }

  return 1.0;
}

fn mask_alpha(pixel_pos: vec2<f32>) -> f32 {
  var alpha = 1.0;
  var idx = viewport.mask_head;
  for (var i = 0u; i < 64u; i = i + 1u) {
    if (i >= viewport.mask_count) {
      break;
    }
    if (viewport.mask_scope_count != 0u && idx == viewport.mask_scope_head) {
      break;
    }
    if (idx == 0xffffffffu) {
      break;
    }
    let m = mask_stack.masks[idx];
    idx = bitcast<u32>(m.inv0.w);
    alpha = alpha * clamp(mask_eval(m, pixel_pos), 0.0, 1.0);
  }
  return alpha;
}

fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
  let a = 0.055;
  let lo = rgb * 12.92;
  let hi = (1.0 + a) * pow(rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(a);
  return select(hi, lo, rgb <= vec3<f32>(0.0031308));
}

// Contrast and gamma correction adapted from the Microsoft Terminal alpha correction work
// (via Zed/GPUI). See ADR 0029/0107/0142.
fn color_brightness(color: vec3<f32>) -> f32 {
  // REC. 601 luminance coefficients for perceived brightness.
  return dot(color, vec3<f32>(0.30, 0.59, 0.11));
}

fn light_on_dark_contrast(enhanced_contrast: f32, color: vec3<f32>) -> f32 {
  let brightness = color_brightness(color);
  let multiplier = clamp(4.0 * (0.75 - brightness), 0.0, 1.0);
  return enhanced_contrast * multiplier;
}

fn enhance_contrast(alpha: f32, k: f32) -> f32 {
  return alpha * (k + 1.0) / (alpha * k + 1.0);
}

fn apply_alpha_correction(alpha: f32, brightness: f32, g: vec4<f32>) -> f32 {
  let brightness_adjustment = g.x * brightness + g.y;
  let correction = brightness_adjustment * alpha + (g.z * brightness + g.w);
  return alpha + alpha * (1.0 - alpha) * correction;
}

fn apply_contrast_and_gamma_correction(sample: f32, color: vec3<f32>) -> f32 {
  let k = light_on_dark_contrast(viewport.text_grayscale_enhanced_contrast, color);
  let contrasted = enhance_contrast(sample, k);
  let b = color_brightness(color);
  return apply_alpha_correction(contrasted, b, viewport.text_gamma_ratios);
}

fn encode_output_premul(c: vec4<f32>) -> vec4<f32> {
  return c;
}

@vertex
fn vs_main(input: VsIn, @builtin(instance_index) instance_index: u32) -> VsOut {
  var out: VsOut;
  let clip_xy = to_clip_space(input.pos_px);
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.uv = input.uv;
  out.color = input.color;
  out.pixel_pos = input.pos_px;
  out.local_pos_px = input.local_pos_px;
  out.paint_index = instance_index;
  out.outline_params = input.outline_params;
  return out;
}

fn glyph_sample_r(uv: vec2<f32>) -> f32 {
  return textureSample(glyph_atlas, glyph_sampler, uv).r;
}

fn glyph_dilate_r(uv: vec2<f32>, radius: u32) -> f32 {
  let dims_u = textureDimensions(glyph_atlas);
  let dims = vec2<f32>(f32(dims_u.x), f32(dims_u.y));
  let texel = vec2<f32>(1.0, 1.0) / dims;

  let s0 = glyph_sample_r(uv);

  let dx = vec2<f32>(texel.x, 0.0);
  let dy = vec2<f32>(0.0, texel.y);
  let d11 = vec2<f32>(texel.x, texel.y);
  let d1m1 = vec2<f32>(texel.x, -texel.y);

  let d1x = 1.0 * dx;
  let d1y = 1.0 * dy;
  let d1d0 = 1.0 * d11;
  let d1d1 = 1.0 * d1m1;
  let max1 = max(
    s0,
    max(
      max(
        max(glyph_sample_r(uv + d1x), glyph_sample_r(uv - d1x)),
        max(glyph_sample_r(uv + d1y), glyph_sample_r(uv - d1y))
      ),
      max(
        max(glyph_sample_r(uv + d1d0), glyph_sample_r(uv - d1d0)),
        max(glyph_sample_r(uv + d1d1), glyph_sample_r(uv - d1d1))
      )
    )
  );

  let d2x = 2.0 * dx;
  let d2y = 2.0 * dy;
  let d2d0 = 2.0 * d11;
  let d2d1 = 2.0 * d1m1;
  let max2 = max(
    max1,
    max(
      max(
        max(glyph_sample_r(uv + d2x), glyph_sample_r(uv - d2x)),
        max(glyph_sample_r(uv + d2y), glyph_sample_r(uv - d2y))
      ),
      max(
        max(glyph_sample_r(uv + d2d0), glyph_sample_r(uv - d2d0)),
        max(glyph_sample_r(uv + d2d1), glyph_sample_r(uv - d2d1))
      )
    )
  );

  let d3x = 3.0 * dx;
  let d3y = 3.0 * dy;
  let d3d0 = 3.0 * d11;
  let d3d1 = 3.0 * d1m1;
  let max3 = max(
    max2,
    max(
      max(
        max(glyph_sample_r(uv + d3x), glyph_sample_r(uv - d3x)),
        max(glyph_sample_r(uv + d3y), glyph_sample_r(uv - d3y))
      ),
      max(
        max(glyph_sample_r(uv + d3d0), glyph_sample_r(uv - d3d0)),
        max(glyph_sample_r(uv + d3d1), glyph_sample_r(uv - d3d1))
      )
    )
  );

  let m1 = select(0.0, 1.0, radius >= 1u);
  let m2 = select(0.0, 1.0, radius >= 2u);
  let m3 = select(0.0, 1.0, radius >= 3u);
  return max(s0, max(max1 * m1, max(max2 * m2, max3 * m3)));
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let clip = clip_alpha(input.pixel_pos);
  let mask = mask_alpha(input.pixel_pos);
  let tex = textureSample(glyph_atlas, glyph_sampler, input.uv);
  let fill_sample = tex.r;
  let p = text_paints.paints[input.paint_index];
  let base = paint_eval(p, input.local_pos_px, input.pixel_pos) * input.color;
  let base_un = select(vec3<f32>(0.0), base.rgb / base.a, base.a > 1e-6);
  let fill_cov = apply_contrast_and_gamma_correction(fill_sample, base_un);
  var out = vec4<f32>(base.rgb * fill_cov, base.a * fill_cov);

  if (FRET_TEXT_OUTLINE_PRESENT != 0u) {
    let outline_params = input.outline_params;
    let outline_radius = min(outline_params & 3u, 3u);
    let outline_enabled = select(0.0, 1.0, outline_radius != 0u);
    let outline_max_sample = glyph_dilate_r(input.uv, outline_radius);
    let ring_sample = saturate(outline_max_sample - fill_sample) * outline_enabled;
    let outline_paint_index = outline_params >> 2u;
    let op = text_paints.paints[outline_paint_index];
    let outline_mul = vec4<f32>(1.0, 1.0, 1.0, input.color.a);
    let outline_base = paint_eval(op, input.local_pos_px, input.pixel_pos) * outline_mul;
    let outline_un = select(
      vec3<f32>(0.0),
      outline_base.rgb / outline_base.a,
      outline_base.a > 1e-6
    );
    let ring_cov = apply_contrast_and_gamma_correction(ring_sample, outline_un);
    out = out + vec4<f32>(outline_base.rgb * ring_cov, outline_base.a * ring_cov);
  }

  out = out * clip * mask;
  return encode_output_premul(out);
}
"#;

pub(super) const TEXT_COLOR_SHADER: &str = r#"
struct ClipRRect {
  rect: vec4<f32>,
  corner_radii: vec4<f32>,
  inv0: vec4<f32>,
  inv1: vec4<f32>,
};

struct Viewport {
  viewport_size: vec2<f32>,
  clip_head: u32,
  clip_count: u32,
  mask_head: u32,
  mask_count: u32,
  mask_scope_head: u32,
  mask_scope_count: u32,
  output_is_srgb: u32,
  _pad0: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
  text_gamma_ratios: vec4<f32>,
  text_grayscale_enhanced_contrast: f32,
  text_subpixel_enhanced_contrast: f32,
  _pad_text_quality0: u32,
  _pad_text_quality1: u32,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;

struct RenderSpace {
  origin_px: vec2<f32>,
  size_px: vec2<f32>,
};

@group(0) @binding(5) var<uniform> render_space: RenderSpace;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

struct MaskGradient {
  bounds: vec4<f32>,
  kind: u32,
  tile_mode: u32,
  stop_count: u32,
  _pad0: u32,
  params0: vec4<f32>,
  inv0: vec4<f32>,
  inv1: vec4<f32>,
  stop_alphas0: vec4<f32>,
  stop_alphas1: vec4<f32>,
  stop_offsets0: vec4<f32>,
  stop_offsets1: vec4<f32>,
};

struct MaskStack {
  masks: array<MaskGradient>,
};

@group(0) @binding(2) var<storage, read> mask_stack: MaskStack;

@group(0) @binding(6) var mask_image_sampler: sampler;
@group(0) @binding(7) var mask_image_texture: texture_2d<f32>;

@group(1) @binding(0) var glyph_sampler: sampler;
@group(1) @binding(1) var glyph_atlas: texture_2d<f32>;

const MAX_STOPS: u32 = 8u;

struct Paint {
  kind: u32,
  tile_mode: u32,
  color_space: u32,
  stop_count: u32,
  eval_space: u32,
  _pad_eval0: u32,
  _pad_eval1: u32,
  _pad_eval2: u32,
  params0: vec4<f32>,
  params1: vec4<f32>,
  params2: vec4<f32>,
  params3: vec4<f32>,
  stop_colors: array<vec4<f32>, 8>,
  stop_offsets0: vec4<f32>,
  stop_offsets1: vec4<f32>,
};

struct TextPaints {
  paints: array<Paint>,
};

@group(2) @binding(0) var<storage, read> text_paints: TextPaints;

struct VsIn {
  @location(0) pos_px: vec2<f32>,
  @location(1) local_pos_px: vec2<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) outline_params: u32,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) pixel_pos: vec2<f32>,
  @location(3) local_pos_px: vec2<f32>,
  @location(4) @interpolate(flat) paint_index: u32,
  @location(5) @interpolate(flat) outline_params: u32,
};

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let local = pixel_pos - render_space.origin_px;
  let ndc_x = (local.x / render_space.size_px.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (local.y / render_space.size_px.y) * 2.0;
  return vec2<f32>(ndc_x, ndc_y);
}

fn pick_corner_radius(center_to_point: vec2<f32>, radii: vec4<f32>) -> f32 {
  if (center_to_point.x < 0.0) {
    if (center_to_point.y < 0.0) { return radii.x; }
    return radii.w;
  }
  if (center_to_point.y < 0.0) { return radii.y; }
  return radii.z;
}

fn quad_sdf_impl(corner_center_to_point: vec2<f32>, corner_radius: f32) -> f32 {
  if (corner_radius == 0.0) {
    return max(corner_center_to_point.x, corner_center_to_point.y);
  }
  let signed_distance_to_inset_quad =
    length(max(vec2<f32>(0.0), corner_center_to_point)) +
    min(0.0, max(corner_center_to_point.x, corner_center_to_point.y));
  return signed_distance_to_inset_quad - corner_radius;
}

fn quad_sdf(point: vec2<f32>, rect_origin: vec2<f32>, rect_size: vec2<f32>, corner_radii: vec4<f32>) -> f32 {
  let center = rect_origin + rect_size * 0.5;
  let center_to_point = point - center;
  let half_size = rect_size * 0.5;
  let corner_radius = pick_corner_radius(center_to_point, corner_radii);
  let corner_to_point = abs(center_to_point) - half_size;
  let corner_center_to_point = corner_to_point + corner_radius;
  return quad_sdf_impl(corner_center_to_point, corner_radius);
}

fn clip_alpha(pixel_pos: vec2<f32>) -> f32 {
  var alpha = 1.0;
  var idx = viewport.clip_head;
  for (var i = 0u; i < 64u; i = i + 1u) {
    if (i >= viewport.clip_count) {
      break;
    }
    if (idx == 0xffffffffu) {
      break;
    }
    let clip = clip_stack.clips[idx];
    idx = bitcast<u32>(clip.inv0.w);
    let clip_local = vec2<f32>(
      dot(clip.inv0.xy, pixel_pos) + clip.inv0.z,
      dot(clip.inv1.xy, pixel_pos) + clip.inv1.z
    );
    let sdf = quad_sdf(clip_local, clip.rect.xy, clip.rect.zw, clip.corner_radii);
    let aa = max(fwidth(sdf), 1e-4);
    let a = 1.0 - smoothstep(-aa, aa, sdf);
    alpha = alpha * a;
  }
  return alpha;
}

fn saturate(x: f32) -> f32 {
  return clamp(x, 0.0, 1.0);
}

fn gradient_tile_mode_apply(t: f32, tile_mode: u32) -> f32 {
  if (tile_mode == 1u) {
    return fract(t);
  }
  if (tile_mode == 2u) {
    let seg = floor(t);
    let r = fract(t);
    let odd = (i32(seg) & 1) != 0;
    return select(r, 1.0 - r, odd);
  }
  return clamp(t, 0.0, 1.0);
}

fn paint_stop_offset(p: Paint, i: u32) -> f32 {
  if (i < 4u) {
    return p.stop_offsets0[i];
  }
  return p.stop_offsets1[i - 4u];
}

fn paint_unpremul_rgb(c: vec4<f32>) -> vec3<f32> {
  let a = max(c.a, 1e-6);
  return c.rgb / a;
}

fn linear_rgb_to_oklab(rgb: vec3<f32>) -> vec3<f32> {
  let c = max(rgb, vec3<f32>(0.0));
  let lms = vec3<f32>(
    0.4122214708 * c.x + 0.5363325363 * c.y + 0.0514459929 * c.z,
    0.2119034982 * c.x + 0.6806995451 * c.y + 0.1073969566 * c.z,
    0.0883024619 * c.x + 0.2817188376 * c.y + 0.6299787005 * c.z
  );
  let lms_cbrt = pow(max(lms, vec3<f32>(0.0)), vec3<f32>(1.0 / 3.0));
  return vec3<f32>(
    0.2104542553 * lms_cbrt.x + 0.7936177850 * lms_cbrt.y - 0.0040720468 * lms_cbrt.z,
    1.9779984951 * lms_cbrt.x - 2.4285922050 * lms_cbrt.y + 0.4505937099 * lms_cbrt.z,
    0.0259040371 * lms_cbrt.x + 0.7827717662 * lms_cbrt.y - 0.8086757660 * lms_cbrt.z
  );
}

fn oklab_to_linear_rgb(lab: vec3<f32>) -> vec3<f32> {
  let lms_cbrt = vec3<f32>(
    lab.x + 0.3963377774 * lab.y + 0.2158037573 * lab.z,
    lab.x - 0.1055613458 * lab.y - 0.0638541728 * lab.z,
    lab.x - 0.0894841775 * lab.y - 1.2914855480 * lab.z
  );
  let lms = lms_cbrt * lms_cbrt * lms_cbrt;
  return vec3<f32>(
    4.0767416621 * lms.x - 3.3077115913 * lms.y + 0.2309699292 * lms.z,
    -1.2684380046 * lms.x + 2.6097574011 * lms.y - 0.3413193965 * lms.z,
    -0.0041960863 * lms.x - 0.7034186147 * lms.y + 1.7076147010 * lms.z
  );
}

fn paint_mix_colorspace(p: Paint, a: vec4<f32>, b: vec4<f32>, u: f32) -> vec4<f32> {
  if (p.color_space == 1u) {
    let a0 = clamp(a.a, 0.0, 1.0);
    let a1 = clamp(b.a, 0.0, 1.0);
    let alpha = clamp(mix(a0, a1, u), 0.0, 1.0);

    let rgb0 = clamp(paint_unpremul_rgb(a), vec3<f32>(0.0), vec3<f32>(1.0));
    let rgb1 = clamp(paint_unpremul_rgb(b), vec3<f32>(0.0), vec3<f32>(1.0));
    let lab0 = linear_rgb_to_oklab(rgb0);
    let lab1 = linear_rgb_to_oklab(rgb1);
    let lab = mix(lab0, lab1, u);
    let rgb = clamp(oklab_to_linear_rgb(lab), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(rgb * alpha, alpha);
  }
  return mix(a, b, u);
}

fn paint_sample_stops(p: Paint, t: f32) -> vec4<f32> {
  let n = min(p.stop_count, MAX_STOPS);
  if (n == 0u) {
    return vec4<f32>(0.0);
  }

  var prev_offset = paint_stop_offset(p, 0u);
  var prev_color = p.stop_colors[0u];
  if (n == 1u || t <= prev_offset) {
    return prev_color;
  }

  for (var i = 1u; i < 8u; i = i + 1u) {
    if (i >= n) {
      break;
    }
    let off = paint_stop_offset(p, i);
    let col = p.stop_colors[i];
    if (t <= off) {
      let denom = max(off - prev_offset, 1e-6);
      let u = saturate((t - prev_offset) / denom);
      return paint_mix_colorspace(p, prev_color, col, u);
    }
    prev_offset = off;
    prev_color = col;
  }
  return prev_color;
}

fn paint_eval(p: Paint, local_pos: vec2<f32>, pixel_pos: vec2<f32>) -> vec4<f32> {
  let pos = select(local_pos, pixel_pos, p.eval_space == 1u);
  if (p.kind == 0u) {
    return p.params0;
  }
  if (p.kind == 1u) {
    let start = p.params0.xy;
    let end = p.params0.zw;
    let dir = end - start;
    let len2 = dot(dir, dir);
    let t = select(0.0, dot(pos - start, dir) / len2, len2 > 1e-6);
    let tt = gradient_tile_mode_apply(t, p.tile_mode);
    return paint_sample_stops(p, tt);
  }
  if (p.kind == 2u) {
    let center = p.params0.xy;
    let radius = max(p.params0.zw, vec2<f32>(1e-6));
    let d = (pos - center) / radius;
    let t = length(d);
    let tt = gradient_tile_mode_apply(t, p.tile_mode);
    return paint_sample_stops(p, tt);
  }
  if (p.kind == 4u) {
    let center = p.params0.xy;
    let start = p.params0.z;
    let span = max(p.params0.w, 1e-6);
    let v = pos - center;
    let a = atan2(v.y, v.x);
    let turns = fract(a * (1.0 / 6.2831853) + 1.0);
    let rel = fract(turns - fract(start) + 1.0);
    let t = rel / span;
    let tt = gradient_tile_mode_apply(t, p.tile_mode);
    return paint_sample_stops(p, tt);
  }
  return vec4<f32>(0.0);
}

fn mask_stop_offset(m: MaskGradient, i: u32) -> f32 {
  if (i < 4u) { return m.stop_offsets0[i]; }
  return m.stop_offsets1[i - 4u];
}

fn mask_stop_alpha(m: MaskGradient, i: u32) -> f32 {
  if (i < 4u) { return m.stop_alphas0[i]; }
  return m.stop_alphas1[i - 4u];
}

fn mask_sample_stops(m: MaskGradient, t: f32) -> f32 {
  let n = min(m.stop_count, 8u);
  if (n == 0u) { return 1.0; }

  var prev_offset = mask_stop_offset(m, 0u);
  var prev_alpha = mask_stop_alpha(m, 0u);
  if (n == 1u || t <= prev_offset) {
    return prev_alpha;
  }

  for (var i = 1u; i < 8u; i = i + 1u) {
    if (i >= n) {
      break;
    }
    let off = mask_stop_offset(m, i);
    let a = mask_stop_alpha(m, i);
    if (t <= off) {
      let denom = max(off - prev_offset, 1e-6);
      let u = saturate((t - prev_offset) / denom);
      return mix(prev_alpha, a, u);
    }
    prev_offset = off;
    prev_alpha = a;
  }
  return prev_alpha;
}

fn mask_image_sample_bilinear_clamp(uv: vec2<f32>) -> vec4<f32> {
  let dims_u = textureDimensions(mask_image_texture);
  if (dims_u.x == 0u || dims_u.y == 0u) {
    return vec4<f32>(0.0);
  }

  let dims = vec2<f32>(f32(dims_u.x), f32(dims_u.y));
  let p_px = uv * dims;

  // Manual bilinear sampling avoids WGSL uniformity restrictions on WebGPU.
  let max_p = vec2<f32>(dims.x - 0.5, dims.y - 0.5);
  let p = clamp(p_px, vec2<f32>(0.5), max_p);

  let t = p - vec2<f32>(0.5);
  let base_f = floor(t);
  let f = fract(t);

  let x0 = clamp(i32(base_f.x), 0, i32(dims_u.x) - 1);
  let y0 = clamp(i32(base_f.y), 0, i32(dims_u.y) - 1);
  let x1 = min(x0 + 1, i32(dims_u.x) - 1);
  let y1 = min(y0 + 1, i32(dims_u.y) - 1);

  let c00 = textureLoad(mask_image_texture, vec2<i32>(x0, y0), 0);
  let c10 = textureLoad(mask_image_texture, vec2<i32>(x1, y0), 0);
  let c01 = textureLoad(mask_image_texture, vec2<i32>(x0, y1), 0);
  let c11 = textureLoad(mask_image_texture, vec2<i32>(x1, y1), 0);

  let cx0 = mix(c00, c10, f.x);
  let cx1 = mix(c01, c11, f.x);
  return mix(cx0, cx1, f.y);
}

fn mask_eval(m: MaskGradient, pixel_pos: vec2<f32>) -> f32 {
  let local_pos = vec2<f32>(
    dot(m.inv0.xy, pixel_pos) + m.inv0.z,
    dot(m.inv1.xy, pixel_pos) + m.inv1.z
  );

  let p = local_pos - m.bounds.xy;
  let in_bounds = p.x >= 0.0 && p.y >= 0.0 && p.x <= m.bounds.z && p.y <= m.bounds.w;

  if (m.kind == 1u) {
    let start = m.params0.xy;
    let end = m.params0.zw;
    let dir = end - start;
    let len2 = dot(dir, dir);
    let t = select(0.0, dot(local_pos - start, dir) / len2, len2 > 1e-6);
    let tt = gradient_tile_mode_apply(t, m.tile_mode);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 2u) {
    let center = m.params0.xy;
    let radius = max(m.params0.zw, vec2<f32>(1e-6));
    let d = (local_pos - center) / radius;
    let t = length(d);
    let tt = gradient_tile_mode_apply(t, m.tile_mode);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 3u) {
    let uv0 = m.params0.xy;
    let uv1 = m.params0.zw;
    let denom = max(m.bounds.zw, vec2<f32>(1e-6));
    let t = clamp(p / denom, vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = mix(uv0, uv1, t);
    let s = mask_image_sample_bilinear_clamp(uv);
    let cov = select(s.r, s.a, m.tile_mode == 1u);
    return select(1.0, clamp(cov, 0.0, 1.0), in_bounds);
  }

  return 1.0;
}

fn mask_alpha(pixel_pos: vec2<f32>) -> f32 {
  var alpha = 1.0;
  var idx = viewport.mask_head;
  for (var i = 0u; i < 64u; i = i + 1u) {
    if (i >= viewport.mask_count) {
      break;
    }
    if (viewport.mask_scope_count != 0u && idx == viewport.mask_scope_head) {
      break;
    }
    if (idx == 0xffffffffu) {
      break;
    }
    let m = mask_stack.masks[idx];
    idx = bitcast<u32>(m.inv0.w);
    alpha = alpha * clamp(mask_eval(m, pixel_pos), 0.0, 1.0);
  }
  return alpha;
}

fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
  let a = 0.055;
  let lo = rgb * 12.92;
  let hi = (1.0 + a) * pow(rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(a);
  return select(hi, lo, rgb <= vec3<f32>(0.0031308));
}

fn encode_output_premul(c: vec4<f32>) -> vec4<f32> {
  return c;
}

@vertex
fn vs_main(input: VsIn, @builtin(instance_index) instance_index: u32) -> VsOut {
  var out: VsOut;
  let clip_xy = to_clip_space(input.pos_px);
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.uv = input.uv;
  out.color = input.color;
  out.pixel_pos = input.pos_px;
  out.local_pos_px = input.local_pos_px;
  out.paint_index = instance_index;
  out.outline_params = input.outline_params;
  return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let clip = clip_alpha(input.pixel_pos);
  let mask = mask_alpha(input.pixel_pos);
  let tex = textureSample(glyph_atlas, glyph_sampler, input.uv);
  let p = text_paints.paints[input.paint_index];
  let base = paint_eval(p, input.local_pos_px, input.pixel_pos) * input.color;
  let a = tex.a * base.a;
  let out = vec4<f32>(tex.rgb * a, a) * clip * mask;
  return encode_output_premul(out);
}
"#;

pub(super) const TEXT_SUBPIXEL_SHADER: &str = r#"
struct ClipRRect {
  rect: vec4<f32>,
  corner_radii: vec4<f32>,
  inv0: vec4<f32>,
  inv1: vec4<f32>,
};

struct Viewport {
  viewport_size: vec2<f32>,
  clip_head: u32,
  clip_count: u32,
  mask_head: u32,
  mask_count: u32,
  mask_scope_head: u32,
  mask_scope_count: u32,
  output_is_srgb: u32,
  _pad0: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
  text_gamma_ratios: vec4<f32>,
  text_grayscale_enhanced_contrast: f32,
  text_subpixel_enhanced_contrast: f32,
  _pad_text_quality0: u32,
  _pad_text_quality1: u32,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;

struct RenderSpace {
  origin_px: vec2<f32>,
  size_px: vec2<f32>,
};

@group(0) @binding(5) var<uniform> render_space: RenderSpace;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

struct MaskGradient {
  bounds: vec4<f32>,
  kind: u32,
  tile_mode: u32,
  stop_count: u32,
  _pad0: u32,
  params0: vec4<f32>,
  inv0: vec4<f32>,
  inv1: vec4<f32>,
  stop_alphas0: vec4<f32>,
  stop_alphas1: vec4<f32>,
  stop_offsets0: vec4<f32>,
  stop_offsets1: vec4<f32>,
};

struct MaskStack {
  masks: array<MaskGradient>,
};

@group(0) @binding(2) var<storage, read> mask_stack: MaskStack;

@group(0) @binding(6) var mask_image_sampler: sampler;
@group(0) @binding(7) var mask_image_texture: texture_2d<f32>;

@group(1) @binding(0) var glyph_sampler: sampler;
@group(1) @binding(1) var glyph_atlas: texture_2d<f32>;

const MAX_STOPS: u32 = 8u;

struct Paint {
  kind: u32,
  tile_mode: u32,
  color_space: u32,
  stop_count: u32,
  eval_space: u32,
  _pad_eval0: u32,
  _pad_eval1: u32,
  _pad_eval2: u32,
  params0: vec4<f32>,
  params1: vec4<f32>,
  params2: vec4<f32>,
  params3: vec4<f32>,
  stop_colors: array<vec4<f32>, 8>,
  stop_offsets0: vec4<f32>,
  stop_offsets1: vec4<f32>,
};

struct TextPaints {
  paints: array<Paint>,
};

@group(2) @binding(0) var<storage, read> text_paints: TextPaints;

struct VsIn {
  @location(0) pos_px: vec2<f32>,
  @location(1) local_pos_px: vec2<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) outline_params: u32,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) pixel_pos: vec2<f32>,
  @location(3) local_pos_px: vec2<f32>,
  @location(4) @interpolate(flat) paint_index: u32,
  @location(5) @interpolate(flat) outline_params: u32,
};

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let local = pixel_pos - render_space.origin_px;
  let ndc_x = (local.x / render_space.size_px.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (local.y / render_space.size_px.y) * 2.0;
  return vec2<f32>(ndc_x, ndc_y);
}

fn pick_corner_radius(center_to_point: vec2<f32>, radii: vec4<f32>) -> f32 {
  if (center_to_point.x < 0.0) {
    if (center_to_point.y < 0.0) { return radii.x; }
    return radii.w;
  }
  if (center_to_point.y < 0.0) { return radii.y; }
  return radii.z;
}

fn quad_sdf_impl(corner_center_to_point: vec2<f32>, corner_radius: f32) -> f32 {
  if (corner_radius == 0.0) {
    return max(corner_center_to_point.x, corner_center_to_point.y);
  }
  let signed_distance_to_inset_quad =
    length(max(vec2<f32>(0.0), corner_center_to_point)) +
    min(0.0, max(corner_center_to_point.x, corner_center_to_point.y));
  return signed_distance_to_inset_quad - corner_radius;
}

fn quad_sdf(point: vec2<f32>, rect_origin: vec2<f32>, rect_size: vec2<f32>, corner_radii: vec4<f32>) -> f32 {
  let center = rect_origin + rect_size * 0.5;
  let center_to_point = point - center;
  let half_size = rect_size * 0.5;
  let corner_radius = pick_corner_radius(center_to_point, corner_radii);
  let corner_to_point = abs(center_to_point) - half_size;
  let corner_center_to_point = corner_to_point + corner_radius;
  return quad_sdf_impl(corner_center_to_point, corner_radius);
}

fn clip_alpha(pixel_pos: vec2<f32>) -> f32 {
  var alpha = 1.0;
  var idx = viewport.clip_head;
  for (var i = 0u; i < 64u; i = i + 1u) {
    if (i >= viewport.clip_count) {
      break;
    }
    if (idx == 0xffffffffu) {
      break;
    }
    let clip = clip_stack.clips[idx];
    idx = bitcast<u32>(clip.inv0.w);
    let clip_local = vec2<f32>(
      dot(clip.inv0.xy, pixel_pos) + clip.inv0.z,
      dot(clip.inv1.xy, pixel_pos) + clip.inv1.z
    );
    let sdf = quad_sdf(clip_local, clip.rect.xy, clip.rect.zw, clip.corner_radii);
    let aa = max(fwidth(sdf), 1e-4);
    let a = 1.0 - smoothstep(-aa, aa, sdf);
    alpha = alpha * a;
  }
  return alpha;
}

fn saturate(x: f32) -> f32 {
  return clamp(x, 0.0, 1.0);
}

fn gradient_tile_mode_apply(t: f32, tile_mode: u32) -> f32 {
  if (tile_mode == 1u) {
    return fract(t);
  }
  if (tile_mode == 2u) {
    let seg = floor(t);
    let r = fract(t);
    let odd = (i32(seg) & 1) != 0;
    return select(r, 1.0 - r, odd);
  }
  return clamp(t, 0.0, 1.0);
}

fn paint_stop_offset(p: Paint, i: u32) -> f32 {
  if (i < 4u) {
    return p.stop_offsets0[i];
  }
  return p.stop_offsets1[i - 4u];
}

fn paint_unpremul_rgb(c: vec4<f32>) -> vec3<f32> {
  let a = max(c.a, 1e-6);
  return c.rgb / a;
}

fn linear_rgb_to_oklab(rgb: vec3<f32>) -> vec3<f32> {
  let c = max(rgb, vec3<f32>(0.0));
  let lms = vec3<f32>(
    0.4122214708 * c.x + 0.5363325363 * c.y + 0.0514459929 * c.z,
    0.2119034982 * c.x + 0.6806995451 * c.y + 0.1073969566 * c.z,
    0.0883024619 * c.x + 0.2817188376 * c.y + 0.6299787005 * c.z
  );
  let lms_cbrt = pow(max(lms, vec3<f32>(0.0)), vec3<f32>(1.0 / 3.0));
  return vec3<f32>(
    0.2104542553 * lms_cbrt.x + 0.7936177850 * lms_cbrt.y - 0.0040720468 * lms_cbrt.z,
    1.9779984951 * lms_cbrt.x - 2.4285922050 * lms_cbrt.y + 0.4505937099 * lms_cbrt.z,
    0.0259040371 * lms_cbrt.x + 0.7827717662 * lms_cbrt.y - 0.8086757660 * lms_cbrt.z
  );
}

fn oklab_to_linear_rgb(lab: vec3<f32>) -> vec3<f32> {
  let lms_cbrt = vec3<f32>(
    lab.x + 0.3963377774 * lab.y + 0.2158037573 * lab.z,
    lab.x - 0.1055613458 * lab.y - 0.0638541728 * lab.z,
    lab.x - 0.0894841775 * lab.y - 1.2914855480 * lab.z
  );
  let lms = lms_cbrt * lms_cbrt * lms_cbrt;
  return vec3<f32>(
    4.0767416621 * lms.x - 3.3077115913 * lms.y + 0.2309699292 * lms.z,
    -1.2684380046 * lms.x + 2.6097574011 * lms.y - 0.3413193965 * lms.z,
    -0.0041960863 * lms.x - 0.7034186147 * lms.y + 1.7076147010 * lms.z
  );
}

fn paint_mix_colorspace(p: Paint, a: vec4<f32>, b: vec4<f32>, u: f32) -> vec4<f32> {
  if (p.color_space == 1u) {
    let a0 = clamp(a.a, 0.0, 1.0);
    let a1 = clamp(b.a, 0.0, 1.0);
    let alpha = clamp(mix(a0, a1, u), 0.0, 1.0);

    let rgb0 = clamp(paint_unpremul_rgb(a), vec3<f32>(0.0), vec3<f32>(1.0));
    let rgb1 = clamp(paint_unpremul_rgb(b), vec3<f32>(0.0), vec3<f32>(1.0));
    let lab0 = linear_rgb_to_oklab(rgb0);
    let lab1 = linear_rgb_to_oklab(rgb1);
    let lab = mix(lab0, lab1, u);
    let rgb = clamp(oklab_to_linear_rgb(lab), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(rgb * alpha, alpha);
  }
  return mix(a, b, u);
}

fn paint_sample_stops(p: Paint, t: f32) -> vec4<f32> {
  let n = min(p.stop_count, MAX_STOPS);
  if (n == 0u) {
    return vec4<f32>(0.0);
  }

  var prev_offset = paint_stop_offset(p, 0u);
  var prev_color = p.stop_colors[0u];
  if (n == 1u || t <= prev_offset) {
    return prev_color;
  }

  for (var i = 1u; i < 8u; i = i + 1u) {
    if (i >= n) {
      break;
    }
    let off = paint_stop_offset(p, i);
    let col = p.stop_colors[i];
    if (t <= off) {
      let denom = max(off - prev_offset, 1e-6);
      let u = saturate((t - prev_offset) / denom);
      return paint_mix_colorspace(p, prev_color, col, u);
    }
    prev_offset = off;
    prev_color = col;
  }
  return prev_color;
}

fn paint_eval(p: Paint, local_pos: vec2<f32>, pixel_pos: vec2<f32>) -> vec4<f32> {
  let pos = select(local_pos, pixel_pos, p.eval_space == 1u);
  if (p.kind == 0u) {
    return p.params0;
  }
  if (p.kind == 1u) {
    let start = p.params0.xy;
    let end = p.params0.zw;
    let dir = end - start;
    let len2 = dot(dir, dir);
    let t = select(0.0, dot(pos - start, dir) / len2, len2 > 1e-6);
    let tt = gradient_tile_mode_apply(t, p.tile_mode);
    return paint_sample_stops(p, tt);
  }
  if (p.kind == 2u) {
    let center = p.params0.xy;
    let radius = max(p.params0.zw, vec2<f32>(1e-6));
    let d = (pos - center) / radius;
    let t = length(d);
    let tt = gradient_tile_mode_apply(t, p.tile_mode);
    return paint_sample_stops(p, tt);
  }
  if (p.kind == 4u) {
    let center = p.params0.xy;
    let start = p.params0.z;
    let span = max(p.params0.w, 1e-6);
    let v = pos - center;
    let a = atan2(v.y, v.x);
    let turns = fract(a * (1.0 / 6.2831853) + 1.0);
    let rel = fract(turns - fract(start) + 1.0);
    let t = rel / span;
    let tt = gradient_tile_mode_apply(t, p.tile_mode);
    return paint_sample_stops(p, tt);
  }
  return vec4<f32>(0.0);
}

fn mask_stop_offset(m: MaskGradient, i: u32) -> f32 {
  if (i < 4u) { return m.stop_offsets0[i]; }
  return m.stop_offsets1[i - 4u];
}

fn mask_stop_alpha(m: MaskGradient, i: u32) -> f32 {
  if (i < 4u) { return m.stop_alphas0[i]; }
  return m.stop_alphas1[i - 4u];
}

fn mask_sample_stops(m: MaskGradient, t: f32) -> f32 {
  let n = min(m.stop_count, 8u);
  if (n == 0u) { return 1.0; }

  var prev_offset = mask_stop_offset(m, 0u);
  var prev_alpha = mask_stop_alpha(m, 0u);
  if (n == 1u || t <= prev_offset) {
    return prev_alpha;
  }

  for (var i = 1u; i < 8u; i = i + 1u) {
    if (i >= n) {
      break;
    }
    let off = mask_stop_offset(m, i);
    let a = mask_stop_alpha(m, i);
    if (t <= off) {
      let denom = max(off - prev_offset, 1e-6);
      let u = saturate((t - prev_offset) / denom);
      return mix(prev_alpha, a, u);
    }
    prev_offset = off;
    prev_alpha = a;
  }
  return prev_alpha;
}

fn mask_image_sample_bilinear_clamp(uv: vec2<f32>) -> vec4<f32> {
  let dims_u = textureDimensions(mask_image_texture);
  if (dims_u.x == 0u || dims_u.y == 0u) {
    return vec4<f32>(0.0);
  }

  let dims = vec2<f32>(f32(dims_u.x), f32(dims_u.y));
  let p_px = uv * dims;

  // Manual bilinear sampling avoids WGSL uniformity restrictions on WebGPU.
  let max_p = vec2<f32>(dims.x - 0.5, dims.y - 0.5);
  let p = clamp(p_px, vec2<f32>(0.5), max_p);

  let t = p - vec2<f32>(0.5);
  let base_f = floor(t);
  let f = fract(t);

  let x0 = clamp(i32(base_f.x), 0, i32(dims_u.x) - 1);
  let y0 = clamp(i32(base_f.y), 0, i32(dims_u.y) - 1);
  let x1 = min(x0 + 1, i32(dims_u.x) - 1);
  let y1 = min(y0 + 1, i32(dims_u.y) - 1);

  let c00 = textureLoad(mask_image_texture, vec2<i32>(x0, y0), 0);
  let c10 = textureLoad(mask_image_texture, vec2<i32>(x1, y0), 0);
  let c01 = textureLoad(mask_image_texture, vec2<i32>(x0, y1), 0);
  let c11 = textureLoad(mask_image_texture, vec2<i32>(x1, y1), 0);

  let cx0 = mix(c00, c10, f.x);
  let cx1 = mix(c01, c11, f.x);
  return mix(cx0, cx1, f.y);
}

fn mask_eval(m: MaskGradient, pixel_pos: vec2<f32>) -> f32 {
  let local_pos = vec2<f32>(
    dot(m.inv0.xy, pixel_pos) + m.inv0.z,
    dot(m.inv1.xy, pixel_pos) + m.inv1.z
  );

  let p = local_pos - m.bounds.xy;
  let in_bounds = p.x >= 0.0 && p.y >= 0.0 && p.x <= m.bounds.z && p.y <= m.bounds.w;

  if (m.kind == 1u) {
    let start = m.params0.xy;
    let end = m.params0.zw;
    let dir = end - start;
    let len2 = dot(dir, dir);
    let t = select(0.0, dot(local_pos - start, dir) / len2, len2 > 1e-6);
    let tt = gradient_tile_mode_apply(t, m.tile_mode);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 2u) {
    let center = m.params0.xy;
    let radius = max(m.params0.zw, vec2<f32>(1e-6));
    let d = (local_pos - center) / radius;
    let t = length(d);
    let tt = gradient_tile_mode_apply(t, m.tile_mode);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 3u) {
    let uv0 = m.params0.xy;
    let uv1 = m.params0.zw;
    let denom = max(m.bounds.zw, vec2<f32>(1e-6));
    let t = clamp(p / denom, vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = mix(uv0, uv1, t);
    let s = mask_image_sample_bilinear_clamp(uv);
    let cov = select(s.r, s.a, m.tile_mode == 1u);
    return select(1.0, clamp(cov, 0.0, 1.0), in_bounds);
  }

  return 1.0;
}

fn mask_alpha(pixel_pos: vec2<f32>) -> f32 {
  var alpha = 1.0;
  var idx = viewport.mask_head;
  for (var i = 0u; i < 64u; i = i + 1u) {
    if (i >= viewport.mask_count) {
      break;
    }
    if (viewport.mask_scope_count != 0u && idx == viewport.mask_scope_head) {
      break;
    }
    if (idx == 0xffffffffu) {
      break;
    }
    let m = mask_stack.masks[idx];
    idx = bitcast<u32>(m.inv0.w);
    alpha = alpha * clamp(mask_eval(m, pixel_pos), 0.0, 1.0);
  }
  return alpha;
}

fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
  let a = 0.055;
  let lo = rgb * 12.92;
  let hi = (1.0 + a) * pow(rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(a);
  return select(hi, lo, rgb <= vec3<f32>(0.0031308));
}

fn color_brightness(color: vec3<f32>) -> f32 {
  return dot(color, vec3<f32>(0.30, 0.59, 0.11));
}

fn light_on_dark_contrast(enhanced_contrast: f32, color: vec3<f32>) -> f32 {
  let brightness = color_brightness(color);
  let multiplier = clamp(4.0 * (0.75 - brightness), 0.0, 1.0);
  return enhanced_contrast * multiplier;
}

fn enhance_contrast3(alpha: vec3<f32>, k: f32) -> vec3<f32> {
  return alpha * (k + 1.0) / (alpha * k + 1.0);
}

fn apply_alpha_correction3(alpha: vec3<f32>, brightness: vec3<f32>, g: vec4<f32>) -> vec3<f32> {
  let brightness_adjustment = g.x * brightness + g.y;
  let correction = brightness_adjustment * alpha + (g.z * brightness + g.w);
  return alpha + alpha * (vec3<f32>(1.0) - alpha) * correction;
}

fn apply_contrast_and_gamma_correction3(sample: vec3<f32>, color: vec3<f32>) -> vec3<f32> {
  let k = light_on_dark_contrast(viewport.text_subpixel_enhanced_contrast, color);
  let contrasted = enhance_contrast3(sample, k);
  return apply_alpha_correction3(contrasted, color, viewport.text_gamma_ratios);
}

const FRET_TEXT_OUTLINE_PRESENT: u32 = 0u;

fn glyph_sample_max_rgb(uv: vec2<f32>) -> f32 {
  let tex = textureSample(glyph_atlas, glyph_sampler, uv);
  return max(max(tex.r, tex.g), tex.b);
}

fn glyph_dilate_max_rgb(uv: vec2<f32>, radius: u32) -> f32 {
  let dims_u = textureDimensions(glyph_atlas);
  let dims = vec2<f32>(f32(dims_u.x), f32(dims_u.y));
  let texel = vec2<f32>(1.0, 1.0) / dims;

  let s0 = glyph_sample_max_rgb(uv);

  let dx = vec2<f32>(texel.x, 0.0);
  let dy = vec2<f32>(0.0, texel.y);
  let d11 = vec2<f32>(texel.x, texel.y);
  let d1m1 = vec2<f32>(texel.x, -texel.y);

  let d1x = 1.0 * dx;
  let d1y = 1.0 * dy;
  let d1d0 = 1.0 * d11;
  let d1d1 = 1.0 * d1m1;
  let max1 = max(
    s0,
    max(
      max(
        max(glyph_sample_max_rgb(uv + d1x), glyph_sample_max_rgb(uv - d1x)),
        max(glyph_sample_max_rgb(uv + d1y), glyph_sample_max_rgb(uv - d1y))
      ),
      max(
        max(glyph_sample_max_rgb(uv + d1d0), glyph_sample_max_rgb(uv - d1d0)),
        max(glyph_sample_max_rgb(uv + d1d1), glyph_sample_max_rgb(uv - d1d1))
      )
    )
  );

  let d2x = 2.0 * dx;
  let d2y = 2.0 * dy;
  let d2d0 = 2.0 * d11;
  let d2d1 = 2.0 * d1m1;
  let max2 = max(
    max1,
    max(
      max(
        max(glyph_sample_max_rgb(uv + d2x), glyph_sample_max_rgb(uv - d2x)),
        max(glyph_sample_max_rgb(uv + d2y), glyph_sample_max_rgb(uv - d2y))
      ),
      max(
        max(glyph_sample_max_rgb(uv + d2d0), glyph_sample_max_rgb(uv - d2d0)),
        max(glyph_sample_max_rgb(uv + d2d1), glyph_sample_max_rgb(uv - d2d1))
      )
    )
  );

  let d3x = 3.0 * dx;
  let d3y = 3.0 * dy;
  let d3d0 = 3.0 * d11;
  let d3d1 = 3.0 * d1m1;
  let max3 = max(
    max2,
    max(
      max(
        max(glyph_sample_max_rgb(uv + d3x), glyph_sample_max_rgb(uv - d3x)),
        max(glyph_sample_max_rgb(uv + d3y), glyph_sample_max_rgb(uv - d3y))
      ),
      max(
        max(glyph_sample_max_rgb(uv + d3d0), glyph_sample_max_rgb(uv - d3d0)),
        max(glyph_sample_max_rgb(uv + d3d1), glyph_sample_max_rgb(uv - d3d1))
      )
    )
  );

  let m1 = select(0.0, 1.0, radius >= 1u);
  let m2 = select(0.0, 1.0, radius >= 2u);
  let m3 = select(0.0, 1.0, radius >= 3u);
  return max(s0, max(max1 * m1, max(max2 * m2, max3 * m3)));
}

fn encode_output_premul(c: vec4<f32>) -> vec4<f32> {
  return c;
}

@vertex
fn vs_main(input: VsIn, @builtin(instance_index) instance_index: u32) -> VsOut {
  var out: VsOut;
  let clip_xy = to_clip_space(input.pos_px);
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.uv = input.uv;
  out.color = input.color;
  out.pixel_pos = input.pos_px;
  out.local_pos_px = input.local_pos_px;
  out.paint_index = instance_index;
  out.outline_params = input.outline_params;
  return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let clip = clip_alpha(input.pixel_pos);
  let mask = mask_alpha(input.pixel_pos);
  let tex = textureSample(glyph_atlas, glyph_sampler, input.uv);
  let p = text_paints.paints[input.paint_index];
  let base = paint_eval(p, input.local_pos_px, input.pixel_pos) * input.color;
  let base_un = select(vec3<f32>(0.0), base.rgb / base.a, base.a > 1e-6);
  let coverage = apply_contrast_and_gamma_correction3(tex.rgb, base_un);
  let a = max(max(coverage.r, coverage.g), coverage.b);
  var out = vec4<f32>(base.rgb * coverage, base.a * a);

  if (FRET_TEXT_OUTLINE_PRESENT != 0u) {
    let outline_params = input.outline_params;
    let outline_radius = min(outline_params & 3u, 3u);
    let outline_enabled = select(0.0, 1.0, outline_radius != 0u);
    let fill_sample = max(max(tex.r, tex.g), tex.b);
    let outline_max_sample = glyph_dilate_max_rgb(input.uv, outline_radius);
    let ring_sample = saturate(outline_max_sample - fill_sample) * outline_enabled;
    let outline_paint_index = outline_params >> 2u;
    let op = text_paints.paints[outline_paint_index];
    let outline_mul = vec4<f32>(1.0, 1.0, 1.0, input.color.a);
    let outline_base = paint_eval(op, input.local_pos_px, input.pixel_pos) * outline_mul;
    let outline_un = select(
      vec3<f32>(0.0),
      outline_base.rgb / outline_base.a,
      outline_base.a > 1e-6
    );
    let ring_cov3 = apply_contrast_and_gamma_correction3(
      vec3<f32>(ring_sample, ring_sample, ring_sample),
      outline_un
    );
    let ring_a = max(max(ring_cov3.r, ring_cov3.g), ring_cov3.b);
    out = out + vec4<f32>(outline_base.rgb * ring_cov3, outline_base.a * ring_a);
  }

  out = out * clip * mask;
  return encode_output_premul(out);
}
"#;

pub(super) const MASK_SHADER: &str = include_str!("pipelines/wgsl/mask.wgsl");

pub(super) const NOISE_SHADER: &str = include_str!("pipelines/wgsl/noise.wgsl");

const NOISE_MASKED_SHADER_PART_A: &str = include_str!("pipelines/wgsl/noise_masked_part_a.wgsl");

const NOISE_MASKED_SHADER_PART_B: &str = include_str!("pipelines/wgsl/noise_masked_part_b.wgsl");

pub(super) fn noise_masked_shader_source() -> String {
    format!("{NOISE_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{NOISE_MASKED_SHADER_PART_B}")
}

pub(super) const NOISE_MASK_SHADER: &str = include_str!("pipelines/wgsl/noise_mask.wgsl");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_shader_wgsl_validates_under_naga() {
        let module = naga::front::wgsl::parse_str(PATH_SHADER).expect("PATH_SHADER must parse");
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator
            .validate(&module)
            .expect("PATH_SHADER must validate under naga");
    }
}
