use crate::{
    Px, SvgFit, ViewportFit,
    geometry::{Corners, Edges, Point, Rect, Transform2D},
    ids::{ImageId, PathId, RenderTargetId, SvgId, TextBlobId},
};
use serde::{Deserialize, Serialize};
use slotmap::Key;

mod composite;
mod fingerprint;
mod image_object_fit;
mod mask;
mod paint;
mod replay;
mod validate;

pub use composite::{BlendMode, CompositeGroupDesc};
use fingerprint::mix_scene_op;
pub use image_object_fit::{ImageObjectFitMapped, map_image_object_fit};
pub use mask::Mask;
pub use paint::{
    ColorSpace, GradientStop, LinearGradient, MAX_STOPS, MaterialParams, Paint, RadialGradient,
    TileMode,
};
pub use validate::{SceneValidationError, SceneValidationErrorKind};

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectStep {
    GaussianBlur {
        radius_px: crate::Px,
        downsample: u32,
    },
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

    PushCompositeGroup {
        desc: CompositeGroupDesc,
    },
    PopCompositeGroup,

    Quad {
        order: DrawOrder,
        rect: Rect,
        background: Paint,
        border: Edges,
        border_paint: Paint,
        corner_radii: Corners,
    },

    Image {
        order: DrawOrder,
        rect: Rect,
        image: ImageId,
        fit: ViewportFit,
        opacity: f32,
    },

    ImageRegion {
        order: DrawOrder,
        rect: Rect,
        image: ImageId,
        uv: UvRect,
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
        color: Color,
    },

    Path {
        order: DrawOrder,
        origin: Point,
        path: PathId,
        color: Color,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Px, Size};

    #[test]
    fn replay_ops_translated_wraps_in_transform_stack() {
        let ops = [SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
            background: Paint::Solid(Color::TRANSPARENT),
            border: Edges::all(Px(0.0)),
            border_paint: Paint::Solid(Color::TRANSPARENT),
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
            background: Paint::Solid(Color::TRANSPARENT),
            border: Edges::all(Px(0.0)),
            border_paint: Paint::Solid(Color::TRANSPARENT),
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
