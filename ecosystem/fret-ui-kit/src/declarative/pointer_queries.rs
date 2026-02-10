use fret_core::PointerType;
use fret_ui::{ElementContext, Invalidation, UiHost};

/// Returns the most recent committed primary pointer type for the window (ADR 1171).
#[track_caller]
pub fn primary_pointer_type<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
) -> PointerType {
    cx.environment_primary_pointer_type(invalidation)
}

/// Returns whether the primary pointer is expected to support hover-driven affordances.
///
/// Notes:
/// - This is a **policy helper**; it is intentionally small and may evolve as runners provide more
///   precise capability data.
/// - When the pointer type is `Unknown`, `default_when_unknown` is returned.
#[track_caller]
pub fn primary_pointer_can_hover<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
    default_when_unknown: bool,
) -> bool {
    cx.environment_primary_pointer_can_hover(invalidation, default_when_unknown)
}

/// Returns whether the primary pointer is expected to be coarse (touch-first).
///
/// When the pointer type is `Unknown`, `default_when_unknown` is returned.
#[track_caller]
pub fn primary_pointer_is_coarse<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
    default_when_unknown: bool,
) -> bool {
    cx.environment_primary_pointer_is_coarse(invalidation, default_when_unknown)
}
