use super::resolve_paint_root_hovered_edge;
use crate::core::EdgeId;
use crate::ui::canvas::state::{
    EdgeInsertDrag, InsertNodeDragPreview, InteractionState, PendingEdgeInsertDrag,
};
use fret_core::Point;
use std::sync::Arc;

#[test]
fn resolve_paint_root_hovered_edge_prefers_edge_insert_drag_then_insert_preview_then_hover() {
    let hovered = EdgeId::new();
    let preview = EdgeId::new();
    let active = EdgeId::new();
    let interaction = InteractionState {
        hover_edge: Some(hovered),
        insert_node_drag_preview: Some(InsertNodeDragPreview {
            label: Arc::<str>::from("preview"),
            pos: Point::default(),
            edge: Some(preview),
        }),
        edge_insert_drag: Some(EdgeInsertDrag {
            edge: active,
            pos: Point::default(),
        }),
        ..Default::default()
    };

    assert_eq!(resolve_paint_root_hovered_edge(&interaction), Some(active));
}

#[test]
fn resolve_paint_root_hovered_edge_uses_pending_insert_before_hover() {
    let hovered = EdgeId::new();
    let pending = EdgeId::new();
    let interaction = InteractionState {
        hover_edge: Some(hovered),
        pending_edge_insert_drag: Some(PendingEdgeInsertDrag {
            edge: pending,
            start_pos: Point::default(),
        }),
        ..Default::default()
    };

    assert_eq!(resolve_paint_root_hovered_edge(&interaction), Some(pending));
}
