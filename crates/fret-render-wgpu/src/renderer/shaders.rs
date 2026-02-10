const CLIP_SDF_CORE_WGSL: &str = r#"
fn saturate(x: f32) -> f32 {
  return clamp(x, 0.0, 1.0);
}

fn sdf_aa(sdf: f32) -> f32 {
  return max(fwidth(sdf), 1e-4);
}

fn sdf_coverage_smooth(sdf: f32) -> f32 {
  let aa = sdf_aa(sdf);
  return 1.0 - smoothstep(-aa, aa, sdf);
}

fn sdf_coverage_linear(sdf: f32) -> f32 {
  let aa = sdf_aa(sdf);
  return saturate(0.5 - sdf / aa);
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
"#;

const QUAD_SHADER_PART_A: &str = r#"
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
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

const MAX_STOPS: u32 = 8u;

struct Paint {
  kind: u32,
  tile_mode: u32,
  color_space: u32,
  stop_count: u32,
  params0: vec4<f32>,
  params1: vec4<f32>,
  stop_colors: array<vec4<f32>, 8>,
  stop_offsets0: vec4<f32>,
  stop_offsets1: vec4<f32>,
};

struct QuadInstance {
  rect: vec4<f32>,
  transform0: vec4<f32>,
  transform1: vec4<f32>,
  fill_paint: Paint,
  border_paint: Paint,
  corner_radii: vec4<f32>,
  border: vec4<f32>,
};

struct QuadInstances {
  instances: array<QuadInstance>,
};

@group(1) @binding(0) var<storage, read> quad_instances: QuadInstances;

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) pixel_pos: vec2<f32>,
  @location(1) local_pos: vec2<f32>,
  @location(2) rect: vec4<f32>,
  @location(3) corner_radii: vec4<f32>,
  @location(4) border: vec4<f32>,
  @location(5) @interpolate(flat) instance_index: u32,
};

fn quad_vertex_xy(vertex_index: u32) -> vec2<f32> {
  switch vertex_index {
    case 0u: { return vec2<f32>(0.0, 0.0); }
    case 1u: { return vec2<f32>(1.0, 0.0); }
    case 2u: { return vec2<f32>(1.0, 1.0); }
    case 3u: { return vec2<f32>(0.0, 0.0); }
    case 4u: { return vec2<f32>(1.0, 1.0); }
    default: { return vec2<f32>(0.0, 1.0); }
  }
}

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let ndc_x = (pixel_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (pixel_pos.y / viewport.viewport_size.y) * 2.0;
  return vec2<f32>(ndc_x, ndc_y);
}

@vertex
fn vs_main(
  @builtin(vertex_index) vertex_index: u32,
  @builtin(instance_index) instance_index: u32,
) -> VsOut {
  let inst = quad_instances.instances[instance_index];
  let rect = inst.rect;
  let transform0 = inst.transform0;
  let transform1 = inst.transform1;
  let uv = quad_vertex_xy(vertex_index);
  let local_pos = rect.xy + uv * rect.zw;
  let pixel_pos = vec2<f32>(
    dot(transform0.xy, local_pos) + transform0.z,
    dot(transform1.xy, local_pos) + transform1.z
  );
  let clip_xy = to_clip_space(pixel_pos);

  var out: VsOut;
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.pixel_pos = pixel_pos;
  out.local_pos = local_pos;
  out.rect = rect;
  out.corner_radii = inst.corner_radii;
  out.border = inst.border;
  out.instance_index = instance_index;
  return out;
}
"#;

const QUAD_SHADER_PART_B: &str = r#"
fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
  let a = 0.055;
  let lo = rgb * 12.92;
  let hi = (1.0 + a) * pow(rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(a);
  return select(hi, lo, rgb <= vec3<f32>(0.0031308));
}

fn encode_output_premul(c: vec4<f32>) -> vec4<f32> {
  if (viewport.output_is_srgb != 0u) {
    return c;
  }
  if (c.a <= 0.0) {
    return c;
  }
  let un = c.rgb / c.a;
  let enc = linear_to_srgb(un);
  return vec4<f32>(enc * c.a, c.a);
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
    alpha = alpha * sdf_coverage_smooth(sdf);
  }
  return alpha;
}

fn paint_stop_offset(p: Paint, i: u32) -> f32 {
  if (i < 4u) {
    return p.stop_offsets0[i];
  }
  return p.stop_offsets1[i - 4u];
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
      return mix(prev_color, col, u);
    }
    prev_offset = off;
    prev_color = col;
  }
  return prev_color;
}

fn paint_eval(p: Paint, local_pos: vec2<f32>) -> vec4<f32> {
  switch p.kind {
    // 0 = Solid
    case 0u: {
      return p.params0;
    }
    // 1 = LinearGradient
    case 1u: {
      let start = p.params0.xy;
      let end = p.params0.zw;
      let dir = end - start;
      let len2 = dot(dir, dir);
      let t = select(0.0, dot(local_pos - start, dir) / len2, len2 > 1e-6);
      let tt = clamp(t, 0.0, 1.0);
      return paint_sample_stops(p, tt);
    }
    // 2 = RadialGradient
    case 2u: {
      let center = p.params0.xy;
      let radius = max(p.params0.zw, vec2<f32>(1e-6));
      let d = (local_pos - center) / radius;
      let t = length(d);
      let tt = clamp(t, 0.0, 1.0);
      return paint_sample_stops(p, tt);
    }
    default: {
      return vec4<f32>(0.0);
    }
  }
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let clip = clip_alpha(input.pixel_pos);
  let inst = quad_instances.instances[input.instance_index];

  let outer_sdf = quad_sdf(input.local_pos, input.rect.xy, input.rect.zw, input.corner_radii);

  // NOTE: AA must scale with derivatives. A fixed threshold (e.g. 0.5) breaks under DPI changes
  // and transforms. See ADR 0030.
  let aa_outer = max(fwidth(outer_sdf), 1e-4);
  let alpha_outer = 1.0 - smoothstep(-aa_outer, aa_outer, outer_sdf);

  // Border alignment: inside. Inner radii are derived by subtracting adjacent border widths.
  let inner_origin = input.rect.xy + vec2<f32>(input.border.x, input.border.y);
  let inner_size = input.rect.zw - vec2<f32>(input.border.x + input.border.z, input.border.y + input.border.w);

  let inner_radii = max(
    vec4<f32>(0.0),
    vec4<f32>(
      input.corner_radii.x - max(input.border.x, input.border.y), // TL
      input.corner_radii.y - max(input.border.z, input.border.y), // TR
      input.corner_radii.z - max(input.border.z, input.border.w), // BR
      input.corner_radii.w - max(input.border.x, input.border.w)  // BL
    )
  );

  let inner_sdf = quad_sdf(input.local_pos, inner_origin, max(inner_size, vec2<f32>(0.0)), inner_radii);
  let aa_inner = max(fwidth(inner_sdf), 1e-4);
  let alpha_inner_raw = 1.0 - smoothstep(-aa_inner, aa_inner, inner_sdf);
  let inner_valid = inner_size.x > 0.0 && inner_size.y > 0.0;
  let alpha_inner = select(0.0, alpha_inner_raw, inner_valid);

  let border_sum = input.border.x + input.border.y + input.border.z + input.border.w;
  let border_present = border_sum > 0.0;

  let alpha_fill = select(alpha_outer, alpha_inner, border_present);
  let border_cov_raw = saturate(alpha_outer - alpha_inner);
  let border_cov = select(0.0, border_cov_raw, border_present);

  let fill = paint_eval(inst.fill_paint, input.local_pos) * alpha_fill;
  let border = paint_eval(inst.border_paint, input.local_pos) * border_cov;

  let out = (fill + border) * clip;
  return encode_output_premul(out);
}
"#;

