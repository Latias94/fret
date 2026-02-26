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

@group(1) @binding(0) var src_texture: texture_2d<f32>;

struct NoiseParams {
  strength: f32,
  scale_px: f32,
  phase: f32,
  _pad0: f32,
};

@group(1) @binding(1) var<uniform> noise: NoiseParams;
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

fn hash_u32(x: u32) -> u32 {
  var v = x;
  v = v ^ (v >> 16u);
  v = v * 0x7feb352du;
  v = v ^ (v >> 15u);
  v = v * 0x846ca68bu;
  v = v ^ (v >> 16u);
  return v;
}

fn hash2(x: u32, y: u32, seed: u32) -> u32 {
  return hash_u32(x ^ hash_u32(y ^ seed));
}

fn noise_rgb(cell_x: i32, cell_y: i32, seed: u32) -> vec3<f32> {
  let ux = u32(cell_x);
  let uy = u32(cell_y);
  let h0 = hash2(ux, uy, seed);
  let h1 = hash2(ux, uy, seed ^ 0x9e3779b9u);
  let h2 = hash2(ux, uy, seed ^ 0x243f6a88u);
  let inv = 1.0 / 4294967296.0;
  let r = f32(h0) * inv;
  let g = f32(h1) * inv;
  let b = f32(h2) * inv;
  return vec3<f32>(r, g, b) * 2.0 - vec3<f32>(1.0);
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

  let scale_px = max(noise.scale_px, 1.0);
  let cell_scale = max(i32(floor(scale_px)), 1);
  let cx = x / cell_scale;
  let cy = y / cell_scale;
  let seed = bitcast<u32>(noise.phase);
  let n = noise_rgb(cx, cy, seed) * noise.strength;
  let rgb = clamp(tex.rgb + n * a, vec3<f32>(0.0), vec3<f32>(1.0));

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
  return vec4<f32>(rgb * mask, mask);
}
