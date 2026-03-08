use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_context_menu<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        menu: &ContextMenuState,
        zoom: f32,
    ) {
        super::paint_overlay_menu::paint_context_menu(self, cx, menu, zoom);
    }

    pub(super) fn paint_marquee<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        marquee: &MarqueeDrag,
        zoom: f32,
    ) {
        super::paint_overlay_guides::paint_marquee(self, cx, marquee, zoom);
    }

    pub(super) fn paint_snap_guides<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        guides: &SnapGuides,
        zoom: f32,
        viewport_origin_x: f32,
        viewport_origin_y: f32,
        viewport_w: f32,
        viewport_h: f32,
    ) {
        super::paint_overlay_guides::paint_snap_guides(
            self,
            cx,
            guides,
            zoom,
            viewport_origin_x,
            viewport_origin_y,
            viewport_w,
            viewport_h,
        );
    }

    pub(super) fn paint_toast<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        toast: &ToastState,
        zoom: f32,
        viewport_origin_x: f32,
        viewport_origin_y: f32,
        viewport_h: f32,
    ) {
        super::paint_overlay_feedback::paint_toast(
            self,
            cx,
            toast,
            zoom,
            viewport_origin_x,
            viewport_origin_y,
            viewport_h,
        );
    }

    pub(super) fn paint_wire_drag_hint<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        wire_drag: &WireDrag,
        zoom: f32,
    ) {
        super::paint_overlay_feedback::paint_wire_drag_hint(self, cx, snapshot, wire_drag, zoom);
    }
}