pub(super) fn quad_shader_source() -> String {
    format!("{QUAD_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{QUAD_SHADER_PART_B}")
}

pub(super) const VIEWPORT_SHADER: &str = r#"
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
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

@group(1) @binding(0) var viewport_sampler: sampler;
@group(1) @binding(1) var viewport_texture: texture_2d<f32>;

struct VsIn {
  @location(0) pos_px: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) opacity: f32,
  @location(3) premul: f32,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) opacity: f32,
  @location(2) pixel_pos: vec2<f32>,
  @location(3) premul: f32,
};

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let ndc_x = (pixel_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (pixel_pos.y / viewport.viewport_size.y) * 2.0;
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

fn saturate(x: f32) -> f32 {
  return clamp(x, 0.0, 1.0);
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

fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
  let a = 0.055;
  let lo = rgb * 12.92;
  let hi = (1.0 + a) * pow(rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(a);
  return select(hi, lo, rgb <= vec3<f32>(0.0031308));
}

fn encode_output_premul(c: vec4<f32>) -> vec4<f32> {
  if (viewport.output_is_srgb != 0u) {
    return c;
  }
  if (c.a <= 0.0) {
    return c;
  }
  let un = c.rgb / c.a;
  let enc = linear_to_srgb(un);
  return vec4<f32>(enc * c.a, c.a);
}

@vertex
fn vs_main(input: VsIn) -> VsOut {
  var out: VsOut;
  let clip_xy = to_clip_space(input.pos_px);
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.uv = input.uv;
  out.opacity = input.opacity;
  out.pixel_pos = input.pos_px;
  out.premul = input.premul;
  return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let clip = clip_alpha(input.pixel_pos);
  let tex = textureSample(viewport_texture, viewport_sampler, input.uv);
  let factor = input.opacity * clip;
  let a = tex.a * factor;
  let premul = input.premul >= 0.5;
  let rgb = select(tex.rgb * a, tex.rgb * factor, premul);
  let out = vec4<f32>(rgb, a);
  return encode_output_premul(out);
}
"#;

pub(super) const BLIT_SHADER: &str = r#"
@group(0) @binding(0) var src_texture: texture_2d<f32>;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0),
  );
  var out: VsOut;
  out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
  return out;
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = textureDimensions(src_texture);
  let x = u32(floor(pos.x));
  let y = u32(floor(pos.y));
  if (x >= dims.x || y >= dims.y) {
    return vec4<f32>(0.0);
  }
  return textureLoad(src_texture, vec2<i32>(i32(x), i32(y)), 0);
}
"#;

pub(super) const DOWNSAMPLE_NEAREST_SHADER: &str = r#"
@group(0) @binding(0) var src_texture: texture_2d<f32>;

struct ScaleParams {
  scale: u32,
  _pad0: u32,
  src_origin: vec2<u32>,
  dst_origin: vec2<u32>,
  _pad1: u32,
  _pad2: u32,
};

@group(0) @binding(1) var<uniform> params: ScaleParams;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0),
  );
  var out: VsOut;
  out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
  return out;
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = textureDimensions(src_texture);
  let x = u32(floor(pos.x));
  let y = u32(floor(pos.y));
  let s = max(params.scale, 1u);
  let local_x_i = i32(x) - i32(params.dst_origin.x);
  let local_y_i = i32(y) - i32(params.dst_origin.y);
  if (local_x_i < 0 || local_y_i < 0) {
    return vec4<f32>(0.0);
  }
  let local_x = u32(local_x_i);
  let local_y = u32(local_y_i);
  let sx = params.src_origin.x + local_x * s;
  let sy = params.src_origin.y + local_y * s;
  if (sx >= dims.x || sy >= dims.y) {
    return vec4<f32>(0.0);
  }
  return textureLoad(src_texture, vec2<i32>(i32(sx), i32(sy)), 0);
}
"#;

pub(super) const UPSCALE_NEAREST_SHADER: &str = r#"
@group(0) @binding(0) var src_texture: texture_2d<f32>;

struct ScaleParams {
  scale: u32,
  _pad0: u32,
  src_origin: vec2<u32>,
  dst_origin: vec2<u32>,
  _pad1: u32,
  _pad2: u32,
};

@group(0) @binding(1) var<uniform> params: ScaleParams;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0),
  );
  var out: VsOut;
  out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
  return out;
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = textureDimensions(src_texture);
  let x = u32(floor(pos.x));
  let y = u32(floor(pos.y));
  let s = max(params.scale, 1u);
  let local_x_i = i32(x) - i32(params.dst_origin.x);
  let local_y_i = i32(y) - i32(params.dst_origin.y);
  if (local_x_i < 0 || local_y_i < 0) {
    return vec4<f32>(0.0);
  }
  let local_x = u32(local_x_i);
  let local_y = u32(local_y_i);
  let sx = params.src_origin.x + local_x / s;
  let sy = params.src_origin.y + local_y / s;
  if (sx >= dims.x || sy >= dims.y) {
    return vec4<f32>(0.0);
  }
  return textureLoad(src_texture, vec2<i32>(i32(sx), i32(sy)), 0);
}
"#;

const UPSCALE_NEAREST_MASKED_SHADER_PART_A: &str = r#"
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
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

@group(1) @binding(0) var src_texture: texture_2d<f32>;

struct ScaleParams {
  scale: u32,
  _pad0: u32,
  src_origin: vec2<u32>,
  dst_origin: vec2<u32>,
  _pad1: u32,
  _pad2: u32,
};

