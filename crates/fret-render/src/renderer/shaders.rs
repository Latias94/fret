pub(super) const QUAD_SHADER: &str = r#"
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
};

@group(0) @binding(0) var<uniform> viewport: Viewport;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

struct QuadInstance {
  rect: vec4<f32>,
  transform0: vec4<f32>,
  transform1: vec4<f32>,
  color: vec4<f32>,
  corner_radii: vec4<f32>,
  border: vec4<f32>,
  border_color: vec4<f32>,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) pixel_pos: vec2<f32>,
  @location(1) local_pos: vec2<f32>,
  @location(2) rect: vec4<f32>,
  @location(3) color: vec4<f32>,
  @location(4) corner_radii: vec4<f32>,
  @location(5) border: vec4<f32>,
  @location(6) border_color: vec4<f32>,
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
  @location(0) rect: vec4<f32>,
  @location(1) transform0: vec4<f32>,
  @location(2) transform1: vec4<f32>,
  @location(3) color: vec4<f32>,
  @location(4) corner_radii: vec4<f32>,
  @location(5) border: vec4<f32>,
  @location(6) border_color: vec4<f32>,
) -> VsOut {
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
  out.color = color;
  out.corner_radii = corner_radii;
  out.border = border;
  out.border_color = border_color;
  return out;
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
  for (var i = 0u; i < viewport.clip_count; i = i + 1u) {
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

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let clip = clip_alpha(input.pixel_pos);

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

  let fill = vec4<f32>(input.color.rgb, input.color.a) * alpha_fill;
  let border = vec4<f32>(input.border_color.rgb, input.border_color.a) * border_cov;

  let out = (fill + border) * clip;
  if (clip <= 0.0) {
    discard;
  }
  return out;
}
"#;

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
  for (var i = 0u; i < viewport.clip_count; i = i + 1u) {
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
  if (clip <= 0.0) {
    discard;
  }
  let tex = textureSample(viewport_texture, viewport_sampler, input.uv);
  let a = tex.a * input.opacity * clip;
  return vec4<f32>(tex.rgb * a, a);
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
  for (var i = 0u; i < viewport.clip_count; i = i + 1u) {
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
  if (clip <= 0.0) {
    discard;
  }
  let sample = textureSample(tex, tex_sampler, input.uv);
  let o = clamp(input.opacity, 0.0, 1.0);
  return vec4<f32>(sample.rgb * o, sample.a * o) * clip;
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
  for (var i = 0u; i < viewport.clip_count; i = i + 1u) {
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
  if (clip <= 0.0) {
    discard;
  }
  return input.color * clip;
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
  for (var i = 0u; i < viewport.clip_count; i = i + 1u) {
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
  if (clip <= 0.0) {
    discard;
  }
  let tex = textureSample(glyph_atlas, glyph_sampler, input.uv);
  let coverage = tex.r;
  return vec4<f32>(input.color.rgb * coverage, input.color.a * coverage) * clip;
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
  for (var i = 0u; i < viewport.clip_count; i = i + 1u) {
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
  if (clip <= 0.0) {
    discard;
  }
  let tex = textureSample(glyph_atlas, glyph_sampler, input.uv);
  let a = tex.a * input.color.a;
  return vec4<f32>(tex.rgb * a, a) * clip;
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
  for (var i = 0u; i < viewport.clip_count; i = i + 1u) {
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
  if (clip <= 0.0) {
    discard;
  }
  let tex = textureSample(mask_texture, mask_sampler, input.uv);
  let coverage = tex.r;
  return vec4<f32>(input.color.rgb * coverage, input.color.a * coverage) * clip;
}
"#;
