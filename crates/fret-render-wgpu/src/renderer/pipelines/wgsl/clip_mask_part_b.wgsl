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
    alpha = alpha * sdf_coverage_smooth(sdf);
  }
  return alpha;
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) f32 {
  let x = floor(pos.x) + 0.5;
  let y = floor(pos.y) + 0.5;
  let scale = viewport.mask_viewport_size / params.dst_size;
  return clip_alpha(viewport.mask_viewport_origin + vec2<f32>(x, y) * scale);
}
