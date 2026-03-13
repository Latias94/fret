@group(0) @binding(0) var src_texture: texture_2d<f32>;

struct Params {
  row0: vec4<f32>,
  row1: vec4<f32>,
  row2: vec4<f32>,
  row3: vec4<f32>,
  bias: vec4<f32>,
};

@group(0) @binding(1) var<uniform> params: Params;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0),
  );
  var out: VsOut;
  out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
  return out;
}

fn saturate(x: f32) -> f32 {
  return clamp(x, 0.0, 1.0);
}

fn saturate4(v: vec4<f32>) -> vec4<f32> {
  return vec4<f32>(saturate(v.x), saturate(v.y), saturate(v.z), saturate(v.w));
}

fn apply_color_matrix(tex: vec4<f32>) -> vec4<f32> {
  let a = tex.a;
  if (a <= 0.0) {
    return vec4<f32>(0.0);
  }
  let rgb = tex.rgb / a;
  let v = vec4<f32>(rgb, a);
  let out = vec4<f32>(
    dot(params.row0, v) + params.bias.x,
    dot(params.row1, v) + params.bias.y,
    dot(params.row2, v) + params.bias.z,
    dot(params.row3, v) + params.bias.w,
  );
  let clamped = saturate4(out);
  return vec4<f32>(clamped.rgb * clamped.a, clamped.a);
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
  return apply_color_matrix(tex);
}
