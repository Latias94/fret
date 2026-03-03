# Workspace TabStrip (Fearless Refactor v1) — Open Questions

## Convergence with docking tab bars

- Do we converge on one shared "tab strip kernel" used by:
  - workspace tabs
  - docking tab bars
  - other header-tab surfaces
  Or do we keep kernels separate and share only math helpers?

Decision (2026-03-02):

- Share **headless primitives** in `ecosystem/fret-ui-headless` (and re-export from
  `ecosystem/fret-ui-kit` when convenient), while keeping adapter-specific policy and integration
  separate.
- Current shared surfaces:
  - surface classification: `tab_strip_surface::{TabStripSurface, classify_tab_strip_surface*}`
  - overflow membership: `tab_strip_overflow::compute_overflowed_tab_indices`
  - click arbitration: `tab_strip_controller::intent_for_click`
  - midpoint drop target resolution: `tab_strip_drop_target::compute_tab_strip_drop_target_midpoint`

Rationale:

- We want docking and workspace to share the same vocabulary and "what counts as header space /
  end-drop / overflow control".
- We still keep adapter-specific policy separate:
  - workspace owns pinned/preview/editor semantics
  - docking owns floating/tear-off/float-zone vocabulary and overlay discipline

Open question (follow-up):

- "Dragged tab exclusion" semantics are adapter-sensitive. If a UI keeps the dragged tab in the
  layout (no placeholder reflow), excluding it from drop candidates can produce confusing previews.
  Prefer adapter-owned behavior unless we also define a shared placeholder/reflow contract.
- Update (2026-03-03): docking now excludes the dragged panel index when resolving tab-bar
  insert targets (still adapter-owned, implemented via the `tab_is_dragged` closure passed into
  the shared headless kernel).

## Reference source of truth (Zed vs dockview vs gpui-component)

- Which reference should we treat as the behavioral source of truth for editor-grade tabs?

Recommendation (v1):

- Use **Zed** as the primary source of truth for workspace tab semantics (preview/pinned/activation
  rules), because it is an editor and its invariants match our target.
- Use **dockview** as the primary source of truth for DnD overlay discipline and droptarget
  vocabulary (e.g. ensuring only a single overlay is active at any time).
- Use **gpui-component** only as a UI composition reference (e.g. "last empty space" as an explicit
  drop target), not as the behavioral authority.

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

## Ensure-visible / scroll-to-active policy

- When activating a tab (click, keyboard, MRU fallback after close), should the strip always scroll
  so the active tab is visible?

Recommendation:

- Treat "ensure active is visible" as **workspace policy**, not mechanism.
- Default to ensuring visibility for non-pointer activations (keyboard commands, programmatic MRU
  fallback), and make pointer activation a no-op if the tab is already visible.
- Keep the mechanism layer limited to: "given tab rects + viewport, compute the scroll delta needed
  to reveal the active tab".

## Keyboard + a11y semantics (APG tablist)

- Do we want full APG-style tablist semantics (roving focus, tab/tabpanel roles, Home/End)?

Recommendation:

- Yes, but track it as a dedicated follow-up milestone (M3), because it touches:
  - focus model (roving index vs per-tab focus),
  - close button focusability (policy: close is reachable, but should not break tab roving),
  - semantics export (stable roles/selected state).
