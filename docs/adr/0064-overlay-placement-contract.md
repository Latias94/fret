---
title: "ADR 0064: Overlay Placement Contract (Anchored Panels)"
---

# ADR 0064: Overlay Placement Contract (Anchored Panels)

## Status

Accepted (MVP 62).

## Context

Fret needs a deterministic, testable contract for placing anchored overlay panels (popover, tooltip,
context menu, hover card-like surfaces). In DOM ecosystems, this is commonly handled by
Floating UI–style middleware (flip/shift/size/arrow) and/or Radix UI primitives.

Fret is **not** a DOM/CSS runtime, so we cannot directly reuse those implementations, but we still
want:

- Stable behavior outcomes aligned with Radix/Floating expectations.
- A small pure function that is easy to regression-test.
- A single shared solver to avoid per-component “placement drift”.

## Decision

We define an **anchored panel placement** contract implemented by a pure function in
`crates/fret-ui/src/overlay_placement.rs`.

### Inputs

- `outer: Rect`: the placement boundary in the same coordinate space as the anchor.
  - For window-scoped overlays, this is the window bounds (or an inset version).
  - For declarative primitives that are not portaled, this is the render root bounds.
- `anchor: Rect`: the anchor bounds.
- `content: Size`: desired panel size (measured independently).
- `side_offset: Px`: gap between anchor and panel on the main axis.
- `preferred_side: Side`: the preferred placement side.
- `align: Align`: alignment along the cross axis.

### Outputs

- `Rect`: final panel bounds.

### Coordinate space and docking/multi-window notes

This contract is intentionally independent from docking and multi-view layout; compatibility is
achieved by choosing correct inputs:

- `outer` and `anchor` must be expressed in the **same window-space coordinate system**.
- For window-scoped overlays (menus/popovers/tooltips) that must escape panel clipping, `outer`
  should be the window bounds and `anchor` should be obtained via a cross-frame geometry query
  (e.g. `elements::bounds_for_element`, or an equivalent retained-widget hook).
- For non-portaled anchored panels (rare, local-only), `outer` should be the render root bounds,
  and the caller must accept that clipping/scrolling in the subtree may constrain visibility.
- Overlays are **per-window**: do not attempt to place a panel using an anchor from a different
  OS window (tear-off docking). Recompute `outer`/`anchor` for the target window.

### Algorithm (deterministic subset)

This contract implements a small, deterministic subset inspired by Floating UI:

1) Compute the preferred placement rect from `(preferred_side, align)` and `side_offset`.
2) Compute the opposite-side rect (flip candidate).
3) If a candidate fits the main axis **without requiring main-axis clamping**, prefer it.
4) If neither fits, choose the candidate with **minimum main-axis overflow**, breaking ties by
   total overflow, then clamp into `outer`.

Notes:

- “Shift” is represented by clamping on the cross axis (and on the main axis if unavoidable).
- “Size” and “arrow” are not part of this contract yet; they will be added as follow-up extensions
  only if demanded by shadcn/Radix parity.
- For scrollable content, the runtime also provides a convenience helper that clamps the *panel
  rect size* to the available space on the chosen side (`anchored_panel_bounds_sized`). This is
  intentionally not full “size middleware”: it only provides a viewport rect; the component is
  responsible for internal scrolling behavior.
  - When neither the preferred side nor the flipped side can fit the desired size without
    main-axis clamping, the sized helper prefers the side with **more available main-axis space**
    ("best fit"), breaking ties with the same overflow heuristic as the unsized solver.

## Consequences

- Overlay widgets in `fret-components-ui` should use this solver rather than bespoke “below/above”
  positioning logic.
- The solver is intentionally conservative: predictable and testable first, then grow capability
  behind regression tests.

## References

- Reference stack: `docs/reference-stack-ui-behavior.md`
- Floating UI (algorithm inspiration): `repo-ref/floating-ui`
- Radix UI Primitives (behavior outcomes): <https://github.com/radix-ui/primitives> (pinned locally; see `docs/repo-ref.md`)
