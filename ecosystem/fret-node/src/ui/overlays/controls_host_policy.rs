use fret_core::{MouseButton, Point};

use super::controls_layout::ControlsLayout;
use super::controls_policy::ControlsButton;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ControlsPointerDownHostPlan {
    pub(super) request_focus: bool,
    pub(super) stop_propagation: bool,
    pub(super) capture_pointer: bool,
    pub(super) repaint: bool,
}

pub(super) fn controls_panel_hit_test(layout: &ControlsLayout, position: Point) -> bool {
    layout.panel.contains(position)
}

pub(super) fn plan_controls_pointer_down_host(
    button: MouseButton,
    target: Option<ControlsButton>,
) -> Option<ControlsPointerDownHostPlan> {
    if button != MouseButton::Left {
        return None;
    }

    Some(ControlsPointerDownHostPlan {
        request_focus: true,
        stop_propagation: true,
        capture_pointer: target.is_some(),
        repaint: target.is_some(),
    })
}

pub(super) fn plan_controls_declarative_panel_pointer_down(
    button: MouseButton,
    hit_is_pressable: bool,
) -> Option<ControlsPointerDownHostPlan> {
    if hit_is_pressable {
        return None;
    }
    plan_controls_pointer_down_host(button, None)
}

#[cfg(test)]
mod tests {
    use fret_core::{MouseButton, Point, Px, Rect, Size};

    use crate::ui::NodeGraphStyle;
    use crate::ui::overlays::OverlayPlacement;
    use crate::ui::overlays::controls_host_policy::{
        controls_panel_hit_test, plan_controls_declarative_panel_pointer_down,
        plan_controls_pointer_down_host,
    };
    use crate::ui::overlays::controls_layout::compute_controls_layout;
    use crate::ui::overlays::controls_policy::ControlsButton;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        )
    }

    fn test_style() -> NodeGraphStyle {
        let mut style = NodeGraphStyle::default();
        style.paint.controls_button_size = 20.0;
        style.paint.controls_padding = 4.0;
        style.paint.controls_gap = 2.0;
        style.paint.controls_margin = 10.0;
        style
    }

    #[test]
    fn controls_panel_hit_test_blocks_only_the_panel_rect() {
        let style = test_style();
        let layout = compute_controls_layout(&style, OverlayPlacement::FloatingInCanvas, bounds());

        assert!(controls_panel_hit_test(
            &layout,
            Point::new(
                Px(layout.panel.origin.x.0 + 1.0),
                Px(layout.panel.origin.y.0 + 1.0)
            )
        ));
        assert!(!controls_panel_hit_test(
            &layout,
            Point::new(
                Px(layout.panel.origin.x.0 - 1.0),
                Px(layout.panel.origin.y.0 - 1.0)
            )
        ));
    }

    #[test]
    fn controls_pointer_down_host_plan_blocks_panel_and_captures_buttons_only() {
        let button_plan =
            plan_controls_pointer_down_host(MouseButton::Left, Some(ControlsButton::ZoomIn))
                .expect("left button pointer down");
        assert!(button_plan.request_focus);
        assert!(button_plan.stop_propagation);
        assert!(button_plan.capture_pointer);
        assert!(button_plan.repaint);

        let panel_plan =
            plan_controls_pointer_down_host(MouseButton::Left, None).expect("panel pointer down");
        assert!(panel_plan.request_focus);
        assert!(panel_plan.stop_propagation);
        assert!(!panel_plan.capture_pointer);
        assert!(!panel_plan.repaint);

        assert!(plan_controls_pointer_down_host(MouseButton::Right, None).is_none());
    }

    #[test]
    fn controls_declarative_panel_pointer_down_preserves_pressable_button_activation() {
        assert!(
            plan_controls_declarative_panel_pointer_down(MouseButton::Left, true).is_none(),
            "button descendants should keep their own pressable activation path"
        );

        let plan = plan_controls_declarative_panel_pointer_down(MouseButton::Left, false)
            .expect("panel blank pointer down");
        assert!(plan.request_focus);
        assert!(plan.stop_propagation);
        assert!(!plan.capture_pointer);
        assert!(!plan.repaint);
    }
}
