# Keyboard Sensor Design Note

Date: 2026-03-13

## Status

Design note only.

This does not add a keyboard sensor in v1. It locks the intended ownership and behavioral shape so
future implementation work stays additive instead of widening the wrong layer.

## Problem statement

Fret's headless DnD stack is pointer-first today.

That is acceptable for v1 because ADR 0157 explicitly defers keyboard-driven drag sensors, but the
next accessibility/parity step needs a clear boundary before implementation starts:

- which layer owns key bindings and keyboard-drag state,
- how keyboard drag activation interacts with focus,
- how keyboard-driven movement interacts with auto-scroll,
- which parts stay generic versus recipe/product-specific.

## Decision summary

### 1) Keep the layer split: focus/routing in UI, drag state in the headless toolbox

Ownership should be:

- `crates/fret-ui`: focus tree, focus traversal, focus-visible state, and key-event routing
- `ecosystem/fret-dnd`: data-only keyboard DnD state/bindings/intents
- `ecosystem/fret-ui-kit::dnd`: focused-handle wiring that translates key events into headless
  keyboard sensor updates
- recipe/product layers (`sortable`, Kanban, docking, workspace, node graph): semantic movement
  policy and post-drop focus policy

The keyboard sensor must not pull focus management into `fret-dnd`, and it must not push DnD policy
down into `crates/fret-ui`.

### 2) Activation is focus-based, not hit-test-based

Keyboard dragging should activate only from a currently focused, enabled drag source or drag handle.

That means:

- the authoring surface must expose a focusable handle or source region,
- the integration layer should attach keyboard handlers to that focused element,
- activation should be prevented for text-input editing surfaces unless a dedicated drag handle owns
  focus.

This matches Fret's existing key-routing model better than any pointer-like region hit-testing.

### 3) Default bindings should stay conservative in Fret

Recommended default bindings:

- start: `Space`, `Enter`
- cancel: `Escape`
- end: `Space`, `Enter`
- directional movement: arrow keys

Do **not** make `Tab` a default drag-end key in Fret.

Reason:

- Fret already uses `Tab` / `Shift+Tab` for focus traversal commands in app-facing defaults.
- `focus_visible` also treats `Tab` as a navigation key that changes focus-visible state.

`Tab` may be allowed as an opt-in recipe/product override later, but it should not be part of the
core default keyboard DnD contract.

### 4) Do not bake fixed pixel stepping into the core contract

`dnd-kit` ships a fixed coordinate-step default and lets higher layers override it.

Fret should invert that emphasis for editor-grade surfaces:

- the headless keyboard sensor should emit **directional intent** (up/down/left/right) and
  start/cancel/end lifecycle,
- the integration or recipe layer should decide how that intent maps to the next drag position or
  target,
- simple surfaces may still use a pixel-step helper, but sortable/docking/node-graph flows should
  be free to resolve movement semantically.

This avoids hard-coding DOM-style coordinate stepping into the long-term contract.

### 5) Auto-scroll remains a pure output contract

Keyboard DnD must preserve the same purity rule as pointer DnD:

- `fret-dnd` computes data-only scroll requests or movement outputs,
- it does not perform scrolling,
- it does not enable/disable scroll drivers,
- it does not scroll a focused node into view directly.

If keyboard drag needs visibility management, the integration layer should use existing focus/scroll
mechanisms or product-local scroll drivers after consuming the headless output.

## Proposed ownership by layer

### `crates/fret-ui`

Owns existing mechanisms that keyboard DnD must consume rather than replace:

- focusability
- focus-visible updates
- key handler registration
- focus traversal
- scroll-into-view for focused descendants

No DnD-specific focus policy should be added here just to support drag authoring.

### `ecosystem/fret-dnd`

May add a data-only keyboard seam later, for example:

- `KeyboardDndBindings` using `fret_core::KeyCode`
- `KeyboardDndIntent::{Start, Cancel, End, Move(Direction)}`
- optional helpers for semantic-axis movement or simple pixel stepping

It should not depend on:

- element IDs,
- focus-tree internals,
- runtime/window registries,
- accessibility announcement systems.

### `ecosystem/fret-ui-kit::dnd`

Should own the integration seam that binds keyboard events on focused drag sources/handles and
converts them into headless DnD updates.

Possible responsibilities:

- focused-handle key hookup
- default prevent-activation logic for text inputs and disabled handles
- bridging keyboard directional intent to a consumer-supplied move strategy
- forwarding pure auto-scroll requests back to the consumer

### Recipe / product layers

Must continue to own:

- which handle is focusable,
- whether keyboard drag is enabled for that surface,
- how directional intent maps to reorder/drop semantics,
- where focus lands after cancel or successful drop,
- announcements/instructions for accessibility.

## Focus semantics

The first implementation should preserve these rules:

1. Focus stays on the active drag source/handle during keyboard drag by default.
2. Collision / over updates must not implicitly move focus to droppables.
3. Cancel should keep or restore focus to the origin handle.
4. Successful drop should keep focus on the logical dragged item by default, but exact restore
   policy remains recipe-owned.
5. Keyboard drag must cooperate with existing roving-focus and focus-scope patterns rather than
   bypassing them.

This keeps keyboard DnD compatible with existing Fret focus primitives and component policies.

## Auto-scroll interaction

Fret should not copy the DOM-specific `dnd-kit` behavior literally.

In `dnd-kit`, the keyboard sensor can call DOM scroll-into-view helpers and temporarily disable the
auto-scroll plugin during keyboard control.

For Fret:

- visibility management should remain integration-owned because scroll containers are framework
  widgets, not DOM nodes,
- `scroll_node_into_view(...)` remains a UI/focus mechanism, not a headless DnD side effect,
- continuous keyboard auto-scroll, if needed, belongs to a later integration-layer driver note.

The keyboard sensor design therefore depends on the already-deferred "continuous auto-scroll driver"
follow-on rather than replacing it.

## Explicit non-goals for the first keyboard landing

- no accessibility announcement/ARIA instruction system in the same change
- no monitor/event-surface extraction as part of keyboard support
- no `Tab` default end-key contract in the headless layer
- no requirement that all surfaces use pixel-step movement
- no focus trapping semantics inside `fret-dnd`

## Minimum gates before implementation

When implementation starts, require at least:

1. `fret-dnd` tests for keyboard binding interpretation and cancel/end cleanup.
2. `fret-ui-kit` tests for focused-handle activation and text-input prevention.
3. One first-party integration gate showing semantic keyboard drag behavior:
   - sortable reorder, or
   - workspace tab-strip reorder, or
   - docking tab movement.
4. `python tools/check_layering.py`.

## Evidence anchors

- `docs/adr/0149-headless-dnd-toolbox-and-ui-integration.md`
- `docs/adr/0157-headless-dnd-v1-contract-surface.md`
- `crates/fret-app/src/app.rs`
- `crates/fret-ui/src/elements/cx.rs`
- `crates/fret-ui/src/focus_visible.rs`
- `crates/fret-ui/src/tree/commands.rs`
- `repo-ref/dnd-kit/packages/dom/src/core/sensors/keyboard/KeyboardSensor.ts`
- `repo-ref/dnd-kit/apps/docs/extend/sensors/keyboard-sensor.mdx`
- `repo-ref/dnd-kit/apps/docs/legacy/api-documentation/sensors/keyboard.mdx`
