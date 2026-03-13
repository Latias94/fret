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

fn apply_color_matrix(tex: vec4<f32>) -> vec4<f32> {
  let a = tex.a;
  if (a <= 0.0) {
    return vec4<f32>(0.0);
  }
  let rgb = tex.rgb / a;
  let v = vec4<f32>(rgb, a);
  let out = vec4<f32>(
    dot(params.row0, v) + params.bias.x,
    dot(params.row1, v) + params.bias.y,
    dot(params.row2, v) + params.bias.z,
    dot(params.row3, v) + params.bias.w,
  );
  let clamped = saturate4(out);
  return vec4<f32>(clamped.rgb * clamped.a, clamped.a);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let clip = clip_alpha(pos.xy);
  let dims = textureDimensions(src_texture);
  let x = i32(floor(pos.x));
  let y = i32(floor(pos.y));
  if (x < 0 || y < 0 || x >= i32(dims.x) || y >= i32(dims.y)) {
    return vec4<f32>(0.0);
  }

  let tex = textureLoad(src_texture, vec2<i32>(x, y), 0);
  let out = apply_color_matrix(tex);

  return vec4<f32>(out.rgb * clip, clip);
}
