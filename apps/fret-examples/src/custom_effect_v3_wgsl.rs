//! Shared WGSL for the CustomV3 "lens" authoring demos (native + web).
//!
//! This shader intentionally mirrors the key pieces of the AndroidLiquidGlass reference:
//! - rounded-rect SDF + gradient-based refraction direction
//! - rim-only refraction ("refraction height" gate)
//! - circle-map displacement taper
//! - optional chromatic dispersion (cheap 3-tap)

pub const CUSTOM_EFFECT_V3_LENS_WGSL: &str = r#"
fn radius_at(centered: vec2<f32>, radii: vec4<f32>) -> f32 {
  if (centered.x >= 0.0) {
    if (centered.y <= 0.0) { return radii.y; }
    return radii.z;
  }
  if (centered.y <= 0.0) { return radii.x; }
  return radii.w;
}

fn sd_rounded_rect(centered: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
  let r = clamp(radius, 0.0, min(half_size.x, half_size.y));
  let corner = abs(centered) - (half_size - vec2<f32>(r));
  let outside = length(max(corner, vec2<f32>(0.0))) - r;
  let inside = min(max(corner.x, corner.y), 0.0);
  return outside + inside;
}

fn grad_sd_rounded_rect(centered: vec2<f32>, half_size: vec2<f32>, radius: f32) -> vec2<f32> {
  let r = clamp(radius, 0.0, min(half_size.x, half_size.y));
  let corner = abs(centered) - (half_size - vec2<f32>(r));
  if (corner.x >= 0.0 || corner.y >= 0.0) {
    return sign(centered) * normalize(max(corner, vec2<f32>(0.0)) + vec2<f32>(1.0e-6, 0.0));
  }
  let grad_x = select(0.0, 1.0, corner.y <= corner.x);
  return sign(centered) * vec2<f32>(grad_x, 1.0 - grad_x);
}

fn circle_map(x: f32) -> f32 {
  let xx = clamp(x, 0.0, 1.0);
  return 1.0 - sqrt(max(1.0 - xx * xx, 0.0));
}

fn hash01(p: vec2<f32>) -> f32 {
  let u = dot(p, vec2<f32>(12.9898, 78.233));
  return fract(sin(u) * 43758.5453);
}

fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, pos_px: vec2<f32>, params: EffectParamsV1) -> vec4<f32> {
  // params.vec4s[0]:
  // - x: refraction_height_px (rim thickness; render px)
  // - y: refraction_amount_px (displacement; render px; positive)
  // - z: pyramid_level
  // - w: frost_mix (0..1)
  let refraction_height_px = clamp(params.vec4s[0].x, 0.0, 96.0);
  let refraction_amount_px = clamp(params.vec4s[0].y, 0.0, 96.0);
  let pyramid_level = u32(clamp(params.vec4s[0].z, 0.0, 6.0));
  let frost_mix = clamp(params.vec4s[0].w, 0.0, 1.0);

  // params.vec4s[1]:
  // - x: corner_radius_px (render px)
  // - y: depth_effect (0..1)
  // - z: dispersion (0..1)
  // - w: noise_alpha (0..0.1)
  let corner_radius_px = clamp(params.vec4s[1].x, 0.0, 256.0);
  let depth_effect = clamp(params.vec4s[1].y, 0.0, 1.0);
  let dispersion = clamp(params.vec4s[1].z, 0.0, 1.0);
  let noise_alpha = clamp(params.vec4s[1].w, 0.0, 0.1);

  // params.vec4s[2]: tint (rgb + alpha)
  let tint = vec4<f32>(
    clamp(params.vec4s[2].x, 0.0, 1.0),
    clamp(params.vec4s[2].y, 0.0, 1.0),
    clamp(params.vec4s[2].z, 0.0, 1.0),
    clamp(params.vec4s[2].w, 0.0, 1.0)
  );

  let local = fret_local_px(pos_px);
  let size = max(render_space.size_px, vec2<f32>(1.0));
  let half_size = size * 0.5;
  let centered = local - half_size;

  let radii = vec4<f32>(corner_radius_px, corner_radius_px, corner_radius_px, corner_radius_px);
  let radius = radius_at(centered, radii);
  let sd = sd_rounded_rect(centered, half_size, radius);
  if (sd > 0.0) {
    return src;
  }

  // Frosted source: chain input (blurred) + optional extra pyramid sampling from raw.
  let pyr = fret_sample_src_pyramid_at_pos(pyramid_level, pos_px);
  let frosted = mix(src, pyr, frost_mix);

  let inside_px = clamp(-sd, 0.0, 4096.0);
  if (inside_px >= refraction_height_px || refraction_height_px <= 0.0 || refraction_amount_px <= 0.0) {
    var out_rgb = mix(frosted.rgb, tint.rgb, tint.a);
    let n = hash01(floor(pos_px) + vec2<f32>(17.0, 91.0)) - 0.5;
    out_rgb = out_rgb + vec3<f32>(n) * noise_alpha;
    return vec4<f32>(clamp(out_rgb, vec3<f32>(0.0), vec3<f32>(1.0)), frosted.a);
  }

  let inside01 = inside_px / max(refraction_height_px, 1.0);

  // Refraction direction from SDF gradient (optionally with "depth" pull toward center).
  let grad_radius = min(radius * 1.5, min(half_size.x, half_size.y));
  let g0 = grad_sd_rounded_rect(centered, half_size, grad_radius);
  let g1 = normalize(g0 + depth_effect * normalize(centered + vec2<f32>(1.0e-6, 0.0)));

  // AndroidLiquidGlass uses a negated refraction amount so the rim refracts inward.
  let d = circle_map(1.0 - inside01) * refraction_amount_px;
  let refract = -d * g1;

  // Cheap dispersion (3 taps).
  let disp_k = dispersion * abs((centered.x * centered.y) / max(half_size.x * half_size.y, 1.0));
  let disp = refract * disp_k;
  let raw_r = fret_sample_src_raw_at_pos(pos_px + refract + disp);
  let raw_g = fret_sample_src_raw_at_pos(pos_px + refract);
  let raw_b = fret_sample_src_raw_at_pos(pos_px + refract - disp);
  let raw = vec4<f32>(raw_r.r, raw_g.g, raw_b.b, raw_g.a);

  let rim = 1.0 - smoothstep(0.0, 1.0, inside01);
  var out_rgb = mix(frosted.rgb, raw.rgb, rim);
  out_rgb = mix(out_rgb, tint.rgb, tint.a);
  let n = hash01(floor(pos_px) + vec2<f32>(17.0, 91.0)) - 0.5;
  out_rgb = out_rgb + vec3<f32>(n) * noise_alpha;
  return vec4<f32>(clamp(out_rgb, vec3<f32>(0.0), vec3<f32>(1.0)), frosted.a);
}
"#;
