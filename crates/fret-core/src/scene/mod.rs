use crate::{
    Px, SvgFit, ViewportFit,
    geometry::{Corners, Edges, Point, Rect, Transform2D},
    ids::{EffectId, ImageId, PathId, RenderTargetId, SvgId, TextBlobId},
};
use serde::{Deserialize, Serialize};
use slotmap::Key;

mod composite;
mod fingerprint;
mod image_object_fit;
mod mask;
mod paint;
mod replay;
mod stroke;
mod validate;

pub use composite::{BlendMode, CompositeGroupDesc};
use fingerprint::mix_scene_op;
pub use image_object_fit::{ImageObjectFitMapped, map_image_object_fit};
pub use mask::Mask;
pub use paint::{
    ColorSpace, GradientStop, LinearGradient, MAX_STOPS, MaterialParams, Paint, PaintBindingV1,
    PaintEvalSpaceV1, RadialGradient, SweepGradient, TileMode,
};
pub use stroke::{DashPatternV1, StrokeStyleV1};
pub use validate::{SceneValidationError, SceneValidationErrorKind};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageSamplingHint {
    /// Renderer-chosen default (typically linear filtering for UI content).
    #[default]
    Default,
    Linear,
    Nearest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawOrder(pub u32);

// `DrawOrder` is intentionally non-semantic for compositing. Scene operation order is authoritative.
// See `docs/adr/0081-draworder-is-non-semantic.md`.

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// Convert an sRGB `0xRRGGBB` hex color into a linear `Color` (alpha = 1.0).
    pub fn from_srgb_hex_rgb(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xff) as u8;
        let g = ((hex >> 8) & 0xff) as u8;
        let b = (hex & 0xff) as u8;
        Self {
            r: srgb_u8_to_linear(r),
            g: srgb_u8_to_linear(g),
            b: srgb_u8_to_linear(b),
            a: 1.0,
        }
    }

    /// Convert a linear `Color` to an sRGB `0xRRGGBB` hex value (alpha ignored).
    pub fn to_srgb_hex_rgb(self) -> u32 {
        let r = linear_to_srgb_u8(self.r) as u32;
        let g = linear_to_srgb_u8(self.g) as u32;
        let b = linear_to_srgb_u8(self.b) as u32;
        (r << 16) | (g << 8) | b
    }
}

fn srgb_f32_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_f32_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn srgb_u8_to_linear(u: u8) -> f32 {
    srgb_f32_to_linear(u as f32 / 255.0)
}

fn linear_to_srgb_u8(c: f32) -> u8 {
    let srgb = linear_f32_to_srgb(c.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    (srgb * 255.0).round() as u8
}

/// A bounded, portable text shadow surface (v1).
///
/// This is intentionally minimal (single layer, no blur) so it remains viable across wasm/mobile
/// backends. Higher-level shadow recipes (multi-layer elevation, blur, color management) remain
/// policy in ecosystem crates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextShadowV1 {
    /// Baseline-origin offset in logical pixels (pre-scale-factor).
    pub offset: Point,
    pub color: Color,
}

impl TextShadowV1 {
    pub const fn new(offset: Point, color: Color) -> Self {
        Self { offset, color }
    }
}

/// A bounded, portable text outline/stroke surface (v1).
///
/// This is intentionally minimal so it can be implemented deterministically across wasm/mobile
/// backends. More advanced strategies (e.g. SDF/MSDF atlases, multi-layer outlines) remain v2+.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextOutlineV1 {
    pub paint: PaintBindingV1,
    /// Outline width in logical pixels (pre-scale-factor).
    pub width_px: crate::Px,
}

impl TextOutlineV1 {
    pub const MAX_WIDTH_PX: crate::Px = crate::Px(8.0);

