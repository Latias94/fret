use super::super::*;
use super::style::EdgeAnchorPaintStyle;

pub(super) fn push_edge_focus_anchor_quad<H: UiHost>(
    cx: &mut PaintCx<'_, H>,
    rect: Rect,
    style: EdgeAnchorPaintStyle,
) {
    let radius = Px(0.5 * rect.size.width.0);

    cx.scene.push(SceneOp::Quad {
        order: DrawOrder(6),
        rect,
        background: fret_core::Paint::Solid(style.background).into(),
        border: Edges::all(style.border),
        border_paint: fret_core::Paint::Solid(style.anchor_color).into(),
        corner_radii: Corners::all(radius),
    });
}
