use fret_core::{AppWindowId, Event, Rect};
use fret_runtime::{CommandId, Model};
use fret_ui::UiHost;
use fret_ui::retained_bridge::{CommandCx, EventCx};

use crate::core::{CanvasPoint, Graph};
use crate::io::NodeGraphViewState;
use crate::ops::GraphTransaction;
use crate::rules::Diagnostic;
use crate::runtime::store::NodeGraphStore;
use crate::ui::edit_queue::NodeGraphEditQueue;
use crate::ui::overlays::NodeGraphOverlayState;
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
    pub store: Option<&'a Model<NodeGraphStore>>,
    pub edit_queue: Option<&'a Model<NodeGraphEditQueue>>,
    pub overlays: Option<&'a Model<NodeGraphOverlayState>>,
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
