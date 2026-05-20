use fret_core::{Color, DrawOrder, Rect, TextConstraints, TextOverflow, TextStyle, TextWrap};

use crate::interaction::NodeGraphConnectionMode;
use crate::ui::style::NodeGraphStyle;

use super::controls_layout::ControlsLayout;
use super::controls_policy::{ControlsButton, controls_button_label};
use super::panel_item_state::panel_item_visual_state;

const PANEL_ORDER: DrawOrder = DrawOrder(21_000);
const BUTTON_BACKGROUND_ORDER: DrawOrder = DrawOrder(21_001);
const TEXT_ORDER: DrawOrder = DrawOrder(21_002);

#[derive(Debug, Clone, Copy)]
pub(super) struct ControlsPaintState {
    pub(super) hovered: Option<ControlsButton>,
    pub(super) pressed: Option<ControlsButton>,
    pub(super) keyboard_active: Option<ControlsButton>,
    pub(super) keyboard_visible: bool,
}

#[derive(Clone)]
pub(super) struct ControlsPaintPlan {
    pub(super) panel: ControlsPanelPaintPlan,
    pub(super) text_style: TextStyle,
    pub(super) text_constraints: TextConstraints,
    pub(super) buttons: Vec<ControlsButtonPaintPlan>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ControlsPanelPaintPlan {
    pub(super) order: DrawOrder,
    pub(super) rect: Rect,
    pub(super) background: Color,
    pub(super) border: Color,
    pub(super) corner_px: f32,
}

#[derive(Debug, Clone)]
pub(super) struct ControlsButtonPaintPlan {
    pub(super) button: ControlsButton,
    pub(super) rect: Rect,
    pub(super) label: &'static str,
    pub(super) background: Color,
    pub(super) text_color: Color,
    pub(super) corner_px: f32,
    pub(super) background_order: DrawOrder,
    pub(super) text_order: DrawOrder,
}

pub(super) fn controls_paint_plan(
    style: &NodeGraphStyle,
    layout: &ControlsLayout,
    state: ControlsPaintState,
    connection_mode: NodeGraphConnectionMode,
    scale_factor: f32,
) -> ControlsPaintPlan {
    ControlsPaintPlan {
        panel: ControlsPanelPaintPlan {
            order: PANEL_ORDER,
            rect: layout.panel,
            background: style.paint.context_menu_background,
            border: style.paint.context_menu_border,
            corner_px: style.paint.context_menu_corner_radius,
        },
        text_style: style.paint.controls_text_style.clone(),
        text_constraints: controls_text_constraints(scale_factor),
        buttons: layout
            .buttons
            .iter()
            .map(|(button, rect)| {
                controls_button_paint_plan(style, *button, *rect, state, connection_mode)
            })
            .collect(),
    }
}

fn controls_button_paint_plan(
    style: &NodeGraphStyle,
    button: ControlsButton,
    rect: Rect,
    state: ControlsPaintState,
    connection_mode: NodeGraphConnectionMode,
) -> ControlsButtonPaintPlan {
    ControlsButtonPaintPlan {
        button,
        rect,
        label: controls_button_label(button, connection_mode),
        background: controls_button_background(style, button, state),
        text_color: style.paint.controls_text,
        corner_px: style.paint.context_menu_corner_radius,
        background_order: BUTTON_BACKGROUND_ORDER,
        text_order: TEXT_ORDER,
    }
}

fn controls_button_background(
    style: &NodeGraphStyle,
    button: ControlsButton,
    state: ControlsPaintState,
) -> Color {
    let visual_state = panel_item_visual_state(
        button,
        state.hovered,
        state.pressed,
        state.keyboard_active,
        state.keyboard_visible,
        true,
    );
    if visual_state.pressed {
        style.paint.controls_active_background
    } else if visual_state.hovered || visual_state.keyboard {
        style.paint.controls_hover_background
    } else {
        Color::TRANSPARENT
    }
}

fn controls_text_constraints(scale_factor: f32) -> TextConstraints {
    TextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor,
    }
}

#[cfg(test)]
mod tests {
    use fret_core::{Color, DrawOrder, Point, Px, Rect, Size, TextOverflow, TextWrap};

    use crate::interaction::NodeGraphConnectionMode;
    use crate::ui::NodeGraphStyle;
    use crate::ui::overlays::OverlayPlacement;
    use crate::ui::overlays::controls_layout::compute_controls_layout;
    use crate::ui::overlays::controls_paint_plan::{ControlsPaintState, controls_paint_plan};
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

    fn idle_state() -> ControlsPaintState {
        ControlsPaintState {
            hovered: None,
            pressed: None,
            keyboard_active: None,
            keyboard_visible: false,
        }
    }

