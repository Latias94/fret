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

fn hash_u32(x: u32) -> u32 {
  var v = x;
  v = v ^ (v >> 16u);
  v = v * 0x7feb352du;
  v = v ^ (v >> 15u);
  v = v * 0x846ca68bu;
  v = v ^ (v >> 16u);
  return v;
}

fn hash2(x: u32, y: u32, seed: u32) -> u32 {
  return hash_u32(x ^ hash_u32(y ^ seed));
}

fn noise_rgb(cell_x: i32, cell_y: i32, seed: u32) -> vec3<f32> {
  let ux = u32(cell_x);
  let uy = u32(cell_y);
  let h0 = hash2(ux, uy, seed);
  let h1 = hash2(ux, uy, seed ^ 0x9e3779b9u);
  let h2 = hash2(ux, uy, seed ^ 0x243f6a88u);
  let inv = 1.0 / 4294967296.0;
  let r = f32(h0) * inv;
  let g = f32(h1) * inv;
  let b = f32(h2) * inv;
  return vec3<f32>(r, g, b) * 2.0 - vec3<f32>(1.0);
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

  let scale_px = max(noise.scale_px, 1.0);
  let cell_scale = max(i32(floor(scale_px)), 1);
  let cx = x / cell_scale;
  let cy = y / cell_scale;
  let seed = bitcast<u32>(noise.phase);
  let n = noise_rgb(cx, cy, seed) * noise.strength;
  let rgb = clamp(tex.rgb + n * a, vec3<f32>(0.0), vec3<f32>(1.0));

  let clip = clip_alpha(pos.xy);
  return vec4<f32>(rgb * clip, clip);
}
