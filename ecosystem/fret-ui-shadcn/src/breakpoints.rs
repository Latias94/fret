//! Shared breakpoint constants used by shadcn-aligned recipes.
//!
//! Important: not all breakpoints are "responsive layout" breakpoints.
//! - Container-query-driven breakpoints should use `fret-ui-kit` container queries (ADR 1170).
//! - Device-level breakpoints (e.g. mobile vs desktop) remain viewport-driven by design.

/// Viewport-driven device breakpoints (mobile vs desktop).
pub(crate) mod device {
    use fret_core::Px;

    /// Tailwind-compatible `md` breakpoint (viewport width).
    pub(crate) const MD: Px = Px(768.0);
}
