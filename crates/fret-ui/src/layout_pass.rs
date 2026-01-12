#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutPassKind {
    /// A layout pass performed only to probe intrinsic/desired sizes or intermediate geometry.
    ///
    /// Probe passes must not be treated as a final layout solve. In particular, they:
    ///
    /// - must not consume one-shot state (e.g. deferred scroll requests),
    /// - must not register viewport roots,
    /// - must not update window-scoped bounds caches used by overlay placement,
    /// - must not clear layout invalidation flags (so a subsequent `Final` pass still runs).
    Probe,
    /// A layout pass performed under the final viewport/root constraints for this frame.
    Final,
}
