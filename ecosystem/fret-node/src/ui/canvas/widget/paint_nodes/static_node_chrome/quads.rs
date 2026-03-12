use super::*;

pub(super) fn paint_static_node_quads(
    scene: &mut fret_core::Scene,
    rect: Rect,
    body_background: fret_core::scene::PaintBindingV1,
    header_background: Option<fret_core::scene::PaintBindingV1>,
    border_paint: fret_core::scene::PaintBindingV1,
    border_w: Px,
    corner: Px,
    title_h: f32,
) {
    scene.push(SceneOp::Quad {
        order: DrawOrder(3),
        rect,
        background: body_background,
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),
        corner_radii: Corners::all(corner),
    });

    if let Some(paint) = header_background {
        scene.push(SceneOp::Quad {
            order: DrawOrder(3),
            rect: Rect::new(
                rect.origin,
                Size::new(rect.size.width, Px(title_h.min(rect.size.height.0))),
            ),
            background: paint,
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT.into(),
            corner_radii: Corners {
                top_left: corner,
                top_right: corner,
                bottom_right: Px(0.0),
                bottom_left: Px(0.0),
            },
        });
    }

    scene.push(SceneOp::Quad {
        order: DrawOrder(3),
        rect,
        background: fret_core::Paint::TRANSPARENT.into(),
        border: Edges::all(border_w),
        border_paint,
        corner_radii: Corners::all(corner),
    });
}
