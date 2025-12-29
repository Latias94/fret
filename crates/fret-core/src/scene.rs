use crate::{
    SvgFit,
    geometry::{Corners, Edges, Point, Rect, Transform2D},
    ids::{ImageId, PathId, RenderTargetId, SvgId, TextBlobId},
};
use slotmap::Key;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawOrder(pub u32);

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Default, Clone)]
pub struct SceneRecording {
    ops: Vec<SceneOp>,
    fingerprint: u64,
}

pub type Scene = SceneRecording;

impl SceneRecording {
    pub fn clear(&mut self) {
        self.ops.clear();
        self.fingerprint = 0;
    }

    pub fn push(&mut self, op: SceneOp) {
        self.fingerprint = mix_scene_op(self.fingerprint, op);
        self.ops.push(op);
    }

    pub fn replay_ops(&mut self, ops: &[SceneOp]) {
        self.ops.reserve(ops.len());
        for &op in ops {
            self.fingerprint = mix_scene_op(self.fingerprint, op);
            self.ops.push(op);
        }
    }

    pub fn replay_ops_translated(&mut self, ops: &[SceneOp], delta: Point) {
        if delta.x.0 == 0.0 && delta.y.0 == 0.0 {
            self.replay_ops(ops);
            return;
        }

        self.push(SceneOp::PushTransform {
            transform: Transform2D::translation(delta),
        });
        self.replay_ops(ops);
        self.push(SceneOp::PopTransform);
    }

    pub fn ops(&self) -> &[SceneOp] {
        &self.ops
    }

    pub fn ops_len(&self) -> usize {
        self.ops.len()
    }

