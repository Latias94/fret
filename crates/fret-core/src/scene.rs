use crate::{
    geometry::{Corners, Edges, Point, Rect},
    ids::{ImageId, RenderTargetId, TextBlobId},
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
        self.ops.reserve(ops.len());
        for &op in ops {
            self.push(translate_scene_op(op, delta));
        }
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
    PushClipRect {
        rect: Rect,
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

    Text {
        order: DrawOrder,
        origin: Point,
        text: TextBlobId,
        color: Color,
    },

    ViewportSurface {
        order: DrawOrder,
        rect: Rect,
        target: RenderTargetId,
        opacity: f32,
    },
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
        SceneOp::PushClipRect { rect } => {
            let state = mix_u64(state, 1);
            mix_rect(state, rect)
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

fn translate_point(p: Point, delta: Point) -> Point {
    Point::new(p.x + delta.x, p.y + delta.y)
}

fn translate_rect(r: Rect, delta: Point) -> Rect {
    Rect::new(translate_point(r.origin, delta), r.size)
}

fn translate_scene_op(op: SceneOp, delta: Point) -> SceneOp {
    match op {
        SceneOp::PushClipRect { rect } => SceneOp::PushClipRect {
            rect: translate_rect(rect, delta),
        },
        SceneOp::PopClip => SceneOp::PopClip,
        SceneOp::Quad {
            order,
            rect,
            background,
            border,
            border_color,
            corner_radii,
        } => SceneOp::Quad {
            order,
            rect: translate_rect(rect, delta),
            background,
            border,
            border_color,
            corner_radii,
        },
        SceneOp::Image {
            order,
            rect,
            image,
            opacity,
        } => SceneOp::Image {
            order,
            rect: translate_rect(rect, delta),
            image,
            opacity,
        },
        SceneOp::Text {
            order,
            origin,
            text,
            color,
        } => SceneOp::Text {
            order,
            origin: translate_point(origin, delta),
            text,
            color,
        },
        SceneOp::ViewportSurface {
            order,
            rect,
            target,
            opacity,
        } => SceneOp::ViewportSurface {
            order,
            rect: translate_rect(rect, delta),
            target,
            opacity,
        },
    }
}
