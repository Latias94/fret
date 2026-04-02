fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
  let a = 0.055;
  let lo = rgb * 12.92;
  let hi = (1.0 + a) * pow(rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(a);
  return select(hi, lo, rgb <= vec3<f32>(0.0031308));
}

fn encode_output_premul(c: vec4<f32>) -> vec4<f32> {
  return c;
}

fn dist2(a: vec2<f32>, b: vec2<f32>) -> f32 {
  let d = a - b;
  return dot(d, d);
}

fn rrect_perimeter_s(p: vec2<f32>, rect: vec4<f32>, corner_radii: vec4<f32>) -> f32 {
  let pi = 3.141592653589793;
  let half_pi = 1.5707963267948966;
  let two_pi = 6.283185307179586;

  let x0 = rect.x;
  let y0 = rect.y;
  let w = rect.z;
  let h = rect.w;
  let x1 = x0 + w;
  let y1 = y0 + h;

  let r_tl = max(corner_radii.x, 0.0);
  let r_tr = max(corner_radii.y, 0.0);
  let r_br = max(corner_radii.z, 0.0);
  let r_bl = max(corner_radii.w, 0.0);

  let l_top = max(0.0, w - r_tl - r_tr);
  let l_right = max(0.0, h - r_tr - r_br);
  let l_bottom = max(0.0, w - r_bl - r_br);
  let l_left = max(0.0, h - r_tl - r_bl);

  let l_tl = half_pi * r_tl;
  let l_tr = half_pi * r_tr;
  let l_br = half_pi * r_br;
  let l_bl = half_pi * r_bl;

  let off_top = 0.0;
  let off_tr = off_top + l_top;
  let off_right = off_tr + l_tr;
  let off_br = off_right + l_right;
  let off_bottom = off_br + l_br;
  let off_bl = off_bottom + l_bottom;
  let off_left = off_bl + l_bl;
  let off_tl = off_left + l_left;

  // Straight segments (clamped to segment extents).
  let q_top = vec2<f32>(clamp(p.x, x0 + r_tl, x1 - r_tr), y0);
  let q_right = vec2<f32>(x1, clamp(p.y, y0 + r_tr, y1 - r_br));
  let q_bottom = vec2<f32>(clamp(p.x, x0 + r_bl, x1 - r_br), y1);
  let q_left = vec2<f32>(x0, clamp(p.y, y0 + r_tl, y1 - r_bl));

  let s_top = off_top + (q_top.x - (x0 + r_tl));
  let s_right = off_right + (q_right.y - (y0 + r_tr));
  let s_bottom = off_bottom + ((x1 - r_br) - q_bottom.x);
  let s_left = off_left + ((y1 - r_bl) - q_left.y);

  var best_d = dist2(p, q_top);
  var best_s = s_top;

  let d_right = dist2(p, q_right);
  if (d_right < best_d) {
    best_d = d_right;
    best_s = s_right;
  }

  let d_bottom = dist2(p, q_bottom);
  if (d_bottom < best_d) {
    best_d = d_bottom;
    best_s = s_bottom;
  }

  let d_left = dist2(p, q_left);
  if (d_left < best_d) {
    best_d = d_left;
    best_s = s_left;
  }

  // Corner arcs (angle-clamped to the quarter arc extents).
  if (r_tr > 0.0) {
    let c = vec2<f32>(x1 - r_tr, y0 + r_tr);
    let a0 = atan2(p.y - c.y, p.x - c.x);
    let a = clamp(a0, -half_pi, 0.0);
    let q = c + vec2<f32>(cos(a), sin(a)) * r_tr;
    let d = dist2(p, q);
    if (d < best_d) {
      let t = (a + half_pi) / half_pi;
      best_d = d;
      best_s = off_tr + t * l_tr;
    }
  }

  if (r_br > 0.0) {
    let c = vec2<f32>(x1 - r_br, y1 - r_br);
    let a0 = atan2(p.y - c.y, p.x - c.x);
    let a = clamp(a0, 0.0, half_pi);
    let q = c + vec2<f32>(cos(a), sin(a)) * r_br;
    let d = dist2(p, q);
    if (d < best_d) {
      let t = a / half_pi;
      best_d = d;
      best_s = off_br + t * l_br;
    }
  }

  if (r_bl > 0.0) {
    let c = vec2<f32>(x0 + r_bl, y1 - r_bl);
    let a0 = atan2(p.y - c.y, p.x - c.x);
    let a = clamp(a0, half_pi, pi);
    let q = c + vec2<f32>(cos(a), sin(a)) * r_bl;
    let d = dist2(p, q);
    if (d < best_d) {
      let t = (a - half_pi) / half_pi;
      best_d = d;
      best_s = off_bl + t * l_bl;
    }
  }

  if (r_tl > 0.0) {
    let c = vec2<f32>(x0 + r_tl, y0 + r_tl);
    var a0 = atan2(p.y - c.y, p.x - c.x);
    // Wrap TL's left endpoint from `π` to `-π` so the clamp range is monotonic.
    if (a0 > half_pi) {
      a0 = a0 - two_pi;
    }
    let a = clamp(a0, -pi, -half_pi);
    let q = c + vec2<f32>(cos(a), sin(a)) * r_tl;
    let d = dist2(p, q);
    if (d < best_d) {
      let t = (a + pi) / half_pi; // [-π..-π/2] -> [0..1]
      best_s = off_tl + t * l_tl;
    }
  }

  return best_s;
}

