@group(0) @binding(0) var src_texture: texture_2d<f32>;

struct EffectParamsV1 {
  vec4s: array<vec4<f32>, 4>,
};

@group(0) @binding(1) var<uniform> params: EffectParamsV1;

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

