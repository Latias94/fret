use super::*;

pub(super) fn handle_background_zoom_double_click<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: fret_core::Modifiers,
    click_count: u8,
    zoom: f32,
) -> bool {
    if click_count != 2
        || !snapshot.interaction.zoom_on_double_click
        || canvas.interaction.searcher.is_some()
        || canvas.interaction.context_menu.is_some()
    {
        return false;
    }

    if !pointer_is_background(canvas, cx, snapshot, position, zoom) {
        return false;
    }

    if let Some(state) = canvas.interaction.viewport_move_debounce.take() {
        cx.app
            .push_effect(Effect::CancelTimer { token: state.timer });
        canvas.emit_move_end(snapshot, state.kind, ViewportMoveEndOutcome::Ended);
    }

    canvas.emit_move_start(snapshot, ViewportMoveKind::ZoomDoubleClick);
    let factor = if modifiers.shift { 0.5 } else { 2.0 };
    canvas.zoom_about_pointer_factor(position, factor);
    let pan = canvas.cached_pan;
    let zoom = canvas.cached_zoom;
    canvas.update_view_state(cx.app, |state| {
        state.pan = pan;
        state.zoom = zoom;
    });
    let snap = canvas.sync_view_state(cx.app);
    canvas.emit_move_end(
        &snap,
        ViewportMoveKind::ZoomDoubleClick,
        ViewportMoveEndOutcome::Ended,
    );
    cx.stop_propagation();
    paint_invalidation::invalidate_paint(cx);
    true
}

fn pointer_is_background<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
    canvas
        .graph
        .read_ref(cx.app, |graph| {
            let mut scratch = HitTestScratch::default();
            let mut ctx = HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);

            if canvas.hit_port(&mut ctx, position).is_some() {
                return false;
            }
            if canvas
                .hit_edge_focus_anchor(graph, snapshot, &mut ctx, position)
                .is_some()
            {
                return false;
            }
            if geom.nodes.values().any(|ng| ng.rect.contains(position)) {
                return false;
            }
            if canvas
                .hit_edge(graph, snapshot, &mut ctx, position)
                .is_some()
            {
                return false;
            }
            !graph.groups.iter().any(|(group_id, group)| {
                let rect0 = canvas.group_rect_with_preview(*group_id, group.rect);
                group_resize::group_rect_to_px(rect0).contains(position)
            })
        })
        .unwrap_or(false)
}