fn rrect_perimeter_len(rect: vec4<f32>, corner_radii: vec4<f32>) -> f32 {
  let half_pi = 1.5707963267948966;

  let w = rect.z;
  let h = rect.w;

  let r_tl = max(corner_radii.x, 0.0);
  let r_tr = max(corner_radii.y, 0.0);
  let r_br = max(corner_radii.z, 0.0);
  let r_bl = max(corner_radii.w, 0.0);

  let l_top = max(0.0, w - r_tl - r_tr);
  let l_right = max(0.0, h - r_tr - r_br);
  let l_bottom = max(0.0, w - r_bl - r_br);
  let l_left = max(0.0, h - r_tl - r_bl);

  let l_corners = half_pi * (r_tl + r_tr + r_br + r_bl);
  return l_top + l_right + l_bottom + l_left + l_corners;
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
    alpha = alpha * sdf_coverage_smooth(sdf);
  }
  return alpha;
}

fn mask_stop_offset(m: MaskGradient, i: u32) -> f32 {
  if (i < 4u) { return m.stop_offsets0[i]; }
  return m.stop_offsets1[i - 4u];
}

fn mask_stop_alpha(m: MaskGradient, i: u32) -> f32 {
  if (i < 4u) { return m.stop_alphas0[i]; }
  return m.stop_alphas1[i - 4u];
}

fn mask_sample_stops(m: MaskGradient, t: f32) -> f32 {
  let n = min(m.stop_count, 8u);
  if (n == 0u) { return 1.0; }

  var prev_offset = mask_stop_offset(m, 0u);
  var prev_alpha = mask_stop_alpha(m, 0u);
  if (n == 1u || t <= prev_offset) {
    return prev_alpha;
  }

  for (var i = 1u; i < 8u; i = i + 1u) {
    if (i >= n) {
      break;
    }
    let off = mask_stop_offset(m, i);
    let a = mask_stop_alpha(m, i);
    if (t <= off) {
      let denom = max(off - prev_offset, 1e-6);
      let u = saturate((t - prev_offset) / denom);
      return mix(prev_alpha, a, u);
    }
    prev_offset = off;
    prev_alpha = a;
  }
  return prev_alpha;
}

fn mask_image_sample_bilinear_clamp(uv: vec2<f32>) -> vec4<f32> {
  let dims_u = textureDimensions(mask_image_texture);
  if (dims_u.x == 0u || dims_u.y == 0u) {
    return vec4<f32>(0.0);
  }

  let dims = vec2<f32>(f32(dims_u.x), f32(dims_u.y));
  let p_px = uv * dims;

  // Manual bilinear sampling avoids WGSL uniformity restrictions on WebGPU.
  let max_p = vec2<f32>(dims.x - 0.5, dims.y - 0.5);
  let p = clamp(p_px, vec2<f32>(0.5), max_p);

  let t = p - vec2<f32>(0.5);
  let base_f = floor(t);
  let f = fract(t);

  let x0 = clamp(i32(base_f.x), 0, i32(dims_u.x) - 1);
  let y0 = clamp(i32(base_f.y), 0, i32(dims_u.y) - 1);
  let x1 = min(x0 + 1, i32(dims_u.x) - 1);
  let y1 = min(y0 + 1, i32(dims_u.y) - 1);

  let c00 = textureLoad(mask_image_texture, vec2<i32>(x0, y0), 0);
  let c10 = textureLoad(mask_image_texture, vec2<i32>(x1, y0), 0);
  let c01 = textureLoad(mask_image_texture, vec2<i32>(x0, y1), 0);
  let c11 = textureLoad(mask_image_texture, vec2<i32>(x1, y1), 0);

  let cx0 = mix(c00, c10, f.x);
  let cx1 = mix(c01, c11, f.x);
  return mix(cx0, cx1, f.y);
}

