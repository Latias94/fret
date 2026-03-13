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
  let sample = textureLoad(src_texture, vec2<i32>(i32(sx), i32(sy)), 0);
  return vec4<f32>(sample.rgb * clip, clip);
}
