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

