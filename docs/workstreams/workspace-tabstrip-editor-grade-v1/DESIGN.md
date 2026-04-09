# Workspace TabStrip (editor-grade) v1 (Design)

## Context

This workstream targets the **editor/workspace tab strip** surface (pane chrome), not in-page
navigation tabs (Radix/shadcn `Tabs`).

This surface is high-frequency and “hard to change” once apps depend on it:

- selection + close
- reorder + cross-pane move
- overflow / scroll-to-reveal
- pinned + preview semantics (editor-grade)
- drag-to-split integration (workspace shell / docking)

The repo is docs/ADR-driven, so the goal is to lock down stable seams and regression gates before
we iterate on visuals.

## Goals (v1)

1. **Modular, testable tab strip kernel** in the correct layer.
2. **Deterministic interaction outcomes** (drop index, focus restore, scroll-to-active).
3. **Scriptable + gateable** surface (stable `test_id` anchors + `fretboard-dev diag` scripts).
4. **One behavior kernel, multiple adapters** (workspace shell + docking).

## Non-goals (v1)

- Pixel-perfect parity with any upstream reference.
- Moving policy into `crates/fret-ui` (mechanism/contract layer).
- Finalizing a public, semver-stable editor chrome API (iterate in `ecosystem/` first).

## Layering (where code should live)

- `crates/fret-core`: stable workspace/docking models + ops (no UI policy, no platform).
- `ecosystem/fret-workspace`: editor/workspace tab policy + default adapter (`WorkspaceTabStrip`).
- `ecosystem/fret-docking`: docking-specific arbitration + integration (drop-to-split, cross-pane).
- `ecosystem/fret-ui-kit`: shared policy helpers only when broadly reusable (roving, menu helpers).
- `crates/fret-ui`: mechanism primitives only (scroll handles, semantics roles, event routing).

## References (what to learn from whom)

- **Zed (`repo-ref/zed`)**: editor semantics (pinned/preview, focus invariants, split-on-drop).
- **dockview (`repo-ref/dockview`)**: overflow pipeline + “header space” drop surfaces + tests.
- **gpui-component (`repo-ref/gpui-component`)**: wiring shape for dock/tab panels + explicit end-drop
  surface concept.

We align outcomes, not DOM/CSS implementation details.

## Proposed internal split (workspace tab strip)

Keep the implementation modular even if it starts in a single crate:

1. **Kernel (pure-ish)**
   - Inputs: tab rects, viewport rect, pointer position, drag payload metadata, tab ordering.
   - Outputs: `DropTarget` / `insert_index`, optional `split_intent`, plus “why” debug info.
   - Deterministic and unit-testable.

2. **Geometry (measured)**
   - Rect caching, visible-range computation, overflow membership.
   - Produces canonical-index mapping (full list ↔ visible subset).

3. **Interaction (policy)**
   - Pointer/keyboard → intents (activate/close/reorder/pin/unpin/start drag).
   - Owns editor-heavy behaviors (middle-click close, double-click policy) in the shell layer.

4. **View (render)**
   - Renders tabs + overflow control + explicit end-drop surface region.
   - Exposes stable `test_id` anchors.

## Contracts to lock early

- **Canonical ordering**: all indices and `insert_index` are expressed in the full tab list order,
  never in “visible-only” index space.
- **End-drop surface**: a stable hit-test region exists that always resolves to
  `insert_index == tab_count`.
- **Pinned boundary**: reorder cannot cross pinned boundary unless explicitly pin/unpin.
- **Focus invariants**:
  - pointer down does not steal focus from the active editor surface,
  - closing the active tab selects a deterministic neighbor (policy-owned, but gated).
- **Overflow**: overflow membership + “activate from overflow” is deterministic under resize.

## Regression gates (required for refactors)

- At least one unit test per kernel-level invariant (drop target, index mapping, pinned boundary).
- At least one promoted `fretboard-dev diag` script per interaction class (reorder, split-drop preview,
  overflow activate, close behaviors).