@group(1) @binding(1) var<uniform> params: ScaleParams;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0),
  );
  var out: VsOut;
  out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
  return out;
}
"#;

const UPSCALE_NEAREST_MASKED_SHADER_PART_B: &str = r#"
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
    alpha = alpha * sdf_coverage_linear(sdf);
  }
  return alpha;
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = textureDimensions(src_texture);
  let x = u32(floor(pos.x));
  let y = u32(floor(pos.y));
  let s = max(params.scale, 1u);
  let local_x_i = i32(x) - i32(params.dst_origin.x);
  let local_y_i = i32(y) - i32(params.dst_origin.y);
  if (local_x_i < 0 || local_y_i < 0) {
    return vec4<f32>(0.0);
  }
  let local_x = u32(local_x_i);
  let local_y = u32(local_y_i);
  let sx = params.src_origin.x + local_x / s;
  let sy = params.src_origin.y + local_y / s;
  if (sx >= dims.x || sy >= dims.y) {
    return vec4<f32>(0.0);
  }
  let sample = textureLoad(src_texture, vec2<i32>(i32(sx), i32(sy)), 0);
  let clip = clip_alpha(pos.xy);
  return vec4<f32>(sample.rgb * clip, clip);
}
"#;

pub(super) fn upscale_nearest_masked_shader_source() -> String {
    format!(
        "{UPSCALE_NEAREST_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{UPSCALE_NEAREST_MASKED_SHADER_PART_B}"
    )
}

pub(super) const UPSCALE_NEAREST_MASK_SHADER: &str = r#"
struct Viewport {
  viewport_size: vec2<f32>,
  clip_head: u32,
  clip_count: u32,
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;

@group(1) @binding(0) var src_texture: texture_2d<f32>;
@group(1) @binding(2) var mask_texture: texture_2d<f32>;

struct ScaleParams {
  scale: u32,
  _pad0: u32,
  src_origin: vec2<u32>,
  dst_origin: vec2<u32>,
  _pad1: u32,
  _pad2: u32,
};

@group(1) @binding(1) var<uniform> params: ScaleParams;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0),
  );
  var out: VsOut;
  out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
  return out;
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = textureDimensions(src_texture);
  let x = u32(floor(pos.x));
  let y = u32(floor(pos.y));
  let s = max(params.scale, 1u);
  let local_x_i = i32(x) - i32(params.dst_origin.x);
  let local_y_i = i32(y) - i32(params.dst_origin.y);
  if (local_x_i < 0 || local_y_i < 0) {
    return vec4<f32>(0.0);
  }
  let sample_x = u32(local_x_i);
  let sample_y = u32(local_y_i);
  let sx = params.src_origin.x + sample_x / s;
  let sy = params.src_origin.y + sample_y / s;
  if (sx >= dims.x || sy >= dims.y) {
    return vec4<f32>(0.0);
  }
  let sample = textureLoad(src_texture, vec2<i32>(i32(sx), i32(sy)), 0);
  let mdims_u = textureDimensions(mask_texture);
  let mdims = vec2<f32>(f32(mdims_u.x), f32(mdims_u.y));
  let local_x = (f32(x) + 0.5) - viewport.mask_viewport_origin.x;
  let local_y = (f32(y) + 0.5) - viewport.mask_viewport_origin.y;
  if (local_x < 0.0 || local_y < 0.0 ||
      local_x >= viewport.mask_viewport_size.x || local_y >= viewport.mask_viewport_size.y) {
    return vec4<f32>(0.0);
  }
  let mx = clamp(i32(floor(local_x * mdims.x / viewport.mask_viewport_size.x)), 0, i32(mdims_u.x) - 1);
  let my = clamp(i32(floor(local_y * mdims.y / viewport.mask_viewport_size.y)), 0, i32(mdims_u.y) - 1);
  let mask = textureLoad(mask_texture, vec2<i32>(mx, my), 0).x;
  return vec4<f32>(sample.rgb * mask, mask);
}
"#;

const CLIP_MASK_SHADER_PART_A: &str = r#"
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
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

struct Params {
  dst_size: vec2<f32>,
  _pad0: vec2<f32>,
};

@group(1) @binding(0) var<uniform> params: Params;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0),
  );
  var out: VsOut;
  out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
  return out;
}
"#;

const CLIP_MASK_SHADER_PART_B: &str = r#"
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
    alpha = alpha * sdf_coverage_smooth(sdf);
  }
  return alpha;
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) f32 {
  let x = floor(pos.x) + 0.5;
  let y = floor(pos.y) + 0.5;
  let scale = viewport.mask_viewport_size / params.dst_size;
  return clip_alpha(viewport.mask_viewport_origin + vec2<f32>(x, y) * scale);
}
"#;

pub(super) fn clip_mask_shader_source() -> String {
    format!("{CLIP_MASK_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{CLIP_MASK_SHADER_PART_B}")
}

pub(super) const COLOR_ADJUST_SHADER: &str = r#"
@group(0) @binding(0) var src_texture: texture_2d<f32>;

struct Params {
  saturation: f32,
  brightness: f32,
  contrast: f32,
  _pad: f32,
};

@group(0) @binding(1) var<uniform> params: Params;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0),
  );
  var out: VsOut;
  out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
  return out;
}

fn saturate(x: f32) -> f32 {
  return clamp(x, 0.0, 1.0);
}

fn saturate3(v: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(saturate(v.x), saturate(v.y), saturate(v.z));
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = textureDimensions(src_texture);
  let x = i32(floor(pos.x));
  let y = i32(floor(pos.y));
  if (x < 0 || y < 0 || x >= i32(dims.x) || y >= i32(dims.y)) {
    return vec4<f32>(0.0);
  }

  let tex = textureLoad(src_texture, vec2<i32>(x, y), 0);
  let a = tex.a;
  if (a <= 0.0) {
    return vec4<f32>(0.0);
  }

  var rgb = tex.rgb / a;
  let s = max(params.saturation, 0.0);
  let c = params.contrast;
  let b = params.brightness;

  // Luma coefficients (linear-ish). This pass treats the stored texture encoding as "working space"
  // to stay consistent with other fullscreen passes.
  let luma = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  rgb = mix(vec3<f32>(luma), rgb, s);
  rgb = (rgb - vec3<f32>(0.5)) * c + vec3<f32>(0.5);
  rgb = rgb + vec3<f32>(b);
  rgb = saturate3(rgb);

  return vec4<f32>(rgb * a, a);
}
"#;

