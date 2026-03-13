use super::ContextMenuPaintLayout;
use super::*;
use fret_core::Scene;

pub(super) fn paint_context_menu_frame<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    scene: &mut Scene,
    menu: &ContextMenuState,
    zoom: f32,
) -> ContextMenuPaintLayout {
    let rect = context_menu_rect_at(&canvas.style, menu.origin, menu.items.len(), zoom);
    let border_w = Px(1.0 / zoom);
    let radius = Px(canvas.style.paint.context_menu_corner_radius / zoom);

    scene.push(SceneOp::Quad {
        order: DrawOrder(50),
        rect,
        background: fret_core::Paint::Solid(canvas.style.paint.context_menu_background).into(),
        border: Edges::all(border_w),
        border_paint: fret_core::Paint::Solid(canvas.style.paint.context_menu_border).into(),
        corner_radii: Corners::all(radius),
    });

    let pad = canvas.style.paint.context_menu_padding / zoom;
    let item_height = Px(canvas.style.paint.context_menu_item_height / zoom);
    let inner_origin = Point::new(Px(rect.origin.x.0 + pad), Px(rect.origin.y.0 + pad));
    let inner_width = Px((rect.size.width.0 - 2.0 * pad).max(0.0));

    let mut text_style = canvas.style.geometry.context_menu_text_style.clone();
    text_style.size = Px(text_style.size.0 / zoom);
    if let Some(line_height) = text_style.line_height.as_mut() {
        line_height.0 /= zoom;
    }

    ContextMenuPaintLayout {
        inner_origin,
        inner_width,
        item_height,
        hover_radius: Px(4.0 / zoom),
        text_style,
    }
}