    pub fn swap_storage(&mut self, other_ops: &mut Vec<SceneOp>, other_fingerprint: &mut u64) {
        std::mem::swap(&mut self.ops, other_ops);
        std::mem::swap(&mut self.fingerprint, other_fingerprint);
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

    Quad {
        order: DrawOrder,
        rect: Rect,
        background: Color,
        border: Edges,
        border_color: Color,
        corner_radii: Corners,
    },

    Image {
        order: DrawOrder,
        rect: Rect,
        image: ImageId,
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

fn mix_u64(mut state: u64, value: u64) -> u64 {
    // A lightweight, deterministic mixing function (not cryptographic).
    // We want stability across platforms and reasonable avalanche for small changes.
    state ^= value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    state = state.rotate_left(7);
    state = state.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    state
}

fn mix_f32(state: u64, value: f32) -> u64 {
    mix_u64(state, u64::from(value.to_bits()))
}

fn mix_px(state: u64, value: crate::Px) -> u64 {
    mix_f32(state, value.0)
}

fn mix_point(mut state: u64, p: Point) -> u64 {
    state = mix_px(state, p.x);
    state = mix_px(state, p.y);
    state
}

fn mix_rect(mut state: u64, r: Rect) -> u64 {
    state = mix_point(state, r.origin);
    state = mix_px(state, r.size.width);
    state = mix_px(state, r.size.height);
    state
}

fn mix_color(mut state: u64, c: Color) -> u64 {
    state = mix_f32(state, c.r);
    state = mix_f32(state, c.g);
    state = mix_f32(state, c.b);
    state = mix_f32(state, c.a);
    state
}

fn mix_edges(mut state: u64, e: Edges) -> u64 {
    state = mix_px(state, e.top);
    state = mix_px(state, e.right);
    state = mix_px(state, e.bottom);
    state = mix_px(state, e.left);
    state
}

fn mix_corners(mut state: u64, c: Corners) -> u64 {
    state = mix_px(state, c.top_left);
    state = mix_px(state, c.top_right);
    state = mix_px(state, c.bottom_right);
    state = mix_px(state, c.bottom_left);
    state
}

fn mix_scene_op(state: u64, op: SceneOp) -> u64 {
    match op {
        SceneOp::PushTransform { transform } => {
            let mut state = mix_u64(state, 100);
            state = mix_f32(state, transform.a);
            state = mix_f32(state, transform.b);
            state = mix_f32(state, transform.c);
            state = mix_f32(state, transform.d);
            state = mix_f32(state, transform.tx);
            mix_f32(state, transform.ty)
        }
        SceneOp::PopTransform => mix_u64(state, 101),
        SceneOp::PushOpacity { opacity } => {
            let state = mix_u64(state, 102);
            mix_f32(state, opacity)
        }
        SceneOp::PopOpacity => mix_u64(state, 103),
        SceneOp::PushLayer { layer } => {
            let state = mix_u64(state, 104);
            mix_u64(state, u64::from(layer))
        }
        SceneOp::PopLayer => mix_u64(state, 105),
        SceneOp::PushClipRect { rect } => {
            let state = mix_u64(state, 1);
            mix_rect(state, rect)
        }
        SceneOp::PushClipRRect { rect, corner_radii } => {
            let mut state = mix_u64(state, 13);
            state = mix_rect(state, rect);
            mix_corners(state, corner_radii)
        }
        SceneOp::PopClip => mix_u64(state, 2),
        SceneOp::Quad {
            order,
            rect,
            background,
            border,
            border_color,
            corner_radii,
        } => {
            let mut state = mix_u64(state, 3);
            state = mix_u64(state, u64::from(order.0));
            state = mix_rect(state, rect);
            state = mix_color(state, background);
            state = mix_edges(state, border);
            state = mix_color(state, border_color);
            mix_corners(state, corner_radii)
        }
        SceneOp::Image {
            order,
            rect,
            image,
            opacity,
        } => {
            let mut state = mix_u64(state, 4);
            state = mix_u64(state, u64::from(order.0));
            state = mix_rect(state, rect);
            state = mix_u64(state, image.data().as_ffi());
            mix_f32(state, opacity)
        }
        SceneOp::ImageRegion {
            order,
            rect,
            image,
            uv,
            opacity,
        } => {
            let mut state = mix_u64(state, 7);
            state = mix_u64(state, u64::from(order.0));
            state = mix_rect(state, rect);
            state = mix_u64(state, image.data().as_ffi());
            state = mix_f32(state, uv.u0);
            state = mix_f32(state, uv.v0);
            state = mix_f32(state, uv.u1);
            state = mix_f32(state, uv.v1);
            mix_f32(state, opacity)
        }
        SceneOp::MaskImage {
            order,
            rect,
            image,
            uv,
            color,
            opacity,
        } => {
            let mut state = mix_u64(state, 9);
            state = mix_u64(state, u64::from(order.0));
            state = mix_rect(state, rect);
            state = mix_u64(state, image.data().as_ffi());
            state = mix_f32(state, uv.u0);
            state = mix_f32(state, uv.v0);
            state = mix_f32(state, uv.u1);
            state = mix_f32(state, uv.v1);
            state = mix_color(state, color);
            mix_f32(state, opacity)
        }
        SceneOp::SvgMaskIcon {
            order,
            rect,
            svg,
            fit,
            color,
            opacity,
        } => {
            let mut state = mix_u64(state, 10);
            state = mix_u64(state, u64::from(order.0));
            state = mix_rect(state, rect);
            state = mix_u64(state, svg.data().as_ffi());
            state = mix_u64(
                state,
                match fit {
                    SvgFit::Contain => 1,
                    SvgFit::Width => 2,
                    SvgFit::Stretch => 3,
                },
            );
            state = mix_color(state, color);
            mix_f32(state, opacity)
        }
        SceneOp::SvgImage {
            order,
            rect,
            svg,
            fit,
            opacity,
        } => {
            let mut state = mix_u64(state, 11);
            state = mix_u64(state, u64::from(order.0));
            state = mix_rect(state, rect);
            state = mix_u64(state, svg.data().as_ffi());
            state = mix_u64(
                state,
                match fit {
                    SvgFit::Contain => 1,
                    SvgFit::Width => 2,
                    SvgFit::Stretch => 3,
                },
            );
            mix_f32(state, opacity)
        }
        SceneOp::Text {
            order,
            origin,
            text,
            color,
        } => {
            let mut state = mix_u64(state, 5);
            state = mix_u64(state, u64::from(order.0));
            state = mix_point(state, origin);
            state = mix_u64(state, text.data().as_ffi());
            mix_color(state, color)
        }
        SceneOp::Path {
            order,
            origin,
            path,
            color,
        } => {
            let mut state = mix_u64(state, 8);
            state = mix_u64(state, u64::from(order.0));
            state = mix_point(state, origin);
            state = mix_u64(state, path.data().as_ffi());
            mix_color(state, color)
        }
        SceneOp::ViewportSurface {
            order,
            rect,
            target,
            opacity,
        } => {
            let mut state = mix_u64(state, 6);
            state = mix_u64(state, u64::from(order.0));
            state = mix_rect(state, rect);
            state = mix_u64(state, target.data().as_ffi());
            mix_f32(state, opacity)
        }
    }
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
            background: Color::TRANSPARENT,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        }];

        let mut scene = Scene::default();
        scene.replay_ops_translated(&ops, Point::new(Px(2.0), Px(3.0)));

        assert_eq!(scene.ops_len(), 3);
        assert!(matches!(scene.ops()[0], SceneOp::PushTransform { .. }));
        assert!(matches!(scene.ops()[1], SceneOp::Quad { .. }));
        assert!(matches!(scene.ops()[2], SceneOp::PopTransform));
    }
}
