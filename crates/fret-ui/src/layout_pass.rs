#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutPassKind {
    /// A layout pass performed only to probe intrinsic/desired sizes or intermediate geometry.
    ///
    /// Probe passes must not consume one-shot state (e.g. deferred scroll requests), must not
    /// register viewport roots, and should avoid expensive precomputation where possible.
    Probe,
    /// A layout pass performed under the final viewport/root constraints for this frame.
    Final,
}
