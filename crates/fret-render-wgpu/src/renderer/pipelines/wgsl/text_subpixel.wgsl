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
