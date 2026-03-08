use super::*;

pub(super) fn append_dot_ops(spec: &super::paint_grid_tiles::GridTileSpec, ops: &mut Vec<SceneOp>) {
    let d = spec.dot_size.max(0.0);
    let r = 0.5 * d;
    if !(d.is_finite() && d > 0.0) {
        return;
    }

    let corner = Corners::all(Px(r));
    for ix in spec.x0..=spec.x1 {
        let x = ix as f32 * spec.spacing;
        let x_local = x - spec.tile_origin.x.0;
        for iy in spec.y0..=spec.y1 {
            let y = iy as f32 * spec.spacing;
            let y_local = y - spec.tile_origin.y.0;

            let is_major =
                ix.rem_euclid(spec.major_every) == 0 && iy.rem_euclid(spec.major_every) == 0;
            let color = if is_major {
                spec.major_color
            } else {
                spec.minor_color
            };

            ops.push(SceneOp::Quad {
                order: DrawOrder(1),
                rect: Rect::new(
                    Point::new(Px(x_local - r), Px(y_local - r)),
                    Size::new(Px(d), Px(d)),
                ),
                background: fret_core::Paint::Solid(color).into(),
                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT.into(),
                corner_radii: corner,
            });
        }
    }
}
