struct Params {
  range: u32,
  matrix: u32,
  _pad0: u32,
  _pad1: u32,
}

@group(0) @binding(0) var y_tex: texture_2d<f32>;
@group(0) @binding(1) var uv_tex: texture_2d<f32>;
@group(0) @binding(2) var<uniform> params: Params;

struct VsOut {
  @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) v: u32) -> VsOut {
  // Fullscreen triangle.
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>( 3.0, -1.0),
    vec2<f32>(-1.0,  3.0),
  );
  var out: VsOut;
  out.position = vec4<f32>(pos[v], 0.0, 1.0);
  return out;
}

fn srgb_channel_to_linear(c: f32) -> f32 {
  if (c <= 0.04045) {
    return c / 12.92;
  }
  return pow((c + 0.055) / 1.055, 2.4);
}

fn srgb_to_linear(rgb: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(
    srgb_channel_to_linear(rgb.r),
    srgb_channel_to_linear(rgb.g),
    srgb_channel_to_linear(rgb.b),
  );
}

fn coeffs_bt601() -> vec4<f32> {
  // (rv, gu, gv, bu)
  return vec4<f32>(1.4020, 0.344136, 0.714136, 1.7720);
}

fn coeffs_bt709() -> vec4<f32> {
  return vec4<f32>(1.5748, 0.187324, 0.468124, 1.8556);
}

fn coeffs_bt2020() -> vec4<f32> {
  return vec4<f32>(1.4746, 0.16455, 0.57135, 1.8814);
}

fn clamp01(x: f32) -> f32 {
  return clamp(x, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let x = i32(pos.x);
  let y = i32(pos.y);

  let yv = textureLoad(y_tex, vec2<i32>(x, y), 0).r;
  let uv = textureLoad(uv_tex, vec2<i32>(x / 2, y / 2), 0).rg;

  var yy: f32;
  var uu: f32;
  var vv: f32;
  if (params.range == 0u) {
    // Full range: Y in [0,1], UV in [-0.5,0.5].
    yy = yv;
    uu = uv.r - 0.5;
    vv = uv.g - 0.5;
  } else {
    // Limited range:
    // Y: [16,235] -> [0,1], UV: [16,240] -> [-0.5,0.5]
    yy = (yv - (16.0 / 255.0)) / (219.0 / 255.0);
    uu = (uv.r - (128.0 / 255.0)) / (224.0 / 255.0);
    vv = (uv.g - (128.0 / 255.0)) / (224.0 / 255.0);
  }

  var c: vec4<f32>;
  if (params.matrix == 0u) {
    c = coeffs_bt601();
  } else if (params.matrix == 2u) {
    c = coeffs_bt2020();
  } else {
    c = coeffs_bt709();
  }

  // Compute RGB in the same domain as the input (R'G'B').
  let rp = clamp01(yy + c.x * vv);
  let gp = clamp01(yy - c.y * uu - c.z * vv);
  let bp = clamp01(yy + c.w * uu);

  // Treat R'G'B' as sRGB-coded values and convert to linear so that writing into an sRGB
  // render target round-trips (and sampling later yields linear values).
  let rgb_linear = srgb_to_linear(vec3<f32>(rp, gp, bp));

  return vec4<f32>(rgb_linear, 1.0);
}

