use crate::core::CanvasPoint;
use crate::ui::canvas::widget::*;

pub(super) fn auto_pan_delta(snapshot: &ViewSnapshot, pos: Point, bounds: Rect) -> CanvasPoint {
    let view = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::viewport_from_snapshot(
        bounds, snapshot,
    )
    .view;
    let tuning = fret_canvas::view::AutoPanTuning {
        margin_screen_px: snapshot.interaction.auto_pan.margin,
        speed_screen_px_per_s: snapshot.interaction.auto_pan.speed,
    };

    let delta = fret_canvas::view::auto_pan_delta_per_tick(
        bounds,
        view,
        pos,
        tuning,
        NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::AUTO_PAN_TICK_HZ,
    );

    CanvasPoint {
        x: delta.x.0,
        y: delta.y.0,
    }
}
