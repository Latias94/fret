use fret_canvas::view::{PanZoom2D, visible_canvas_rect};
use fret_core::{KeyCode, Point, Px, Rect};

use crate::core::CanvasPoint;
use crate::io::NodeGraphViewState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MiniMapKeyboardAction {
    PanLeft,
    PanRight,
    PanUp,
    PanDown,
    ZoomIn,
    ZoomOut,
    FocusCanvas,
}

pub(super) fn minimap_keyboard_action_from_key(key: KeyCode) -> Option<MiniMapKeyboardAction> {
    match key {
        KeyCode::ArrowLeft => Some(MiniMapKeyboardAction::PanLeft),
        KeyCode::ArrowRight => Some(MiniMapKeyboardAction::PanRight),
        KeyCode::ArrowUp => Some(MiniMapKeyboardAction::PanUp),
        KeyCode::ArrowDown => Some(MiniMapKeyboardAction::PanDown),
        KeyCode::Equal | KeyCode::NumpadAdd => Some(MiniMapKeyboardAction::ZoomIn),
        KeyCode::Minus | KeyCode::NumpadSubtract => Some(MiniMapKeyboardAction::ZoomOut),
        KeyCode::Escape => Some(MiniMapKeyboardAction::FocusCanvas),
        _ => None,
    }
}

pub(super) fn plan_minimap_keyboard_pan(
    view_state: &NodeGraphViewState,
    action: MiniMapKeyboardAction,
    step_screen_px: f32,
) -> Option<CanvasPoint> {
    let zoom = normalized_zoom(view_state.zoom);
    let step = step_screen_px / zoom;
    let mut pan = view_state.pan;

    match action {
        MiniMapKeyboardAction::PanLeft => pan.x += step,
        MiniMapKeyboardAction::PanRight => pan.x -= step,
        MiniMapKeyboardAction::PanUp => pan.y += step,
        MiniMapKeyboardAction::PanDown => pan.y -= step,
        _ => return None,
    }

    Some(pan)
}

pub(super) fn plan_minimap_keyboard_zoom(
    view_state: &NodeGraphViewState,
    canvas_bounds: Rect,
    min_zoom: f32,
    max_zoom: f32,
    zoom_step_mul: f32,
    action: MiniMapKeyboardAction,
) -> Option<(CanvasPoint, f32)> {
    let zoom = normalized_zoom(view_state.zoom);
    let factor = match action {
        MiniMapKeyboardAction::ZoomIn => zoom_step_mul,
        MiniMapKeyboardAction::ZoomOut => 1.0 / zoom_step_mul,
        _ => return None,
    };

    let new_zoom = clamp_zoom(zoom * factor, min_zoom, max_zoom);
    let view = PanZoom2D {
        pan: Point::new(Px(view_state.pan.x), Px(view_state.pan.y)),
        zoom,
    };
    let visible = visible_canvas_rect(canvas_bounds, view);
    let cx_canvas = visible.origin.x.0 + 0.5 * visible.size.width.0;
    let cy_canvas = visible.origin.y.0 + 0.5 * visible.size.height.0;

    let new_pan = CanvasPoint {
        x: canvas_bounds.size.width.0 / (2.0 * new_zoom) - cx_canvas,
        y: canvas_bounds.size.height.0 / (2.0 * new_zoom) - cy_canvas,
    };
    Some((new_pan, new_zoom))
}

fn normalized_zoom(zoom: f32) -> f32 {
    if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    }
}

fn clamp_zoom(zoom: f32, min_zoom: f32, max_zoom: f32) -> f32 {
    if min_zoom.is_finite() && max_zoom.is_finite() && min_zoom > 0.0 && max_zoom > 0.0 {
        let (min_z, max_z) = if min_zoom <= max_zoom {
            (min_zoom, max_zoom)
        } else {
            (max_zoom, min_zoom)
        };
        zoom.clamp(min_z, max_z)
    } else if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MiniMapKeyboardAction, minimap_keyboard_action_from_key, plan_minimap_keyboard_pan,
        plan_minimap_keyboard_zoom,
    };
    use crate::core::CanvasPoint;
    use crate::io::NodeGraphViewState;
    use fret_core::{KeyCode, Point, Px, Rect, Size};

    #[test]
    fn minimap_keyboard_mapping_stays_stable() {
        assert_eq!(
            minimap_keyboard_action_from_key(KeyCode::ArrowLeft),
            Some(MiniMapKeyboardAction::PanLeft)
        );
        assert_eq!(
            minimap_keyboard_action_from_key(KeyCode::NumpadAdd),
            Some(MiniMapKeyboardAction::ZoomIn)
        );
        assert_eq!(
            minimap_keyboard_action_from_key(KeyCode::Escape),
            Some(MiniMapKeyboardAction::FocusCanvas)
        );
        assert_eq!(minimap_keyboard_action_from_key(KeyCode::Enter), None);
    }

    #[test]
    fn minimap_keyboard_pan_and_zoom_planning_keep_center_based_math() {
        let view_state = NodeGraphViewState {
            pan: CanvasPoint { x: 0.0, y: 0.0 },
            zoom: 1.0,
            ..Default::default()
        };
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        let pan = plan_minimap_keyboard_pan(&view_state, MiniMapKeyboardAction::PanRight, 24.0)
            .expect("pan plan");
        assert_eq!(pan.x, -24.0);

        let (zoom_pan, zoom) = plan_minimap_keyboard_zoom(
            &view_state,
            bounds,
            0.05,
            64.0,
            1.1,
            MiniMapKeyboardAction::ZoomIn,
        )
        .expect("zoom plan");
        assert!((zoom - 1.1).abs() <= 1.0e-6);
        assert!((zoom_pan.x - (800.0 / (2.0 * 1.1) - 400.0)).abs() <= 1.0e-3);
        assert!((zoom_pan.y - (600.0 / (2.0 * 1.1) - 300.0)).abs() <= 1.0e-3);
    }
}
