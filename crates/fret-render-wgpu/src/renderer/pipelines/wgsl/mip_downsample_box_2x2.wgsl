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
  let dims_u = textureDimensions(src_texture);
  if (dims_u.x == 0u || dims_u.y == 0u) {
    return vec4<f32>(0.0);
  }

  let dst_x = i32(floor(pos.x));
  let dst_y = i32(floor(pos.y));
  let src_x = dst_x * 2;
  let src_y = dst_y * 2;

  let max_x = i32(dims_u.x) - 1;
  let max_y = i32(dims_u.y) - 1;

  let x0 = clamp(src_x, 0, max_x);
  let y0 = clamp(src_y, 0, max_y);
  let x1 = clamp(src_x + 1, 0, max_x);
  let y1 = clamp(src_y + 1, 0, max_y);

  let c00 = textureLoad(src_texture, vec2<i32>(x0, y0), 0);
  let c10 = textureLoad(src_texture, vec2<i32>(x1, y0), 0);
  let c01 = textureLoad(src_texture, vec2<i32>(x0, y1), 0);
  let c11 = textureLoad(src_texture, vec2<i32>(x1, y1), 0);
  return (c00 + c10 + c01 + c11) * 0.25;
}
