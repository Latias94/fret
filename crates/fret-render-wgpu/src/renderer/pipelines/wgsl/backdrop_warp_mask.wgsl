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
  origin_px: vec2<f32>,
  bounds_size_px: vec2<f32>,
  strength_px: f32,
  scale_px: f32,
  phase: f32,
  chroma_px: f32,
  kind: u32,
  warp_encoding: u32,
  warp_sampling: u32,
  _pad0: u32,
  uv0: vec2<f32>,
  uv1: vec2<f32>,
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

fn sample_premul_bilinear(p_px: vec2<f32>) -> vec4<f32> {
  let dims_u = textureDimensions(src_texture);
  if (dims_u.x == 0u || dims_u.y == 0u) {
    return vec4<f32>(0.0);
  }

  let max_p = vec2<f32>(f32(dims_u.x) - 0.5, f32(dims_u.y) - 0.5);
  let p = clamp(p_px, vec2<f32>(0.5), max_p);
  let t = p - vec2<f32>(0.5);
  let base_f = floor(t);
  let f = fract(t);

  let x0 = clamp(i32(base_f.x), 0, i32(dims_u.x) - 1);
  let y0 = clamp(i32(base_f.y), 0, i32(dims_u.y) - 1);
  let x1 = min(x0 + 1, i32(dims_u.x) - 1);
  let y1 = min(y0 + 1, i32(dims_u.y) - 1);

  let c00 = textureLoad(src_texture, vec2<i32>(x0, y0), 0);
  let c10 = textureLoad(src_texture, vec2<i32>(x1, y0), 0);
  let c01 = textureLoad(src_texture, vec2<i32>(x0, y1), 0);
  let c11 = textureLoad(src_texture, vec2<i32>(x1, y1), 0);

  let cx0 = mix(c00, c10, f.x);
  let cx1 = mix(c01, c11, f.x);
  return mix(cx0, cx1, f.y);
}

fn warp_offset_px(pixel_pos_px: vec2<f32>) -> vec2<f32> {
  let local = pixel_pos_px - params.origin_px;
  let s = clamp(params.strength_px, 0.0, 24.0);
  let scale = max(params.scale_px, 1.0);
  let phase = params.phase;

  let tau = 6.283185307179586;

  let bounds = max(params.bounds_size_px, vec2<f32>(1.0));
  let p01 = clamp(local / bounds, vec2<f32>(0.0), vec2<f32>(1.0));
  let p = (p01 - vec2<f32>(0.5)) * 2.0;
  let r = length(p);
  let falloff = clamp(1.0 - r, 0.0, 1.0);
  let falloff2 = falloff * falloff;

  let t = (local / scale) + vec2<f32>(phase, phase);

  let wave = vec2<f32>(sin(t.y * tau), sin(t.x * tau)) * falloff2;

  let inv_r = select(0.0, 1.0 / r, r > 1e-4);
  let dir = select(vec2<f32>(1.0, 0.0), p * inv_r, r > 1e-4);
  let swirl = vec2<f32>(-dir.y, dir.x);
  let swirl_phase = sin(phase * 1.7);
  let lens = (dir * 0.9 + swirl * (0.1 * swirl_phase)) * (falloff2 * falloff);

  let use_lens = params.kind == 1u;
  return select(wave, lens, use_lens) * s;
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = textureDimensions(src_texture);
  let x = i32(floor(pos.x));
  let y = i32(floor(pos.y));
  if (x < 0 || y < 0 || x >= i32(dims.x) || y >= i32(dims.y)) {
    return vec4<f32>(0.0);
  }

  let base_px = pos.xy;
  let d = warp_offset_px(base_px);
  let chroma = clamp(params.chroma_px, 0.0, 8.0);

  var color = vec4<f32>(0.0);
  if (chroma <= 0.0) {
    color = sample_premul_bilinear(base_px + d);
  } else {
    let len = length(d);
    let dir = select(vec2<f32>(1.0, 0.0), d / len, len > 1e-4);

    let center = sample_premul_bilinear(base_px + d);
    let r_s = sample_premul_bilinear(base_px + d + dir * chroma);
    let b_s = sample_premul_bilinear(base_px + d - dir * chroma);

    let a = center.a;
    if (a <= 0.0) {
      return vec4<f32>(0.0);
    }
    let ru = r_s.rgb / max(r_s.a, 1e-4);
    let cu = center.rgb / a;
    let bu = b_s.rgb / max(b_s.a, 1e-4);
    let rgb_u = vec3<f32>(ru.r, cu.g, bu.b);
    color = vec4<f32>(rgb_u * a, a);
  }

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
  return vec4<f32>(color.rgb * mask, mask);
}
