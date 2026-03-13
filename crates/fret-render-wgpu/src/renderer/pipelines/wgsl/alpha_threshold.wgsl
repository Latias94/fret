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
