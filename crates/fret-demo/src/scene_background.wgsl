struct Uniforms {
  v0: vec4<f32>,
  v1: vec4<f32>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VsOut {
  var positions = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>( 3.0, -1.0),
    vec2<f32>(-1.0,  3.0),
  );
  var out: VsOut;
  out.pos = vec4<f32>(positions[i], 0.0, 1.0);
  return out;
}

fn rotate(v: vec2<f32>, a: f32) -> vec2<f32> {
  let s = sin(a);
  let c = cos(a);
  return vec2<f32>(v.x * c - v.y * s, v.x * s + v.y * c);
}

fn line_aa(dist: f32, thickness: f32, aa: f32) -> f32 {
  return 1.0 - smoothstep(thickness, thickness + aa, dist);
}

@fragment
fn fs_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
  let target_px = u.v0.xy;
  let world_span = u.v0.z;
  let time = u.v0.w;
  let center = u.v1.xy;
  let zoom = max(u.v1.z, 0.0001);
  let rotation = u.v1.w;

  // Convert fragment pixel coords to NDC [-1, 1] with Y up.
  var ndc = (p.xy / target_px) * 2.0 - vec2<f32>(1.0, 1.0);
  ndc.y = -ndc.y;
  // Preserve square world cells across viewport aspect ratios by scaling X.
  ndc.x *= target_px.x / max(target_px.y, 1.0);

  // Time is used to animate preview viewports (e.g. Game view) without affecting editor viewports.
  let time_offset = vec2<f32>(time * 0.35, time * 0.15);
  let world = center + rotate(ndc * (world_span * 0.5) / zoom, rotation) + time_offset;

  // Checker cell color from world-space integer grid.
  let cell = floor(world);
  let check = (i32(cell.x) + i32(cell.y)) & 1;
  let base0 = vec3<f32>(0.09, 0.11, 0.16);
  let base1 = vec3<f32>(0.14, 0.18, 0.29);
  var col = select(base0, base1, check == 1);

  // Minor grid lines.
  let frac = fract(world);
  let dist = min(min(frac.x, 1.0 - frac.x), min(frac.y, 1.0 - frac.y));
  let g = line_aa(dist, 0.02, 0.02);
  col = mix(col, vec3<f32>(0.20, 0.22, 0.28), g * 0.6);

  // World axes.
  let ax_x = line_aa(abs(world.y), 0.03, 0.03);
  let ax_y = line_aa(abs(world.x), 0.03, 0.03);
  col = mix(col, vec3<f32>(0.70, 0.20, 0.20), ax_x);
  col = mix(col, vec3<f32>(0.20, 0.70, 0.20), ax_y);

  return vec4<f32>(col, 1.0);
}
