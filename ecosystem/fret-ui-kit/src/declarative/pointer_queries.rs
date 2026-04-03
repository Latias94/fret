use fret_core::PointerType;
use fret_ui::{ElementContextAccess, Invalidation, UiHost};

/// Returns the most recent committed primary pointer type for the window (ADR 0232).
#[track_caller]
pub fn primary_pointer_type<'a, H: UiHost + 'a, Cx>(
    cx: &mut Cx,
    invalidation: Invalidation,
) -> PointerType
where
    Cx: ElementContextAccess<'a, H>,
{
    cx.elements().environment_primary_pointer_type(invalidation)
}

/// Returns whether the primary pointer is expected to support hover-driven affordances.
///
/// Notes:
/// - This is a **policy helper**; it is intentionally small and may evolve as runners provide more
///   precise capability data.
/// - When the pointer type is `Unknown`, `default_when_unknown` is returned.
#[track_caller]
pub fn primary_pointer_can_hover<'a, H: UiHost + 'a, Cx>(
    cx: &mut Cx,
    invalidation: Invalidation,
    default_when_unknown: bool,
) -> bool
where
    Cx: ElementContextAccess<'a, H>,
{
    cx.elements()
        .environment_primary_pointer_can_hover(invalidation, default_when_unknown)
}

/// Returns whether the primary pointer is expected to be coarse (touch-first).
///
/// When the pointer type is `Unknown`, `default_when_unknown` is returned.
#[track_caller]
pub fn primary_pointer_is_coarse<'a, H: UiHost + 'a, Cx>(
    cx: &mut Cx,
    invalidation: Invalidation,
    default_when_unknown: bool,
) -> bool
where
    Cx: ElementContextAccess<'a, H>,
{
    cx.elements()
        .environment_primary_pointer_is_coarse(invalidation, default_when_unknown)
}
