use crate::ui::canvas::widget::*;

use super::super::item_builders;

pub(super) fn try_show_group_context_menu<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    let Some(group_id) = canvas.hit_group_context_target(cx.app, snapshot, position, zoom) else {
        return false;
    };

    let items = item_builders::build_group_context_menu_items();
    canvas.select_group_context_target(cx.app, group_id);
    canvas.show_context_menu(
        cx,
        snapshot,
        position,
        ContextMenuTarget::Group(group_id),
        items,
        Vec::new(),
        true,
    )
}
