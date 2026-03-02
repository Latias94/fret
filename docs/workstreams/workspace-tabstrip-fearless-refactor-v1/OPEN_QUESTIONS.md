# Workspace TabStrip (Fearless Refactor v1) — Open Questions

## Convergence with docking tab bars

- Do we converge on one shared "tab strip kernel" used by:
  - workspace tabs
  - docking tab bars
  - other header-tab surfaces
  Or do we keep kernels separate and share only math helpers?

Recommendation (v1):

- Share **math helpers** in `ecosystem/fret-ui-headless` and keep kernels separate until both sides
  stabilize their invariants.

## Explicit surfaces vs diagnostics-only

- For self-drawn strips, do we require explicit internal surfaces (header space, controls) even if
  we cannot assign fine-grained `test_id`s?

Recommendation:

- Model explicit surfaces in the kernel and add a small number of stable diagnostic anchors for
  scripts (end-drop, overflow button, overflow row).

## Insert index under overflow

- Is `insert_index` canonical order or visible index?

Recommendation:

- Canonical order always.

## Preview + pinned model ownership

- Should pinned/preview be represented in the mechanism layer?

Recommendation:

- Keep pinned/preview as workspace policy-layer semantics (do not extend `fret-ui` contracts).
