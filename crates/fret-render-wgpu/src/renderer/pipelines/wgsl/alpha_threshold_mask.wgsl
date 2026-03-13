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
