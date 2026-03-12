use crate::ui::canvas::widget::*;
use fret_core::PathId;
use fret_core::scene::{PaintBindingV1, PaintEvalSpaceV1};

use super::super::markers::WireHighlightPaint;

pub(super) fn marker_paint_binding_for_wire(paint: PaintBindingV1, color: Color) -> PaintBindingV1 {
    if paint.eval_space == PaintEvalSpaceV1::StrokeS01 {
        color.into()
    } else {
        paint
    }
}

pub(super) fn push_marker_path(
    scene: &mut fret_core::Scene,
    path: Option<PathId>,
    paint: PaintBindingV1,
) {
    if let Some(path) = path {
        scene.push(SceneOp::Path {
            order: DrawOrder(2),
            origin: Point::new(Px(0.0), Px(0.0)),
            path,
            paint,
        });
    }
}

pub(super) fn push_wire_highlight_path(
    scene: &mut fret_core::Scene,
    path: Option<PathId>,
    highlight: Option<WireHighlightPaint>,
) {
    if let Some(highlight) = highlight
        && highlight.width.is_finite()
        && highlight.width > 1.0e-3
        && highlight.color.a > 0.0
    {
        push_marker_path(scene, path, highlight.color.into());
    }
}