    #[test]
    fn controls_paint_plan_emits_panel_and_button_draw_decisions() {
        let style = test_style();
        let layout = compute_controls_layout(&style, OverlayPlacement::FloatingInCanvas, bounds());
        let plan = controls_paint_plan(
            &style,
            &layout,
            idle_state(),
            NodeGraphConnectionMode::Strict,
            2.0,
        );

        assert_eq!(plan.panel.order, DrawOrder(21_000));
        assert_eq!(plan.panel.rect, layout.panel);
        assert_eq!(plan.panel.background, style.paint.context_menu_background);
        assert_eq!(plan.panel.border, style.paint.context_menu_border);
        assert_eq!(plan.panel.corner_px, style.paint.context_menu_corner_radius);
        assert_eq!(plan.text_constraints.scale_factor, 2.0);
        assert_eq!(plan.text_constraints.wrap, TextWrap::None);
        assert_eq!(plan.text_constraints.overflow, TextOverflow::Clip);

        assert_eq!(plan.buttons.len(), layout.buttons.len());
        let first = &plan.buttons[0];
        assert_eq!(first.button, ControlsButton::ToggleConnectionMode);
        assert_eq!(first.rect, layout.buttons[0].1);
        assert_eq!(first.label, "S");
        assert_eq!(first.background, Color::TRANSPARENT);
        assert_eq!(first.text_color, style.paint.controls_text);
        assert_eq!(first.corner_px, style.paint.context_menu_corner_radius);
        assert_eq!(first.background_order, DrawOrder(21_001));
        assert_eq!(first.text_order, DrawOrder(21_002));
    }

    #[test]
    fn controls_paint_plan_labels_follow_connection_mode() {
        let style = test_style();
        let layout = compute_controls_layout(&style, OverlayPlacement::FloatingInCanvas, bounds());

        let strict = controls_paint_plan(
            &style,
            &layout,
            idle_state(),
            NodeGraphConnectionMode::Strict,
            1.0,
        );
        let loose = controls_paint_plan(
            &style,
            &layout,
            idle_state(),
            NodeGraphConnectionMode::Loose,
            1.0,
        );

        assert_eq!(strict.buttons[0].label, "S");
        assert_eq!(loose.buttons[0].label, "L");
        assert_eq!(loose.buttons[1].label, "+");
        assert_eq!(loose.buttons[5].label, "1:1");
    }

    #[test]
    fn controls_paint_plan_uses_active_hover_keyboard_and_idle_backgrounds() {
        let style = test_style();
        let layout = compute_controls_layout(&style, OverlayPlacement::FloatingInCanvas, bounds());

        let pressed = controls_paint_plan(
            &style,
            &layout,
            ControlsPaintState {
                hovered: Some(ControlsButton::ZoomIn),
                pressed: Some(ControlsButton::ZoomIn),
                keyboard_active: None,
                keyboard_visible: true,
            },
            NodeGraphConnectionMode::Strict,
            1.0,
        );
        assert_eq!(
            pressed.buttons[1].background,
            style.paint.controls_active_background
        );

        let hovered = controls_paint_plan(
            &style,
            &layout,
            ControlsPaintState {
                hovered: Some(ControlsButton::ZoomOut),
                pressed: None,
                keyboard_active: None,
                keyboard_visible: true,
            },
            NodeGraphConnectionMode::Strict,
            1.0,
        );
        assert_eq!(
            hovered.buttons[2].background,
            style.paint.controls_hover_background
        );

        let keyboard = controls_paint_plan(
            &style,
            &layout,
            ControlsPaintState {
                hovered: None,
                pressed: None,
                keyboard_active: Some(ControlsButton::FrameAll),
                keyboard_visible: true,
            },
            NodeGraphConnectionMode::Strict,
            1.0,
        );
        assert_eq!(
            keyboard.buttons[3].background,
            style.paint.controls_hover_background
        );

        let hidden_keyboard = controls_paint_plan(
            &style,
            &layout,
            ControlsPaintState {
                hovered: None,
                pressed: None,
                keyboard_active: Some(ControlsButton::FrameAll),
                keyboard_visible: false,
            },
            NodeGraphConnectionMode::Strict,
            1.0,
        );
        assert_eq!(hidden_keyboard.buttons[3].background, Color::TRANSPARENT);

        let pointer_active_suppresses_keyboard = controls_paint_plan(
            &style,
            &layout,
            ControlsPaintState {
                hovered: Some(ControlsButton::ZoomIn),
                pressed: None,
                keyboard_active: Some(ControlsButton::FrameAll),
                keyboard_visible: true,
            },
            NodeGraphConnectionMode::Strict,
            1.0,
        );
        assert_eq!(
            pointer_active_suppresses_keyboard.buttons[1].background,
            style.paint.controls_hover_background
        );
        assert_eq!(
            pointer_active_suppresses_keyboard.buttons[3].background,
            Color::TRANSPARENT
        );
    }
}
