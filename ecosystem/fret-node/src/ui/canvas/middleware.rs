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
