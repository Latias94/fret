fn clip_alpha(pixel_pos: vec2<f32>) -> f32 {
  var alpha = 1.0;
  var idx = viewport.clip_head;
  for (var i = 0u; i < 64u; i = i + 1u) {
    if (i >= viewport.clip_count) {
      break;
    }
    if (idx == 0xffffffffu) {
      break;
    }
    let clip = clip_stack.clips[idx];
    idx = bitcast<u32>(clip.inv0.w);
    let clip_local = vec2<f32>(
      dot(clip.inv0.xy, pixel_pos) + clip.inv0.z,
      dot(clip.inv1.xy, pixel_pos) + clip.inv1.z
    );
    let sdf = quad_sdf(clip_local, clip.rect.xy, clip.rect.zw, clip.corner_radii);
    alpha = alpha * sdf_coverage_linear(sdf);
  }
  return alpha;
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let clip = clip_alpha(pos.xy);

  let dims_u = textureDimensions(src_texture);
  if (dims_u.x == 0u || dims_u.y == 0u) {
    return vec4<f32>(0.0);
  }

  let x = i32(floor(pos.x));
  let y = i32(floor(pos.y));
  let max_x = i32(dims_u.x) - 1;
  let max_y = i32(dims_u.y) - 1;
  let in_bounds = x >= 0 && y >= 0 && x <= max_x && y <= max_y;
  let x0 = clamp(x, 0, max_x);
  let y0 = clamp(y, 0, max_y);

  let tex = textureLoad(src_texture, vec2<i32>(x0, y0), 0);
  let dims = vec2<f32>(f32(dims_u.x), f32(dims_u.y));
  // Keep `pos_px` stable for out-of-bounds fragments so user WGSL can safely use derivatives.
  let pos_px = vec2<f32>(f32(x0) + 0.5, f32(y0) + 0.5);
  let uv = pos_px / dims;
  let out = fret_custom_effect(tex, uv, pos_px, params);
  let a = clip * select(0.0, 1.0, in_bounds);
  return vec4<f32>(out.rgb * a, out.a * a);
}
