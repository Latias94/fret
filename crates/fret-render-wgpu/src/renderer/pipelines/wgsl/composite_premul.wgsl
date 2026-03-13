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

@group(1) @binding(0) var tex_sampler: sampler;
@group(1) @binding(1) var tex: texture_2d<f32>;

struct VsIn {
  @location(0) pos_px: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) opacity: f32,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) opacity: f32,
  @location(2) pixel_pos: vec2<f32>,
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
fn vs_main(input: VsIn) -> VsOut {
  var out: VsOut;
  let clip_xy = to_clip_space(input.pos_px);
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.uv = input.uv;
  out.opacity = input.opacity;
  out.pixel_pos = input.pos_px;
  return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let clip = clip_alpha(input.pixel_pos);
  let mask = mask_alpha(input.pixel_pos);
  let sample = textureSample(tex, tex_sampler, input.uv);
  let o = clamp(input.opacity, 0.0, 1.0);
  let out = vec4<f32>(sample.rgb * o, sample.a * o) * clip * mask;
  return encode_output_premul(out);
}
