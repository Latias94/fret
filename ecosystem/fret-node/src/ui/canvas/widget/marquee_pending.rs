use fret_canvas::scale::canvas_units_from_screen_px;
use fret_core::{Modifiers, MouseButton, Point};
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn handle_pending_marquee<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    if canvas.interaction.node_drag.is_some() {
        return false;
    }

    let Some(pending) = canvas.interaction.pending_marquee.clone() else {
        return false;
    };

    let selection_key_pressed = snapshot.interaction.selection_key.is_pressed(modifiers);
    let threshold_screen = if selection_key_pressed {
        0.0
    } else {
        snapshot.interaction.pane_click_distance.max(0.0)
    };
    let threshold_graph = canvas_units_from_screen_px(threshold_screen, zoom);
    let dx = position.x.0 - pending.start_pos.x.0;
    let dy = position.y.0 - pending.start_pos.y.0;
    if threshold_graph > 0.0 && dx * dx + dy * dy < threshold_graph * threshold_graph {
        return false;
    }

    let selection_box_active = snapshot.interaction.selection_on_drag || selection_key_pressed;
    if selection_box_active {
        return super::marquee_selection::activate_pending_marquee(
            canvas,
            cx,
            snapshot,
            pending.start_pos,
            position,
        );
    }

    if snapshot.interaction.pan_on_drag.left {
        canvas.interaction.pending_marquee = None;
        let _ = super::pan_zoom::begin_panning(
            canvas,
            cx,
            snapshot,
            pending.start_pos,
            MouseButton::Left,
        );
        return true;
    }

    false
}
