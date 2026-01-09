# ADR 0011: Overlays and Multi-Root UI Composition

Status: Accepted

## Context

Editor UIs require multiple overlay layers:

- menus and context menus,
- tooltips,
- drag-and-drop previews and drop-zone highlights,
- modal dialogs,
- global “command palette”-style surfaces.

A single-root tree that routes events only to one root is not sufficient. If overlays are bolted
on later, the input model, focus model, and rendering model will require invasive changes.

References:

- Dear ImGui docking + multi-viewport UX (conceptual target for editor workflows):
  - https://github.com/ocornut/imgui (Docking branch / viewports)
- Zed/GPUI (non-normative implementation reference for overlay-style behavior):
  - content masks and painting outside parent bounds: `repo-ref/zed/crates/gpui/src/window.rs`
    (`Window::with_content_mask`), `repo-ref/zed/crates/gpui/src/element.rs`
  - mouse occlusion semantics for overlays (block underlay hover/click, optionally allow scroll):
    `repo-ref/zed/crates/gpui/src/window.rs` (`HitboxBehavior::{BlockMouse, BlockMouseExceptScroll}`)

## Decision

### 1) Multi-root is a first-class concept

Each window’s UI is composed from **multiple roots** with an explicit z-order, e.g.:

1. Base UI root (dock layout, panels)
2. Overlay root (drag previews, drop hints, tooltips)
3. Popup root (menus, context menus)
4. Modal root (dialogs)

The exact root set can evolve, but the concept is part of the core UI contract.

### 2) Event routing considers root z-order

Hit-testing and pointer targeting evaluate roots from top-most to bottom-most.

Focus, capture, and modal blocking rules are defined in the UI runtime:

- Pointer capture remains authoritative.
- Modal roots can block events from reaching lower roots, except for explicitly allowed pass-through.

Implementation note (MVP): the UI runtime models modal blocking as a per-root flag
(`blocks_underlay_input`). When a blocking root is visible, hit-testing is restricted to that root
and any roots above it; if no widget is hit, the blocking root still receives the pointer event so
it can implement “click outside to close”.

#### Ownership vs z-order (the “logical parent” problem)

Overlays like menus, popovers, and tooltips often:

- render in a top overlay root (z-order),
- but are logically owned by a deep widget (e.g. a button in a panel).

The contract is:

- visual stacking is determined by roots/z-order,
- logical ownership is modeled explicitly by the widget/element that created the overlay (e.g. a menu state owned by the button/view model),
- dismissal rules (“click outside”) are implemented at the overlay root by checking pointer hits against the overlay bounds and the owner’s policy.

To support anchored positioning (menus near a button, IME near caret), the UI runtime must provide a way to compute
window-space bounds for an element/node after layout (see ADR 0012 / ADR 0028).

#### Why overlays are not "just absolute positioning"

In DOM/CSS systems, many "floating" surfaces are implemented by mixing `position: absolute/fixed`
with portals and `z-index`. In Fret we intentionally separate these concerns:

- Docking layouts commonly introduce **clipping and scroll containers**; a popover that must extend
  beyond a dock panel cannot be reliably implemented by `absolute` positioning inside that panel.
- Fret does not provide a global element-level `z-index` (ADR 0062). Cross-container stacking must
  be expressed via explicit overlay roots and their deterministic window-scoped ordering.
- Therefore, menus/popovers/tooltips/drag previews that need to escape local clipping should be
  treated as **overlay roots** (portal), while `absolute` positioning remains a local layout tool
  for in-tree decoration.

### 3) Rendering order matches root order

The display list is built so that overlay roots paint after base roots, preserving expected composition.

## Consequences

- Docking drag previews, menus, and tooltips become predictable and consistent across platforms.
- The retained tree can stay simple per-root; cross-root policies live in the UI runtime.

## Future Work

- Define keyboard focus traversal across roots (tab order, focus scopes).
- Add an accessibility/semantics parallel tree hook that supports overlays.
