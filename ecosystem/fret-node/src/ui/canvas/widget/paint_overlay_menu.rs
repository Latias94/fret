#[path = "paint_overlay_menu/frame.rs"]
mod frame;
#[path = "paint_overlay_menu/items.rs"]
mod items;

use super::*;

struct ContextMenuPaintLayout {
    inner_origin: Point,
    inner_width: Px,
    item_height: Px,
    hover_radius: Px,
    text_style: fret_core::TextStyle,
}

pub(super) fn paint_context_menu<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    menu: &ContextMenuState,
    zoom: f32,
) {
    let layout = frame::paint_context_menu_frame(canvas, cx.scene, menu, zoom);
    items::paint_context_menu_items(canvas, cx, menu, zoom, &layout);
}
