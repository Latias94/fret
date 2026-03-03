use std::sync::Arc;

/// Why a close action was requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceCloseReason {
    CloseActive,
    CloseById,
    CloseOthers,
    CloseLeftOfActive,
    CloseRightOfActive,
}

/// A policy-layer request emitted when a close command would close one or more dirty tabs.
///
/// This lives in `fret-workspace` (ecosystem) on purpose: it is editor policy and must not leak
/// into `fret-ui` runtime contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDirtyCloseRequest {
    pub reason: WorkspaceCloseReason,
    /// Tabs the command intends to close, in tab-strip order.
    pub target_tabs_in_order: Vec<Arc<str>>,
    /// Dirty tabs among the target set, in tab-strip order.
    pub dirty_tabs_in_order: Vec<Arc<str>>,
    /// The active tab id at the time of the request (if any).
    pub active_tab_id: Option<Arc<str>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceDirtyCloseDecision {
    Allow,
    Block,
}

/// Policy hook for dirty close confirmation.
///
/// Implementations can show a prompt (save/discard/cancel) and then re-dispatch an appropriate
/// command based on the user choice.
pub trait WorkspaceDirtyClosePolicy {
    fn decide_dirty_close(
        &mut self,
        request: &WorkspaceDirtyCloseRequest,
    ) -> WorkspaceDirtyCloseDecision;
}
