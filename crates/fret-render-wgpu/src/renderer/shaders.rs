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
  params0: vec4<f32>,
  params1: vec4<f32>,
  params2: vec4<f32>,
  params3: vec4<f32>,
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
  dash_params: vec4<f32>,
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
  let local = pixel_pos - render_space.origin_px;
  let ndc_x = (local.x / render_space.size_px.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (local.y / render_space.size_px.y) * 2.0;
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

fn dist2(a: vec2<f32>, b: vec2<f32>) -> f32 {
  let d = a - b;
  return dot(d, d);
}

fn rrect_perimeter_s(p: vec2<f32>, rect: vec4<f32>, corner_radii: vec4<f32>) -> f32 {
  let pi = 3.141592653589793;
  let half_pi = 1.5707963267948966;
  let two_pi = 6.283185307179586;

  let x0 = rect.x;
  let y0 = rect.y;
  let w = rect.z;
  let h = rect.w;
  let x1 = x0 + w;
  let y1 = y0 + h;

  let r_tl = max(corner_radii.x, 0.0);
  let r_tr = max(corner_radii.y, 0.0);
  let r_br = max(corner_radii.z, 0.0);
  let r_bl = max(corner_radii.w, 0.0);

  let l_top = max(0.0, w - r_tl - r_tr);
  let l_right = max(0.0, h - r_tr - r_br);
  let l_bottom = max(0.0, w - r_bl - r_br);
  let l_left = max(0.0, h - r_tl - r_bl);

  let l_tl = half_pi * r_tl;
  let l_tr = half_pi * r_tr;
  let l_br = half_pi * r_br;
  let l_bl = half_pi * r_bl;

  let off_top = 0.0;
  let off_tr = off_top + l_top;
  let off_right = off_tr + l_tr;
  let off_br = off_right + l_right;
  let off_bottom = off_br + l_br;
  let off_bl = off_bottom + l_bottom;
  let off_left = off_bl + l_bl;
  let off_tl = off_left + l_left;

  // Straight segments (clamped to segment extents).
  let q_top = vec2<f32>(clamp(p.x, x0 + r_tl, x1 - r_tr), y0);
  let q_right = vec2<f32>(x1, clamp(p.y, y0 + r_tr, y1 - r_br));
  let q_bottom = vec2<f32>(clamp(p.x, x0 + r_bl, x1 - r_br), y1);
  let q_left = vec2<f32>(x0, clamp(p.y, y0 + r_tl, y1 - r_bl));

  let s_top = off_top + (q_top.x - (x0 + r_tl));
  let s_right = off_right + (q_right.y - (y0 + r_tr));
  let s_bottom = off_bottom + ((x1 - r_br) - q_bottom.x);
  let s_left = off_left + ((y1 - r_bl) - q_left.y);

  var best_d = dist2(p, q_top);
  var best_s = s_top;

  let d_right = dist2(p, q_right);
  if (d_right < best_d) {
    best_d = d_right;
    best_s = s_right;
  }

  let d_bottom = dist2(p, q_bottom);
  if (d_bottom < best_d) {
    best_d = d_bottom;
    best_s = s_bottom;
  }

  let d_left = dist2(p, q_left);
  if (d_left < best_d) {
    best_d = d_left;
    best_s = s_left;
  }

  // Corner arcs (angle-clamped to the quarter arc extents).
  if (r_tr > 0.0) {
    let c = vec2<f32>(x1 - r_tr, y0 + r_tr);
    let a0 = atan2(p.y - c.y, p.x - c.x);
    let a = clamp(a0, -half_pi, 0.0);
    let q = c + vec2<f32>(cos(a), sin(a)) * r_tr;
    let d = dist2(p, q);
    if (d < best_d) {
      let t = (a + half_pi) / half_pi;
      best_d = d;
      best_s = off_tr + t * l_tr;
    }
  }

  if (r_br > 0.0) {
    let c = vec2<f32>(x1 - r_br, y1 - r_br);
    let a0 = atan2(p.y - c.y, p.x - c.x);
    let a = clamp(a0, 0.0, half_pi);
    let q = c + vec2<f32>(cos(a), sin(a)) * r_br;
    let d = dist2(p, q);
    if (d < best_d) {
      let t = a / half_pi;
      best_d = d;
      best_s = off_br + t * l_br;
    }
  }

  if (r_bl > 0.0) {
    let c = vec2<f32>(x0 + r_bl, y1 - r_bl);
    let a0 = atan2(p.y - c.y, p.x - c.x);
    let a = clamp(a0, half_pi, pi);
    let q = c + vec2<f32>(cos(a), sin(a)) * r_bl;
    let d = dist2(p, q);
    if (d < best_d) {
      let t = (a - half_pi) / half_pi;
      best_d = d;
      best_s = off_bl + t * l_bl;
    }
  }

  if (r_tl > 0.0) {
    let c = vec2<f32>(x0 + r_tl, y0 + r_tl);
    var a0 = atan2(p.y - c.y, p.x - c.x);
    // Wrap TL's left endpoint from `π` to `-π` so the clamp range is monotonic.
    if (a0 > half_pi) {
      a0 = a0 - two_pi;
    }
    let a = clamp(a0, -pi, -half_pi);
    let q = c + vec2<f32>(cos(a), sin(a)) * r_tl;
    let d = dist2(p, q);
    if (d < best_d) {
      let t = (a + pi) / half_pi; // [-π..-π/2] -> [0..1]
      best_s = off_tl + t * l_tl;
    }
  }

  return best_s;
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
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 2u) {
    let center = m.params0.xy;
    let radius = max(m.params0.zw, vec2<f32>(1e-6));
    let d = (local_pos - center) / radius;
    let t = length(d);
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 3u) {
    let uv0 = m.params0.xy;
    let uv1 = m.params0.zw;
    let denom = max(m.bounds.zw, vec2<f32>(1e-6));
    let t = clamp(p / denom, vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = mix(uv0, uv1, t);
    let s = textureSample(mask_image_texture, mask_image_sampler, uv);
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

fn paint_eval(p: Paint, local_pos: vec2<f32>) -> vec4<f32> {
  // WebGPU/Tint uniformity rule: derivative ops (e.g. fwidth) must not be called from
  // control flow that depends on non-uniform values (e.g. storage-buffer driven enums).
  // Keep the evaluation branchless and select the final result by kind/tile_mode.

  let kind = p.kind;

  // 0 = Solid
  let solid = p.params0;

  // 1 = LinearGradient
  let lg_start = p.params0.xy;
  let lg_end = p.params0.zw;
  let lg_dir = lg_end - lg_start;
  let lg_len2 = dot(lg_dir, lg_dir);
  let lg_t = select(0.0, dot(local_pos - lg_start, lg_dir) / lg_len2, lg_len2 > 1e-6);
  let linear = paint_sample_stops(p, clamp(lg_t, 0.0, 1.0));

  // 2 = RadialGradient
  let rg_center = p.params0.xy;
  let rg_radius = max(p.params0.zw, vec2<f32>(1e-6));
  let rg_d = (local_pos - rg_center) / rg_radius;
  let radial = paint_sample_stops(p, clamp(length(rg_d), 0.0, 1.0));

  // 3 = Material (Tier B procedural patterns)
  let base = p.params0;
  let fg = p.params1;
  let pos = local_pos + p.params3.zw;
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
  let noise_xi = noise_cell.x & 63u;
  let noise_yi = noise_cell.y & 63u;
  let noise_uv = (vec2<f32>(f32(noise_xi) + 0.5, f32(noise_yi) + 0.5) / 64.0);
  let noise_layer = select(0, i32(p.color_space), p.stop_count == 1u);
  let noise_r1 = textureSample(material_catalog_texture, material_catalog_sampler, noise_uv, noise_layer).r;
  let noise_r = select(noise_r0, noise_r1, p.stop_count == 1u);
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

  var out = solid;
  out = select(out, linear, kind == 1u);
  out = select(out, radial, kind == 2u);
  out = select(out, material, kind == 3u);
  return select(vec4<f32>(0.0), out, kind <= 3u);
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let clip = clip_alpha(input.pixel_pos);
  let mask = mask_alpha(input.pixel_pos);
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
  let dash_enabled = inst.dash_params.w > 0.5 && border_present;
  let dash = inst.dash_params.x;
  let gap = inst.dash_params.y;
  let phase = inst.dash_params.z;
  let period = dash + gap;
  let dash_valid = dash_enabled && period > 0.0 && dash > 0.0;
  let period_safe = max(period, 1e-6);
  let s = rrect_perimeter_s(input.local_pos, input.rect, input.corner_radii);
  let tt = s + phase;
  let m = tt - floor(tt / period_safe) * period_safe;
  let aa = max(fwidth(s), 1e-4);
  let on_start = smoothstep(0.0, aa, m);
  let on_end = 1.0 - smoothstep(dash - aa, dash + aa, m);
  let dash_mask = select(1.0, on_start * on_end, dash_valid);
  let border = paint_eval(inst.border_paint, input.local_pos) * border_cov * dash_mask;

  let out = (fill + border) * clip * mask;
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

fn mask_eval(m: MaskGradient, pixel_pos: vec2<f32>) -> f32 {
  let local_pos = vec2<f32>(
    dot(m.inv0.xy, pixel_pos) + m.inv0.z,
    dot(m.inv1.xy, pixel_pos) + m.inv1.z
  );

  let p = local_pos - m.bounds.xy;
  let in_bounds = p.x >= 0.0 && p.y >= 0.0 && p.x <= m.bounds.z && p.y <= m.bounds.w;

  // 1 = LinearGradient
  if (m.kind == 1u) {
    let start = m.params0.xy;
    let end = m.params0.zw;
    let dir = end - start;
    let len2 = dot(dir, dir);
    let t = select(0.0, dot(local_pos - start, dir) / len2, len2 > 1e-6);
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  // 2 = RadialGradient
  if (m.kind == 2u) {
    let center = m.params0.xy;
    let radius = max(m.params0.zw, vec2<f32>(1e-6));
    let d = (local_pos - center) / radius;
    let t = length(d);
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  // 3 = Image mask
  if (m.kind == 3u) {
    let uv0 = m.params0.xy;
    let uv1 = m.params0.zw;
    let denom = max(m.bounds.zw, vec2<f32>(1e-6));
    let t = clamp(p / denom, vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = mix(uv0, uv1, t);
    let s = textureSample(mask_image_texture, mask_image_sampler, uv);
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
  let mask = mask_alpha(input.pixel_pos);
  let tex = textureSample(viewport_texture, viewport_sampler, input.uv);
  let factor = input.opacity * clip * mask;
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
  mask_head: u32,
  mask_count: u32,
  mask_scope_head: u32,
  mask_scope_count: u32,
  output_is_srgb: u32,
  _pad0: u32,
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
  mask_head: u32,
  mask_count: u32,
  mask_scope_head: u32,
  mask_scope_count: u32,
  output_is_srgb: u32,
  _pad0: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
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
  mask_head: u32,
  mask_count: u32,
  mask_scope_head: u32,
  mask_scope_count: u32,
  output_is_srgb: u32,
  _pad0: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
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

pub(super) const COLOR_MATRIX_SHADER: &str = r#"
@group(0) @binding(0) var src_texture: texture_2d<f32>;

struct Params {
  row0: vec4<f32>,
  row1: vec4<f32>,
  row2: vec4<f32>,
  row3: vec4<f32>,
  bias: vec4<f32>,
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

fn saturate4(v: vec4<f32>) -> vec4<f32> {
  return vec4<f32>(saturate(v.x), saturate(v.y), saturate(v.z), saturate(v.w));
}

fn apply_color_matrix(tex: vec4<f32>) -> vec4<f32> {
  let a = tex.a;
  if (a <= 0.0) {
    return vec4<f32>(0.0);
  }
  let rgb = tex.rgb / a;
  let v = vec4<f32>(rgb, a);
  let out = vec4<f32>(
    dot(params.row0, v) + params.bias.x,
    dot(params.row1, v) + params.bias.y,
    dot(params.row2, v) + params.bias.z,
    dot(params.row3, v) + params.bias.w,
  );
  let clamped = saturate4(out);
  return vec4<f32>(clamped.rgb * clamped.a, clamped.a);
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
  return apply_color_matrix(tex);
}
"#;

const COLOR_MATRIX_MASKED_SHADER_PART_A: &str = r#"
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

@group(1) @binding(0) var src_texture: texture_2d<f32>;

struct Params {
  row0: vec4<f32>,
  row1: vec4<f32>,
  row2: vec4<f32>,
  row3: vec4<f32>,
  bias: vec4<f32>,
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

fn saturate4(v: vec4<f32>) -> vec4<f32> {
  return clamp(v, vec4<f32>(0.0), vec4<f32>(1.0));
}
"#;

const COLOR_MATRIX_MASKED_SHADER_PART_B: &str = r#"
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

fn apply_color_matrix(tex: vec4<f32>) -> vec4<f32> {
  let a = tex.a;
  if (a <= 0.0) {
    return vec4<f32>(0.0);
  }
  let rgb = tex.rgb / a;
  let v = vec4<f32>(rgb, a);
  let out = vec4<f32>(
    dot(params.row0, v) + params.bias.x,
    dot(params.row1, v) + params.bias.y,
    dot(params.row2, v) + params.bias.z,
    dot(params.row3, v) + params.bias.w,
  );
  let clamped = saturate4(out);
  return vec4<f32>(clamped.rgb * clamped.a, clamped.a);
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
  let out = apply_color_matrix(tex);

  let clip = clip_alpha(pos.xy);
  return vec4<f32>(out.rgb * clip, clip);
}
"#;

pub(super) fn color_matrix_masked_shader_source() -> String {
    format!(
        "{COLOR_MATRIX_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{COLOR_MATRIX_MASKED_SHADER_PART_B}"
    )
}

pub(super) const COLOR_MATRIX_MASK_SHADER: &str = r#"
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

@group(1) @binding(0) var src_texture: texture_2d<f32>;

struct Params {
  row0: vec4<f32>,
  row1: vec4<f32>,
  row2: vec4<f32>,
  row3: vec4<f32>,
  bias: vec4<f32>,
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

fn saturate4(v: vec4<f32>) -> vec4<f32> {
  return vec4<f32>(saturate(v.x), saturate(v.y), saturate(v.z), saturate(v.w));
}

fn apply_color_matrix(tex: vec4<f32>) -> vec4<f32> {
  let a = tex.a;
  if (a <= 0.0) {
    return vec4<f32>(0.0);
  }
  let rgb = tex.rgb / a;
  let v = vec4<f32>(rgb, a);
  let out = vec4<f32>(
    dot(params.row0, v) + params.bias.x,
    dot(params.row1, v) + params.bias.y,
    dot(params.row2, v) + params.bias.z,
    dot(params.row3, v) + params.bias.w,
  );
  let clamped = saturate4(out);
  return vec4<f32>(clamped.rgb * clamped.a, clamped.a);
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
  let out = apply_color_matrix(tex);

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

pub(super) const ALPHA_THRESHOLD_SHADER: &str = r#"
@group(0) @binding(0) var src_texture: texture_2d<f32>;

struct Params {
  cutoff: f32,
  soft: f32,
  _pad0: f32,
  _pad1: f32,
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

fn threshold_t(a: f32) -> f32 {
  if (params.soft <= 0.0) {
    return select(0.0, 1.0, a >= params.cutoff);
  }
  return smoothstep(params.cutoff - params.soft, params.cutoff + params.soft, a);
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
  let t = threshold_t(tex.a);
  return tex * t;
}
"#;

const ALPHA_THRESHOLD_MASKED_SHADER_PART_A: &str = r#"
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

@group(1) @binding(0) var src_texture: texture_2d<f32>;

struct Params {
  cutoff: f32,
  soft: f32,
  _pad0: f32,
  _pad1: f32,
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

const ALPHA_THRESHOLD_MASKED_SHADER_PART_B: &str = r#"
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

fn threshold_t(a: f32) -> f32 {
  if (params.soft <= 0.0) {
    return select(0.0, 1.0, a >= params.cutoff);
  }
  return smoothstep(params.cutoff - params.soft, params.cutoff + params.soft, a);
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
  let t = threshold_t(tex.a);
  let out = tex * t;

  let clip = clip_alpha(pos.xy);
  return vec4<f32>(out.rgb * clip, clip);
}
"#;

pub(super) fn alpha_threshold_masked_shader_source() -> String {
    format!(
        "{ALPHA_THRESHOLD_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{ALPHA_THRESHOLD_MASKED_SHADER_PART_B}"
    )
}

pub(super) const ALPHA_THRESHOLD_MASK_SHADER: &str = r#"
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

@group(1) @binding(0) var src_texture: texture_2d<f32>;

struct Params {
  cutoff: f32,
  soft: f32,
  _pad0: f32,
  _pad1: f32,
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

fn threshold_t(a: f32) -> f32 {
  if (params.soft <= 0.0) {
    return select(0.0, 1.0, a >= params.cutoff);
  }
  return smoothstep(params.cutoff - params.soft, params.cutoff + params.soft, a);
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
  let t = threshold_t(tex.a);
  let out = tex * t;

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
  mask_head: u32,
  mask_count: u32,
  mask_scope_head: u32,
  mask_scope_count: u32,
  output_is_srgb: u32,
  _pad0: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
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
  mask_head: u32,
  mask_count: u32,
  mask_scope_head: u32,
  mask_scope_count: u32,
  output_is_srgb: u32,
  _pad0: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
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
  mask_head: u32,
  mask_count: u32,
  mask_scope_head: u32,
  mask_scope_count: u32,
  output_is_srgb: u32,
  _pad0: u32,
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
  mask_head: u32,
  mask_count: u32,
  mask_scope_head: u32,
  mask_scope_count: u32,
  output_is_srgb: u32,
  _pad0: u32,
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
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 2u) {
    let center = m.params0.xy;
    let radius = max(m.params0.zw, vec2<f32>(1e-6));
    let d = (local_pos - center) / radius;
    let t = length(d);
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 3u) {
    let uv0 = m.params0.xy;
    let uv1 = m.params0.zw;
    let denom = max(m.bounds.zw, vec2<f32>(1e-6));
    let t = clamp(p / denom, vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = mix(uv0, uv1, t);
    let s = textureSample(mask_image_texture, mask_image_sampler, uv);
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
  let mask = mask_alpha(input.pixel_pos);
  let sample = textureSample(tex, tex_sampler, input.uv);
  let o = clamp(input.opacity, 0.0, 1.0);
  let out = vec4<f32>(sample.rgb * o, sample.a * o) * clip * mask;
  return encode_output_premul(out);
}
"#;

pub(super) const COMPOSITE_PREMUL_MASK_SHADER: &str = r#"
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
  let local = pixel_pos - render_space.origin_px;
  let ndc_x = (local.x / render_space.size_px.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (local.y / render_space.size_px.y) * 2.0;
  return vec2<f32>(ndc_x, ndc_y);
}

fn saturate(x: f32) -> f32 {
  return clamp(x, 0.0, 1.0);
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
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 2u) {
    let center = m.params0.xy;
    let radius = max(m.params0.zw, vec2<f32>(1e-6));
    let d = (local_pos - center) / radius;
    let t = length(d);
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 3u) {
    let uv0 = m.params0.xy;
    let uv1 = m.params0.zw;
    let denom = max(m.bounds.zw, vec2<f32>(1e-6));
    let t = clamp(p / denom, vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = mix(uv0, uv1, t);
    let s = textureSample(mask_image_texture, mask_image_sampler, uv);
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
  let mask_tex = textureLoad(mask_texture, vec2<i32>(mx, my), 0).x * select(0.0, 1.0, inside);
  let mask_stack = mask_alpha(input.pixel_pos);
  let o = clamp(input.opacity, 0.0, 1.0);
  let out = vec4<f32>(sample.rgb * o, sample.a * o) * mask_tex * mask_stack;
  return encode_output_premul(out);
}
"#;

pub(super) const PATH_CLIP_MASK_SHADER: &str = r#"
struct RenderSpace {
  origin_px: vec2<f32>,
  size_px: vec2<f32>,
};

@group(0) @binding(5) var<uniform> render_space: RenderSpace;

struct VsIn {
  @location(0) pos_px: vec2<f32>,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
};

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let local = pixel_pos - render_space.origin_px;
  let ndc_x = (local.x / render_space.size_px.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (local.y / render_space.size_px.y) * 2.0;
  return vec2<f32>(ndc_x, ndc_y);
}

@vertex
fn vs_main(input: VsIn) -> VsOut {
  var out: VsOut;
  let clip_xy = to_clip_space(input.pos_px);
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  return out;
}

@fragment
fn fs_main() -> @location(0) f32 {
  return 1.0;
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
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 2u) {
    let center = m.params0.xy;
    let radius = max(m.params0.zw, vec2<f32>(1e-6));
    let d = (local_pos - center) / radius;
    let t = length(d);
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 3u) {
    let uv0 = m.params0.xy;
    let uv1 = m.params0.zw;
    let denom = max(m.bounds.zw, vec2<f32>(1e-6));
    let t = clamp(p / denom, vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = mix(uv0, uv1, t);
    let s = textureSample(mask_image_texture, mask_image_sampler, uv);
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
  let mask = mask_alpha(input.pixel_pos);
  let out = input.color * clip * mask;
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
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 2u) {
    let center = m.params0.xy;
    let radius = max(m.params0.zw, vec2<f32>(1e-6));
    let d = (local_pos - center) / radius;
    let t = length(d);
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 3u) {
    let uv0 = m.params0.xy;
    let uv1 = m.params0.zw;
    let denom = max(m.bounds.zw, vec2<f32>(1e-6));
    let t = clamp(p / denom, vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = mix(uv0, uv1, t);
    let s = textureSample(mask_image_texture, mask_image_sampler, uv);
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
  let mask = mask_alpha(input.pixel_pos);
  let tex = textureSample(glyph_atlas, glyph_sampler, input.uv);
  let coverage = apply_contrast_and_gamma_correction(tex.r, input.color.rgb);
  let out = vec4<f32>(input.color.rgb * coverage, input.color.a * coverage) * clip * mask;
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
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 2u) {
    let center = m.params0.xy;
    let radius = max(m.params0.zw, vec2<f32>(1e-6));
    let d = (local_pos - center) / radius;
    let t = length(d);
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 3u) {
    let uv0 = m.params0.xy;
    let uv1 = m.params0.zw;
    let denom = max(m.bounds.zw, vec2<f32>(1e-6));
    let t = clamp(p / denom, vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = mix(uv0, uv1, t);
    let s = textureSample(mask_image_texture, mask_image_sampler, uv);
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
  let mask = mask_alpha(input.pixel_pos);
  let tex = textureSample(glyph_atlas, glyph_sampler, input.uv);
  let a = tex.a * input.color.a;
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
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 2u) {
    let center = m.params0.xy;
    let radius = max(m.params0.zw, vec2<f32>(1e-6));
    let d = (local_pos - center) / radius;
    let t = length(d);
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 3u) {
    let uv0 = m.params0.xy;
    let uv1 = m.params0.zw;
    let denom = max(m.bounds.zw, vec2<f32>(1e-6));
    let t = clamp(p / denom, vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = mix(uv0, uv1, t);
    let s = textureSample(mask_image_texture, mask_image_sampler, uv);
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
  let mask = mask_alpha(input.pixel_pos);
  let tex = textureSample(glyph_atlas, glyph_sampler, input.uv);
  let coverage = apply_contrast_and_gamma_correction3(tex.rgb, input.color.rgb);
  let a = max(max(coverage.r, coverage.g), coverage.b);
  let out = vec4<f32>(input.color.rgb * coverage, input.color.a * a) * clip * mask;
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
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 2u) {
    let center = m.params0.xy;
    let radius = max(m.params0.zw, vec2<f32>(1e-6));
    let d = (local_pos - center) / radius;
    let t = length(d);
    let tt = clamp(t, 0.0, 1.0);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 3u) {
    let uv0 = m.params0.xy;
    let uv1 = m.params0.zw;
    let denom = max(m.bounds.zw, vec2<f32>(1e-6));
    let t = clamp(p / denom, vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = mix(uv0, uv1, t);
    let s = textureSample(mask_image_texture, mask_image_sampler, uv);
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
  let mask = mask_alpha(input.pixel_pos);
  let tex = textureSample(mask_texture, mask_sampler, input.uv);
  let coverage = tex.r;
  let out = vec4<f32>(input.color.rgb * coverage, input.color.a * coverage) * clip * mask;
  return encode_output_premul(out);
}
"#;
