@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let dims_u = textureDimensions(src_texture);
  let x = i32(floor(pos.x));
  let y = i32(floor(pos.y));
  if (x < 0 || y < 0 || x >= i32(dims_u.x) || y >= i32(dims_u.y)) {
    return vec4<f32>(0.0);
  }

  let tex = textureLoad(src_texture, vec2<i32>(x, y), 0);
  let dims = vec2<f32>(f32(dims_u.x), f32(dims_u.y));
  let pos_px = vec2<f32>(f32(x) + 0.5, f32(y) + 0.5);
  let uv = pos_px / dims;
  return fret_custom_effect(tex, uv, pos_px, params);
}

