//! Shared helpers for scripted interaction regression tests.
//!
//! These tests are intentionally renderer-agnostic: they assert high-level stability and invariants
//! by inspecting the `SceneOp` stream (op kinds + draw ordering), rather than pixel snapshots.

use fret_core::{DrawOrder, Scene, SceneOp};

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
