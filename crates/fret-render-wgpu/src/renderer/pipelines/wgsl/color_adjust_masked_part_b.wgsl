fn saturate3(v: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(saturate(v.x), saturate(v.y), saturate(v.z));
}

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

  var rgb = tex.rgb / a;
  let s = max(params.saturation, 0.0);
  let c = params.contrast;
  let b = max(params.brightness, 0.0);

  let luma = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  rgb = mix(vec3<f32>(luma), rgb, s);
  rgb = rgb * vec3<f32>(b);
  rgb = (rgb - vec3<f32>(0.5)) * c + vec3<f32>(0.5);
  rgb = saturate3(rgb);

  return vec4<f32>(rgb * a * clip, clip);
}
