use fret_core::{KeyCode, MouseButton, Point, Rect};

use crate::core::CanvasPoint;
use crate::io::NodeGraphViewState;

use super::minimap_drag_policy::{MiniMapDragPlan, plan_minimap_drag_start};
use super::minimap_policy::{
    MiniMapKeyboardAction, minimap_keyboard_action_from_key, plan_minimap_keyboard_pan,
    plan_minimap_keyboard_zoom,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum MiniMapKeyboardInteractionPlan {
    Viewport { pan: CanvasPoint, zoom: f32 },
    FocusCanvas,
    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct MiniMapPointerDownInteractionPlan {
    pub(super) drag: MiniMapDragPlan,
    pub(super) focus_canvas: bool,
    pub(super) capture_pointer: bool,
    pub(super) stop_propagation: bool,
    pub(super) repaint: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MiniMapPointerUpInteractionPlan {
    pub(super) release_capture: bool,
    pub(super) finish_event: bool,
}

pub(super) fn plan_minimap_keyboard_interaction(
    key: KeyCode,
    view_state: &NodeGraphViewState,
    canvas_bounds: Rect,
    min_zoom: f32,
    max_zoom: f32,
    pan_step_screen_px: f32,
    zoom_step_mul: f32,
) -> MiniMapKeyboardInteractionPlan {
    let Some(action) = minimap_keyboard_action_from_key(key) else {
        return MiniMapKeyboardInteractionPlan::Ignore;
    };

    match action {
        MiniMapKeyboardAction::PanLeft
        | MiniMapKeyboardAction::PanRight
        | MiniMapKeyboardAction::PanUp
        | MiniMapKeyboardAction::PanDown => {
            plan_minimap_keyboard_pan(view_state, action, pan_step_screen_px)
                .map(|pan| MiniMapKeyboardInteractionPlan::Viewport {
                    pan,
                    zoom: view_state.zoom,
                })
                .unwrap_or(MiniMapKeyboardInteractionPlan::Ignore)
        }
        MiniMapKeyboardAction::ZoomIn | MiniMapKeyboardAction::ZoomOut => {
            plan_minimap_keyboard_zoom(
                view_state,
                canvas_bounds,
                min_zoom,
                max_zoom,
                zoom_step_mul,
                action,
            )
            .map(|(pan, zoom)| MiniMapKeyboardInteractionPlan::Viewport { pan, zoom })
            .unwrap_or(MiniMapKeyboardInteractionPlan::Ignore)
        }
        MiniMapKeyboardAction::FocusCanvas => MiniMapKeyboardInteractionPlan::FocusCanvas,
    }
}

pub(super) fn plan_minimap_pointer_down_interaction(
    button: MouseButton,
    minimap: Rect,
    world: Rect,
    viewport: Rect,
    position: Point,
    current_pan: CanvasPoint,
    zoom: f32,
    canvas_bounds: Rect,
) -> Option<MiniMapPointerDownInteractionPlan> {
    if button != MouseButton::Left || !minimap.contains(position) {
        return None;
    }

    let drag = plan_minimap_drag_start(
        minimap,
        world,
        viewport,
        position,
        current_pan,
        zoom,
        canvas_bounds,
    )?;

    Some(MiniMapPointerDownInteractionPlan {
        drag,
        focus_canvas: true,
        capture_pointer: true,
        stop_propagation: true,
        repaint: true,
    })
}

pub(super) fn plan_minimap_pointer_up_interaction(
    button: MouseButton,
    drag_active: bool,
) -> Option<MiniMapPointerUpInteractionPlan> {
    (button == MouseButton::Left && drag_active).then_some(MiniMapPointerUpInteractionPlan {
        release_capture: true,
        finish_event: true,
    })
}

#[cfg(test)]
mod tests {
    use fret_core::{KeyCode, MouseButton, Point, Px, Rect, Size};

    use crate::core::CanvasPoint;
    use crate::io::NodeGraphViewState;
    use crate::ui::overlays::minimap_interaction_policy::{
        MiniMapKeyboardInteractionPlan, plan_minimap_keyboard_interaction,
        plan_minimap_pointer_down_interaction, plan_minimap_pointer_up_interaction,
    };

    fn canvas_bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        )
    }

    fn minimap_bounds() -> Rect {
        Rect::new(
            Point::new(Px(590.0), Px(470.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn minimap_keyboard_interaction_plans_pan_zoom_focus_and_ignore() {
        let view_state = NodeGraphViewState {
            pan: CanvasPoint { x: 0.0, y: 0.0 },
            zoom: 1.0,
            ..Default::default()
        };

        assert_eq!(
            plan_minimap_keyboard_interaction(
                KeyCode::ArrowRight,
                &view_state,
                canvas_bounds(),
                0.05,
                64.0,
                24.0,
                1.1,
            ),
            MiniMapKeyboardInteractionPlan::Viewport {
                pan: CanvasPoint { x: -24.0, y: 0.0 },
                zoom: 1.0,
            }
        );

        let zoom_plan = plan_minimap_keyboard_interaction(
            KeyCode::NumpadAdd,
            &view_state,
            canvas_bounds(),
            0.05,
            64.0,
            24.0,
            1.1,
        );
        assert!(matches!(
            zoom_plan,
            MiniMapKeyboardInteractionPlan::Viewport { zoom, .. } if (zoom - 1.1).abs() <= 1.0e-6
        ));

        assert_eq!(
            plan_minimap_keyboard_interaction(
                KeyCode::Escape,
                &view_state,
                canvas_bounds(),
                0.05,
                64.0,
                24.0,
                1.1,
            ),
            MiniMapKeyboardInteractionPlan::FocusCanvas,
        );
        assert_eq!(
            plan_minimap_keyboard_interaction(
                KeyCode::Enter,
                &view_state,
                canvas_bounds(),
                0.05,
                64.0,
                24.0,
                1.1,
            ),
            MiniMapKeyboardInteractionPlan::Ignore,
        );
    }

    #[test]
    fn minimap_pointer_down_interaction_plans_focus_capture_drag_and_repaint() {
        let plan = plan_minimap_pointer_down_interaction(
            MouseButton::Left,
            minimap_bounds(),
            canvas_bounds(),
            canvas_bounds(),
            Point::new(Px(690.0), Px(530.0)),
            CanvasPoint { x: 10.0, y: 20.0 },
            1.0,
            canvas_bounds(),
        )
        .expect("pointer down plan");

        assert!(plan.focus_canvas);
        assert!(plan.capture_pointer);
        assert!(plan.stop_propagation);
        assert!(plan.repaint);
        assert_eq!(plan.drag.start_pan, CanvasPoint { x: 10.0, y: 20.0 });
        assert_eq!(plan.drag.immediate_pan, None);
    }

    #[test]
    fn minimap_pointer_down_interaction_ignores_non_left_or_outside_pointer() {
        assert!(
            plan_minimap_pointer_down_interaction(
                MouseButton::Right,
                minimap_bounds(),
                canvas_bounds(),
                canvas_bounds(),
                Point::new(Px(690.0), Px(530.0)),
                CanvasPoint { x: 0.0, y: 0.0 },
                1.0,
                canvas_bounds(),
            )
            .is_none()
        );
        assert!(
            plan_minimap_pointer_down_interaction(
                MouseButton::Left,
                minimap_bounds(),
                canvas_bounds(),
                canvas_bounds(),
                Point::new(Px(10.0), Px(10.0)),
                CanvasPoint { x: 0.0, y: 0.0 },
                1.0,
                canvas_bounds(),
            )
            .is_none()
        );
    }

    #[test]
    fn minimap_pointer_up_interaction_only_finishes_active_left_drag() {
        let plan =
            plan_minimap_pointer_up_interaction(MouseButton::Left, true).expect("pointer up plan");
        assert!(plan.release_capture);
        assert!(plan.finish_event);

        assert!(plan_minimap_pointer_up_interaction(MouseButton::Left, false).is_none());
        assert!(plan_minimap_pointer_up_interaction(MouseButton::Right, true).is_none());
    }
}
