pub(super) use std::sync::Arc;

pub(super) use fret_core::{Point, Px, Rect};
pub(super) use fret_ui::UiHost;

pub(super) use crate::core::{EdgeId, PortId};
pub(super) use crate::ops::{GraphOp, GraphTransaction, apply_transaction};
pub(super) use crate::rules::{ConnectDecision, DiagnosticSeverity, EdgeEndpoint};
pub(super) use crate::runtime::callbacks::ConnectEndOutcome;
pub(super) use crate::ui::presenter::InsertNodeCandidate;

pub(super) use crate::ui::canvas::conversion;
pub(super) use crate::ui::canvas::searcher::SEARCHER_MAX_VISIBLE_ROWS;
pub(super) use crate::ui::canvas::state::{
    ContextMenuTarget, LastConversionContext, SearcherState, ViewSnapshot, WireDrag, WireDragKind,
};

pub(super) use super::{
    HitTestCtx, HitTestScratch, NodeGraphCanvasMiddleware, NodeGraphCanvasWith, WireCommitCx,
};

pub(super) struct CommitEmit {
    pub target: Option<PortId>,
    pub outcome: ConnectEndOutcome,
}
