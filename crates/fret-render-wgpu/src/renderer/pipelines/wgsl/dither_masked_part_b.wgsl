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

fn bayer4x4(x: i32, y: i32) -> f32 {
  let xx = u32(x) & 3u;
  let yy = u32(y) & 3u;
  let idx = (yy << 2u) | xx;
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
  let out = vec4<f32>(rgb * a, a);

  let clip = clip_alpha(pos.xy);
  return vec4<f32>(out.rgb * clip, clip);
}
