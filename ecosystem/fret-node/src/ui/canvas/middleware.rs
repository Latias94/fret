use fret_core::{AppWindowId, Event, Rect};
use fret_runtime::{CommandId, Model};
use fret_ui::UiHost;
use fret_ui::retained_bridge::{CommandCx, EventCx};

use crate::core::{CanvasPoint, Graph};
use crate::io::NodeGraphViewState;
use crate::ops::GraphTransaction;
use crate::rules::{Diagnostic, DiagnosticSeverity, DiagnosticTarget};
use crate::ui::style::NodeGraphStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeGraphCanvasEventOutcome {
    NotHandled,
    Handled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeGraphCanvasCommandOutcome {
    NotHandled,
    Handled,
}

#[derive(Debug, Clone)]
pub enum NodeGraphCanvasCommitOutcome {
    Continue,
    Reject { diagnostics: Vec<Diagnostic> },
}

#[derive(Debug, Clone, Copy)]
pub struct NodeGraphCanvasMiddlewareCx<'a> {
    pub graph: &'a Model<Graph>,
    pub view_state: &'a Model<NodeGraphViewState>,
    pub style: &'a NodeGraphStyle,
    pub bounds: Option<Rect>,
    pub pan: CanvasPoint,
    pub zoom: f32,
}

pub trait NodeGraphCanvasMiddleware: 'static {
    fn handle_event<H: UiHost>(
        &mut self,
        _cx: &mut EventCx<'_, H>,
        _ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        _event: &Event,
    ) -> NodeGraphCanvasEventOutcome {
        NodeGraphCanvasEventOutcome::NotHandled
    }

    fn handle_command<H: UiHost>(
        &mut self,
        _cx: &mut CommandCx<'_, H>,
        _ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        _command: &CommandId,
    ) -> NodeGraphCanvasCommandOutcome {
        NodeGraphCanvasCommandOutcome::NotHandled
    }

    fn before_commit<H: UiHost>(
        &mut self,
        _host: &mut H,
        _window: Option<AppWindowId>,
        _ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        _tx: &mut GraphTransaction,
    ) -> NodeGraphCanvasCommitOutcome {
        NodeGraphCanvasCommitOutcome::Continue
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopNodeGraphCanvasMiddleware;

impl NodeGraphCanvasMiddleware for NoopNodeGraphCanvasMiddleware {}

#[derive(Debug, Clone)]
pub struct NodeGraphCanvasMiddlewareChain<A, B> {
    pub first: A,
    pub second: B,
}

impl<A, B> NodeGraphCanvasMiddlewareChain<A, B> {
    pub fn new(first: A, second: B) -> Self {
        Self { first, second }
    }
}

impl<A, B> NodeGraphCanvasMiddleware for NodeGraphCanvasMiddlewareChain<A, B>
where
    A: NodeGraphCanvasMiddleware,
    B: NodeGraphCanvasMiddleware,
{
    fn handle_event<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        event: &Event,
    ) -> NodeGraphCanvasEventOutcome {
        match self.first.handle_event(cx, ctx, event) {
            NodeGraphCanvasEventOutcome::NotHandled => self.second.handle_event(cx, ctx, event),
            other => other,
        }
    }

    fn handle_command<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        command: &CommandId,
    ) -> NodeGraphCanvasCommandOutcome {
        match self.first.handle_command(cx, ctx, command) {
            NodeGraphCanvasCommandOutcome::NotHandled => {
                self.second.handle_command(cx, ctx, command)
            }
            other => other,
        }
    }

    fn before_commit<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        tx: &mut GraphTransaction,
    ) -> NodeGraphCanvasCommitOutcome {
        match self.first.before_commit(host, window, ctx, tx) {
            NodeGraphCanvasCommitOutcome::Continue => {
                self.second.before_commit(host, window, ctx, tx)
            }
            other => other,
        }
    }
}

/// Rejects transactions that introduce non-finite geometry (`NaN`, `Inf`) into the graph document.
#[derive(Debug, Default, Clone, Copy)]
pub struct RejectNonFiniteTx;