const COLOR_ADJUST_MASKED_SHADER_PART_A: &str = r#"
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
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

@group(1) @binding(0) var src_texture: texture_2d<f32>;

struct Params {
  saturation: f32,
  brightness: f32,
  contrast: f32,
  _pad: f32,
};

@group(1) @binding(1) var<uniform> params: Params;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0),
  );
  var out: VsOut;
  out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
  return out;
}
"#;

const COLOR_ADJUST_MASKED_SHADER_PART_B: &str = r#"
fn saturate3(v: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(saturate(v.x), saturate(v.y), saturate(v.z));
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
    alpha = alpha * sdf_coverage_linear(sdf);
  }
  return alpha;
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = textureDimensions(src_texture);
  let x = i32(floor(pos.x));
  let y = i32(floor(pos.y));
  if (x < 0 || y < 0 || x >= i32(dims.x) || y >= i32(dims.y)) {
    return vec4<f32>(0.0);
  }

  let tex = textureLoad(src_texture, vec2<i32>(x, y), 0);
  let a = tex.a;
  if (a <= 0.0) {
    return vec4<f32>(0.0);
  }

  var rgb = tex.rgb / a;
  let s = max(params.saturation, 0.0);
  let c = params.contrast;
  let b = params.brightness;

  let luma = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  rgb = mix(vec3<f32>(luma), rgb, s);
  rgb = (rgb - vec3<f32>(0.5)) * c + vec3<f32>(0.5);
  rgb = rgb + vec3<f32>(b);
  rgb = saturate3(rgb);

  let clip = clip_alpha(pos.xy);
  return vec4<f32>(rgb * a * clip, clip);
}
"#;

pub(super) fn color_adjust_masked_shader_source() -> String {
    format!(
        "{COLOR_ADJUST_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{COLOR_ADJUST_MASKED_SHADER_PART_B}"
    )
}

pub(super) const COLOR_ADJUST_MASK_SHADER: &str = r#"
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
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

@group(1) @binding(0) var src_texture: texture_2d<f32>;

struct Params {
  saturation: f32,
  brightness: f32,
  contrast: f32,
  _pad: f32,
};

@group(1) @binding(1) var<uniform> params: Params;
@group(1) @binding(2) var mask_texture: texture_2d<f32>;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0),
  );
  var out: VsOut;
  out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
  return out;
}

fn saturate(x: f32) -> f32 {
  return clamp(x, 0.0, 1.0);
}

fn saturate3(v: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(saturate(v.x), saturate(v.y), saturate(v.z));
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = textureDimensions(src_texture);
  let x = i32(floor(pos.x));
  let y = i32(floor(pos.y));
  if (x < 0 || y < 0 || x >= i32(dims.x) || y >= i32(dims.y)) {
    return vec4<f32>(0.0);
  }

  let tex = textureLoad(src_texture, vec2<i32>(x, y), 0);
  let a = tex.a;
  if (a <= 0.0) {
    return vec4<f32>(0.0);
  }

  var rgb = tex.rgb / a;
  let s = max(params.saturation, 0.0);
  let c = params.contrast;
  let b = params.brightness;

  let luma = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  rgb = mix(vec3<f32>(luma), rgb, s);
  rgb = (rgb - vec3<f32>(0.5)) * c + vec3<f32>(0.5);
  rgb = rgb + vec3<f32>(b);
  rgb = saturate3(rgb);

  let mdims_u = textureDimensions(mask_texture);
  let mdims = vec2<f32>(f32(mdims_u.x), f32(mdims_u.y));
  let local_x = (f32(x) + 0.5) - viewport.mask_viewport_origin.x;
  let local_y = (f32(y) + 0.5) - viewport.mask_viewport_origin.y;
  if (local_x < 0.0 || local_y < 0.0 ||
      local_x >= viewport.mask_viewport_size.x || local_y >= viewport.mask_viewport_size.y) {
    return vec4<f32>(0.0);
  }
  let mx = clamp(i32(floor(local_x * mdims.x / viewport.mask_viewport_size.x)), 0, i32(mdims_u.x) - 1);
  let my = clamp(i32(floor(local_y * mdims.y / viewport.mask_viewport_size.y)), 0, i32(mdims_u.y) - 1);
  let mask = textureLoad(mask_texture, vec2<i32>(mx, my), 0).x;
  return vec4<f32>(rgb * a * mask, mask);
}
"#;

pub(super) const BLUR_H_SHADER: &str = r#"
@group(0) @binding(0) var src_texture: texture_2d<f32>;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0),
  );
  var out: VsOut;
  out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
  return out;
}

fn clamp_i32(x: i32, lo: i32, hi: i32) -> i32 {
  return min(max(x, lo), hi);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = textureDimensions(src_texture);
  let x = i32(floor(pos.x));
  let y = i32(floor(pos.y));
  if (x < 0 || y < 0 || x >= i32(dims.x) || y >= i32(dims.y)) {
    return vec4<f32>(0.0);
  }

  // 9-tap separable gaussian-ish kernel (radius 4).
  let w0 = 0.227027;
  let w1 = 0.1945946;
  let w2 = 0.1216216;
  let w3 = 0.054054;
  let w4 = 0.016216;

  let max_x = i32(dims.x) - 1;
  let sx0 = clamp_i32(x, 0, max_x);
  let c0 = textureLoad(src_texture, vec2<i32>(sx0, y), 0) * w0;

  let sx1p = clamp_i32(x + 1, 0, max_x);
  let sx1n = clamp_i32(x - 1, 0, max_x);
  let c1 = (textureLoad(src_texture, vec2<i32>(sx1p, y), 0) +
            textureLoad(src_texture, vec2<i32>(sx1n, y), 0)) * w1;

  let sx2p = clamp_i32(x + 2, 0, max_x);
  let sx2n = clamp_i32(x - 2, 0, max_x);
  let c2 = (textureLoad(src_texture, vec2<i32>(sx2p, y), 0) +
            textureLoad(src_texture, vec2<i32>(sx2n, y), 0)) * w2;

  let sx3p = clamp_i32(x + 3, 0, max_x);
  let sx3n = clamp_i32(x - 3, 0, max_x);
  let c3 = (textureLoad(src_texture, vec2<i32>(sx3p, y), 0) +
            textureLoad(src_texture, vec2<i32>(sx3n, y), 0)) * w3;

  let sx4p = clamp_i32(x + 4, 0, max_x);
  let sx4n = clamp_i32(x - 4, 0, max_x);
  let c4 = (textureLoad(src_texture, vec2<i32>(sx4p, y), 0) +
            textureLoad(src_texture, vec2<i32>(sx4n, y), 0)) * w4;

  return c0 + c1 + c2 + c3 + c4;
}
"#;

