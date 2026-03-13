use fret_core::{AppWindowId, Event, Rect};
use fret_runtime::{CommandId, Model};
use fret_ui::UiHost;
use fret_ui::retained_bridge::{CommandCx, EventCx};

use crate::core::{CanvasPoint, Graph};
use crate::io::NodeGraphViewState;
use crate::ops::GraphTransaction;
use crate::rules::{Diagnostic, DiagnosticSeverity, DiagnosticTarget};
use crate::ui::style::NodeGraphStyle;

mod middleware_chain;
mod middleware_validation;
pub use middleware_chain::NodeGraphCanvasMiddlewareChain;
pub use middleware_validation::{RejectInvalidSizeTx, RejectNonFiniteTx};

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