impl NodeGraphCanvasMiddleware for RejectNonFiniteTx {
    fn before_commit<H: UiHost>(
        &mut self,
        _host: &mut H,
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
                severity: DiagnosticSeverity::Error,
                target: DiagnosticTarget::Graph,
                message,
                fixes: Vec::new(),
            }],
        }
    }
}

/// Rejects transactions that introduce invalid node sizes (`<= 0`) into the graph document.
#[derive(Debug, Default, Clone, Copy)]
pub struct RejectInvalidSizeTx;

impl NodeGraphCanvasMiddleware for RejectInvalidSizeTx {
    fn before_commit<H: UiHost>(
        &mut self,
        _host: &mut H,
        _window: Option<AppWindowId>,
        _ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        tx: &mut GraphTransaction,
    ) -> NodeGraphCanvasCommitOutcome {
        let Some((key, message)) = find_invalid_size_in_tx(tx) else {
            return NodeGraphCanvasCommitOutcome::Continue;
        };

        NodeGraphCanvasCommitOutcome::Reject {
            diagnostics: vec![Diagnostic {
                key,
                severity: DiagnosticSeverity::Error,
                target: DiagnosticTarget::Graph,
                message,
                fixes: Vec::new(),
            }],
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

    match op {
        GraphOp::AddNode { node, .. } => node
            .size
            .and_then(|s| (!size_is_finite(s)).then_some("AddNode.node.size"))
            .or_else(|| (!point_is_finite(node.pos)).then_some("AddNode.node.pos")),
        GraphOp::AddGroup { group, .. } => (!rect_is_finite(group.rect)).then_some("AddGroup.rect"),
        GraphOp::AddStickyNote { note, .. } => {
            (!rect_is_finite(note.rect)).then_some("AddStickyNote.rect")
        }

        GraphOp::SetNodePos { to, .. } => (!point_is_finite(*to)).then_some("SetNodePos.to"),
        GraphOp::SetGroupRect { to, .. } => (!rect_is_finite(*to)).then_some("SetGroupRect.to"),
        GraphOp::SetNodeSize { to, .. } => {
            to.and_then(|s| (!size_is_finite(s)).then_some("SetNodeSize.to"))
        }

        GraphOp::SetEdgeEndpoints { from, to, .. } => (!endpoints_is_finite(*from))
            .then_some("SetEdgeEndpoints.from")
            .or_else(|| (!endpoints_is_finite(*to)).then_some("SetEdgeEndpoints.to")),

        GraphOp::RemoveNode { .. }
        | GraphOp::SetNodeParent { .. }
        | GraphOp::SetNodeCollapsed { .. }
        | GraphOp::SetNodePorts { .. }
        | GraphOp::SetNodeData { .. }
        | GraphOp::SetGroupTitle { .. }
        | GraphOp::AddPort { .. }
        | GraphOp::RemovePort { .. }
        | GraphOp::AddEdge { .. }
        | GraphOp::RemoveEdge { .. }
        | GraphOp::SetEdgeKind { .. }
        | GraphOp::AddSymbol { .. }
        | GraphOp::RemoveSymbol { .. }
        | GraphOp::SetSymbolMeta { .. }
        | GraphOp::RemoveGroup { .. }
        | GraphOp::RemoveStickyNote { .. } => None,
    }
}

fn find_invalid_size_in_tx(tx: &GraphTransaction) -> Option<(String, String)> {
    for (ix, op) in tx.ops.iter().enumerate() {
        if let Some(field) = op_invalid_size_field(op) {
            return Some((
                "tx.invalid_size".to_string(),
                format!("transaction contains invalid size at op[{ix}] ({field})"),
            ));
        }
    }
    None
}

fn op_invalid_size_field(op: &crate::ops::GraphOp) -> Option<&'static str> {
    use crate::core::CanvasSize;
    use crate::ops::GraphOp;

    fn size_is_valid(s: CanvasSize) -> bool {
        s.width.is_finite() && s.height.is_finite() && s.width > 0.0 && s.height > 0.0
    }

    match op {
        GraphOp::AddNode { node, .. } => node
            .size
            .and_then(|s| (!size_is_valid(s)).then_some("AddNode.node.size")),
        GraphOp::SetNodeSize { to, .. } => {
            to.and_then(|s| (!size_is_valid(s)).then_some("SetNodeSize.to"))
        }
        _ => None,
    }
}
