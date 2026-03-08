use super::*;

pub(super) fn handle_edge_insert_picker_double_click<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: fret_core::Modifiers,
    click_count: u8,
    zoom: f32,
) -> bool {
    if click_count != 2
        || !(modifiers.alt || modifiers.alt_gr)
        || canvas.interaction.searcher.is_some()
        || canvas.interaction.context_menu.is_some()
    {
        return false;
    }

    let Some(edge_id) = edge_double_click_target(canvas, cx, snapshot, position, zoom) else {
        return false;
    };

    canvas.update_view_state(cx.app, |state| {
        state.selected_nodes.clear();
        state.selected_groups.clear();
        if !state.selected_edges.iter().any(|id| *id == edge_id) {
            state.selected_edges.clear();
            state.selected_edges.push(edge_id);
        }
    });
    canvas.open_edge_insert_node_picker(cx.app, cx.window, edge_id, position);
    finish_double_click(cx);
    true
}

pub(super) fn handle_edge_reroute_double_click<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    click_count: u8,
    zoom: f32,
) -> bool {
    if click_count != 2
        || !snapshot.interaction.reroute_on_edge_double_click
        || canvas.interaction.searcher.is_some()
        || canvas.interaction.context_menu.is_some()
    {
        return false;
    }

    let Some(edge_id) = edge_double_click_target(canvas, cx, snapshot, position, zoom) else {
        return false;
    };

    let outcome = canvas.plan_canvas_split_edge_reroute(cx.app, edge_id, position);
    canvas.execute_split_edge_reroute_outcome(cx.app, cx.window, Some("Insert Reroute"), outcome);
    finish_double_click(cx);
    true
}

fn edge_double_click_target<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> Option<EdgeId> {
    let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
    canvas
        .graph
        .read_ref(cx.app, |graph| {
            let mut scratch = HitTestScratch::default();
            let mut ctx = HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);

            if canvas.hit_port(&mut ctx, position).is_some() {
                return None;
            }
            if canvas
                .hit_edge_focus_anchor(graph, snapshot, &mut ctx, position)
                .is_some()
            {
                return None;
            }
            if geom.nodes.values().any(|ng| ng.rect.contains(position)) {
                return None;
            }
            if graph.groups.iter().any(|(group_id, group)| {
                let rect0 = canvas.group_rect_with_preview(*group_id, group.rect);
                group_resize::group_rect_to_px(rect0).contains(position)
            }) {
                return None;
            }
            canvas.hit_edge(graph, snapshot, &mut ctx, position)
        })
        .ok()
        .flatten()
}

fn finish_double_click<H: UiHost>(cx: &mut EventCx<'_, H>) {
    cx.stop_propagation();
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
}
