struct ClipRRect {
  rect: vec4<f32>,
  corner_radii: vec4<f32>,
  inv0: vec4<f32>,
  inv1: vec4<f32>,
};

struct Viewport {
  viewport_size: vec2<f32>,
  clip_head: u32,
  clip_count: u32,
  mask_head: u32,
  mask_count: u32,
  mask_scope_head: u32,
  mask_scope_count: u32,
  output_is_srgb: u32,
  _pad0: u32,
  mask_viewport_origin: vec2<f32>,
  mask_viewport_size: vec2<f32>,
  text_gamma_ratios: vec4<f32>,
  text_grayscale_enhanced_contrast: f32,
  text_subpixel_enhanced_contrast: f32,
  _pad_text_quality0: u32,
  _pad_text_quality1: u32,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;

struct RenderSpace {
  origin_px: vec2<f32>,
  size_px: vec2<f32>,
};

@group(0) @binding(5) var<uniform> render_space: RenderSpace;
struct ClipStack {
  clips: array<ClipRRect>,
};

@group(0) @binding(1) var<storage, read> clip_stack: ClipStack;

struct MaskGradient {
  bounds: vec4<f32>,
  kind: u32,
  tile_mode: u32,
  stop_count: u32,
  _pad0: u32,
  params0: vec4<f32>,
  inv0: vec4<f32>,
  inv1: vec4<f32>,
  stop_alphas0: vec4<f32>,
  stop_alphas1: vec4<f32>,
  stop_offsets0: vec4<f32>,
  stop_offsets1: vec4<f32>,
};

struct MaskStack {
  masks: array<MaskGradient>,
};

@group(0) @binding(2) var<storage, read> mask_stack: MaskStack;

@group(0) @binding(6) var mask_image_sampler: sampler;
@group(0) @binding(7) var mask_image_texture: texture_2d<f32>;

@group(0) @binding(3) var material_catalog_texture: texture_2d_array<f32>;
@group(0) @binding(4) var material_catalog_sampler: sampler;

const MAX_STOPS: u32 = 8u;

override FRET_FILL_KIND: u32 = 0u;
override FRET_BORDER_KIND: u32 = 0u;
override FRET_BORDER_PRESENT: u32 = 1u;
override FRET_DASH_ENABLED: u32 = 0u;
override FRET_FILL_MATERIAL_SAMPLED: u32 = 0u;
override FRET_BORDER_MATERIAL_SAMPLED: u32 = 0u;

struct Paint {
  kind: u32,
  tile_mode: u32,
  color_space: u32,
  stop_count: u32,
  eval_space: u32,
  _pad_eval0: u32,
  _pad_eval1: u32,
  _pad_eval2: u32,
  params0: vec4<f32>,
  params1: vec4<f32>,
  params2: vec4<f32>,
  params3: vec4<f32>,
  stop_colors: array<vec4<f32>, 8>,
  stop_offsets0: vec4<f32>,
  stop_offsets1: vec4<f32>,
};

struct QuadInstance {
  rect: vec4<f32>,
  transform0: vec4<f32>,
  transform1: vec4<f32>,
  fill_paint: Paint,
  border_paint: Paint,
  corner_radii: vec4<f32>,
  border: vec4<f32>,
  dash_params: vec4<f32>,
};

struct QuadInstances {
  instances: array<QuadInstance>,
};

@group(1) @binding(0) var<storage, read> quad_instances: QuadInstances;

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) pixel_pos: vec2<f32>,
  @location(1) local_pos: vec2<f32>,
  @location(2) rect: vec4<f32>,
  @location(3) corner_radii: vec4<f32>,
  @location(4) border: vec4<f32>,
  @location(5) @interpolate(flat) instance_index: u32,
};

fn quad_vertex_xy(vertex_index: u32) -> vec2<f32> {
  switch vertex_index {
    case 0u: { return vec2<f32>(0.0, 0.0); }
    case 1u: { return vec2<f32>(1.0, 0.0); }
    case 2u: { return vec2<f32>(1.0, 1.0); }
    case 3u: { return vec2<f32>(0.0, 0.0); }
    case 4u: { return vec2<f32>(1.0, 1.0); }
    default: { return vec2<f32>(0.0, 1.0); }
  }
}

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let local = pixel_pos - render_space.origin_px;
  let ndc_x = (local.x / render_space.size_px.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (local.y / render_space.size_px.y) * 2.0;
  return vec2<f32>(ndc_x, ndc_y);
}

@vertex
fn vs_main(
  @builtin(vertex_index) vertex_index: u32,
  @builtin(instance_index) instance_index: u32,
) -> VsOut {
  let inst = quad_instances.instances[instance_index];
  let rect = inst.rect;
  let transform0 = inst.transform0;
  let transform1 = inst.transform1;
  let uv = quad_vertex_xy(vertex_index);
  let local_pos = rect.xy + uv * rect.zw;
  let pixel_pos = vec2<f32>(
    dot(transform0.xy, local_pos) + transform0.z,
    dot(transform1.xy, local_pos) + transform1.z
  );
  let clip_xy = to_clip_space(pixel_pos);

  var out: VsOut;
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.pixel_pos = pixel_pos;
  out.local_pos = local_pos;
  out.rect = rect;
  out.corner_radii = inst.corner_radii;
  out.border = inst.border;
  out.instance_index = instance_index;
  return out;
}