fn mask_eval(m: MaskGradient, pixel_pos: vec2<f32>) -> f32 {
  let local_pos = vec2<f32>(
    dot(m.inv0.xy, pixel_pos) + m.inv0.z,
    dot(m.inv1.xy, pixel_pos) + m.inv1.z
  );

  let p = local_pos - m.bounds.xy;
  let in_bounds = p.x >= 0.0 && p.y >= 0.0 && p.x <= m.bounds.z && p.y <= m.bounds.w;

  if (m.kind == 1u) {
    let start = m.params0.xy;
    let end = m.params0.zw;
    let dir = end - start;
    let len2 = dot(dir, dir);
    let t = select(0.0, dot(local_pos - start, dir) / len2, len2 > 1e-6);
    let tt = gradient_tile_mode_apply(t, m.tile_mode);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 2u) {
    let center = m.params0.xy;
    let radius = max(m.params0.zw, vec2<f32>(1e-6));
    let d = (local_pos - center) / radius;
    let t = length(d);
    let tt = gradient_tile_mode_apply(t, m.tile_mode);
    return select(1.0, mask_sample_stops(m, tt), in_bounds);
  }

  if (m.kind == 3u) {
    let uv0 = m.params0.xy;
    let uv1 = m.params0.zw;
    let denom = max(m.bounds.zw, vec2<f32>(1e-6));
    let t = clamp(p / denom, vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = mix(uv0, uv1, t);
    let s = mask_image_sample_bilinear_clamp(uv);
    let cov = select(s.r, s.a, m.tile_mode == 1u);
    return select(1.0, clamp(cov, 0.0, 1.0), in_bounds);
  }

  return 1.0;
}

fn mask_alpha(pixel_pos: vec2<f32>) -> f32 {
  var alpha = 1.0;
  var idx = viewport.mask_head;
  for (var i = 0u; i < 64u; i = i + 1u) {
    if (i >= viewport.mask_count) {
      break;
    }
    if (viewport.mask_scope_count != 0u && idx == viewport.mask_scope_head) {
      break;
    }
    if (idx == 0xffffffffu) {
      break;
    }
    let m = mask_stack.masks[idx];
    idx = bitcast<u32>(m.inv0.w);
    alpha = alpha * clamp(mask_eval(m, pixel_pos), 0.0, 1.0);
  }
  return alpha;
}

fn paint_stop_offset(p: Paint, i: u32) -> f32 {
  if (i < 4u) {
    return p.stop_offsets0[i];
  }
  return p.stop_offsets1[i - 4u];
}

fn paint_unpremul_rgb(c: vec4<f32>) -> vec3<f32> {
  let a = max(c.a, 1e-6);
  return c.rgb / a;
}

fn linear_rgb_to_oklab(rgb: vec3<f32>) -> vec3<f32> {
  let c = max(rgb, vec3<f32>(0.0));
  let lms = vec3<f32>(
    0.4122214708 * c.x + 0.5363325363 * c.y + 0.0514459929 * c.z,
    0.2119034982 * c.x + 0.6806995451 * c.y + 0.1073969566 * c.z,
    0.0883024619 * c.x + 0.2817188376 * c.y + 0.6299787005 * c.z
  );
  let lms_cbrt = pow(max(lms, vec3<f32>(0.0)), vec3<f32>(1.0 / 3.0));
  return vec3<f32>(
    0.2104542553 * lms_cbrt.x + 0.7936177850 * lms_cbrt.y - 0.0040720468 * lms_cbrt.z,
    1.9779984951 * lms_cbrt.x - 2.4285922050 * lms_cbrt.y + 0.4505937099 * lms_cbrt.z,
    0.0259040371 * lms_cbrt.x + 0.7827717662 * lms_cbrt.y - 0.8086757660 * lms_cbrt.z
  );
}

fn oklab_to_linear_rgb(lab: vec3<f32>) -> vec3<f32> {
  let lms_cbrt = vec3<f32>(
    lab.x + 0.3963377774 * lab.y + 0.2158037573 * lab.z,
    lab.x - 0.1055613458 * lab.y - 0.0638541728 * lab.z,
    lab.x - 0.0894841775 * lab.y - 1.2914855480 * lab.z
  );
  let lms = lms_cbrt * lms_cbrt * lms_cbrt;
  let rgb = vec3<f32>(
    4.0767416621 * lms.x - 3.3077115913 * lms.y + 0.2309699292 * lms.z,
    -1.2684380046 * lms.x + 2.6097574011 * lms.y - 0.3413193965 * lms.z,
    -0.0041960863 * lms.x - 0.7034186147 * lms.y + 1.7076147010 * lms.z
  );
  return rgb;
}

fn paint_mix_colorspace(p: Paint, a: vec4<f32>, b: vec4<f32>, u: f32) -> vec4<f32> {
  if (p.color_space == 1u) {
    let a0 = clamp(a.a, 0.0, 1.0);
    let a1 = clamp(b.a, 0.0, 1.0);
    let alpha = clamp(mix(a0, a1, u), 0.0, 1.0);

    let rgb0 = clamp(paint_unpremul_rgb(a), vec3<f32>(0.0), vec3<f32>(1.0));
    let rgb1 = clamp(paint_unpremul_rgb(b), vec3<f32>(0.0), vec3<f32>(1.0));
    let lab0 = linear_rgb_to_oklab(rgb0);
    let lab1 = linear_rgb_to_oklab(rgb1);
    let lab = mix(lab0, lab1, u);
    let rgb = clamp(oklab_to_linear_rgb(lab), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(rgb * alpha, alpha);
  }
  return mix(a, b, u);
}

fn paint_sample_stops(p: Paint, t: f32) -> vec4<f32> {
  let n = min(p.stop_count, MAX_STOPS);
  if (n == 0u) {
    return vec4<f32>(0.0);
  }

  var prev_offset = paint_stop_offset(p, 0u);
  var prev_color = p.stop_colors[0u];
  if (n == 1u || t <= prev_offset) {
    return prev_color;
  }

  for (var i = 1u; i < 8u; i = i + 1u) {
    if (i >= n) {
      break;
    }
    let off = paint_stop_offset(p, i);
    let col = p.stop_colors[i];
    if (t <= off) {
      let denom = max(off - prev_offset, 1e-6);
      let u = saturate((t - prev_offset) / denom);
      return paint_mix_colorspace(p, prev_color, col, u);
    }
    prev_offset = off;
    prev_color = col;
  }
  return prev_color;
}

fn mat_hash_u32(x: u32) -> u32 {
  var v = x;
  v = v ^ (v >> 16u);
  v = v * 0x7feb352du;
  v = v ^ (v >> 15u);
  v = v * 0x846ca68bu;
  v = v ^ (v >> 16u);
  return v;
}

fn mat_hash2(p: vec2<u32>, seed: u32) -> u32 {
  let h = p.x ^ (p.y * 0x9e3779b9u) ^ (seed * 0x85ebca6bu);
  return mat_hash_u32(h);
}

fn mat_rand01(p: vec2<u32>, seed: u32) -> f32 {
  let h = mat_hash2(p, seed);
  return f32(h) * (1.0 / 4294967295.0);
}

fn mat_rot(v: vec2<f32>, a: f32) -> vec2<f32> {
  let s = sin(a);
  let c = cos(a);
  return vec2<f32>(c * v.x - s * v.y, s * v.x + c * v.y);
}

fn material_eval(p: Paint, local_pos: vec2<f32>, sample_catalog: bool) -> vec4<f32> {
  let base = p.params0;
  let fg = p.params1;
  let pos = local_pos + p.params3.zw;

  // params2: primary (x/y), thickness/radius (z), seed (w)
  // params3: time/phase (x), angle/softness (y), offset (z/w)
  let spacing = max(p.params2.x, 1.0);
  let spacing_y = max(p.params2.y, 1.0);
  let thickness = max(p.params2.z, 0.0);
  let seed = u32(max(p.params2.w, 0.0));
  let time = p.params3.x;
  let angle = p.params3.y;

  let tm0 = p.tile_mode == 0u;
  let tm1 = p.tile_mode == 1u;
  let tm2 = p.tile_mode == 2u;
  let tm3 = p.tile_mode == 3u;
  let tm4 = p.tile_mode == 4u;
  let tm5 = p.tile_mode == 5u;
  let tm6 = p.tile_mode == 6u;
  let tm7 = p.tile_mode == 7u;

  // 0 DotGrid
  let dot_cell = pos / spacing;
  let dot_frac = fract(dot_cell) - vec2<f32>(0.5);
  let dot_r = select(spacing * 0.12, thickness, thickness > 0.0);
  let dot_d = length(dot_frac) * spacing;
  let dot_aa = max(fwidth(dot_d), 1e-4);
  let dot_cov = 1.0 - smoothstep(dot_r, dot_r + dot_aa, dot_d);
  let mat0 = base * (1.0 - dot_cov) + fg * dot_cov;

  // 1 Grid
  let grid_cell = pos / vec2<f32>(spacing, spacing_y);
  let grid_frac = abs(fract(grid_cell) - vec2<f32>(0.5));
  let grid_dx = grid_frac.x * spacing;
  let grid_dy = grid_frac.y * spacing_y;
  let grid_w = select(1.0, thickness, thickness > 0.0);
  let grid_aa_x = max(fwidth(grid_dx), 1e-4);
  let grid_aa_y = max(fwidth(grid_dy), 1e-4);
  let grid_cov_x = 1.0 - smoothstep(grid_w * 0.5, grid_w * 0.5 + grid_aa_x, grid_dx);
  let grid_cov_y = 1.0 - smoothstep(grid_w * 0.5, grid_w * 0.5 + grid_aa_y, grid_dy);
  let grid_cov = max(grid_cov_x, grid_cov_y);
  let mat1 = base * (1.0 - grid_cov) + fg * grid_cov;

  // 2 Checkerboard
  let chk_cell = vec2<u32>(
    u32(floor(pos.x / spacing)),
    u32(floor(pos.y / spacing_y))
  );
  let chk_parity = (chk_cell.x + chk_cell.y) & 1u;
  let mat2 = select(base, fg, chk_parity == 1u);

  // 3 Stripe
  let stripe_p2 = mat_rot(pos, angle);
  let stripe_u = stripe_p2.x / spacing;
  let stripe_du = abs(fract(stripe_u) - 0.5) * spacing;
  let stripe_w = select(spacing * 0.25, thickness, thickness > 0.0);
  let stripe_aa = max(fwidth(stripe_du), 1e-4);
  let stripe_cov = 1.0 - smoothstep(stripe_w * 0.5, stripe_w * 0.5 + stripe_aa, stripe_du);
  let mat3 = base * (1.0 - stripe_cov) + fg * stripe_cov;

  // 4 Noise (deterministic cell noise; optionally sampled from a renderer-owned catalog texture)
  let noise_scale = spacing;
  let noise_cell = vec2<u32>(
    u32(floor(pos.x / noise_scale + 0.5)),
    u32(floor(pos.y / noise_scale + 0.5))
  );
  let noise_r0 = mat_rand01(noise_cell, seed);
  var noise_r = noise_r0;
  if (sample_catalog) {
    let noise_xi = i32(noise_cell.x & 63u);
    let noise_yi = i32(noise_cell.y & 63u);
    let noise_layer = clamp(i32(p.color_space), 0, 1);
    noise_r = textureLoad(
      material_catalog_texture,
      vec2<i32>(noise_xi, noise_yi),
      noise_layer,
      0
    ).r;
  }
  let noise_intensity = clamp(p.params2.y, 0.0, 1.0);
  let noise_cov = noise_intensity * noise_r;
  let mat4 = base * (1.0 - noise_cov) + fg * noise_cov;

  // 5 Beam (caller-driven phase via `time`)
  let beam_p2 = mat_rot(pos, angle);
  let beam_u = beam_p2.x;
  let beam_center = time;
  let beam_width = max(p.params2.x, 1.0);
  let beam_soft = max(p.params2.y, 0.0);
  let beam_d = abs(beam_u - beam_center);
  let beam_aa = max(fwidth(beam_d), 1e-4);
  let beam_cov = 1.0 - smoothstep(beam_width * 0.5, beam_width * 0.5 + beam_soft + beam_aa, beam_d);
  let mat5 = base * (1.0 - beam_cov) + fg * beam_cov;

  // 6 Sparkle (cell-based, explicit `time`, explicit `seed`)
  let sp_cell_size = max(p.params2.x, 1.0);
  let sp_cell = vec2<u32>(
    u32(floor(pos.x / sp_cell_size)),
    u32(floor(pos.y / sp_cell_size))
  );
  let sp_r0 = mat_rand01(sp_cell, seed);
  let sp_density = clamp(p.params2.y, 0.0, 1.0);
  let sp_enabled = sp_r0 <= sp_density;
  let sp_rx = mat_rand01(sp_cell, seed ^ 0x68bc21ebu);
  let sp_ry = mat_rand01(sp_cell, seed ^ 0x02e5be93u);
  let sp_phase = mat_rand01(sp_cell, seed ^ 0xa1b3c5d7u) * 6.2831853;
  let sp_p_cell = (fract(pos / sp_cell_size) - vec2<f32>(sp_rx, sp_ry)) * sp_cell_size;
  let sp_radius = select(sp_cell_size * 0.08, thickness, thickness > 0.0);
  let sp_d = length(sp_p_cell);
  let sp_aa = max(fwidth(sp_d), 1e-4);
  let sp_cov = 1.0 - smoothstep(sp_radius, sp_radius + sp_aa, sp_d);
  let sp_twinkle = 0.5 + 0.5 * sin(time * 2.0 + sp_phase);
  let sp_k = sp_cov * sp_twinkle;
  let sp_out = base * (1.0 - sp_k) + fg * sp_k;
  let mat6 = select(base, sp_out, sp_enabled);

  // 7 ConicSweep (center in params2.xy, width in params2.z (turns), phase in params3.x (turns))
  let con_center = p.params2.xy;
  let con_v = local_pos - con_center;
  let con_a = atan2(con_v.y, con_v.x);
  let con_turns = fract(con_a * (1.0 / 6.2831853) + fract(p.params3.x));
  let con_d = abs(fract(con_turns + 0.5) - 0.5);
  let con_w = clamp(p.params2.z, 0.0, 0.5);
  let con_soft = max(p.params3.y, 0.0);
  let con_aa = max(fwidth(con_d), 1e-4);
  let con_cov = 1.0 - smoothstep(con_w, con_w + con_soft + con_aa, con_d);
  let mat7 = base * (1.0 - con_cov) + fg * con_cov;

  var material = base;
  material = select(material, mat0, tm0);
  material = select(material, mat1, tm1);
  material = select(material, mat2, tm2);
  material = select(material, mat3, tm3);
  material = select(material, mat4, tm4);
  material = select(material, mat5, tm5);
  material = select(material, mat6, tm6);
  material = select(material, mat7, tm7);
  return material;
}

fn paint_eval_fill(p: Paint, local_pos: vec2<f32>, pixel_pos: vec2<f32>) -> vec4<f32> {
  let pos = select(local_pos, pixel_pos, p.eval_space == 1u);
  if (FRET_FILL_KIND == 0u) {
    return p.params0;
  }
  if (FRET_FILL_KIND == 1u) {
    let start = p.params0.xy;
    let end = p.params0.zw;
    let dir = end - start;
    let len2 = dot(dir, dir);
    let t = select(0.0, dot(pos - start, dir) / len2, len2 > 1e-6);
    let tt = gradient_tile_mode_apply(t, p.tile_mode);
    return paint_sample_stops(p, tt);
  }
  if (FRET_FILL_KIND == 2u) {
    let center = p.params0.xy;
    let radius = max(p.params0.zw, vec2<f32>(1e-6));
    let d = (pos - center) / radius;
    let t = length(d);
    let tt = gradient_tile_mode_apply(t, p.tile_mode);
    return paint_sample_stops(p, tt);
  }
  if (FRET_FILL_KIND == 4u) {
    let center = p.params0.xy;
    let start = p.params0.z;
    let span = max(p.params0.w, 1e-6);
    let v = pos - center;
    let a = atan2(v.y, v.x);
    let turns = fract(a * (1.0 / 6.2831853) + 1.0);
    let rel = fract(turns - fract(start) + 1.0);
    let t = rel / span;
    let tt = gradient_tile_mode_apply(t, p.tile_mode);
    return paint_sample_stops(p, tt);
  }
  if (FRET_FILL_KIND == 3u) {
    let sampled = FRET_FILL_MATERIAL_SAMPLED != 0u;
    return material_eval(p, pos, sampled);
  }
  return vec4<f32>(0.0);
}

fn paint_eval_border(p: Paint, local_pos: vec2<f32>, pixel_pos: vec2<f32>) -> vec4<f32> {
  let pos = select(local_pos, pixel_pos, p.eval_space == 1u);
  if (FRET_BORDER_KIND == 0u) {
    return p.params0;
  }
  if (FRET_BORDER_KIND == 1u) {
    let start = p.params0.xy;
    let end = p.params0.zw;
    let dir = end - start;
    let len2 = dot(dir, dir);
    let t = select(0.0, dot(pos - start, dir) / len2, len2 > 1e-6);
    let tt = gradient_tile_mode_apply(t, p.tile_mode);
    return paint_sample_stops(p, tt);
  }
  if (FRET_BORDER_KIND == 2u) {
    let center = p.params0.xy;
    let radius = max(p.params0.zw, vec2<f32>(1e-6));
    let d = (pos - center) / radius;
    let t = length(d);
    let tt = gradient_tile_mode_apply(t, p.tile_mode);
    return paint_sample_stops(p, tt);
  }
  if (FRET_BORDER_KIND == 4u) {
    let center = p.params0.xy;
    let start = p.params0.z;
    let span = max(p.params0.w, 1e-6);
    let v = pos - center;
    let a = atan2(v.y, v.x);
    let turns = fract(a * (1.0 / 6.2831853) + 1.0);
    let rel = fract(turns - fract(start) + 1.0);
    let t = rel / span;
    let tt = gradient_tile_mode_apply(t, p.tile_mode);
    return paint_sample_stops(p, tt);
  }
  if (FRET_BORDER_KIND == 3u) {
    let sampled = FRET_BORDER_MATERIAL_SAMPLED != 0u;
    return material_eval(p, pos, sampled);
  }
  return vec4<f32>(0.0);
}

fn shadow_gaussian(x: f32, sigma: f32) -> f32 {
  return exp(-(x * x) / (2.0 * sigma * sigma)) / (sqrt(2.0 * 3.141592653589793) * sigma);
}

fn shadow_erf(v: vec2<f32>) -> vec2<f32> {
  let s = sign(v);
  let a = abs(v);
  let r1 = 1.0 + (0.278393 + (0.230389 + (0.000972 + 0.078108 * a) * a) * a) * a;
  let r2 = r1 * r1;
  return s - s / (r2 * r2);
}

fn shadow_blur_along_x(
  x: f32,
  y: f32,
  sigma: f32,
  corner_radius: f32,
  half_size: vec2<f32>,
) -> f32 {
  let delta = min(half_size.y - corner_radius - abs(y), 0.0);
  let curved =
    half_size.x - corner_radius + sqrt(max(0.0, corner_radius * corner_radius - delta * delta));
  let integral =
    0.5 + 0.5 * shadow_erf((x + vec2<f32>(-curved, curved)) * (sqrt(0.5) / sigma));
  return integral.y - integral.x;
}

fn shadow_source_rect(inst: QuadInstance, base_rect: vec4<f32>) -> vec4<f32> {
  let spread = inst.shadow_params.z;
  return vec4<f32>(
    base_rect.xy + inst.shadow_params.xy - vec2<f32>(spread),
    max(base_rect.zw + vec2<f32>(2.0 * spread), vec2<f32>(0.0))
  );
}

fn shadow_source_radii(
  base_radii: vec4<f32>,
  source_size: vec2<f32>,
  spread: f32,
) -> vec4<f32> {
  let max_radius = max(min(source_size.x, source_size.y) * 0.5, 0.0);
  return clamp(base_radii + vec4<f32>(spread), vec4<f32>(0.0), vec4<f32>(max_radius));
}

fn shadow_blurred_alpha(
  point: vec2<f32>,
  source_rect: vec4<f32>,
  source_radii: vec4<f32>,
  sigma: f32,
) -> f32 {
  // Four midpoint samples are sufficient for small soft shadows, but shadcn-style `shadow-lg` /
  // `shadow-xl` profiles combine large blur radii with negative spread. That makes corner error
  // more visible along the bottom shoulder. Eight midpoint samples remain cheap while tracking a
  // higher-sample reference much more closely for those profiles.
  let sample_count = 8.0;
  let half_size = source_rect.zw * 0.5;
  let center = source_rect.xy + half_size;
  let center_to_point = point - center;
  let corner_radius = pick_corner_radius(center_to_point, source_radii);

  let low = center_to_point.y - half_size.y;
  let high = center_to_point.y + half_size.y;
  let start = clamp(-3.0 * sigma, low, high);
  let end = clamp(3.0 * sigma, low, high);
  let step = (end - start) / sample_count;

  var y = start + step * 0.5;
  var alpha = 0.0;
  for (var i = 0; i < 8; i = i + 1) {
    let blur =
      shadow_blur_along_x(center_to_point.x, center_to_point.y - y, sigma, corner_radius, half_size);
    alpha = alpha + blur * shadow_gaussian(y, sigma) * step;
    y = y + step;
  }

  return clamp(alpha, 0.0, 1.0);
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let clip = clip_alpha(input.pixel_pos);
  let mask = mask_alpha(input.pixel_pos);
  let inst = quad_instances.instances[input.instance_index];

  if (FRET_SHADOW_MODE != 0u) {
    let base_rect = input.rect;
    let source_rect = shadow_source_rect(inst, base_rect);
    if (source_rect.z <= 0.0 || source_rect.w <= 0.0) {
      return vec4<f32>(0.0);
    }

    let spread = inst.shadow_params.z;
    let blur_radius = max(inst.shadow_params.w, 0.0);
    let source_radii = shadow_source_radii(input.corner_radii, source_rect.zw, spread);
    let source_sdf = quad_sdf(input.local_pos, source_rect.xy, source_rect.zw, source_radii);
    let hard_source_alpha = sdf_coverage_smooth(source_sdf);
    let blurred_alpha = shadow_blurred_alpha(
      input.local_pos,
      source_rect,
      source_radii,
      max(blur_radius, 1e-3)
    );
    let shadow_alpha = select(blurred_alpha, hard_source_alpha, blur_radius <= 1e-3);

    let content_sdf = quad_sdf(input.local_pos, base_rect.xy, base_rect.zw, input.corner_radii);
    let content_alpha = sdf_coverage_smooth(content_sdf);
    let shadow_only_alpha = shadow_alpha * (1.0 - content_alpha);
    let out = inst.fill_paint.params0 * shadow_only_alpha * clip * mask;
    return encode_output_premul(out);
  }

  let outer_sdf = quad_sdf(input.local_pos, input.rect.xy, input.rect.zw, input.corner_radii);

  // NOTE: AA must scale with derivatives. A fixed threshold (e.g. 0.5) breaks under DPI changes
  // and transforms. See ADR 0030.
  let aa_outer = max(fwidth(outer_sdf), 1e-4);
  let alpha_outer = 1.0 - smoothstep(-aa_outer, aa_outer, outer_sdf);

  var alpha_fill = alpha_outer;
  var border_cov = 0.0;
  if (FRET_BORDER_PRESENT != 0u) {
    // Border alignment: inside. Inner radii are derived by subtracting adjacent border widths.
    let inner_origin = input.rect.xy + vec2<f32>(input.border.x, input.border.y);
    let inner_size = input.rect.zw - vec2<f32>(input.border.x + input.border.z, input.border.y + input.border.w);

    let inner_radii = max(
      vec4<f32>(0.0),
      vec4<f32>(
        input.corner_radii.x - max(input.border.x, input.border.y), // TL
        input.corner_radii.y - max(input.border.z, input.border.y), // TR
        input.corner_radii.z - max(input.border.z, input.border.w), // BR
        input.corner_radii.w - max(input.border.x, input.border.w)  // BL
      )
    );

    let inner_sdf = quad_sdf(input.local_pos, inner_origin, max(inner_size, vec2<f32>(0.0)), inner_radii);
    let aa_inner = max(fwidth(inner_sdf), 1e-4);
    let alpha_inner_raw = 1.0 - smoothstep(-aa_inner, aa_inner, inner_sdf);
    let inner_valid = inner_size.x > 0.0 && inner_size.y > 0.0;
    let alpha_inner = select(0.0, alpha_inner_raw, inner_valid);

    alpha_fill = alpha_inner;
    border_cov = alpha_outer * (1.0 - alpha_inner);
  }

  let fill = paint_eval_fill(inst.fill_paint, input.local_pos, input.pixel_pos) * alpha_fill;
  var border = vec4<f32>(0.0);
  if (FRET_BORDER_PRESENT != 0u) {
    var dash_mask = 1.0;
    if (FRET_DASH_ENABLED != 0u) {
      let dash = inst.dash_params.x;
      let gap = inst.dash_params.y;
      let phase = inst.dash_params.z;
      let period = dash + gap;
      let period_safe = max(period, 1e-6);
      let s = rrect_perimeter_s(input.local_pos, input.rect, input.corner_radii);
      let tt = s + phase;
      let m = tt - floor(tt / period_safe) * period_safe;
      let aa = max(fwidth(s), 1e-4);
      let on_start = smoothstep(0.0, aa, m);
      let on_end = 1.0 - smoothstep(dash - aa, dash + aa, m);
      dash_mask = on_start * on_end;
    }
    var border_local_pos = input.local_pos;
    if (inst.border_paint.eval_space == 2u) {
      let s = rrect_perimeter_s(input.local_pos, input.rect, input.corner_radii);
      let len = rrect_perimeter_len(input.rect, input.corner_radii);
      let s01 = select(0.0, clamp(s / len, 0.0, 1.0), len > 1e-6);
      border_local_pos = vec2<f32>(s01, 0.0);
    }
    border = paint_eval_border(inst.border_paint, border_local_pos, input.pixel_pos) * border_cov * dash_mask;
  }

  let out = (fill + border) * clip * mask;
  return encode_output_premul(out);
}