pub(super) const BLUR_V_SHADER: &str = r#"
@group(0) @binding(0) var src_texture: texture_2d<f32>;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0),
  );
  var out: VsOut;
  out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
  return out;
}

fn clamp_i32(x: i32, lo: i32, hi: i32) -> i32 {
  return min(max(x, lo), hi);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = textureDimensions(src_texture);
  let x = i32(floor(pos.x));
  let y = i32(floor(pos.y));
  if (x < 0 || y < 0 || x >= i32(dims.x) || y >= i32(dims.y)) {
    return vec4<f32>(0.0);
  }

  // 9-tap separable gaussian-ish kernel (radius 4).
  let w0 = 0.227027;
  let w1 = 0.1945946;
  let w2 = 0.1216216;
  let w3 = 0.054054;
  let w4 = 0.016216;

  let max_y = i32(dims.y) - 1;
  let sy0 = clamp_i32(y, 0, max_y);
  let c0 = textureLoad(src_texture, vec2<i32>(x, sy0), 0) * w0;

  let sy1p = clamp_i32(y + 1, 0, max_y);
  let sy1n = clamp_i32(y - 1, 0, max_y);
  let c1 = (textureLoad(src_texture, vec2<i32>(x, sy1p), 0) +
            textureLoad(src_texture, vec2<i32>(x, sy1n), 0)) * w1;

  let sy2p = clamp_i32(y + 2, 0, max_y);
  let sy2n = clamp_i32(y - 2, 0, max_y);
  let c2 = (textureLoad(src_texture, vec2<i32>(x, sy2p), 0) +
            textureLoad(src_texture, vec2<i32>(x, sy2n), 0)) * w2;

  let sy3p = clamp_i32(y + 3, 0, max_y);
  let sy3n = clamp_i32(y - 3, 0, max_y);
  let c3 = (textureLoad(src_texture, vec2<i32>(x, sy3p), 0) +
            textureLoad(src_texture, vec2<i32>(x, sy3n), 0)) * w3;

  let sy4p = clamp_i32(y + 4, 0, max_y);
  let sy4n = clamp_i32(y - 4, 0, max_y);
  let c4 = (textureLoad(src_texture, vec2<i32>(x, sy4p), 0) +
            textureLoad(src_texture, vec2<i32>(x, sy4n), 0)) * w4;

  return c0 + c1 + c2 + c3 + c4;
}
"#;

const BLUR_H_MASKED_SHADER_PART_A: &str = r#"
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
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

@group(1) @binding(0) var src_texture: texture_2d<f32>;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0),
  );
  var out: VsOut;
  out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
  return out;
}
"#;

const BLUR_H_MASKED_SHADER_PART_B: &str = r#"
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
    alpha = alpha * sdf_coverage_linear(sdf);
  }
  return alpha;
}

fn clamp_i32(x: i32, lo: i32, hi: i32) -> i32 {
  return min(max(x, lo), hi);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = textureDimensions(src_texture);
  let x = i32(floor(pos.x));
  let y = i32(floor(pos.y));
  if (x < 0 || y < 0 || x >= i32(dims.x) || y >= i32(dims.y)) {
    return vec4<f32>(0.0);
  }

  // 9-tap separable gaussian-ish kernel (radius 4).
  let w0 = 0.227027;
  let w1 = 0.1945946;
  let w2 = 0.1216216;
  let w3 = 0.054054;
  let w4 = 0.016216;

  let max_x = i32(dims.x) - 1;
  let sx0 = clamp_i32(x, 0, max_x);
  let c0 = textureLoad(src_texture, vec2<i32>(sx0, y), 0) * w0;

  let sx1p = clamp_i32(x + 1, 0, max_x);
  let sx1n = clamp_i32(x - 1, 0, max_x);
  let c1 = (textureLoad(src_texture, vec2<i32>(sx1p, y), 0) +
            textureLoad(src_texture, vec2<i32>(sx1n, y), 0)) * w1;

  let sx2p = clamp_i32(x + 2, 0, max_x);
  let sx2n = clamp_i32(x - 2, 0, max_x);
  let c2 = (textureLoad(src_texture, vec2<i32>(sx2p, y), 0) +
            textureLoad(src_texture, vec2<i32>(sx2n, y), 0)) * w2;

  let sx3p = clamp_i32(x + 3, 0, max_x);
  let sx3n = clamp_i32(x - 3, 0, max_x);
  let c3 = (textureLoad(src_texture, vec2<i32>(sx3p, y), 0) +
            textureLoad(src_texture, vec2<i32>(sx3n, y), 0)) * w3;

  let sx4p = clamp_i32(x + 4, 0, max_x);
  let sx4n = clamp_i32(x - 4, 0, max_x);
  let c4 = (textureLoad(src_texture, vec2<i32>(sx4p, y), 0) +
            textureLoad(src_texture, vec2<i32>(sx4n, y), 0)) * w4;

  let out = c0 + c1 + c2 + c3 + c4;
  let clip = clip_alpha(pos.xy);
  return vec4<f32>(out.rgb * clip, clip);
}
"#;

pub(super) fn blur_h_masked_shader_source() -> String {
    format!("{BLUR_H_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{BLUR_H_MASKED_SHADER_PART_B}")
}

const BLUR_V_MASKED_SHADER_PART_A: &str = r#"
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
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

@group(1) @binding(0) var src_texture: texture_2d<f32>;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0),
  );
  var out: VsOut;
  out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
  return out;
}
"#;

const BLUR_V_MASKED_SHADER_PART_B: &str = r#"
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
    alpha = alpha * sdf_coverage_linear(sdf);
  }
  return alpha;
}

