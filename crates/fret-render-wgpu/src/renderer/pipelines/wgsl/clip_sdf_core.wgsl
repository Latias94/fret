
fn saturate(x: f32) -> f32 {
  return clamp(x, 0.0, 1.0);
}

fn sdf_aa(sdf: f32) -> f32 {
  // `fwidth(sdf)` uses `abs(dpdx) + abs(dpdy)`, which makes diagonal edges (notably rounded
  // corners) appear softer/thinner than axis-aligned edges. Using the isotropic gradient length
  // keeps coverage more uniform across edge angles.
  let g = vec2<f32>(dpdx(sdf), dpdy(sdf));
  return max(length(g), 1e-4);
}

fn sdf_coverage_smooth(sdf: f32) -> f32 {
  let aa = sdf_aa(sdf);
  return 1.0 - smoothstep(-aa, aa, sdf);
}

fn sdf_coverage_linear(sdf: f32) -> f32 {
  let aa = sdf_aa(sdf);
  return saturate(0.5 - sdf / aa);
}

fn gradient_tile_mode_apply(t: f32, tile_mode: u32) -> f32 {
  if (tile_mode == 1u) {
    return fract(t);
  }
  if (tile_mode == 2u) {
    let seg = floor(t);
    let r = fract(t);
    let odd = (i32(seg) & 1) != 0;
    return select(r, 1.0 - r, odd);
  }
  return clamp(t, 0.0, 1.0);
}

fn pick_corner_radius(center_to_point: vec2<f32>, radii: vec4<f32>) -> f32 {
  // IMPORTANT (WebGPU/wasm): Keep this function branchless so that callers can safely use
  // derivative ops (dpdx/dpdy) later in the call chain from uniform control flow.
  let left = center_to_point.x < 0.0;
  let top = center_to_point.y < 0.0;

  let r_top = select(radii.y, radii.x, left);
  let r_bottom = select(radii.z, radii.w, left);
  return select(r_bottom, r_top, top);
}

fn quad_sdf_impl(corner_center_to_point: vec2<f32>, corner_radius: f32) -> f32 {
  // Branchless variant of the standard rectangle / rounded-rectangle SDF:
  // https://iquilezles.org/articles/distfunctions2d/
  let signed_distance_to_inset_quad =
    length(max(vec2<f32>(0.0), corner_center_to_point)) +
    min(0.0, max(corner_center_to_point.x, corner_center_to_point.y));
  return signed_distance_to_inset_quad - corner_radius;
}

fn quad_sdf(point: vec2<f32>, rect_origin: vec2<f32>, rect_size: vec2<f32>, corner_radii: vec4<f32>) -> f32 {
  let center = rect_origin + rect_size * 0.5;
  let center_to_point = point - center;
  let half_size = rect_size * 0.5;
  let corner_radius = pick_corner_radius(center_to_point, corner_radii);
  let corner_to_point = abs(center_to_point) - half_size;
  let corner_center_to_point = corner_to_point + corner_radius;
  return quad_sdf_impl(corner_center_to_point, corner_radius);
}
