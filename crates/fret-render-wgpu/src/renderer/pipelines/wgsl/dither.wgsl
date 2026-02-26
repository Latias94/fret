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

fn bayer4x4(x: i32, y: i32) -> f32 {
  let xx = u32(x) & 3u;
  let yy = u32(y) & 3u;
  let idx = (yy << 2u) | xx;
  // Standard 4x4 Bayer matrix values in [0, 15].
  let m = array<f32, 16>(
    0.0,  8.0,  2.0, 10.0,
   12.0,  4.0, 14.0,  6.0,
    3.0, 11.0,  1.0,  9.0,
   15.0,  7.0, 13.0,  5.0
  );
  return m[idx];
}

fn dither_u8(rgb: vec3<f32>, x: i32, y: i32) -> vec3<f32> {
  let levels = 255.0;
  let t = (bayer4x4(x, y) + 0.5) / 16.0;
  let q = floor(rgb * levels + vec3<f32>(t)) / levels;
  return clamp(q, vec3<f32>(0.0), vec3<f32>(1.0));
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
  var rgb = tex.rgb / max(a, 1e-4);
  rgb = dither_u8(rgb, x, y);
  return vec4<f32>(rgb * a, a);
}
