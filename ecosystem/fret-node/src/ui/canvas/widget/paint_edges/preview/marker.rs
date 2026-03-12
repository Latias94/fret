use super::*;

pub(super) fn push_drop_marker(scene: &mut fret_core::Scene, pos: Point, color: Color, zoom: f32) {
    let z = zoom.max(1.0e-6);
    let r = 7.0 / z;
    let border_w = 2.0 / z;
    let rect = Rect::new(
        Point::new(Px(pos.x.0 - r), Px(pos.y.0 - r)),
        Size::new(Px(2.0 * r), Px(2.0 * r)),
    );
    scene.push(SceneOp::Quad {
        order: DrawOrder(4),
        rect,
        background: fret_core::Paint::TRANSPARENT.into(),
        border: Edges::all(Px(border_w)),
        border_paint: fret_core::Paint::Solid(color).into(),
        corner_radii: Corners::all(Px(r)),
    });

    let arm = 10.0 / z;
    let thick = (2.0 / z).max(0.5 / z);
    let h_rect = Rect::new(
        Point::new(Px(pos.x.0 - arm * 0.5), Px(pos.y.0 - thick * 0.5)),
        Size::new(Px(arm), Px(thick)),
    );
    let v_rect = Rect::new(
        Point::new(Px(pos.x.0 - thick * 0.5), Px(pos.y.0 - arm * 0.5)),
        Size::new(Px(thick), Px(arm)),
    );
    scene.push(SceneOp::Quad {
        order: DrawOrder(4),
        rect: h_rect,
        background: fret_core::Paint::Solid(color).into(),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(4),
        rect: v_rect,
        background: fret_core::Paint::Solid(color).into(),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    });
}
