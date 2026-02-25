@group(0) @binding(0) var src_texture: texture_2d<f32>;

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

fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
  let a = 0.055;
  let lo = rgb * 12.92;
  let hi = (1.0 + a) * pow(rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(a);
  return select(hi, lo, rgb <= vec3<f32>(0.0031308));
}

fn encode_premul_srgb(c: vec4<f32>) -> vec4<f32> {
  if (c.a <= 0.0) {
    return c;
  }
  // Do not clamp the upper range: callers may intentionally output values where `rgb > a`
  // (e.g. treating a straight-alpha source as premultiplied). Let the output target clamp.
  let un = max(c.rgb / c.a, vec3<f32>(0.0));
  let enc = linear_to_srgb(un);
  return vec4<f32>(enc * c.a, c.a);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = textureDimensions(src_texture);
  let x = u32(floor(pos.x));
  let y = u32(floor(pos.y));
  if (x >= dims.x || y >= dims.y) {
    return vec4<f32>(0.0);
  }
  let c = textureLoad(src_texture, vec2<i32>(i32(x), i32(y)), 0);
  return encode_premul_srgb(c);
}
