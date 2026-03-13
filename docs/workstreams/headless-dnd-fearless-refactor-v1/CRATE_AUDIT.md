# Headless DnD Crate Audit

Date: 2026-03-13

## Scope

- `ecosystem/fret-dnd`
- `ecosystem/fret-ui-kit::dnd`
- first-party adopters used as evidence:
  - `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`
  - `ecosystem/fret-ui-shadcn/src/extras/kanban.rs`
  - `ecosystem/fret-workspace/src/tab_strip/mod.rs`
  - `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/pending.rs`

## Boundary conclusion

No top-level crate rewrite is needed.

The current split is still the right one:

- `fret-dnd` is a clean headless crate with only `fret-core` as a direct dependency.
- `fret-ui-kit::dnd` is the correct home for runtime-facing sensor wiring, per-window registries,
  and authoring ergonomics.

The fearless-refactor justification remains implementation-focused, not boundary-focused: keep the
crate split, keep moving reusable drag truth into the headless engine, and only widen the headless
surface when at least two consumers require the new data.

## Snapshot

### `fret-dnd`

- Purpose:
  - activation constraints and pointer sensor state
  - data-only engine/operation state
  - collision ordering
  - modifier and auto-scroll request computation
- Boundary posture:
  - depends only on `fret-core`
  - no `fret-ui`, runtime, runner, or platform coupling
- Evidence anchors:
  - `ecosystem/fret-dnd/src/lib.rs`
  - `ecosystem/fret-dnd/src/engine.rs`
  - `ecosystem/fret-dnd/src/registry.rs`

### `fret-ui-kit::dnd`

- Purpose:
  - window/frame/scope registry lifecycle
  - event-to-engine translation
  - ergonomic forwarders and activation-only seams
- Boundary posture:
  - depends on runtime/UI layers as expected for an adapter crate
  - should keep product-specific drag semantics outside the generic adapter
- Evidence anchors:
  - `ecosystem/fret-ui-kit/src/dnd/mod.rs`
  - `ecosystem/fret-ui-kit/src/dnd/controller/mod.rs`
  - `ecosystem/fret-ui-kit/src/dnd/forwarders.rs`
  - `ecosystem/fret-ui-kit/src/dnd/activation_probe.rs`

## Current hazards and parity gaps

### 1) Pointer sensor ergonomics previously lagged behind `dnd-kit`

Before this branch, `DndPointerForwardersConfig` only exposed a single activation constraint and no
prevent-activation hook, which forced text-input / nested-pressable / pointer-kind arbitration back
into each consumer.

This branch closes that gap for the UI-kit adapter layer by adding:

- pointer-type-aware activation constraints
- a generic prevent-activation hook
- text-input and nested-pressable guards that do not depend on DOM selectors

Evidence anchors:

- `ecosystem/fret-ui-kit/src/dnd/forwarders.rs`
- `ecosystem/fret-ui-kit/src/dnd/tests.rs`
- `repo-ref/dnd-kit/packages/dom/src/core/sensors/pointer/PointerSensor.ts`

### 2) Droppable metadata is still intentionally minimal

`fret-dnd::Droppable` still contains only:

- `id`
- `rect`
- `disabled`
- `z_index`

That keeps the headless registry portable, but it also means `dnd-kit`-style `type` / `accept` /
per-droppable collision policy hooks are not available yet.

Evidence anchors:

- `ecosystem/fret-dnd/src/registry.rs`
- `repo-ref/dnd-kit/packages/dom/src/core/entities/droppable/droppable.ts`

Recommendation:

- do not widen `fret-dnd` yet;
- first confirm at least two consumers need shared metadata semantics.

### 3) Monitor/event surface is still absent

There is no small monitor-style event stream yet for:

- sensor activation
- over/collision transitions
- cancel/end observation by product policy

That is acceptable for v1, and this workstream now explicitly defers that extraction until two
shared consumers need the same observer contract.

Evidence anchors:

- `docs/workstreams/headless-dnd-fearless-refactor-v1/MONITOR_SURFACE_DECISION.md`
- `ecosystem/fret-dnd/src/engine.rs`
- `ecosystem/fret-ui-kit/src/dnd/controller/`
- `repo-ref/dnd-kit/packages/dom/src/core/manager/manager.ts`

### 4) Cross-window hand-off remains surface-specific after activation

Workspace tab tear-out and node insert promotion now share the same activation-only seam, but their
post-activation hand-off is still product-owned.

That is the right boundary for now, but it is still a hotspot if another cross-window activation
consumer appears.

Evidence anchors:

- `ecosystem/fret-workspace/src/tab_strip/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/pending.rs`
- `docs/workstreams/headless-dnd-fearless-refactor-v1/MANUAL_FORWARDING_INVENTORY.md`

### 5) Keyboard sensor and continuous auto-scroll driver implementations remain follow-on work

The current stack keeps auto-scroll pure/data-only in the headless crate and does not ship a
keyboard sensor yet. That matches ADR scope. A design note now exists for keyboard ownership and
focus semantics, and a second design note now exists for continuous auto-scroll driver extraction.
Both implementation lanes remain follow-on work rather than fully closed outcomes.

Evidence anchors:

- `docs/workstreams/headless-dnd-fearless-refactor-v1/AUTO_SCROLL_DRIVER_DESIGN_NOTE.md`
- `docs/workstreams/headless-dnd-fearless-refactor-v1/KEYBOARD_SENSOR_DESIGN_NOTE.md`
- `ecosystem/fret-dnd/src/scroll.rs`
- `docs/adr/0157-headless-dnd-v1-contract-surface.md`
- `repo-ref/dnd-kit/packages/dom/src/core/sensors/keyboard/KeyboardSensor.ts`
- `repo-ref/dnd-kit/packages/dom/src/core/plugins/scrolling/AutoScroller.ts`

## Recommended next steps

1. Keep the current crate split. Do not move DnD policy into `crates/fret-ui` and do not widen
   `fret-dnd` without two real consumers.
2. Treat the newly added forwarder sensor configuration seam as the default path for handle/text
   input arbitration before adding more bespoke product-local checks.
3. Evaluate lightweight droppable metadata only when at least two consumers need the same shared
   contract:
   - likely candidates: node graph typed drop zones and workspace/docking target filtering
4. Revisit a small monitor/event surface only after two consumers can demonstrate the same observer
   need, as documented in `MONITOR_SURFACE_DECISION.md`.
5. Implement keyboard sensor and any shared continuous auto-scroll driver from the design notes,
   not as opportunistic API widening.

## Gates run for this audit checkpoint

- `cargo nextest run -p fret-dnd`
- `cargo nextest run -p fret-ui-kit --features dnd dnd::tests::`
- `cargo nextest run -p fret-ui-shadcn --test kanban_dnd_forwarders --test carousel_dnd_arbitration`
- `python tools/check_layering.py`
