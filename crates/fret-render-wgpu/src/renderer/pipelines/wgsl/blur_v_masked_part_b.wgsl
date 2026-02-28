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

fn clamp_i32(x: i32, lo: i32, hi: i32) -> i32 {
  return min(max(x, lo), hi);
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

  // 9-tap separable gaussian-ish kernel (radius 4).
  let w0 = 0.227027;
  let w1 = 0.1945946;
  let w2 = 0.1216216;
  let w3 = 0.054054;
  let w4 = 0.016216;

  let max_y = i32(dims.y) - 1;
  let sy0 = clamp_i32(y, 0, max_y);
  let c0 = textureLoad(src_texture, vec2<i32>(x, sy0), 0) * w0;

  let sy1p = clamp_i32(y + 1, 0, max_y);
  let sy1n = clamp_i32(y - 1, 0, max_y);
  let c1 = (textureLoad(src_texture, vec2<i32>(x, sy1p), 0) +
            textureLoad(src_texture, vec2<i32>(x, sy1n), 0)) * w1;

  let sy2p = clamp_i32(y + 2, 0, max_y);
  let sy2n = clamp_i32(y - 2, 0, max_y);
  let c2 = (textureLoad(src_texture, vec2<i32>(x, sy2p), 0) +
            textureLoad(src_texture, vec2<i32>(x, sy2n), 0)) * w2;

  let sy3p = clamp_i32(y + 3, 0, max_y);
  let sy3n = clamp_i32(y - 3, 0, max_y);
  let c3 = (textureLoad(src_texture, vec2<i32>(x, sy3p), 0) +
            textureLoad(src_texture, vec2<i32>(x, sy3n), 0)) * w3;

  let sy4p = clamp_i32(y + 4, 0, max_y);
  let sy4n = clamp_i32(y - 4, 0, max_y);
  let c4 = (textureLoad(src_texture, vec2<i32>(x, sy4p), 0) +
            textureLoad(src_texture, vec2<i32>(x, sy4n), 0)) * w4;

  let out = c0 + c1 + c2 + c3 + c4;
  return vec4<f32>(out.rgb * clip, clip);
}