fn clamp_i32(x: i32, lo: i32, hi: i32) -> i32 {
  return min(max(x, lo), hi);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = textureDimensions(src_texture);
  let x = i32(floor(pos.x));
  let y = i32(floor(pos.y));
  if (x < 0 || y < 0 || x >= i32(dims.x) || y >= i32(dims.y)) {
    return vec4<f32>(0.0);
  }

  // 9-tap separable gaussian-ish kernel (radius 4).
  let w0 = 0.227027;
  let w1 = 0.1945946;
  let w2 = 0.1216216;
  let w3 = 0.054054;
  let w4 = 0.016216;

  let max_y = i32(dims.y) - 1;
  let sy0 = clamp_i32(y, 0, max_y);
  let c0 = textureLoad(src_texture, vec2<i32>(x, sy0), 0) * w0;

  let sy1p = clamp_i32(y + 1, 0, max_y);
  let sy1n = clamp_i32(y - 1, 0, max_y);
  let c1 = (textureLoad(src_texture, vec2<i32>(x, sy1p), 0) +
            textureLoad(src_texture, vec2<i32>(x, sy1n), 0)) * w1;

  let sy2p = clamp_i32(y + 2, 0, max_y);
  let sy2n = clamp_i32(y - 2, 0, max_y);
  let c2 = (textureLoad(src_texture, vec2<i32>(x, sy2p), 0) +
            textureLoad(src_texture, vec2<i32>(x, sy2n), 0)) * w2;

  let sy3p = clamp_i32(y + 3, 0, max_y);
  let sy3n = clamp_i32(y - 3, 0, max_y);
  let c3 = (textureLoad(src_texture, vec2<i32>(x, sy3p), 0) +
            textureLoad(src_texture, vec2<i32>(x, sy3n), 0)) * w3;

  let sy4p = clamp_i32(y + 4, 0, max_y);
  let sy4n = clamp_i32(y - 4, 0, max_y);
  let c4 = (textureLoad(src_texture, vec2<i32>(x, sy4p), 0) +
            textureLoad(src_texture, vec2<i32>(x, sy4n), 0)) * w4;

  let out = c0 + c1 + c2 + c3 + c4;
  let clip = clip_alpha(pos.xy);
  return vec4<f32>(out.rgb * clip, clip);
}
"#;

pub(super) fn blur_v_masked_shader_source() -> String {
    format!("{BLUR_V_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{BLUR_V_MASKED_SHADER_PART_B}")
}

pub(super) const BLUR_H_MASK_SHADER: &str = r#"
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
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

@group(1) @binding(0) var src_texture: texture_2d<f32>;
@group(1) @binding(1) var mask_texture: texture_2d<f32>;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0),
  );
  var out: VsOut;
  out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
  return out;
}

fn clamp_i32(x: i32, lo: i32, hi: i32) -> i32 {
  return min(max(x, lo), hi);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = textureDimensions(src_texture);
  let x = i32(floor(pos.x));
  let y = i32(floor(pos.y));
  if (x < 0 || y < 0 || x >= i32(dims.x) || y >= i32(dims.y)) {
    return vec4<f32>(0.0);
  }

  // 9-tap separable gaussian-ish kernel (radius 4).
  let w0 = 0.227027;
  let w1 = 0.1945946;
  let w2 = 0.1216216;
  let w3 = 0.054054;
  let w4 = 0.016216;

  let max_x = i32(dims.x) - 1;
  let sx0 = clamp_i32(x, 0, max_x);
  let c0 = textureLoad(src_texture, vec2<i32>(sx0, y), 0) * w0;

  let sx1p = clamp_i32(x + 1, 0, max_x);
  let sx1n = clamp_i32(x - 1, 0, max_x);
  let c1 = (textureLoad(src_texture, vec2<i32>(sx1p, y), 0) +
            textureLoad(src_texture, vec2<i32>(sx1n, y), 0)) * w1;

  let sx2p = clamp_i32(x + 2, 0, max_x);
  let sx2n = clamp_i32(x - 2, 0, max_x);
  let c2 = (textureLoad(src_texture, vec2<i32>(sx2p, y), 0) +
            textureLoad(src_texture, vec2<i32>(sx2n, y), 0)) * w2;

  let sx3p = clamp_i32(x + 3, 0, max_x);
  let sx3n = clamp_i32(x - 3, 0, max_x);
  let c3 = (textureLoad(src_texture, vec2<i32>(sx3p, y), 0) +
            textureLoad(src_texture, vec2<i32>(sx3n, y), 0)) * w3;

  let sx4p = clamp_i32(x + 4, 0, max_x);
  let sx4n = clamp_i32(x - 4, 0, max_x);
  let c4 = (textureLoad(src_texture, vec2<i32>(sx4p, y), 0) +
            textureLoad(src_texture, vec2<i32>(sx4n, y), 0)) * w4;

  let out = c0 + c1 + c2 + c3 + c4;
  let mdims_u = textureDimensions(mask_texture);
  let mdims = vec2<f32>(f32(mdims_u.x), f32(mdims_u.y));
  let local_x = (f32(x) + 0.5) - viewport.mask_viewport_origin.x;
  let local_y = (f32(y) + 0.5) - viewport.mask_viewport_origin.y;
  if (local_x < 0.0 || local_y < 0.0 ||
      local_x >= viewport.mask_viewport_size.x || local_y >= viewport.mask_viewport_size.y) {
    return vec4<f32>(0.0);
  }
  let mx = clamp(i32(floor(local_x * mdims.x / viewport.mask_viewport_size.x)), 0, i32(mdims_u.x) - 1);
  let my = clamp(i32(floor(local_y * mdims.y / viewport.mask_viewport_size.y)), 0, i32(mdims_u.y) - 1);
  let mask = textureLoad(mask_texture, vec2<i32>(mx, my), 0).x;
  return vec4<f32>(out.rgb * mask, mask);
}
"#;

pub(super) const BLUR_V_MASK_SHADER: &str = r#"
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
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

@group(1) @binding(0) var src_texture: texture_2d<f32>;
@group(1) @binding(1) var mask_texture: texture_2d<f32>;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0),
  );
  var out: VsOut;
  out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
  return out;
}

