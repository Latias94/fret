use super::SearcherPaintLayout;
use super::*;
use fret_core::Scene;

pub(super) fn paint_searcher_frame<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    scene: &mut Scene,
    searcher: &SearcherState,
    scale_factor: f32,
    zoom: f32,
) -> SearcherPaintLayout {
    let visible_rows = searcher_visible_rows(searcher);
    let rect = searcher_rect_at(&canvas.style, searcher.origin, visible_rows, zoom);
    let border_w = Px(1.0 / zoom);
    let radius = Px(canvas.style.paint.context_menu_corner_radius / zoom);

    scene.push(SceneOp::Quad {
        order: DrawOrder(55),
        rect,
        background: fret_core::Paint::Solid(canvas.style.paint.context_menu_background).into(),
        border: Edges::all(border_w),
        border_paint: fret_core::Paint::Solid(canvas.style.paint.context_menu_border).into(),
        corner_radii: Corners::all(radius),
    });

    let pad = canvas.style.paint.context_menu_padding / zoom;
    let item_h = canvas.style.paint.context_menu_item_height / zoom;
    let inner_x = rect.origin.x.0 + pad;
    let inner_y = rect.origin.y.0 + pad;
    let inner_w = (rect.size.width.0 - 2.0 * pad).max(0.0);

    let mut text_style = canvas.style.geometry.context_menu_text_style.clone();
    text_style.size = Px(text_style.size.0 / zoom);
    if let Some(line_height) = text_style.line_height.as_mut() {
        line_height.0 /= zoom;
    }

    let constraints = TextConstraints {
        max_width: Some(Px(inner_w)),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: effective_scale_factor(scale_factor, zoom),
    };

    SearcherPaintLayout {
        inner_x,
        inner_y,
        inner_w,
        item_h,
        pad,
        text_style,
        constraints,
    }
}
