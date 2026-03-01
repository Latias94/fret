# Docking TabBar Fearless Refactor v1 (Open Questions)

## Sharing with workspace tab strip

- Do we converge on a single shared “tab strip kernel” (geometry + insert-index resolution) used by:
  - workspace tabs
  - docking tab bars
  - other “header tabs” surfaces
  Or do we keep kernels separate but share only small math helpers in `ecosystem/fret-dnd`?

Recommendation (for v1):

- Share **math helpers** (`fret-dnd`) and keep **kernels separate** until both sides stabilize their
  zone vocabularies and invariants.

## Explicit surfaces vs diagnostics-only

- For self-drawn tab bars: should we introduce explicit internal “drop surfaces” even if we cannot
  attach `test_id` at fine granularity?

Recommendation:

- Keep diagnostics predicates as the gate, but still model explicit surfaces in the kernel so future
  rendering refactors do not change hit-test semantics accidentally.

## Insert index semantics under overflow

- When tabs overflow, some tabs are hidden in a dropdown.
  - Is `insert_index` expressed in canonical list order (recommended), or “visible index”?

Recommendation:

- Always canonical list order. UI should map to it via a stable index mapping layer.

## Pinned and preview regions

- If we adopt pinned/preview:
  - do we represent them as regions (pinned | normal | preview) or as per-tab flags with sorting?
  - can preview be “single slot” (Zed-like) or multiple preview tabs?

Recommendation:

- Start with region model (pinned | normal | preview-slot) in workspace policy, not in docking core.