fn clamp_i32(x: i32, lo: i32, hi: i32) -> i32 {
  return min(max(x, lo), hi);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = textureDimensions(src_texture);
  let x = i32(floor(pos.x));
  let y = i32(floor(pos.y));
  if (x < 0 || y < 0 || x >= i32(dims.x) || y >= i32(dims.y)) {
    return vec4<f32>(0.0);
  }

  // 9-tap separable gaussian-ish kernel (radius 4).
  let w0 = 0.227027;
  let w1 = 0.1945946;
  let w2 = 0.1216216;
  let w3 = 0.054054;
  let w4 = 0.016216;

  let max_y = i32(dims.y) - 1;
  let sy0 = clamp_i32(y, 0, max_y);
  let c0 = textureLoad(src_texture, vec2<i32>(x, sy0), 0) * w0;

  let sy1p = clamp_i32(y + 1, 0, max_y);
  let sy1n = clamp_i32(y - 1, 0, max_y);
  let c1 = (textureLoad(src_texture, vec2<i32>(x, sy1p), 0) +
            textureLoad(src_texture, vec2<i32>(x, sy1n), 0)) * w1;

  let sy2p = clamp_i32(y + 2, 0, max_y);
  let sy2n = clamp_i32(y - 2, 0, max_y);
  let c2 = (textureLoad(src_texture, vec2<i32>(x, sy2p), 0) +
            textureLoad(src_texture, vec2<i32>(x, sy2n), 0)) * w2;

  let sy3p = clamp_i32(y + 3, 0, max_y);
  let sy3n = clamp_i32(y - 3, 0, max_y);
  let c3 = (textureLoad(src_texture, vec2<i32>(x, sy3p), 0) +
            textureLoad(src_texture, vec2<i32>(x, sy3n), 0)) * w3;

  let sy4p = clamp_i32(y + 4, 0, max_y);
  let sy4n = clamp_i32(y - 4, 0, max_y);
  let c4 = (textureLoad(src_texture, vec2<i32>(x, sy4p), 0) +
            textureLoad(src_texture, vec2<i32>(x, sy4n), 0)) * w4;

  let out = c0 + c1 + c2 + c3 + c4;
  let mdims_u = textureDimensions(mask_texture);
  let mdims = vec2<f32>(f32(mdims_u.x), f32(mdims_u.y));
  let local_x = (f32(x) + 0.5) - viewport.mask_viewport_origin.x;
  let local_y = (f32(y) + 0.5) - viewport.mask_viewport_origin.y;
  if (local_x < 0.0 || local_y < 0.0 ||
      local_x >= viewport.mask_viewport_size.x || local_y >= viewport.mask_viewport_size.y) {
    return vec4<f32>(0.0);
  }
  let mx = clamp(i32(floor(local_x * mdims.x / viewport.mask_viewport_size.x)), 0, i32(mdims_u.x) - 1);
  let my = clamp(i32(floor(local_y * mdims.y / viewport.mask_viewport_size.y)), 0, i32(mdims_u.y) - 1);
  let mask = textureLoad(mask_texture, vec2<i32>(mx, my), 0).x;
  return vec4<f32>(out.rgb * mask, mask);
}
"#;

