/// A minimal drag lifecycle that maps well to undo/redo boundaries.
///
/// This intentionally mirrors the "begin/update/commit/cancel" vocabulary used by editor tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragPhase {
    /// Start a drag operation (capture, snapshot).
    Begin,
    /// Update a drag operation (preview).
    Update,
    /// Commit a drag operation (write model, record undo).
    Commit,
    /// Cancel a drag operation (revert preview, release capture).
    Cancel,
}
