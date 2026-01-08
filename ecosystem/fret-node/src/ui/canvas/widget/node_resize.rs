use fret_core::{Modifiers, Point, Px};
use fret_ui::UiHost;

use crate::core::CanvasSize;

use super::super::state::ViewSnapshot;
use super::NodeGraphCanvas;

pub(super) fn handle_node_resize_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    _snapshot: &ViewSnapshot,
    position: Point,
    _modifiers: Modifiers,
    _zoom: f32,
) -> bool {
    let Some(resize) = canvas.interaction.node_resize.clone() else {
        return false;
    };

    let delta_screen = Point::new(
        Px(position.x.0 - resize.start_pos.x.0),
        Px(position.y.0 - resize.start_pos.y.0),
    );

    let mut new_size = CanvasSize {
        width: resize.start_size.width + delta_screen.x.0,
        height: resize.start_size.height + delta_screen.y.0,
    };

    let min_w = 120.0;
    let min_h = 80.0;
    if new_size.width.is_finite() {
        new_size.width = new_size.width.max(min_w);
    } else {
        new_size.width = min_w;
    }
    if new_size.height.is_finite() {
        new_size.height = new_size.height.max(min_h);
    } else {
        new_size.height = min_h;
    }

    let _ = canvas.graph.update(cx.app, |g, _cx| {
        let Some(node) = g.nodes.get_mut(&resize.node) else {
            return;
        };

        if let Some(parent) = node.parent
            && let Some(group) = g.groups.get(&parent)
        {
            let max_w = (group.rect.origin.x + group.rect.size.width - node.pos.x).max(min_w);
            let max_h = (group.rect.origin.y + group.rect.size.height - node.pos.y).max(min_h);
            new_size.width = new_size.width.min(max_w);
            new_size.height = new_size.height.min(max_h);
        }

        node.size = Some(new_size);
    });

    // Invalidate derived geometry caches that depend on node bounds.
    canvas.geometry.key = None;
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}
