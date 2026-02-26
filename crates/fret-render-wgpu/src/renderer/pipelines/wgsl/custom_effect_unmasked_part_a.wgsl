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

// Renderer-owned pattern atlas (deterministic utility inputs).
//
// Layers (current contract):
// - 0: hash noise (64x64)
// - 1: Bayer 8x8 repeated (64x64)
//
// Notes:
// - This is intentionally small and deterministic. It exists to unlock high-end recipes (acrylic,
//   grain, scanlines, ordered dither) without introducing user-provided textures in CustomV1.
// - The atlas is populated by the renderer; see `GpuTextures::ensure_material_catalog_uploaded`.
@group(0) @binding(3) var fret_material_catalog_texture: texture_2d_array<f32>;
@group(0) @binding(4) var fret_material_catalog_sampler: sampler;

const FRET_MATERIAL_CATALOG_LAYER_HASH_NOISE: i32 = 0;
const FRET_MATERIAL_CATALOG_LAYER_BAYER8X8: i32 = 1;

fn fret_local_px(pos_px: vec2<f32>) -> vec2<f32> {
  return pos_px - render_space.origin_px;
}

fn fret_catalog_hash_noise01(pos_px: vec2<f32>) -> f32 {
  // Atlas is 64x64; use bitmasking for stable tiling and avoid `%` on negative.
  let x = i32(floor(pos_px.x)) & 63;
  let y = i32(floor(pos_px.y)) & 63;
  return textureLoad(
    fret_material_catalog_texture,
    vec2<i32>(x, y),
    FRET_MATERIAL_CATALOG_LAYER_HASH_NOISE,
    0
  ).r;
}

fn fret_catalog_bayer8x8_01(pos_px: vec2<f32>) -> f32 {
  let x = i32(floor(pos_px.x)) & 63;
  let y = i32(floor(pos_px.y)) & 63;
  return textureLoad(
    fret_material_catalog_texture,
    vec2<i32>(x, y),
    FRET_MATERIAL_CATALOG_LAYER_BAYER8X8,
    0
  ).r;
}

@group(1) @binding(0) var src_texture: texture_2d<f32>;

struct EffectParamsV1 {
  vec4s: array<vec4<f32>, 4>,
};

@group(1) @binding(1) var<uniform> params: EffectParamsV1;

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
