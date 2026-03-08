use super::*;

pub(super) fn append_line_ops(
    spec: &super::paint_grid_tiles::GridTileSpec,
    ops: &mut Vec<SceneOp>,
) {
    for ix in spec.x0..=spec.x1 {
        let x = ix as f32 * spec.spacing;
        let color = if ix.rem_euclid(spec.major_every) == 0 {
            spec.major_color
        } else {
            spec.minor_color
        };
        ops.push(SceneOp::Quad {
            order: DrawOrder(1),
            rect: Rect::new(
                Point::new(
                    Px(x - spec.tile_origin.x.0 - 0.5 * spec.thickness.0),
                    Px(0.0),
                ),
                Size::new(spec.thickness, Px(spec.tile_size_canvas)),
            ),
            background: fret_core::Paint::Solid(color).into(),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT.into(),
            corner_radii: Corners::all(Px(0.0)),
        });
    }

    for iy in spec.y0..=spec.y1 {
        let y = iy as f32 * spec.spacing;
        let color = if iy.rem_euclid(spec.major_every) == 0 {
            spec.major_color
        } else {
            spec.minor_color
        };
        ops.push(SceneOp::Quad {
            order: DrawOrder(1),
            rect: Rect::new(
                Point::new(
                    Px(0.0),
                    Px(y - spec.tile_origin.y.0 - 0.5 * spec.thickness.0),
                ),
                Size::new(Px(spec.tile_size_canvas), spec.thickness),
            ),
            background: fret_core::Paint::Solid(color).into(),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT.into(),
            corner_radii: Corners::all(Px(0.0)),
        });
    }
}
