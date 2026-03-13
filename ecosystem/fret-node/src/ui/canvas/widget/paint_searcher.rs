#[path = "paint_searcher/frame.rs"]
mod frame;

use super::*;

struct SearcherPaintLayout {
    inner_x: f32,
    inner_y: f32,
    inner_w: f32,
    item_h: f32,
    pad: f32,
    text_style: fret_core::TextStyle,
    constraints: TextConstraints,
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_searcher<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        searcher: &SearcherState,
        zoom: f32,
    ) {
        let layout = frame::paint_searcher_frame(self, cx.scene, searcher, cx.scale_factor, zoom);

        let list_y0 = super::paint_searcher_query::paint_searcher_query(
            self,
            cx,
            searcher,
            &layout.text_style,
            layout.constraints,
            layout.inner_x,
            layout.inner_y,
            layout.inner_w,
            layout.item_h,
            layout.pad,
            zoom,
        );
        super::paint_searcher_rows::paint_searcher_rows(
            self,
            cx,
            searcher,
            &layout.text_style,
            layout.constraints,
            layout.inner_x,
            list_y0,
            layout.inner_w,
            layout.item_h,
            zoom,
        );
    }
}
