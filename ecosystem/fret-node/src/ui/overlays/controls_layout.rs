use fret_core::{Point, Px, Rect, Size};

use crate::ui::NodeGraphStyle;
use crate::ui::screen_space_placement::{AxisAlign, rect_in_bounds};

use super::OverlayPlacement;
use super::controls_policy::{ControlsButton, controls_buttons};

#[derive(Debug, Clone)]
pub(super) struct ControlsLayout {
    pub(super) panel: Rect,
    pub(super) buttons: Vec<(ControlsButton, Rect)>,
}

pub(super) fn controls_panel_size(style: &NodeGraphStyle) -> Size {
    let pad = style.paint.controls_padding.max(0.0);
    let gap = style.paint.controls_gap.max(0.0);
    let button = style.paint.controls_button_size.max(10.0);

    let panel_w = button + 2.0 * pad;
    let item_count = controls_buttons().len() as f32;
    let panel_h = item_count * button + (item_count - 1.0) * gap + 2.0 * pad;
    Size::new(Px(panel_w), Px(panel_h))
}

pub(super) fn compute_controls_layout(
    style: &NodeGraphStyle,
    placement: OverlayPlacement,
    bounds: Rect,
) -> ControlsLayout {
    let margin = style.paint.controls_margin.max(0.0);
    let pad = style.paint.controls_padding.max(0.0);
    let gap = style.paint.controls_gap.max(0.0);
    let button = style.paint.controls_button_size.max(10.0);

    let panel = match placement {
        OverlayPlacement::FloatingInCanvas => rect_in_bounds(
            bounds,
            controls_panel_size(style),
            AxisAlign::End,
            AxisAlign::Start,
            margin,
            Point::new(Px(0.0), Px(0.0)),
        ),
        OverlayPlacement::PanelBounds => bounds,
    };

    let mut buttons = Vec::with_capacity(controls_buttons().len());
    let mut cy = panel.origin.y.0 + pad;
    for item in controls_buttons().iter().copied() {
        let rect = Rect::new(
            Point::new(Px(panel.origin.x.0 + pad), Px(cy)),
            Size::new(Px(button), Px(button)),
        );
        buttons.push((item, rect));
        cy += button + gap;
    }

    ControlsLayout { panel, buttons }
}

pub(super) fn controls_button_at(
    layout: &ControlsLayout,
    position: Point,
) -> Option<ControlsButton> {
    for (button, rect) in &layout.buttons {
        if rect.contains(position) {
            return Some(*button);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{compute_controls_layout, controls_button_at, controls_panel_size};
    use crate::ui::NodeGraphStyle;
    use crate::ui::overlays::OverlayPlacement;
    use crate::ui::overlays::controls_policy::ControlsButton;
    use fret_core::{Point, Px, Rect, Size};

    fn test_style() -> NodeGraphStyle {
        let mut style = NodeGraphStyle::default();
        style.paint.controls_button_size = 20.0;
        style.paint.controls_padding = 4.0;
        style.paint.controls_gap = 2.0;
        style.paint.controls_margin = 10.0;
        style
    }

    #[test]
    fn controls_layout_places_panel_and_buttons_in_expected_slots() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let layout =
            compute_controls_layout(&test_style(), OverlayPlacement::FloatingInCanvas, bounds);

        assert_eq!(layout.panel.origin, Point::new(Px(762.0), Px(10.0)));
        assert_eq!(layout.panel.size, controls_panel_size(&test_style()));
        assert_eq!(layout.buttons.len(), 6);
        assert_eq!(layout.buttons[0].0, ControlsButton::ToggleConnectionMode);
        assert!(layout.buttons[1].1.origin.y.0 > layout.buttons[0].1.origin.y.0);
    }

    #[test]
    fn controls_button_hit_testing_matches_button_rects() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let layout =
            compute_controls_layout(&test_style(), OverlayPlacement::FloatingInCanvas, bounds);
        let first = layout.buttons[0].1;
        let last = layout.buttons[5].1;

        assert_eq!(
            controls_button_at(
                &layout,
                Point::new(Px(first.origin.x.0 + 1.0), Px(first.origin.y.0 + 1.0))
            ),
            Some(ControlsButton::ToggleConnectionMode)
        );
        assert_eq!(
            controls_button_at(
                &layout,
                Point::new(Px(last.origin.x.0 + 1.0), Px(last.origin.y.0 + 1.0))
            ),
            Some(ControlsButton::ResetView)
        );
        assert_eq!(
            controls_button_at(
                &layout,
                Point::new(
                    Px(layout.panel.origin.x.0 + 1.0),
                    Px(layout.panel.origin.y.0 + 1.0)
                )
            ),
            None
        );
    }

    #[test]
    fn controls_panel_bounds_mode_uses_host_bounds_directly() {
        let bounds = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(120.0), Px(90.0)),
        );
        let layout = compute_controls_layout(&test_style(), OverlayPlacement::PanelBounds, bounds);
        assert_eq!(layout.panel, bounds);
    }
}
