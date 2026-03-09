use super::*;

pub(super) fn sync_semantics<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut SemanticsCx<'_, H>,
) {
    let theme = Theme::global(&*cx.app).snapshot();
    canvas.sync_style_from_color_mode(theme, None);
    canvas.sync_skin(None);
    canvas.sync_paint_overrides(None);
    canvas.interaction.last_bounds = Some(cx.bounds);
    let snapshot = canvas.sync_view_state(cx.app);

    cx.set_role(fret_core::SemanticsRole::Viewport);
    cx.set_focusable(true);
    cx.set_label(canvas.presenter.a11y_canvas_label().as_ref());
    cx.set_test_id("node_graph.canvas");
    cx.set_active_descendant(super::retained_widget_semantics_focus::active_descendant(
        canvas, cx,
    ));
    cx.set_value(
        super::retained_widget_semantics_value::build_semantics_value(canvas, cx, &snapshot),
    );
}
