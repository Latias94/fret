//! Direction primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/direction/src/direction.tsx`
//!
//! Radix `direction` is a tiny helper that resolves a local `dir` override against an optional
//! inherited/global direction, defaulting to LTR.
//!
//! Fret does not provide a built-in "direction context" mechanism yet. Callers are expected to
//! thread an inherited direction through their component surfaces (or derive it from app/theme
//! configuration) and use `use_direction(...)` to apply the Radix resolution rule.

pub use fret_ui::overlay_placement::LayoutDirection;

/// Resolve direction using the Radix rule: `local || inherited || Ltr`.
pub fn use_direction(
    local: Option<LayoutDirection>,
    inherited: Option<LayoutDirection>,
) -> LayoutDirection {
    local.or(inherited).unwrap_or_default()
}
