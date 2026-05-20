mod item;
mod retained_cx;

use super::*;

pub(in crate::ui::canvas::widget) trait SearcherRowActivationCx<M: NodeGraphCanvasMiddleware> {
    fn activate_searcher_context_item(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        target: &ContextMenuTarget,
        invoked_at: Point,
        item: NodeGraphContextMenuItem,
        menu_candidates: &[InsertNodeCandidate],
    );
}

pub(super) fn try_activate_searcher_row<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl SearcherRowActivationCx<M>,
    row_ix: usize,
) -> bool {
    let Some(searcher) = super::searcher_ui::take_searcher_overlay(&mut canvas.interaction) else {
        return false;
    };

    let Some(item) = item::searcher_row_activation_item(&searcher, row_ix) else {
        super::searcher_ui::restore_searcher_overlay(&mut canvas.interaction, searcher);
        return false;
    };
    cx.activate_searcher_context_item(
        canvas,
        &searcher.target,
        searcher.invoked_at,
        item,
        &searcher.candidates,
    );
    true
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::SearcherRowActivationCx;
    use crate::core::{CanvasPoint, Graph, GraphId, NodeKindKey};
    use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
    use crate::ui::canvas::searcher::{SearcherRow, SearcherRowKind};
    use crate::ui::canvas::state::{ContextMenuTarget, SearcherRowsMode, SearcherState};
    use crate::ui::canvas::widget::{NodeGraphCanvasWith, NoopNodeGraphCanvasMiddleware};
    use crate::ui::presenter::{
        InsertNodeCandidate, NodeGraphContextMenuAction, NodeGraphContextMenuItem,
    };
    use fret_core::{Point, Px};
    use fret_runtime::ModelStore;
    use serde_json::Value;

    #[derive(Debug)]
    struct ActivationCall {
        target: ContextMenuTarget,
        invoked_at: Point,
        item: NodeGraphContextMenuItem,
        candidates: usize,
    }

    #[derive(Default)]
    struct StubCx {
        calls: Vec<ActivationCall>,
    }

    impl SearcherRowActivationCx<NoopNodeGraphCanvasMiddleware> for StubCx {
        fn activate_searcher_context_item(
            &mut self,
            _canvas: &mut NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
            target: &ContextMenuTarget,
            invoked_at: Point,
            item: NodeGraphContextMenuItem,
            menu_candidates: &[InsertNodeCandidate],
        ) {
            self.calls.push(ActivationCall {
                target: target.clone(),
                invoked_at,
                item,
                candidates: menu_candidates.len(),
            });
        }
    }

    fn test_canvas() -> NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware> {
        let mut models = ModelStore::default();
        let graph = models.insert(Graph::new(GraphId::new()));
        let view = models.insert(NodeGraphViewState::default());
        let editor_config = models.insert(NodeGraphEditorConfig::default());
        NodeGraphCanvasWith::new_with_middleware(
            graph,
            view,
            editor_config,
            NoopNodeGraphCanvasMiddleware,
        )
    }

    fn candidate(label: &str) -> InsertNodeCandidate {
        InsertNodeCandidate {
            kind: NodeKindKey::new(format!("demo.{label}")),
            label: Arc::from(label),
            enabled: true,
            template: None,
            payload: Value::Null,
        }
    }

    fn searcher(rows: Vec<SearcherRow>, target: ContextMenuTarget) -> SearcherState {
        SearcherState {
            origin: Point::new(Px(10.0), Px(20.0)),
            invoked_at: Point::new(Px(30.0), Px(40.0)),
            target,
            rows_mode: SearcherRowsMode::Catalog,
            query: String::new(),
            candidates: vec![candidate("alpha"), candidate("beta")],
            recent_kinds: Vec::new(),
            rows,
            hovered_row: None,
            active_row: 0,
            scroll: 0,
        }
    }

    #[test]
    fn searcher_row_activation_without_searcher_is_side_effect_free() {
        let mut canvas = test_canvas();
        let mut cx = StubCx::default();

        assert!(!super::try_activate_searcher_row(&mut canvas, &mut cx, 0));

        assert!(cx.calls.is_empty());
        assert!(canvas.interaction.searcher.is_none());
    }

    #[test]
    fn searcher_row_activation_restores_unactivatable_row() {
        let mut canvas = test_canvas();
        canvas.interaction.searcher = Some(searcher(
            vec![SearcherRow {
                kind: SearcherRowKind::Header,
                label: Arc::from("Header"),
                enabled: true,
            }],
            ContextMenuTarget::Background,
        ));
        let mut cx = StubCx::default();

        assert!(!super::try_activate_searcher_row(&mut canvas, &mut cx, 0));

        assert!(cx.calls.is_empty());
        assert!(canvas.interaction.searcher.is_some());
    }

    #[test]
    fn searcher_row_activation_delegates_candidate_item_to_context_action() {
        let target = ContextMenuTarget::BackgroundInsertNodePicker {
            at: CanvasPoint { x: 12.0, y: 24.0 },
        };
        let mut canvas = test_canvas();
        canvas.interaction.searcher = Some(searcher(
            vec![SearcherRow {
                kind: SearcherRowKind::Candidate { candidate_ix: 1 },
                label: Arc::from("Beta"),
                enabled: true,
            }],
            target.clone(),
        ));
        let mut cx = StubCx::default();

        assert!(super::try_activate_searcher_row(&mut canvas, &mut cx, 0));

        assert!(canvas.interaction.searcher.is_none());
        assert_eq!(cx.calls.len(), 1);
        let call = &cx.calls[0];
        assert!(matches!(
            call.item.action,
            NodeGraphContextMenuAction::InsertNodeCandidate(1)
        ));
        assert_eq!(call.item.label.as_ref(), "Beta");
        assert!(matches!(
            call.target,
            ContextMenuTarget::BackgroundInsertNodePicker { at }
                if at == CanvasPoint { x: 12.0, y: 24.0 }
        ));
        assert_eq!(call.invoked_at, Point::new(Px(30.0), Px(40.0)));
        assert_eq!(call.candidates, 2);
    }
}
