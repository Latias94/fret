use crate::{
    geometry::{Corners, Edges, Point, Rect},
    ids::{ImageId, RenderTargetId, TextBlobId},
};

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
pub struct Scene {
    pub ops: Vec<SceneOp>,
}

impl Scene {
    pub fn clear(&mut self) {
        self.ops.clear();
    }

    pub fn push(&mut self, op: SceneOp) {
        self.ops.push(op);
    }
}

#[derive(Debug, Clone)]
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