    pub fn sanitize(self) -> Option<Self> {
        if !self.width_px.0.is_finite() {
            return None;
        }
        let width_px = crate::Px(self.width_px.0.clamp(0.0, Self::MAX_WIDTH_PX.0));
        if width_px.0 <= 0.0 {
            return None;
        }
        Some(Self {
            paint: self.paint.sanitize(),
            width_px,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectMode {
    /// Render children to an offscreen intermediate, then filter and composite the result.
    FilterContent,
    /// Sample already-rendered backdrop behind the group, filter it, then draw children on top.
    Backdrop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectQuality {
    /// Renderer-chosen quality within budgets (ADR 0118).
    Auto,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DitherMode {
    Bayer4x4,
}

/// Fixed-size custom effect parameters (v1).
///
/// This is intentionally a small, bounded payload for capability-gated custom effects.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectParamsV1 {
    pub vec4s: [[f32; 4]; 4],
}

impl EffectParamsV1 {
    pub const ZERO: Self = Self {
        vec4s: [[0.0; 4]; 4],
    };

    pub fn sanitize(self) -> Self {
        let mut out = self;
        for v in &mut out.vec4s {
            for x in v {
                if !x.is_finite() {
                    *x = 0.0;
                }
            }
        }
        out
    }

    pub fn is_finite(self) -> bool {
        self.vec4s.iter().flatten().all(|&x| x.is_finite())
    }
}

/// Bounded procedural noise parameters (v1).
///
/// This is a mechanism-level surface intended to enable authored “grain” layers that are useful
/// for acrylic/glass recipes (e.g. subtle noise after backdrop blur) while remaining deterministic
/// (no hidden time dependency).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoiseV1 {
    /// Additive noise magnitude in linear space. Recommended range is ~[0, 0.1].
    pub strength: f32,
    /// Spatial scale for the noise field in logical pixels (pre-scale-factor).
    pub scale_px: crate::Px,
    /// Deterministic phase/seed value (no hidden time dependency).
    pub phase: f32,
}

impl NoiseV1 {
    pub const MAX_STRENGTH: f32 = 1.0;
    pub const MIN_SCALE_PX: crate::Px = crate::Px(1.0);
    pub const MAX_SCALE_PX: crate::Px = crate::Px(1024.0);

    pub fn sanitize(self) -> Self {
        let strength = if self.strength.is_finite() {
            self.strength.clamp(0.0, Self::MAX_STRENGTH)
        } else {
            0.0
        };
        let scale_px = if self.scale_px.0.is_finite() {
            crate::Px(
                self.scale_px
                    .0
                    .clamp(Self::MIN_SCALE_PX.0, Self::MAX_SCALE_PX.0),
            )
        } else {
            Self::MIN_SCALE_PX
        };
        let phase = if self.phase.is_finite() {
            self.phase
        } else {
            0.0
        };

        Self {
            strength,
            scale_px,
            phase,
        }
    }
}

/// Bounded backdrop warp parameters (v1).
///
/// This is a mechanism-level surface intended to enable refraction-like liquid glass effects by
/// sampling the already-rendered backdrop with a deterministic UV displacement. Higher-level
/// recipes (normal-map assets, interaction curves, multi-layer stacks) remain ecosystem policy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackdropWarpV1 {
    /// Displacement strength in logical pixels (pre-scale-factor).
    pub strength_px: crate::Px,
    /// Spatial scale for the warp field in logical pixels.
    pub scale_px: crate::Px,
    /// Deterministic phase/seed value (no hidden time dependency).
    pub phase: f32,
    /// Optional chromatic aberration magnitude in logical pixels.
    pub chromatic_aberration_px: crate::Px,
    pub kind: BackdropWarpKindV1,
}

impl BackdropWarpV1 {
    pub const MAX_STRENGTH_PX: crate::Px = crate::Px(24.0);
    pub const MIN_SCALE_PX: crate::Px = crate::Px(1.0);
    pub const MAX_SCALE_PX: crate::Px = crate::Px(1024.0);
    pub const MAX_CHROMATIC_ABERRATION_PX: crate::Px = crate::Px(8.0);

    pub fn sanitize(self) -> Self {
        let strength_px = if self.strength_px.0.is_finite() {
            crate::Px(self.strength_px.0.clamp(0.0, Self::MAX_STRENGTH_PX.0))
        } else {
            crate::Px(0.0)
        };

        let scale_px = if self.scale_px.0.is_finite() {
            crate::Px(
                self.scale_px
                    .0
                    .clamp(Self::MIN_SCALE_PX.0, Self::MAX_SCALE_PX.0),
            )
        } else {
            Self::MIN_SCALE_PX
        };

        let phase = if self.phase.is_finite() {
            self.phase
        } else {
            0.0
        };

        let chromatic_aberration_px = if self.chromatic_aberration_px.0.is_finite() {
            crate::Px(
                self.chromatic_aberration_px
                    .0
                    .clamp(0.0, Self::MAX_CHROMATIC_ABERRATION_PX.0),
            )
        } else {
            crate::Px(0.0)
        };

        Self {
            strength_px,
            scale_px,
            phase,
            chromatic_aberration_px,
            kind: self.kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarpMapEncodingV1 {
    /// Decode displacement from RG in [0, 1] mapped to [-1, 1].
    ///
    /// This is a good default for authored displacement maps.
    RgSigned,
    /// Decode a normal from RGB in [0, 1] mapped to [-1, 1], and use XY as displacement.
    ///
    /// This is convenient when the warp field is stored as a normal map.
    NormalRgb,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackdropWarpFieldV2 {
    /// Use the procedural v1 warp field (`BackdropWarpV1`).
    Procedural,
    /// Use an image-driven displacement/normal map as the warp field.
    ImageDisplacementMap {
        image: ImageId,
        uv: UvRect,
        sampling: ImageSamplingHint,
        encoding: WarpMapEncodingV1,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackdropWarpV2 {
    /// The v1 parameters are retained as the portable base (and deterministic fallback).
    pub base: BackdropWarpV1,
    pub field: BackdropWarpFieldV2,
}

impl BackdropWarpV2 {
    pub fn sanitize(self) -> Self {
        let base = self.base.sanitize();
        let field = match self.field {
            BackdropWarpFieldV2::Procedural => BackdropWarpFieldV2::Procedural,
            BackdropWarpFieldV2::ImageDisplacementMap {
                image,
                uv,
                sampling,
                encoding,
            } => {
                let uv = if uv.u0.is_finite()
                    && uv.v0.is_finite()
                    && uv.u1.is_finite()
                    && uv.v1.is_finite()
                {
                    uv
                } else {
                    UvRect::FULL
                };
                BackdropWarpFieldV2::ImageDisplacementMap {
                    image,
                    uv,
                    sampling,
                    encoding,
                }
            }
        };

        Self { base, field }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackdropWarpKindV1 {
    Wave,
    /// Reserved for a lens-like warp in a future v1.x/v2. Renderers may treat this as `Wave`.
    LensReserved,
}

/// Bounded drop shadow parameters (v1).
///
/// This is a mechanism-level, blur-based shadow surface intended for general UI content (cards,
/// popovers, overlays). It is explicitly bounded and deterministic so it remains viable on
/// wasm/WebGPU and mobile GPUs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropShadowV1 {
    /// Shadow offset in logical pixels (pre-scale-factor).
    pub offset_px: Point,
    /// Blur radius in logical pixels (pre-scale-factor).
    pub blur_radius_px: crate::Px,
    /// Downsample hint (1–4). Renderers may degrade deterministically under budgets.
    pub downsample: u32,
    /// Solid shadow color (unpremultiplied RGBA in [0, 1]).
    pub color: Color,
}

impl DropShadowV1 {
    pub const MAX_BLUR_RADIUS_PX: crate::Px = crate::Px(64.0);

    pub fn sanitize(self) -> Self {
        let offset_px = Point::new(
            crate::Px(if self.offset_px.x.0.is_finite() {
                self.offset_px.x.0
            } else {
                0.0
            }),
            crate::Px(if self.offset_px.y.0.is_finite() {
                self.offset_px.y.0
            } else {
                0.0
            }),
        );
        let blur_radius_px = if self.blur_radius_px.0.is_finite() {
            crate::Px(self.blur_radius_px.0.clamp(0.0, Self::MAX_BLUR_RADIUS_PX.0))
        } else {
            crate::Px(0.0)
        };
        let downsample = self.downsample.clamp(1, 4);
        let color = Color {
            r: if self.color.r.is_finite() {
                self.color.r.clamp(0.0, 1.0)
            } else {
                0.0
            },
            g: if self.color.g.is_finite() {
                self.color.g.clamp(0.0, 1.0)
            } else {
                0.0
            },
            b: if self.color.b.is_finite() {
                self.color.b.clamp(0.0, 1.0)
            } else {
                0.0
            },
            a: if self.color.a.is_finite() {
                self.color.a.clamp(0.0, 1.0)
            } else {
                0.0
            },
        };

        Self {
            offset_px,
            blur_radius_px,
            downsample,
            color,
        }
    }
}

/// Optional user image input for bounded custom effects (v2).
///
/// This is intentionally small and portable: it references an `ImageId` registered through the
/// existing image service and uses the existing `ImageSamplingHint` vocabulary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CustomEffectImageInputV1 {
    pub image: ImageId,
    pub uv: UvRect,
    pub sampling: ImageSamplingHint,
}

impl CustomEffectImageInputV1 {
    pub const fn new(image: ImageId) -> Self {
        Self {
            image,
            uv: UvRect::FULL,
            sampling: ImageSamplingHint::Default,
        }
    }
}

/// Custom effect source selection for CustomV3.
///
/// This stays bounded and deterministic: callers can request a distinct `src_raw` source and an
/// optional bounded pyramid, but backends may degrade by aliasing sources under budgets or
/// unsupported capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CustomEffectSourcesV3 {
    pub want_raw: bool,
    pub pyramid: Option<CustomEffectPyramidRequestV1>,
}

/// Bounded request for a renderer-owned blur pyramid derived from `src_raw` (v3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CustomEffectPyramidRequestV1 {
    pub max_levels: u8,
    pub max_radius_px: crate::Px,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectStep {
    GaussianBlur {
        radius_px: crate::Px,
        downsample: u32,
    },
    DropShadowV1(DropShadowV1),
    BackdropWarpV1(BackdropWarpV1),
    BackdropWarpV2(BackdropWarpV2),
    NoiseV1(NoiseV1),
    ColorAdjust {
        saturation: f32,
        brightness: f32,
        contrast: f32,
    },
    ColorMatrix {
        m: [f32; 20],
    },
    AlphaThreshold {
        cutoff: f32,
        soft: f32,
    },
    Pixelate {
        scale: u32,
    },
    Dither {
        mode: DitherMode,
    },
    CustomV1 {
        id: EffectId,
        params: EffectParamsV1,
        /// Maximum sampling offset (in logical px) that the custom effect may use when reading
        /// from its source texture.
        ///
        /// This is a bounded contract input used by renderers to deterministically allocate
        /// enough context ("padding") for effect chains (e.g. blur -> custom refraction) without
        /// introducing edge artifacts near clipped/scissored bounds.
        ///
        /// Backends may clamp or degrade behavior under tight budgets.
        max_sample_offset_px: crate::Px,
    },
    CustomV2 {
        id: EffectId,
        params: EffectParamsV1,
        /// Maximum sampling offset (in logical px) that the custom effect may use when reading
        /// from its source texture.
        ///
        /// This preserves the deterministic chain padding story from v1.
        max_sample_offset_px: crate::Px,
        /// Optional user-provided image input (v2 ceiling bump).
        input_image: Option<CustomEffectImageInputV1>,
    },
    CustomV3 {
        id: EffectId,
        params: EffectParamsV1,
        /// Maximum sampling offset (in logical px) that the custom effect may use when reading
        /// from its source textures.
        ///
        /// This preserves the deterministic chain padding story from v1/v2.
        max_sample_offset_px: crate::Px,
        /// Optional user-provided image input 0 (v2-compatible).
        user0: Option<CustomEffectImageInputV1>,
        /// Optional user-provided image input 1 (v3 ceiling bump).
        user1: Option<CustomEffectImageInputV1>,
        /// Renderer-provided sources request (raw + optional pyramid).
        sources: CustomEffectSourcesV3,
    },
}

impl EffectStep {
    pub fn sanitize(self) -> Self {
        match self {
            EffectStep::ColorMatrix { mut m } => {
                for v in &mut m {
                    if !v.is_finite() {
                        *v = 0.0;
                    }
                }
                EffectStep::ColorMatrix { m }
            }
            EffectStep::BackdropWarpV1(w) => EffectStep::BackdropWarpV1(w.sanitize()),
            EffectStep::BackdropWarpV2(w) => EffectStep::BackdropWarpV2(w.sanitize()),
            EffectStep::DropShadowV1(s) => EffectStep::DropShadowV1(s.sanitize()),
            EffectStep::NoiseV1(n) => EffectStep::NoiseV1(n.sanitize()),
            EffectStep::CustomV1 {
                id,
                params,
                max_sample_offset_px,
            } => EffectStep::CustomV1 {
                id,
                params: params.sanitize(),
                max_sample_offset_px: if max_sample_offset_px.0.is_finite() {
                    crate::Px(max_sample_offset_px.0.max(0.0))
                } else {
                    crate::Px(0.0)
                },
            },
            EffectStep::CustomV2 {
                id,
                params,
                max_sample_offset_px,
                input_image,
            } => EffectStep::CustomV2 {
                id,
                params: params.sanitize(),
                max_sample_offset_px: if max_sample_offset_px.0.is_finite() {
                    crate::Px(max_sample_offset_px.0.max(0.0))
                } else {
                    crate::Px(0.0)
                },
                input_image: input_image.map(|mut input| {
                    if !input.uv.u0.is_finite() {
                        input.uv.u0 = 0.0;
                    }
                    if !input.uv.v0.is_finite() {
                        input.uv.v0 = 0.0;
                    }
                    if !input.uv.u1.is_finite() {
                        input.uv.u1 = 1.0;
                    }
                    if !input.uv.v1.is_finite() {
                        input.uv.v1 = 1.0;
                    }
                    input
                }),
            },
            EffectStep::CustomV3 {
                id,
                params,
                max_sample_offset_px,
                user0,
                user1,
                mut sources,
            } => {
                let sanitize_input = |input: Option<CustomEffectImageInputV1>| {
                    input.map(|mut input| {
                        if !input.uv.u0.is_finite() {
                            input.uv.u0 = 0.0;
                        }
                        if !input.uv.v0.is_finite() {
                            input.uv.v0 = 0.0;
                        }
                        if !input.uv.u1.is_finite() {
                            input.uv.u1 = 1.0;
                        }
                        if !input.uv.v1.is_finite() {
                            input.uv.v1 = 1.0;
                        }
                        input
                    })
                };

                sources.pyramid = sources.pyramid.map(|req| {
                    let max_levels = req.max_levels.clamp(1, 7);
                    let max_radius_px = if req.max_radius_px.0.is_finite() {
                        crate::Px(req.max_radius_px.0.max(0.0))
                    } else {
                        crate::Px(0.0)
                    };
                    CustomEffectPyramidRequestV1 {
                        max_levels,
                        max_radius_px,
                    }
                });

                EffectStep::CustomV3 {
                    id,
                    params: params.sanitize(),
                    max_sample_offset_px: if max_sample_offset_px.0.is_finite() {
                        crate::Px(max_sample_offset_px.0.max(0.0))
                    } else {
                        crate::Px(0.0)
                    },
                    user0: sanitize_input(user0),
                    user1: sanitize_input(user1),
                    sources,
                }
            }
            EffectStep::AlphaThreshold { cutoff, soft } => EffectStep::AlphaThreshold {
                cutoff: if cutoff.is_finite() { cutoff } else { 0.0 },
                soft: if soft.is_finite() { soft.max(0.0) } else { 0.0 },
            },
            other => other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectChain {
    steps: [Option<EffectStep>; 4],
}

impl EffectChain {
    pub const MAX_STEPS: usize = 4;
    pub const EMPTY: Self = Self {
        steps: [None, None, None, None],
    };

    pub fn from_steps(steps: &[EffectStep]) -> Self {
        assert!(
            steps.len() <= Self::MAX_STEPS,
            "EffectChain supports up to {} steps",
            Self::MAX_STEPS
        );
        let mut out = Self::EMPTY;
        for (idx, step) in steps.iter().copied().enumerate() {
            out.steps[idx] = Some(step);
        }
        out
    }

    pub fn is_empty(&self) -> bool {
        self.steps.iter().all(|s| s.is_none())
    }

    pub fn iter(&self) -> impl Iterator<Item = EffectStep> + '_ {
        self.steps.iter().copied().flatten()
    }

    pub fn sanitize(self) -> Self {
        let mut out = self;
        for step in &mut out.steps {
            *step = step.map(EffectStep::sanitize);
        }
        out
    }
}

impl Default for EffectChain {
    fn default() -> Self {
        Self::EMPTY
    }
}

#[derive(Debug, Default, Clone)]
pub struct SceneRecording {
    ops: Vec<SceneOp>,
    fingerprint: u64,
    #[cfg(debug_assertions)]
    storage_swapped_since_clear: bool,
}

pub type Scene = SceneRecording;

impl SceneRecording {
    pub fn clear(&mut self) {
        self.ops.clear();
        self.fingerprint = 0;
        #[cfg(debug_assertions)]
        {
            self.storage_swapped_since_clear = false;
        }
    }

    pub fn push(&mut self, op: SceneOp) {
        // Clamp quad corner radii to the local rect size (CSS-style effective border radius).
        //
        // Browsers constrain border radii to half the corresponding box dimension. Many shadcn
        // components use `rounded-full`, which maps to an arbitrarily large radius that becomes
        // `min(width, height) / 2` in practice. Keeping this normalization at the scene layer makes
        // renderer backends and scripted tests agree on the effective shape.
        let op = match op {
            SceneOp::Quad {
                order,
                rect,
                background,
                border,
                border_paint,
                mut corner_radii,
            } => {
                let max = rect.size.width.0.min(rect.size.height.0) * 0.5;
                let max = if max.is_finite() { max.max(0.0) } else { 0.0 };
                corner_radii.top_left = Px(corner_radii.top_left.0.min(max));
                corner_radii.top_right = Px(corner_radii.top_right.0.min(max));
                corner_radii.bottom_left = Px(corner_radii.bottom_left.0.min(max));
                corner_radii.bottom_right = Px(corner_radii.bottom_right.0.min(max));

                SceneOp::Quad {
                    order,
                    rect,
                    background: background.sanitize(),
                    border,
                    border_paint: border_paint.sanitize(),
                    corner_radii,
                }
            }
            SceneOp::ShadowRRect {
                order,
                rect,
                mut corner_radii,
                offset,
                spread,
                blur_radius,
                color,
            } => {
                let effective_width = (rect.size.width.0 + spread.0 * 2.0).max(0.0);
                let effective_height = (rect.size.height.0 + spread.0 * 2.0).max(0.0);
                let max = effective_width.min(effective_height) * 0.5;
                let max = if max.is_finite() { max.max(0.0) } else { 0.0 };
                corner_radii.top_left = Px((corner_radii.top_left.0 + spread.0).max(0.0).min(max));
                corner_radii.top_right =
                    Px((corner_radii.top_right.0 + spread.0).max(0.0).min(max));
                corner_radii.bottom_left =
                    Px((corner_radii.bottom_left.0 + spread.0).max(0.0).min(max));
                corner_radii.bottom_right =
                    Px((corner_radii.bottom_right.0 + spread.0).max(0.0).min(max));

                let blur_radius = if blur_radius.0.is_finite() {
                    Px(blur_radius
                        .0
                        .clamp(0.0, SHADOW_RRECT_V1_MAX_BLUR_RADIUS_PX.0))
                } else {
                    Px(0.0)
                };

                SceneOp::ShadowRRect {
                    order,
                    rect,
                    corner_radii,
                    offset,
                    spread,
                    blur_radius,
                    color,
                }
            }
            SceneOp::PushEffect {
                bounds,
                mode,
                chain,
                quality,
            } => SceneOp::PushEffect {
                bounds,
                mode,
                chain: chain.sanitize(),
                quality,
            },
            SceneOp::Text {
                order,
                origin,
                text,
                paint,
                outline,
                shadow,
            } => SceneOp::Text {
                order,
                origin,
                text,
                paint: paint.sanitize(),
                outline: outline.and_then(|o| o.sanitize()),
                shadow,
            },
            other => other,
        };

        self.fingerprint = mix_scene_op(self.fingerprint, op);
        self.ops.push(op);
    }

    pub fn with_transform<T>(
        &mut self,
        transform: Transform2D,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.push(SceneOp::PushTransform { transform });
        let out = f(self);
        self.push(SceneOp::PopTransform);
        out
    }

    pub fn with_opacity<T>(&mut self, opacity: f32, f: impl FnOnce(&mut Self) -> T) -> T {
        self.push(SceneOp::PushOpacity { opacity });
        let out = f(self);
        self.push(SceneOp::PopOpacity);
        out
    }

    pub fn with_layer<T>(&mut self, layer: u32, f: impl FnOnce(&mut Self) -> T) -> T {
        self.push(SceneOp::PushLayer { layer });
        let out = f(self);
        self.push(SceneOp::PopLayer);
        out
    }

    pub fn with_clip_rect<T>(&mut self, rect: Rect, f: impl FnOnce(&mut Self) -> T) -> T {
        self.push(SceneOp::PushClipRect { rect });
        let out = f(self);
        self.push(SceneOp::PopClip);
        out
    }

    pub fn with_clip_rrect<T>(
        &mut self,
        rect: Rect,
        corner_radii: Corners,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.push(SceneOp::PushClipRRect { rect, corner_radii });
        let out = f(self);
        self.push(SceneOp::PopClip);
        out
    }

    pub fn with_clip_path<T>(
        &mut self,
        bounds: Rect,
        origin: Point,
        path: PathId,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.push(SceneOp::PushClipPath {
            bounds,
            origin,
            path,
        });
        let out = f(self);
        self.push(SceneOp::PopClip);
        out
    }

    pub fn with_mask<T>(&mut self, bounds: Rect, mask: Mask, f: impl FnOnce(&mut Self) -> T) -> T {
        self.push(SceneOp::PushMask { bounds, mask });
        let out = f(self);
        self.push(SceneOp::PopMask);
        out
    }

    pub fn with_effect<T>(
        &mut self,
        bounds: Rect,
        mode: EffectMode,
        chain: EffectChain,
        quality: EffectQuality,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.push(SceneOp::PushEffect {
            bounds,
            mode,
            chain,
            quality,
        });
        let out = f(self);
        self.push(SceneOp::PopEffect);
        out
    }

    pub fn with_composite_group<T>(
        &mut self,
        desc: CompositeGroupDesc,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.push(SceneOp::PushCompositeGroup { desc });
        let out = f(self);
        self.push(SceneOp::PopCompositeGroup);
        out
    }

    pub fn with_backdrop_source_group_v1<T>(
        &mut self,
        bounds: Rect,
        pyramid: Option<CustomEffectPyramidRequestV1>,
        quality: EffectQuality,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.push(SceneOp::PushBackdropSourceGroupV1 {
            bounds,
            pyramid,
            quality,
        });
        let out = f(self);
        self.push(SceneOp::PopBackdropSourceGroup);
        out
    }

    pub fn ops(&self) -> &[SceneOp] {
        &self.ops
    }

    pub fn ops_len(&self) -> usize {
        self.ops.len()
    }

    /// Swap the internal op storage with an external buffer.
    ///
    /// This is a performance-oriented API used by subsystems like the UI paint cache to "take"
    /// the previous frame's ops without copying.
    ///
    /// In debug builds, this asserts if called more than once without an intervening `clear()`,
    /// because repeated swaps typically indicate multiple paint-cache ingestions from the same scene.
    pub fn swap_storage(&mut self, other_ops: &mut Vec<SceneOp>, other_fingerprint: &mut u64) {
        #[cfg(debug_assertions)]
        debug_assert!(
            !self.storage_swapped_since_clear,
            "Scene::swap_storage() was called more than once without an intervening Scene::clear(); \
this is not supported because swap_storage() is destructive and typically indicates multiple paint-cache ingestions"
        );
        std::mem::swap(&mut self.ops, other_ops);
        std::mem::swap(&mut self.fingerprint, other_fingerprint);
        #[cfg(debug_assertions)]
        {
            self.storage_swapped_since_clear = true;
        }
    }

    pub fn fingerprint(&self) -> u64 {
        self.fingerprint
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SceneOp {
    PushTransform {
        transform: Transform2D,
    },
    PopTransform,

    /// Opacity multiplier applied to subsequent draw ops.
    ///
    /// The opacity stack composes multiplicatively (parent * child).
    PushOpacity {
        opacity: f32,
    },
    PopOpacity,

    /// Reserved layer stack marker (ADR 0019).
    PushLayer {
        layer: u32,
    },
    PopLayer,

    PushClipRect {
        rect: Rect,
    },
    PushClipRRect {
        rect: Rect,
        corner_radii: Corners,
    },
    /// Push a path-based clip entry (clip-path).
    ///
    /// `bounds` is a computation bound (not an implicit clip) used to bound GPU work and enable
    /// deterministic budgeting/degradation. The clip geometry is given by a prepared path handle.
    ///
    /// v1 note: renderers may implement this via an offscreen intermediate + mask composite.
    PushClipPath {
        bounds: Rect,
        origin: Point,
        path: PathId,
    },
    PopClip,

    PushMask {
        /// Computation bounds (not an implicit clip), see ADR 0239.
        bounds: Rect,
        mask: Mask,
    },
    PopMask,

    PushEffect {
        /// Computation bounds (not an implicit clip), see ADR 0117.
        bounds: Rect,
        mode: EffectMode,
        chain: EffectChain,
        quality: EffectQuality,
    },
    PopEffect,

    /// Backdrop source group (v1): a mechanism-level scope that enables renderers to share a raw
    /// backdrop snapshot (and optional pyramid) across multiple CustomV3 “liquid glass” surfaces.
    ///
    /// `bounds` are computation bounds (not an implicit clip). `pyramid` is an optional bounded
    /// request for a shared renderer-owned blur pyramid derived from the group snapshot.
    ///
    /// See ADR 0302.
    PushBackdropSourceGroupV1 {
        /// Computation bounds (not an implicit clip).
        bounds: Rect,
        /// Optional bounded pyramid request shared by the group (upper bound).
        pyramid: Option<CustomEffectPyramidRequestV1>,
        /// Quality hint used for deterministic budgeting/degradation.
        quality: EffectQuality,
    },
    PopBackdropSourceGroup,

    PushCompositeGroup {
        desc: CompositeGroupDesc,
    },
    PopCompositeGroup,

    Quad {
        order: DrawOrder,
        rect: Rect,
        background: PaintBindingV1,
        border: Edges,
        border_paint: PaintBindingV1,
        corner_radii: Corners,
    },

    StrokeRRect {
        order: DrawOrder,
        rect: Rect,
        stroke: Edges,
        stroke_paint: PaintBindingV1,
        corner_radii: Corners,
        style: StrokeStyleV1,
    },

    /// Draw a single rounded-rect box shadow layer.
    ///
    /// This is a first-class geometric shadow primitive for container chrome. Unlike
    /// `EffectStep::DropShadowV1`, it is not content-derived and does not require a FilterContent
    /// intermediate.
    ShadowRRect {
        order: DrawOrder,
        rect: Rect,
        corner_radii: Corners,
        offset: Point,
        spread: Px,
        blur_radius: Px,
        color: Color,
    },

    Image {
        order: DrawOrder,
        rect: Rect,
        image: ImageId,
        fit: ViewportFit,
        sampling: ImageSamplingHint,
        opacity: f32,
    },

    ImageRegion {
        order: DrawOrder,
        rect: Rect,
        image: ImageId,
        uv: UvRect,
        sampling: ImageSamplingHint,
        opacity: f32,
    },

    /// Draw an alpha mask image tinted with a solid color.
    ///
    /// The referenced `image` is expected to store coverage in the red channel (e.g. `R8Unorm`).
    MaskImage {
        order: DrawOrder,
        rect: Rect,
        image: ImageId,
        uv: UvRect,
        sampling: ImageSamplingHint,
        color: Color,
        opacity: f32,
    },

    /// Draw an SVG as a monochrome icon: rasterize to an alpha mask, then tint with a solid color.
    SvgMaskIcon {
        order: DrawOrder,
        rect: Rect,
        svg: SvgId,
        fit: SvgFit,
        color: Color,
        opacity: f32,
    },

    /// Draw an SVG as an RGBA image: rasterize and upload as an image texture.
    SvgImage {
        order: DrawOrder,
        rect: Rect,
        svg: SvgId,
        fit: SvgFit,
        opacity: f32,
    },

    Text {
        order: DrawOrder,
        origin: Point,
        text: TextBlobId,
        paint: PaintBindingV1,
        outline: Option<TextOutlineV1>,
        shadow: Option<TextShadowV1>,
    },

    Path {
        order: DrawOrder,
        origin: Point,
        path: PathId,
        paint: PaintBindingV1,
    },

    ViewportSurface {
        order: DrawOrder,
        rect: Rect,
        target: RenderTargetId,
        opacity: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UvRect {
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
}

impl UvRect {
    pub const FULL: Self = Self {
        u0: 0.0,
        v0: 0.0,
        u1: 1.0,
        v1: 1.0,
    };
}

pub const SHADOW_RRECT_V1_MAX_BLUR_RADIUS_PX: crate::Px = crate::Px(64.0);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Px, Size};

    #[test]
    fn replay_ops_translated_wraps_in_transform_stack() {
        let ops = [SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
            background: Paint::Solid(Color::TRANSPARENT).into(),
            border: Edges::all(Px(0.0)),
            border_paint: Paint::Solid(Color::TRANSPARENT).into(),
            corner_radii: Corners::all(Px(0.0)),
        }];

        let mut scene = Scene::default();
        scene.replay_ops_translated(&ops, Point::new(Px(2.0), Px(3.0)));

        assert_eq!(scene.ops_len(), 3);
        assert!(matches!(scene.ops()[0], SceneOp::PushTransform { .. }));
        assert!(matches!(scene.ops()[1], SceneOp::Quad { .. }));
        assert!(matches!(scene.ops()[2], SceneOp::PopTransform));
    }

    #[test]
    fn validate_rejects_transform_underflow() {
        let mut scene = Scene::default();
        scene.push(SceneOp::PopTransform);
        assert!(matches!(
            scene.validate(),
            Err(SceneValidationError {
                kind: SceneValidationErrorKind::TransformUnderflow,
                ..
            })
        ));
    }

    #[test]
    fn validate_rejects_unbalanced_clip_stack() {
        let mut scene = Scene::default();
        scene.push(SceneOp::PushClipRect {
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(1.0), Px(1.0))),
        });
        assert!(matches!(
            scene.validate(),
            Err(SceneValidationError {
                kind: SceneValidationErrorKind::UnbalancedClipStack { remaining: 1 },
                ..
            })
        ));
    }

    #[test]
    fn validate_rejects_effect_underflow() {
        let mut scene = Scene::default();
        scene.push(SceneOp::PopEffect);
        assert!(matches!(
            scene.validate(),
            Err(SceneValidationError {
                kind: SceneValidationErrorKind::EffectUnderflow,
                ..
            })
        ));
    }

    #[test]
    fn validate_rejects_unbalanced_effect_stack() {
        let mut scene = Scene::default();
        scene.push(SceneOp::PushEffect {
            bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(1.0), Px(1.0))),
            mode: EffectMode::Backdrop,
            chain: EffectChain::from_steps(&[EffectStep::GaussianBlur {
                radius_px: Px(2.0),
                downsample: 2,
            }]),
            quality: EffectQuality::Auto,
        });
        assert!(matches!(
            scene.validate(),
            Err(SceneValidationError {
                kind: SceneValidationErrorKind::UnbalancedEffectStack { remaining: 1 },
                ..
            })
        ));
    }

    #[test]
    fn validate_rejects_opacity_underflow() {
        let mut scene = Scene::default();
        scene.push(SceneOp::PopOpacity);
        assert!(matches!(
            scene.validate(),
            Err(SceneValidationError {
                kind: SceneValidationErrorKind::OpacityUnderflow,
                ..
            })
        ));
    }

    #[test]
    fn validate_rejects_layer_underflow() {
        let mut scene = Scene::default();
        scene.push(SceneOp::PopLayer);
        assert!(matches!(
            scene.validate(),
            Err(SceneValidationError {
                kind: SceneValidationErrorKind::LayerUnderflow,
                ..
            })
        ));
    }

    #[test]
    fn validate_rejects_clip_underflow() {
        let mut scene = Scene::default();
        scene.push(SceneOp::PopClip);
        assert!(matches!(
            scene.validate(),
            Err(SceneValidationError {
                kind: SceneValidationErrorKind::ClipUnderflow,
                ..
            })
        ));
    }

    #[test]
    fn validate_rejects_nonfinite_opacity() {
        let mut scene = Scene::default();
        scene.push(SceneOp::PushOpacity { opacity: f32::NAN });
        assert!(matches!(
            scene.validate(),
            Err(SceneValidationError {
                kind: SceneValidationErrorKind::NonFiniteOpacity,
                ..
            })
        ));
    }

    #[test]
    fn validate_rejects_nonfinite_transform() {
        let mut scene = Scene::default();
        scene.push(SceneOp::PushTransform {
            transform: Transform2D {
                a: f32::NAN,
                ..Transform2D::IDENTITY
            },
        });
        assert!(matches!(
            scene.validate(),
            Err(SceneValidationError {
                kind: SceneValidationErrorKind::NonFiniteTransform,
                ..
            })
        ));
    }

    #[test]
    fn validate_rejects_nonfinite_draw_op_data() {
        let mut scene = Scene::default();
        scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(
                Point::new(Px(f32::NAN), Px(0.0)),
                Size::new(Px(10.0), Px(10.0)),
            ),
            background: Paint::Solid(Color::TRANSPARENT).into(),
            border: Edges::all(Px(0.0)),
            border_paint: Paint::Solid(Color::TRANSPARENT).into(),
            corner_radii: Corners::all(Px(0.0)),
        });
        assert!(matches!(
            scene.validate(),
            Err(SceneValidationError {
                kind: SceneValidationErrorKind::NonFiniteOpData,
                ..
            })
        ));
    }

    #[test]
    fn push_shadow_rrect_clamps_blur_and_corner_radii_after_spread() {
        let mut scene = Scene::default();
        scene.push(SceneOp::ShadowRRect {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(20.0), Px(12.0))),
            corner_radii: Corners::all(Px(9999.0)),
            offset: Point::new(Px(0.0), Px(4.0)),
            spread: Px(-4.0),
            blur_radius: Px(4096.0),
            color: Color::TRANSPARENT,
        });

        let SceneOp::ShadowRRect {
            corner_radii,
            blur_radius,
            ..
        } = scene.ops()[0]
        else {
            panic!("expected shadow rrect");
        };

        assert_eq!(blur_radius, SHADOW_RRECT_V1_MAX_BLUR_RADIUS_PX);
        assert_eq!(corner_radii, Corners::all(Px(2.0)));
    }

    #[test]
    fn validate_rejects_nonfinite_shadow_rrect_data() {
        let mut scene = Scene::default();
        scene.push(SceneOp::ShadowRRect {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
            corner_radii: Corners::all(Px(4.0)),
            offset: Point::new(Px(f32::NAN), Px(0.0)),
            spread: Px(0.0),
            blur_radius: Px(8.0),
            color: Color::TRANSPARENT,
        });

        assert!(matches!(
            scene.validate(),
            Err(SceneValidationError {
                kind: SceneValidationErrorKind::NonFiniteOpData,
                ..
            })
        ));
    }

    #[test]
    fn validate_rejects_unbalanced_opacity_stack() {
        let mut scene = Scene::default();
        scene.push(SceneOp::PushOpacity { opacity: 0.5 });
        assert!(matches!(
            scene.validate(),
            Err(SceneValidationError {
                kind: SceneValidationErrorKind::UnbalancedOpacityStack { remaining: 1 },
                ..
            })
        ));
    }

    #[test]
    fn validate_rejects_unbalanced_layer_stack() {
        let mut scene = Scene::default();
        scene.push(SceneOp::PushLayer { layer: 1 });
        assert!(matches!(
            scene.validate(),
            Err(SceneValidationError {
                kind: SceneValidationErrorKind::UnbalancedLayerStack { remaining: 1 },
                ..
            })
        ));
    }
}
