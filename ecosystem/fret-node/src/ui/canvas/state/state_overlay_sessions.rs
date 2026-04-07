use std::sync::Arc;

use fret_core::{ClipboardToken, Point};
use fret_runtime::TimerToken;

use super::state_overlay_policy::{ContextMenuTarget, SearcherRowsMode};
use crate::core::{CanvasPoint, NodeKindKey, PortId};
use crate::rules::DiagnosticSeverity;
use crate::ui::canvas::searcher::SearcherRow;
use crate::ui::presenter::{InsertNodeCandidate, NodeGraphContextMenuItem};

#[derive(Debug, Clone)]
pub(crate) struct SearcherState {
    pub(crate) origin: Point,
    pub(crate) invoked_at: Point,
    pub(crate) target: ContextMenuTarget,
    pub(crate) rows_mode: SearcherRowsMode,
    pub(crate) query: String,
    pub(crate) candidates: Vec<InsertNodeCandidate>,
    pub(crate) recent_kinds: Vec<NodeKindKey>,
    pub(crate) rows: Vec<SearcherRow>,
    pub(crate) hovered_row: Option<usize>,
    pub(crate) active_row: usize,
    pub(crate) scroll: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct ContextMenuState {
    pub(crate) origin: Point,
    pub(crate) invoked_at: Point,
    pub(crate) target: ContextMenuTarget,
    pub(crate) items: Vec<NodeGraphContextMenuItem>,
    pub(crate) candidates: Vec<InsertNodeCandidate>,
    pub(crate) hovered_item: Option<usize>,
    pub(crate) active_item: usize,
    pub(crate) typeahead: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ToastState {
    pub(crate) timer: TimerToken,
    pub(crate) severity: DiagnosticSeverity,
    pub(crate) message: Arc<str>,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingPaste {
    pub(crate) token: ClipboardToken,
    pub(crate) at: CanvasPoint,
}

#[derive(Debug, Clone)]
pub(crate) struct LastConversionContext {
    pub(crate) from: PortId,
    pub(crate) to: PortId,
    pub(crate) at: CanvasPoint,
    pub(crate) candidates: Vec<InsertNodeCandidate>,
}
