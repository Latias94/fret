use fret_core::{AppWindowId, Rect};

use crate::core::{CanvasPoint, Graph};
use crate::io::NodeGraphViewState;
use crate::ops::GraphTransaction;
use crate::rules::Diagnostic;
use crate::ui::style::NodeGraphStyle;

#[derive(Debug, Clone)]
pub enum NodeGraphCanvasCommitOutcome {
    Continue,
    Reject { diagnostics: Vec<Diagnostic> },
}

#[derive(Debug, Clone, Copy)]
pub struct NodeGraphCanvasMiddlewareCx<'a> {
    pub graph: &'a Graph,
    pub view_state: &'a NodeGraphViewState,
    pub style: &'a NodeGraphStyle,
    pub bounds: Option<Rect>,
    pub pan: CanvasPoint,
    pub zoom: f32,
}

pub trait NodeGraphCanvasTxMiddleware: 'static {
    fn before_commit(
        &mut self,
        _window: Option<AppWindowId>,
        _ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        _tx: &mut GraphTransaction,
    ) -> NodeGraphCanvasCommitOutcome {
        NodeGraphCanvasCommitOutcome::Continue
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopNodeGraphCanvasTxMiddleware;

impl NodeGraphCanvasTxMiddleware for NoopNodeGraphCanvasTxMiddleware {}

#[derive(Debug, Default, Clone, Copy)]
pub struct RejectNonFiniteTx;

impl NodeGraphCanvasTxMiddleware for RejectNonFiniteTx {
    fn before_commit(
        &mut self,
        _window: Option<AppWindowId>,
        _ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        tx: &mut GraphTransaction,
    ) -> NodeGraphCanvasCommitOutcome {
        let Some((key, message)) = find_non_finite_in_tx(tx) else {
            return NodeGraphCanvasCommitOutcome::Continue;
        };

        NodeGraphCanvasCommitOutcome::Reject {
            diagnostics: vec![Diagnostic {
                key,
                severity: crate::rules::DiagnosticSeverity::Error,
                target: crate::rules::DiagnosticTarget::Graph,
                message,
                fixes: Vec::new(),
            }],
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeGraphCanvasTxMiddlewareChain<A, B> {
    pub first: A,
    pub second: B,
}

impl<A, B> NodeGraphCanvasTxMiddlewareChain<A, B> {
    pub fn new(first: A, second: B) -> Self {
        Self { first, second }
    }
}

impl<A, B> NodeGraphCanvasTxMiddleware for NodeGraphCanvasTxMiddlewareChain<A, B>
where
    A: NodeGraphCanvasTxMiddleware,
    B: NodeGraphCanvasTxMiddleware,
{
    fn before_commit(
        &mut self,
        window: Option<AppWindowId>,
        ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        tx: &mut GraphTransaction,
    ) -> NodeGraphCanvasCommitOutcome {
        match self.first.before_commit(window, ctx, tx) {
            NodeGraphCanvasCommitOutcome::Continue => self.second.before_commit(window, ctx, tx),
            other => other,
        }
    }
}

fn find_non_finite_in_tx(tx: &GraphTransaction) -> Option<(String, String)> {
    for (ix, op) in tx.ops.iter().enumerate() {
        if let Some(field) = op_non_finite_field(op) {
            return Some((
                "tx.non_finite".to_string(),
                format!("transaction contains non-finite geometry at op[{ix}] ({field})"),
            ));
        }
    }
    None
}

fn op_non_finite_field(op: &crate::ops::GraphOp) -> Option<&'static str> {
    use crate::core::{CanvasRect, CanvasSize};
    use crate::ops::{EdgeEndpoints, GraphOp};

    fn point_is_finite(p: CanvasPoint) -> bool {
        p.x.is_finite() && p.y.is_finite()
    }

    fn size_is_finite(s: CanvasSize) -> bool {
        s.width.is_finite() && s.height.is_finite()
    }

    fn rect_is_finite(r: CanvasRect) -> bool {
        point_is_finite(r.origin) && size_is_finite(r.size)
    }

    fn endpoints_is_finite(_e: EdgeEndpoints) -> bool {
        true
    }

    match *op {
        GraphOp::SetNodePos { from, to, .. } => (!point_is_finite(from))
            .then_some("SetNodePos.from")
            .or_else(|| (!point_is_finite(to)).then_some("SetNodePos.to")),
        GraphOp::SetGroupRect { from, to, .. } => (!rect_is_finite(from))
            .then_some("SetGroupRect.from")
            .or_else(|| (!rect_is_finite(to)).then_some("SetGroupRect.to")),
        GraphOp::SetNodeSize { from, to, .. } => from
            .and_then(|s| (!size_is_finite(s)).then_some("SetNodeSize.from"))
            .or_else(|| to.and_then(|s| (!size_is_finite(s)).then_some("SetNodeSize.to"))),

        GraphOp::SetEdgeEndpoints { from, to, .. } => (!endpoints_is_finite(from))
            .then_some("SetEdgeEndpoints.from")
            .or_else(|| (!endpoints_is_finite(to)).then_some("SetEdgeEndpoints.to")),

        GraphOp::AddNode { .. }
        | GraphOp::RemoveNode { .. }
        | GraphOp::SetNodeParent { .. }
        | GraphOp::SetNodeCollapsed { .. }
        | GraphOp::SetNodePorts { .. }
        | GraphOp::SetNodeData { .. }
        | GraphOp::AddPort { .. }
        | GraphOp::RemovePort { .. }
        | GraphOp::AddEdge { .. }
        | GraphOp::RemoveEdge { .. }
        | GraphOp::SetEdgeKind { .. }
        | GraphOp::AddSymbol { .. }
        | GraphOp::RemoveSymbol { .. }
        | GraphOp::SetSymbolMeta { .. }
        | GraphOp::AddGroup { .. }
        | GraphOp::RemoveGroup { .. }
        | GraphOp::AddStickyNote { .. }
        | GraphOp::RemoveStickyNote { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::{CanvasPoint, CanvasRect, CanvasSize, GroupId, NodeId};
    use crate::io::NodeGraphViewState;
    use crate::ops::{GraphOp, GraphTransaction};
    use crate::ui::style::NodeGraphStyle;

    #[test]
    fn reject_non_finite_tx_accepts_finite_geometry() {
        let graph = Graph::default();
        let view_state = NodeGraphViewState::default();
        let style = NodeGraphStyle::default();
        let ctx = NodeGraphCanvasMiddlewareCx {
            graph: &graph,
            view_state: &view_state,
            style: &style,
            bounds: None,
            pan: CanvasPoint::default(),
            zoom: 1.0,
        };

        let mut tx = GraphTransaction {
            label: None,
            ops: vec![GraphOp::SetNodePos {
                id: NodeId::new(),
                from: CanvasPoint { x: 0.0, y: 0.0 },
                to: CanvasPoint { x: 10.0, y: 20.0 },
            }],
        };

        let mut gate = RejectNonFiniteTx;
        assert!(matches!(
            gate.before_commit(None, &ctx, &mut tx),
            NodeGraphCanvasCommitOutcome::Continue
        ));
    }

    #[test]
    fn reject_non_finite_tx_rejects_nan_in_node_pos() {
        let graph = Graph::default();
        let view_state = NodeGraphViewState::default();
        let style = NodeGraphStyle::default();
        let ctx = NodeGraphCanvasMiddlewareCx {
            graph: &graph,
            view_state: &view_state,
            style: &style,
            bounds: None,
            pan: CanvasPoint::default(),
            zoom: 1.0,
        };

        let mut tx = GraphTransaction {
            label: None,
            ops: vec![GraphOp::SetNodePos {
                id: NodeId::new(),
                from: CanvasPoint { x: 0.0, y: 0.0 },
                to: CanvasPoint {
                    x: f32::NAN,
                    y: 20.0,
                },
            }],
        };

        let mut gate = RejectNonFiniteTx;
        assert!(matches!(
            gate.before_commit(None, &ctx, &mut tx),
            NodeGraphCanvasCommitOutcome::Reject { .. }
        ));
    }

    #[test]
    fn reject_non_finite_tx_rejects_inf_in_group_rect() {
        let graph = Graph::default();
        let view_state = NodeGraphViewState::default();
        let style = NodeGraphStyle::default();
        let ctx = NodeGraphCanvasMiddlewareCx {
            graph: &graph,
            view_state: &view_state,
            style: &style,
            bounds: None,
            pan: CanvasPoint::default(),
            zoom: 1.0,
        };

        let group_id = GroupId::new();
        let mut tx = GraphTransaction {
            label: None,
            ops: vec![GraphOp::SetGroupRect {
                id: group_id,
                from: CanvasRect {
                    origin: CanvasPoint { x: 0.0, y: 0.0 },
                    size: CanvasSize {
                        width: 10.0,
                        height: 20.0,
                    },
                },
                to: CanvasRect {
                    origin: CanvasPoint {
                        x: f32::INFINITY,
                        y: 0.0,
                    },
                    size: CanvasSize {
                        width: 10.0,
                        height: 20.0,
                    },
                },
            }],
        };

        let mut gate = RejectNonFiniteTx;
        assert!(matches!(
            gate.before_commit(None, &ctx, &mut tx),
            NodeGraphCanvasCommitOutcome::Reject { .. }
        ));
    }
}
