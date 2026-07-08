use fret_core::{AppWindowId, Point, Px, Rect, Size, WindowAnchor, WindowMetricsService};
use fret_runtime::UiHost;

use crate::dock::DockManager;

/// Recenter in-window floating containers back into the visible bounds of a window.
///
/// This is intended as a "recovery" affordance for editor-grade layouts where floatings can end up
/// fully off-screen (or stacked) due to persisted state, DPI changes, or window resizes.
pub fn recenter_in_window_floatings<H: UiHost>(app: &mut H, window: AppWindowId) {
    let bounds = visible_bounds(app, window);

    app.with_global_mut(DockManager::default, |dock, _app| {
        let floatings = dock.graph.floating_windows_mut(window);
        for (ix, floating) in floatings.iter_mut().enumerate() {
            let size = floating.rect.size;
            let dx = (ix as f32) * 16.0;
            let dy = (ix as f32) * 16.0;
            let origin = Point::new(
                Px(bounds.origin.x.0 + (bounds.size.width.0 - size.width.0) * 0.5 + dx),
                Px(bounds.origin.y.0 + (bounds.size.height.0 - size.height.0) * 0.5 + dy),
            );
            floating.rect = clamp_rect_to_bounds(Rect::new(origin, size), bounds);
        }
    });

    super::request_dock_invalidation(app, [window]);
}

pub(super) fn default_in_window_float_rect<H: UiHost>(
    app: &H,
    target_window: AppWindowId,
    anchor: Option<WindowAnchor>,
) -> Rect {
    let bounds = visible_bounds(app, target_window);
    let size = Size::new(Px(480.0), Px(360.0));

    let origin = if let Some(anchor) = anchor {
        Point::new(
            Px(anchor.position.x.0 - size.width.0 * 0.25),
            Px(anchor.position.y.0 - size.height.0 * 0.25),
        )
    } else {
        Point::new(
            Px(bounds.size.width.0 * 0.5 - size.width.0 * 0.5),
            Px(bounds.size.height.0 * 0.5 - size.height.0 * 0.5),
        )
    };

    clamp_rect_to_bounds(Rect::new(origin, size), bounds)
}

fn visible_bounds<H: UiHost>(app: &H, target_window: AppWindowId) -> Rect {
    app.global::<WindowMetricsService>()
        .and_then(|svc| svc.inner_bounds(target_window))
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(960.0), Px(720.0)),
            )
        })
}

fn clamp_rect_to_bounds(rect: Rect, bounds: Rect) -> Rect {
    let mut out = rect;
    if bounds.size.width.0 > 0.0 && bounds.size.height.0 > 0.0 {
        let min_x = bounds.origin.x.0;
        let min_y = bounds.origin.y.0;
        let max_x = bounds.origin.x.0 + (bounds.size.width.0 - out.size.width.0).max(0.0);
        let max_y = bounds.origin.y.0 + (bounds.size.height.0 - out.size.height.0).max(0.0);
        out.origin.x = Px(out.origin.x.0.clamp(min_x, max_x.max(min_x)));
        out.origin.y = Px(out.origin.y.0.clamp(min_y, max_y.max(min_y)));
    }
    out
}