pub(super) const COMPOSITE_PREMUL_SHADER: &str = r#"
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
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

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
  let ndc_x = (pixel_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (pixel_pos.y / viewport.viewport_size.y) * 2.0;
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

fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
  let a = 0.055;
  let lo = rgb * 12.92;
  let hi = (1.0 + a) * pow(rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(a);
  return select(hi, lo, rgb <= vec3<f32>(0.0031308));
}

fn encode_output_premul(c: vec4<f32>) -> vec4<f32> {
  if (viewport.output_is_srgb != 0u) {
    return c;
  }
  if (c.a <= 0.0) {
    return c;
  }
  let un = c.rgb / c.a;
  let enc = linear_to_srgb(un);
  return vec4<f32>(enc * c.a, c.a);
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
  let sample = textureSample(tex, tex_sampler, input.uv);
  let o = clamp(input.opacity, 0.0, 1.0);
  let out = vec4<f32>(sample.rgb * o, sample.a * o) * clip;
  return encode_output_premul(out);
}
"#;

pub(super) const COMPOSITE_PREMUL_MASK_SHADER: &str = r#"
struct Viewport {
  viewport_size: vec2<f32>,
  clip_head: u32,
  clip_count: u32,
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;

@group(1) @binding(0) var tex_sampler: sampler;
@group(1) @binding(1) var tex: texture_2d<f32>;
@group(1) @binding(2) var mask_texture: texture_2d<f32>;

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
  let ndc_x = (pixel_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (pixel_pos.y / viewport.viewport_size.y) * 2.0;
  return vec2<f32>(ndc_x, ndc_y);
}

fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
  let a = 0.055;
  let lo = rgb * 12.92;
  let hi = (1.0 + a) * pow(rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(a);
  return select(hi, lo, rgb <= vec3<f32>(0.0031308));
}

fn encode_output_premul(c: vec4<f32>) -> vec4<f32> {
  if (viewport.output_is_srgb != 0u) {
    return c;
  }
  if (c.a <= 0.0) {
    return c;
  }
  let un = c.rgb / c.a;
  let enc = linear_to_srgb(un);
  return vec4<f32>(enc * c.a, c.a);
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
  let x = i32(floor(input.pixel_pos.x));
  let y = i32(floor(input.pixel_pos.y));
  let mdims_u = textureDimensions(mask_texture);
  let mdims = vec2<f32>(f32(mdims_u.x), f32(mdims_u.y));
  let local_x = (f32(x) + 0.5) - viewport.mask_viewport_origin.x;
  let local_y = (f32(y) + 0.5) - viewport.mask_viewport_origin.y;
  let inside = local_x >= 0.0 && local_y >= 0.0 &&
      local_x < viewport.mask_viewport_size.x && local_y < viewport.mask_viewport_size.y;
  let mx = clamp(i32(floor(local_x * mdims.x / viewport.mask_viewport_size.x)), 0, i32(mdims_u.x) - 1);
  let my = clamp(i32(floor(local_y * mdims.y / viewport.mask_viewport_size.y)), 0, i32(mdims_u.y) - 1);
  let sample = textureSample(tex, tex_sampler, input.uv);
  let mask = textureLoad(mask_texture, vec2<i32>(mx, my), 0).x * select(0.0, 1.0, inside);
  let o = clamp(input.opacity, 0.0, 1.0);
  let out = vec4<f32>(sample.rgb * o, sample.a * o) * mask;
  return encode_output_premul(out);
}
"#;

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
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

struct VsIn {
  @location(0) pos_px: vec2<f32>,
  @location(1) color: vec4<f32>,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) color: vec4<f32>,
  @location(1) pixel_pos: vec2<f32>,
};

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let ndc_x = (pixel_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (pixel_pos.y / viewport.viewport_size.y) * 2.0;
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

fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
  let a = 0.055;
  let lo = rgb * 12.92;
  let hi = (1.0 + a) * pow(rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(a);
  return select(hi, lo, rgb <= vec3<f32>(0.0031308));
}

fn encode_output_premul(c: vec4<f32>) -> vec4<f32> {
  if (viewport.output_is_srgb != 0u) {
    return c;
  }
  if (c.a <= 0.0) {
    return c;
  }
  let un = c.rgb / c.a;
  let enc = linear_to_srgb(un);
  return vec4<f32>(enc * c.a, c.a);
}

@vertex
fn vs_main(input: VsIn) -> VsOut {
  var out: VsOut;
  let clip_xy = to_clip_space(input.pos_px);
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.color = input.color;
  out.pixel_pos = input.pos_px;
  return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let clip = clip_alpha(input.pixel_pos);
  let out = input.color * clip;
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
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
  text_gamma_ratios: vec4<f32>,
  text_grayscale_enhanced_contrast: f32,
  text_subpixel_enhanced_contrast: f32,
  _pad_text_quality0: u32,
  _pad_text_quality1: u32,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

@group(1) @binding(0) var glyph_sampler: sampler;
@group(1) @binding(1) var glyph_atlas: texture_2d<f32>;

struct VsIn {
  @location(0) pos_px: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) color: vec4<f32>,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) pixel_pos: vec2<f32>,
};

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let ndc_x = (pixel_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (pixel_pos.y / viewport.viewport_size.y) * 2.0;
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

fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
  let a = 0.055;
  let lo = rgb * 12.92;
  let hi = (1.0 + a) * pow(rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(a);
  return select(hi, lo, rgb <= vec3<f32>(0.0031308));
}

// Contrast and gamma correction adapted from the Microsoft Terminal alpha correction work
// (via Zed/GPUI). See ADR 0029/0109/0157.
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
  if (viewport.output_is_srgb != 0u) {
    return c;
  }
  if (c.a <= 0.0) {
    return c;
  }
  let un = c.rgb / c.a;
  let enc = linear_to_srgb(un);
  return vec4<f32>(enc * c.a, c.a);
}

@vertex
fn vs_main(input: VsIn) -> VsOut {
  var out: VsOut;
  let clip_xy = to_clip_space(input.pos_px);
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.uv = input.uv;
  out.color = input.color;
  out.pixel_pos = input.pos_px;
  return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let clip = clip_alpha(input.pixel_pos);
  let tex = textureSample(glyph_atlas, glyph_sampler, input.uv);
  let coverage = apply_contrast_and_gamma_correction(tex.r, input.color.rgb);
  let out = vec4<f32>(input.color.rgb * coverage, input.color.a * coverage) * clip;
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
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
  text_gamma_ratios: vec4<f32>,
  text_grayscale_enhanced_contrast: f32,
  text_subpixel_enhanced_contrast: f32,
  _pad_text_quality0: u32,
  _pad_text_quality1: u32,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

@group(1) @binding(0) var glyph_sampler: sampler;
@group(1) @binding(1) var glyph_atlas: texture_2d<f32>;

struct VsIn {
  @location(0) pos_px: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) color: vec4<f32>,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) pixel_pos: vec2<f32>,
};

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let ndc_x = (pixel_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (pixel_pos.y / viewport.viewport_size.y) * 2.0;
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

fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
  let a = 0.055;
  let lo = rgb * 12.92;
  let hi = (1.0 + a) * pow(rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(a);
  return select(hi, lo, rgb <= vec3<f32>(0.0031308));
}

fn encode_output_premul(c: vec4<f32>) -> vec4<f32> {
  if (viewport.output_is_srgb != 0u) {
    return c;
  }
  if (c.a <= 0.0) {
    return c;
  }
  let un = c.rgb / c.a;
  let enc = linear_to_srgb(un);
  return vec4<f32>(enc * c.a, c.a);
}

@vertex
fn vs_main(input: VsIn) -> VsOut {
  var out: VsOut;
  let clip_xy = to_clip_space(input.pos_px);
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.uv = input.uv;
  out.color = input.color;
  out.pixel_pos = input.pos_px;
  return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let clip = clip_alpha(input.pixel_pos);
  let tex = textureSample(glyph_atlas, glyph_sampler, input.uv);
  let a = tex.a * input.color.a;
  let out = vec4<f32>(tex.rgb * a, a) * clip;
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
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
  text_gamma_ratios: vec4<f32>,
  text_grayscale_enhanced_contrast: f32,
  text_subpixel_enhanced_contrast: f32,
  _pad_text_quality0: u32,
  _pad_text_quality1: u32,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

@group(1) @binding(0) var glyph_sampler: sampler;
@group(1) @binding(1) var glyph_atlas: texture_2d<f32>;

struct VsIn {
  @location(0) pos_px: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) color: vec4<f32>,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) pixel_pos: vec2<f32>,
};

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let ndc_x = (pixel_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (pixel_pos.y / viewport.viewport_size.y) * 2.0;
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

fn encode_output_premul(c: vec4<f32>) -> vec4<f32> {
  if (viewport.output_is_srgb != 0u) {
    return c;
  }
  if (c.a <= 0.0) {
    return c;
  }
  let un = c.rgb / c.a;
  let enc = linear_to_srgb(un);
  return vec4<f32>(enc * c.a, c.a);
}

@vertex
fn vs_main(input: VsIn) -> VsOut {
  var out: VsOut;
  let clip_xy = to_clip_space(input.pos_px);
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.uv = input.uv;
  out.color = input.color;
  out.pixel_pos = input.pos_px;
  return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let clip = clip_alpha(input.pixel_pos);
  let tex = textureSample(glyph_atlas, glyph_sampler, input.uv);
  let coverage = apply_contrast_and_gamma_correction3(tex.rgb, input.color.rgb);
  let a = max(max(coverage.r, coverage.g), coverage.b);
  let out = vec4<f32>(input.color.rgb * coverage, input.color.a * a) * clip;
  return encode_output_premul(out);
}
"#;

pub(super) const MASK_SHADER: &str = r#"
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
  output_is_srgb: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

@group(1) @binding(0) var mask_sampler: sampler;
@group(1) @binding(1) var mask_texture: texture_2d<f32>;

struct VsIn {
  @location(0) pos_px: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) color: vec4<f32>,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) pixel_pos: vec2<f32>,
};

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let ndc_x = (pixel_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (pixel_pos.y / viewport.viewport_size.y) * 2.0;
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

fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
  let a = 0.055;
  let lo = rgb * 12.92;
  let hi = (1.0 + a) * pow(rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(a);
  return select(hi, lo, rgb <= vec3<f32>(0.0031308));
}

fn encode_output_premul(c: vec4<f32>) -> vec4<f32> {
  if (viewport.output_is_srgb != 0u) {
    return c;
  }
  if (c.a <= 0.0) {
    return c;
  }
  let un = c.rgb / c.a;
  let enc = linear_to_srgb(un);
  return vec4<f32>(enc * c.a, c.a);
}

@vertex
fn vs_main(input: VsIn) -> VsOut {
  var out: VsOut;
  let clip_xy = to_clip_space(input.pos_px);
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.uv = input.uv;
  out.color = input.color;
  out.pixel_pos = input.pos_px;
  return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let clip = clip_alpha(input.pixel_pos);
  let tex = textureSample(mask_texture, mask_sampler, input.uv);
  let coverage = tex.r;
  let out = vec4<f32>(input.color.rgb * coverage, input.color.a * coverage) * clip;
  return encode_output_premul(out);
}
"#;
