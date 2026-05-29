use std::sync::Arc;

use fret_core::Point;

use crate::core::{CanvasPoint, Graph};
use crate::ops::{GraphOp, GraphTransaction};
use crate::runtime::events::ConnectDragKind;
use crate::ui::{InsertNodeCandidate, NodeGraphPresenter};

use super::NodeGraphDeclarativeInsertNodePickerRequest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeGraphDeclarativeInsertNodePickerOpenOutcome {
    Opened { candidate_count: usize },
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeGraphDeclarativeInsertNodePickerPlanError {
    NoActiveSession,
    CandidateOutOfRange { index: usize, len: usize },
    CandidateDisabled { index: usize },
    InvalidCanvasPosition,
    Rejected(Arc<str>),
    EmptyTransaction,
}

#[derive(Debug, Clone)]
pub struct NodeGraphDeclarativeInsertNodePickerSession {
    pub request: NodeGraphDeclarativeInsertNodePickerRequest,
    pub candidates: Vec<InsertNodeCandidate>,
}

#[derive(Debug, Default, Clone)]
pub struct NodeGraphDeclarativeInsertNodePickerState {
    active: Option<NodeGraphDeclarativeInsertNodePickerSession>,
}

pub trait NodeGraphDeclarativeInsertNodePickerCandidateProvider {
    fn list_insert_node_candidates(
        &mut self,
        graph: &Graph,
        request: &NodeGraphDeclarativeInsertNodePickerRequest,
    ) -> Vec<InsertNodeCandidate>;

    fn plan_insert_node_candidate(
        &mut self,
        graph: &Graph,
        request: &NodeGraphDeclarativeInsertNodePickerRequest,
        candidate: &InsertNodeCandidate,
    ) -> Result<Vec<GraphOp>, Arc<str>>;
}

impl<T> NodeGraphDeclarativeInsertNodePickerCandidateProvider for T
where
    T: NodeGraphPresenter + ?Sized,
{
    fn list_insert_node_candidates(
        &mut self,
        graph: &Graph,
        request: &NodeGraphDeclarativeInsertNodePickerRequest,
    ) -> Vec<InsertNodeCandidate> {
        match &request.connect_end.kind {
            ConnectDragKind::New { from, .. } => {
                self.list_insertable_nodes_for_connection(graph, *from)
            }
            ConnectDragKind::Reconnect { fixed, .. } => {
                self.list_insertable_nodes_for_connection(graph, *fixed)
            }
            ConnectDragKind::ReconnectMany { .. } => self.list_insertable_nodes(graph),
        }
    }

    fn plan_insert_node_candidate(
        &mut self,
        graph: &Graph,
        request: &NodeGraphDeclarativeInsertNodePickerRequest,
        candidate: &InsertNodeCandidate,
    ) -> Result<Vec<GraphOp>, Arc<str>> {
        let at = request_canvas_point(request)
            .map_err(|_| Arc::<str>::from("insert-node picker request has invalid canvas point"))?;
        self.plan_create_node(graph, candidate, at)
    }
}

impl NodeGraphDeclarativeInsertNodePickerState {
    pub fn active(&self) -> Option<&NodeGraphDeclarativeInsertNodePickerSession> {
        self.active.as_ref()
    }

    pub fn is_open(&self) -> bool {
        self.active.is_some()
    }

    pub fn open_with_candidates(
        &mut self,
        request: NodeGraphDeclarativeInsertNodePickerRequest,
        candidates: Vec<InsertNodeCandidate>,
    ) -> NodeGraphDeclarativeInsertNodePickerOpenOutcome {
        if candidates.is_empty() {
            self.active = None;
            return NodeGraphDeclarativeInsertNodePickerOpenOutcome::Empty;
        }

        let candidate_count = candidates.len();
        self.active = Some(NodeGraphDeclarativeInsertNodePickerSession {
            request,
            candidates,
        });
        NodeGraphDeclarativeInsertNodePickerOpenOutcome::Opened { candidate_count }
    }

    pub fn open_from_provider(
        &mut self,
        graph: &Graph,
        request: NodeGraphDeclarativeInsertNodePickerRequest,
        provider: &mut dyn NodeGraphDeclarativeInsertNodePickerCandidateProvider,
    ) -> NodeGraphDeclarativeInsertNodePickerOpenOutcome {
        let candidates = provider.list_insert_node_candidates(graph, &request);
        self.open_with_candidates(request, candidates)
    }

    pub fn cancel(&mut self) -> bool {
        self.active.take().is_some()
    }

    pub fn close_after_commit(&mut self) -> bool {
        self.cancel()
    }

    pub fn plan_candidate_with_provider(
        &self,
        graph: &Graph,
        provider: &mut dyn NodeGraphDeclarativeInsertNodePickerCandidateProvider,
        index: usize,
    ) -> Result<GraphTransaction, NodeGraphDeclarativeInsertNodePickerPlanError> {
        let Some(session) = self.active.as_ref() else {
            return Err(NodeGraphDeclarativeInsertNodePickerPlanError::NoActiveSession);
        };
        let Some(candidate) = session.candidates.get(index) else {
            return Err(
                NodeGraphDeclarativeInsertNodePickerPlanError::CandidateOutOfRange {
                    index,
                    len: session.candidates.len(),
                },
            );
        };
        if !candidate.enabled {
            return Err(NodeGraphDeclarativeInsertNodePickerPlanError::CandidateDisabled { index });
        }

        let ops = provider
            .plan_insert_node_candidate(graph, &session.request, candidate)
            .map_err(NodeGraphDeclarativeInsertNodePickerPlanError::Rejected)?;
        if ops.is_empty() {
            return Err(NodeGraphDeclarativeInsertNodePickerPlanError::EmptyTransaction);
        }
        Ok(GraphTransaction { label: None, ops }.with_label("Insert Node"))
    }
}

fn request_canvas_point(
    request: &NodeGraphDeclarativeInsertNodePickerRequest,
) -> Result<CanvasPoint, NodeGraphDeclarativeInsertNodePickerPlanError> {
    canvas_point_from_core_point(request.canvas_position)
        .ok_or(NodeGraphDeclarativeInsertNodePickerPlanError::InvalidCanvasPosition)
}

fn canvas_point_from_core_point(point: Point) -> Option<CanvasPoint> {
    let x = point.x.0;
    let y = point.y.0;
    (x.is_finite() && y.is_finite()).then_some(CanvasPoint { x, y })
}
