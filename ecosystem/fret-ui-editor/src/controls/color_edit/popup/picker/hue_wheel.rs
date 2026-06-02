mod cursor;
mod path;
mod ring;
mod triangle;

use fret_ui::canvas::CanvasPainter;
use fret_ui::element::{AnyElement, CanvasProps};
use fret_ui::{ElementContext, UiHost};

use super::super::super::model::{HsvColor, hue_wheel_geometry};
use super::super::preview::fill_preview_layout;

use cursor::paint_hue_wheel_cursors;
use ring::paint_hue_wheel_ring;
use triangle::paint_hue_wheel_triangle;

pub(in crate::controls::color_edit::popup) fn hue_wheel_canvas<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    hsv: HsvColor,
) -> AnyElement {
    cx.canvas(
        CanvasProps {
            layout: fill_preview_layout(),
            ..Default::default()
        },
        move |painter| paint_hue_wheel_canvas(painter, hsv),
    )
}

fn paint_hue_wheel_canvas(painter: &mut CanvasPainter<'_>, hsv: HsvColor) {
    let bounds = painter.bounds();
    let geometry = hue_wheel_geometry(bounds.size.width.0, bounds.size.height.0);
    if geometry.wheel_r_outer <= f32::EPSILON || geometry.wheel_thickness <= f32::EPSILON {
        return;
    }

    let scale = painter.scale_factor().max(1.0);
    let origin = bounds.origin;
    let base = painter.key_scope(&"fret-ui-editor.color_edit.hue_wheel");
    paint_hue_wheel_ring(painter, base, origin, geometry, scale);
    paint_hue_wheel_triangle(painter, base, origin, geometry, hsv, scale);
    paint_hue_wheel_cursors(painter, base, origin, geometry, hsv, scale);
}
