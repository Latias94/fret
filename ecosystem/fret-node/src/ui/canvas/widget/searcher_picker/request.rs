use super::super::*;

pub(in crate::ui::canvas::widget) struct SearcherPickerRequest {
    pub(in crate::ui::canvas::widget) invoked_at: Point,
    pub(in crate::ui::canvas::widget) target: ContextMenuTarget,
    pub(in crate::ui::canvas::widget) candidates: Vec<InsertNodeCandidate>,
    pub(in crate::ui::canvas::widget) rows_mode: SearcherRowsMode,
}

pub(super) fn background_searcher_picker_request<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    at: CanvasPoint,
) -> SearcherPickerRequest {
    SearcherPickerRequest {
        invoked_at: Point::new(Px(at.x), Px(at.y)),
        target: ContextMenuTarget::BackgroundInsertNodePicker { at },
        candidates: canvas.list_background_insert_candidates(host),
        rows_mode: SearcherRowsMode::Catalog,
    }
}

pub(super) fn connection_searcher_picker_request<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    from: PortId,
    at: CanvasPoint,
) -> SearcherPickerRequest {
    SearcherPickerRequest {
        invoked_at: Point::new(Px(at.x), Px(at.y)),
        target: ContextMenuTarget::ConnectionInsertNodePicker { from, at },
        candidates: canvas.list_connection_insert_candidates(host, from),
        rows_mode: SearcherRowsMode::Catalog,
    }
}

pub(super) fn conversion_searcher_picker_request(
    from: PortId,
    to: PortId,
    at: CanvasPoint,
    candidates: Vec<InsertNodeCandidate>,
) -> SearcherPickerRequest {
    SearcherPickerRequest {
        invoked_at: Point::new(Px(at.x), Px(at.y)),
        target: ContextMenuTarget::ConnectionConvertPicker { from, to, at },
        candidates,
        rows_mode: SearcherRowsMode::Flat,
    }
}

pub(super) fn edge_insert_searcher_picker_request(
    edge: EdgeId,
    invoked_at: Point,
    candidates: Vec<InsertNodeCandidate>,
) -> SearcherPickerRequest {
    SearcherPickerRequest {
        invoked_at,
        target: ContextMenuTarget::EdgeInsertNodePicker(edge),
        candidates,
        rows_mode: SearcherRowsMode::Catalog,
    }
}

#[cfg(test)]
mod tests {
    use super::{conversion_searcher_picker_request, edge_insert_searcher_picker_request};
    use crate::core::{CanvasPoint, EdgeId, NodeKindKey, PortId};
    use crate::ui::presenter::InsertNodeCandidate;
    use fret_core::{Point, Px};
    use std::sync::Arc;

    #[test]
    fn conversion_request_uses_flat_rows_mode_and_convert_target() {
        let from = PortId::new();
        let to = PortId::new();
        let at = CanvasPoint { x: 120.0, y: 48.0 };
        let candidates = vec![InsertNodeCandidate {
            kind: NodeKindKey::new("math.add"),
            label: Arc::<str>::from("Math/Add"),
            enabled: true,
            template: None,
            payload: serde_json::Value::Null,
        }];

        let request = conversion_searcher_picker_request(from, to, at, candidates);

        assert!(matches!(
            request.rows_mode,
            crate::ui::canvas::state::SearcherRowsMode::Flat
        ));
        assert!(matches!(
            request.target,
            crate::ui::canvas::state::ContextMenuTarget::ConnectionConvertPicker {
                from: request_from,
                to: request_to,
                at: request_at,
            } if request_from == from && request_to == to && request_at.x == at.x && request_at.y == at.y
        ));
        assert_eq!(request.invoked_at.x.0, at.x);
        assert_eq!(request.invoked_at.y.0, at.y);
        assert_eq!(request.candidates.len(), 1);
    }

    #[test]
    fn edge_insert_request_uses_catalog_rows_mode_and_edge_target() {
        let edge = EdgeId::new();
        let invoked_at = Point::new(Px(12.0), Px(34.0));
        let candidates = vec![InsertNodeCandidate {
            kind: NodeKindKey::new("math.mul"),
            label: Arc::<str>::from("Math/Mul"),
            enabled: true,
            template: None,
            payload: serde_json::Value::Null,
        }];

        let request = edge_insert_searcher_picker_request(edge, invoked_at, candidates);

        assert!(matches!(
            request.rows_mode,
            crate::ui::canvas::state::SearcherRowsMode::Catalog
        ));
        assert!(matches!(
            request.target,
            crate::ui::canvas::state::ContextMenuTarget::EdgeInsertNodePicker(request_edge)
                if request_edge == edge
        ));
        assert_eq!(request.invoked_at, invoked_at);
        assert_eq!(request.candidates.len(), 1);
    }
}
