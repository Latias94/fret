//! Shared helpers for scripted interaction regression tests.
//!
//! These tests are intentionally renderer-agnostic: they assert high-level stability and invariants
//! by inspecting the `SceneOp` stream (op kinds + draw ordering), rather than pixel snapshots.

#![cfg(feature = "material3_full")]
#![allow(dead_code)]

use fret_core::{Color, Corners, DrawOrder, Edges, Px, Rect, Scene, SceneOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneSig {
    PushTransform,
    PopTransform,
    PushOpacity,
    PopOpacity,
    PushLayer,
    PopLayer,
    PushClipRect,
    PushClipRRect,
    PopClip,
    PushEffect,
    PopEffect,
    Quad(DrawOrder),
    Image(DrawOrder),
    ImageRegion(DrawOrder),
    MaskImage(DrawOrder),
    SvgMaskIcon(DrawOrder),
    SvgImage(DrawOrder),
    Text(DrawOrder),
    Path(DrawOrder),
    ViewportSurface(DrawOrder),
}

pub fn scene_signature(scene: &Scene) -> Vec<SceneSig> {
    scene
        .ops()
        .iter()
        .map(|op| match *op {
            SceneOp::PushTransform { .. } => SceneSig::PushTransform,
            SceneOp::PopTransform => SceneSig::PopTransform,
            SceneOp::PushOpacity { .. } => SceneSig::PushOpacity,
            SceneOp::PopOpacity => SceneSig::PopOpacity,
            SceneOp::PushLayer { .. } => SceneSig::PushLayer,
            SceneOp::PopLayer => SceneSig::PopLayer,
            SceneOp::PushClipRect { .. } => SceneSig::PushClipRect,
            SceneOp::PushClipRRect { .. } => SceneSig::PushClipRRect,
            SceneOp::PopClip => SceneSig::PopClip,
            SceneOp::PushEffect { .. } => SceneSig::PushEffect,
            SceneOp::PopEffect => SceneSig::PopEffect,
            SceneOp::Quad { order, .. } => SceneSig::Quad(order),
            SceneOp::Image { order, .. } => SceneSig::Image(order),
            SceneOp::ImageRegion { order, .. } => SceneSig::ImageRegion(order),
            SceneOp::MaskImage { order, .. } => SceneSig::MaskImage(order),
            SceneOp::SvgMaskIcon { order, .. } => SceneSig::SvgMaskIcon(order),
            SceneOp::SvgImage { order, .. } => SceneSig::SvgImage(order),
            SceneOp::Text { order, .. } => SceneSig::Text(order),
            SceneOp::Path { order, .. } => SceneSig::Path(order),
            SceneOp::ViewportSurface { order, .. } => SceneSig::ViewportSurface(order),
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadSig {
    pub order: DrawOrder,
    pub rect: RectSig,
    pub background: ColorSig,
    pub border: EdgesSig,
    pub corner_radii: CornersSig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadGeomSig {
    pub order: DrawOrder,
    pub rect: RectSig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RectSig {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorSig {
    pub r: i32,
    pub g: i32,
    pub b: i32,
    pub a: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgesSig {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CornersSig {
    pub top_left: i32,
    pub top_right: i32,
    pub bottom_right: i32,
    pub bottom_left: i32,
}

pub fn scene_quad_signature(scene: &Scene) -> Vec<QuadSig> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match *op {
            SceneOp::Quad {
                order,
                rect,
                background,
                border,
                corner_radii,
                ..
            } => Some(QuadSig {
                order,
                rect: rect_sig(rect),
                background: color_sig(background),
                border: edges_sig(border),
                corner_radii: corners_sig(corner_radii),
            }),
            _ => None,
        })
        .collect()
}

pub fn scene_quad_geometry_signature(scene: &Scene) -> Vec<QuadGeomSig> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match *op {
            SceneOp::Quad { order, rect, .. } => Some(QuadGeomSig {
                order,
                rect: rect_sig(rect),
            }),
            _ => None,
        })
        .collect()
}

fn rect_sig(rect: Rect) -> RectSig {
    RectSig {
        x: px_sig(rect.origin.x),
        y: px_sig(rect.origin.y),
        w: px_sig(rect.size.width),
        h: px_sig(rect.size.height),
    }
}

fn edges_sig(edges: Edges) -> EdgesSig {
    EdgesSig {
        top: px_sig(edges.top),
        right: px_sig(edges.right),
        bottom: px_sig(edges.bottom),
        left: px_sig(edges.left),
    }
}

fn corners_sig(corners: Corners) -> CornersSig {
    CornersSig {
        top_left: px_sig(corners.top_left),
        top_right: px_sig(corners.top_right),
        bottom_right: px_sig(corners.bottom_right),
        bottom_left: px_sig(corners.bottom_left),
    }
}

fn color_sig(color: Color) -> ColorSig {
    ColorSig {
        r: unit_sig(color.r),
        g: unit_sig(color.g),
        b: unit_sig(color.b),
        a: unit_sig(color.a),
    }
}

fn px_sig(px: Px) -> i32 {
    // Quantize to 0.1px to avoid false positives from float rounding drift while still catching
    // perceptual layout jitter.
    ((px.0 * 10.0).round()) as i32
}

fn unit_sig(v: f32) -> i32 {
    ((v.clamp(0.0, 1.0) * 1000.0).round()) as i32
}
