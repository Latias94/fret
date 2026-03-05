# Editor TabStrip Unification Fearless Refactor v1 (Design)

## Context

Fret currently has two “editor-grade tab strip” implementations:

- Workspace tab strip: `ecosystem/fret-workspace/src/tab_strip/` (policy-heavy, focus/keyboard-aware)
- Docking tab bar: `ecosystem/fret-docking/src/dock/` (docking arbitration + self-drawn overflow menu)

Both surfaces share the same high-frequency interaction outcomes:

- Activate / close tabs without accidental focus/drag
- Drag reorder + end-drop insertion
- Overflow membership + dropdown/menu behaviors
- Scroll guarantees (“active stays reachable/visible”)

This workstream converges the two implementations onto shared **mechanism vocabulary** while keeping
adapter-specific policy in ecosystem crates.

## Non-goals

- Visual parity with any upstream (Zed/dockview) beyond hit rect correctness.
- Replacing retained UI trees with declarative rebuild (tracked elsewhere).
- Full APG/a11y closure (tracked in a11y workstreams); only tabstrip-relevant pieces here.

## Layering (where things live)

Mechanism (shared, deterministic, unit-testable):

- `ecosystem/fret-ui-headless`
  - surface classification (`TabStripSurface`)
  - drop target resolution (`TabStripDropTarget`)
  - overflow membership + dropdown indices
  - scroll-to-visible math
  - click arbitration intent mapping (small, shared rules)

Policy (adapter-owned, varies by product surface):

- `ecosystem/fret-workspace`
  - pinned/preview semantics
  - focus restore + keyboard command wiring
  - context menu items and editor-specific commands
- `ecosystem/fret-docking`
  - dock graph ops + cross-window drag routing
  - self-drawn overflow menu rendering + hit-test glue

Toolbox re-exports (optional):

- `ecosystem/fret-ui-kit/src/headless/*` may re-export `fret-ui-headless` helpers to reduce import
  churn in policy crates.

## Shared vocabulary (v1)

We treat the tab strip as a set of explicit “surfaces”:

- `TabsViewport`: where tabs live and where tab-half insert indices resolve
- `HeaderSpace`: explicit end-drop surface (“drop at end”)
- `OverflowControl`: opens overflow dropdown/menu (explicit non-drop surface)
- `ScrollControls`: explicit non-drop surface
- `PinnedBoundary`: a boundary region between pinned/unpinned (workspace-only in v1)
- `Outside`

These surfaces exist even when the UI is self-drawn and cannot attach fine-grained `test_id`s:
diagnostics predicates gate the outcomes when semantics IDs are unavailable.

## Overflow dropdown policy (decision)

Overflow dropdown item policy is **adapter-owned**.

Current defaults:

- Workspace tab strip: overflowed-only (dockview-like).
- Docking tab bar: overflowed + active (to keep the active tab reachable under different scroll/geometry pipelines).

Rationale:

- Keep policy in ecosystem layers; do not force a single global default when the surfaces have different UX goals.
- Keep the mechanism helper deterministic (`compute_overflow_menu_item_indices`), and pass policy knobs from adapters.

The shared headless helper (`compute_overflow_menu_item_indices`) remains the mechanism source of
truth; adapters pass:

- `overflowed` indices (computed from their own geometry)
- `active` index (best effort)
- `OverflowMenuActivePolicy::{Include|Exclude}`
 - `OverflowMenuEmptyOverflowedPolicy::{Empty|AllTabs}`

## Next steps (M2+)

- Expand shared gates around end-drop semantics under overflow (canonical insert-index).
- Decide whether pinned tabs become multi-row in workspace (policy-only).
- Document and gate a minimal keyboard/focus semantics bundle for editor surfaces.
