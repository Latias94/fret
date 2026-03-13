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
