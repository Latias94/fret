use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

use super::prepare::PreparedEdgePaintBatches;
use super::preview::push_drop_marker;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn paint_edges<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        render: &RenderData,
        geom: &CanvasGeometry,
        zoom: f32,
        view_interacting: bool,
    ) {
        let interaction_hint = if let Some(skin) = self.skin.as_ref() {
            self.graph
                .read_ref(cx.app, |g| skin.interaction_chrome_hint(g, &self.style))
                .ok()
                .unwrap_or_default()
        } else {
            crate::ui::InteractionChromeHint::default()
        };

        let custom_paths = self.collect_custom_edge_paths(&*cx.app, &render.edges, zoom);
        let bezier_steps = usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));
        let PreparedEdgePaintBatches {
            edges_normal,
            edges_selected,
            edges_hovered,
            edge_insert_marker,
            insert_node_drag_marker,
        } = self.prepare_edge_paint_batches(snapshot, render, &custom_paths, zoom);

        let marker_budget_limit = Self::EDGE_MARKER_BUILD_BUDGET_PER_FRAME.select(view_interacting);
        let mut marker_budget = WorkBudget::new(marker_budget_limit);
        let mut marker_budget_skipped: u32 = 0;
        let mut wire_budget = WorkBudget::new(u32::MAX / 2);
        let outline_budget_limit =
            Self::EDGE_WIRE_OUTLINE_BUILD_BUDGET_PER_FRAME.select(view_interacting);
        let mut outline_budget = WorkBudget::new(outline_budget_limit);
        let mut outline_budget_skipped: u32 = 0;
        let highlight_budget_limit =
            Self::EDGE_WIRE_HIGHLIGHT_BUILD_BUDGET_PER_FRAME.select(view_interacting);
        let mut highlight_budget = WorkBudget::new(highlight_budget_limit);
        let mut highlight_budget_skipped: u32 = 0;

        for edge in edges_normal
            .into_iter()
            .chain(edges_selected)
            .chain(edges_hovered)
        {
            let custom = custom_paths.get(&edge.id);
            let chrome = self.prepare_edge_chrome(
                cx,
                custom,
                interaction_hint,
                edge.selected,
                edge.hovered,
                edge.id,
                edge.from,
                edge.to,
                edge.route,
                edge.color,
                edge.width,
                edge.dash,
                zoom,
                &mut outline_budget,
                &mut outline_budget_skipped,
                &mut highlight_budget,
                &mut highlight_budget_skipped,
            );

            let (_stop, skipped) = if let Some(custom) = custom {
                let fallback = Point::new(
                    Px(edge.to.x.0 - edge.from.x.0),
                    Px(edge.to.y.0 - edge.from.y.0),
                );
                let (t0, t1) =
                    path_start_end_tangents(&custom.commands).unwrap_or((fallback, fallback));
                self.push_edge_custom_wire_and_markers_budgeted(
                    cx.scene,
                    cx.services,
                    custom.cache_key,
                    &custom.commands,
                    t0,
                    t1,
                    zoom,
                    cx.scale_factor,
                    edge.from,
                    edge.to,
                    edge.paint,
                    edge.color,
                    edge.width,
                    edge.dash,
                    chrome.highlight,
                    edge.start_marker.as_ref(),
                    edge.end_marker.as_ref(),
                    &mut wire_budget,
                    &mut marker_budget,
                    false,
                )
            } else {
                self.push_edge_wire_and_markers_budgeted(
                    cx.scene,
                    cx.services,
                    zoom,
                    cx.scale_factor,
                    edge.route,
                    edge.from,
                    edge.to,
                    edge.paint,
                    edge.color,
                    edge.width,
                    edge.dash,
                    chrome.highlight,
                    edge.start_marker.as_ref(),
                    edge.end_marker.as_ref(),
                    &mut wire_budget,
                    &mut marker_budget,
                    false,
                )
            };
            marker_budget_skipped = marker_budget_skipped.saturating_add(skipped);

            if chrome.glow_pushed {
                cx.scene.push(SceneOp::PopEffect);
            }
        }

        super::super::redraw_request::request_paint_redraw_if(cx, marker_budget_skipped > 0);
        super::super::redraw_request::request_paint_redraw_if(cx, outline_budget_skipped > 0);
        super::super::redraw_request::request_paint_redraw_if(cx, highlight_budget_skipped > 0);

        if let Some((pos, color)) = edge_insert_marker {
            push_drop_marker(cx.scene, pos, color, zoom);
        }
        if let Some((pos, color)) = insert_node_drag_marker {
            push_drop_marker(cx.scene, pos, color, zoom);
        }

        self.paint_edge_labels_tail(
            cx,
            render,
            &custom_paths,
            bezier_steps,
            zoom,
            view_interacting,
        );
        Self::record_edge_marker_budget_stat(
            cx,
            marker_budget_limit,
            marker_budget.used(),
            marker_budget_skipped,
        );

        self.paint_wire_drag_preview(
            cx,
            render,
            geom,
            zoom,
            interaction_hint,
            &mut outline_budget,
            &mut outline_budget_skipped,
        );
    }
}
