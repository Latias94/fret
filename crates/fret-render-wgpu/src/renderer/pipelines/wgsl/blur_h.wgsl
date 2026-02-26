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

