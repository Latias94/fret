use fret_core::{AppWindowId, Rect};
use fret_runtime::Model;
use fret_ui::UiHost;

use crate::core::{CanvasPoint, Graph};
use crate::io::NodeGraphViewState;
use crate::ops::GraphTransaction;
use crate::rules::{Diagnostic, DiagnosticSeverity, DiagnosticTarget};
use crate::ui::style::NodeGraphStyle;

mod middleware_chain;
mod middleware_validation;

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
