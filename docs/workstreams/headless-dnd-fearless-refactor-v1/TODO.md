# Headless DnD (dnd-kit Alignment) — TODO

This file tracks the execution checklist for `DESIGN.md`.

## M0 — Workstream setup

- [x] Create the workstream directory.
- [x] Write `README.md`, `DESIGN.md`, `TODO.md`, and `MILESTONES.md`.
- [ ] Link this workstream from any higher-level tracker if we decide it should become a repo-wide
  active initiative.

## M1 — Safety fixes and truthful integration outputs

- [x] Fix `clamp_rect_translation(...)` so oversized dragged rects do not panic.
  - Gate: add a `fret-dnd` regression test for `bounds < rect`.
- [x] Make `fret-ui-kit::dnd::DndUpdate.collisions` truthful.
  - Route collision results from the headless frame computation into the public update surface.
  - Gate: add a focused UI-kit test that asserts deterministic collision ordering.
- [x] Remove duplicate `over` / auto-scroll computation paths in the controller when the headless
  frame output already provides them.
  - Gate: add a test that compares UI-kit update output with direct `compute_dnd_frame(...)`.
- [x] Keep layering green.
  - Gate: `python tools/check_layering.py`

## M2 — Headless engine / operation extraction

- [x] Add a central data-only operation/engine model inside `ecosystem/fret-dnd`.
- [x] Decide the internal module split:
  - `engine.rs` vs `operation.rs`
  - sensor modules
  - entity metadata placement
- [x] Move more drag truth into the headless engine:
  - active pointer/source
  - translation
  - collisions
  - `over`
  - cancel/end lifecycle
- [x] Keep `fret-dnd` free of `fret-ui`, runtime, and runner dependencies.
  - Gate: `python tools/check_layering.py`
- [x] Add engine-level tests that cover:
  - multi-pointer independence
  - activation lifecycle
  - cancel/end cleanup
  - deterministic collision ordering

## M3 — Thin UI-kit adapter and forwarder-first adoption

- [x] Split `ecosystem/fret-ui-kit/src/dnd/controller/` by responsibility.
- [x] Recenter the UI-kit controller around the headless engine output instead of hand-assembling
  partial `DndUpdate` values.
- [x] Make `DndPointerForwarders` the preferred path for new integrations.
- [x] Migrate at least one first-party production integration away from hand-written forwarding:
  - sortable recipe
  - Kanban
  - or both
- [x] Add a small inventory note for remaining manual forwarding call sites.
  - Evidence anchors:
    - `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`
    - `ecosystem/fret-ui-shadcn/src/extras/kanban.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/carousel/demo.rs`

## M4 — First `dnd-kit`-style extension seams

- [x] Add an activation-only seam for pending gesture flows that need threshold tracking without
  full pointer-region updates.
  - Evidence anchors:
    - `ecosystem/fret-ui-kit/src/dnd/activation_probe.rs`
    - `ecosystem/fret-workspace/src/tab_strip/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/pending.rs`
  - Gates:
    - `ecosystem/fret-ui-kit/src/dnd/tests.rs`
    - `ecosystem/fret-workspace/tests/tab_strip_drag_activation_threshold.rs`
- [x] Add richer pointer sensor configuration without introducing DOM assumptions:
  - pointer-type-aware activation constraints for forwarder-backed integrations
  - prevent-activation policy hook for consumer-owned arbitration
  - pressable/text-input guards that support container-vs-handle authoring without DOM selectors
  - Evidence anchors:
    - `ecosystem/fret-ui-kit/src/dnd/forwarders.rs`
    - `ecosystem/fret-ui-kit/src/dnd/tests.rs`
- [x] Decide whether to add lightweight draggable/droppable metadata now:
  - Decision: defer from v1 for now; do not widen `fret-dnd` yet
  - Reason: Kanban is one heterogeneous consumer, but there is not yet a second first-party
    registry-driven consumer that needs the same shared contract
  - Follow-up trigger: revisit once a second real consumer needs shared `type` / `accept` /
    collision-policy semantics
  - Evidence anchors:
    - `docs/workstreams/headless-dnd-fearless-refactor-v1/DROPPABLE_METADATA_DECISION.md`
    - `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`
    - `ecosystem/fret-ui-shadcn/src/extras/kanban.rs`
    - `ecosystem/fret-workspace/src/tab_strip/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/pending.rs`
- [x] Decide whether to extract a small monitor/event surface now:
  - Decision: defer from v1 for now; keep observation product-local through `on_update(...)`
  - Reason: current first-party consumers are local recipe/product observers, not two shared
    cross-cutting monitor consumers
  - Follow-up trigger: revisit once at least two shared observers need the same contract
    (diagnostics, optimistic sorting, accessibility, auto-scroll integration)
  - Evidence anchors:
    - `docs/workstreams/headless-dnd-fearless-refactor-v1/MONITOR_SURFACE_DECISION.md`
    - `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`
    - `ecosystem/fret-ui-shadcn/src/extras/kanban.rs`
    - `ecosystem/fret-workspace/src/tab_strip/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/pending.rs`
- [x] Preserve the rule that modifiers and auto-scroll core remain pure/data-only.
  - Evidence: ADR 0157 already locks this as a contract invariant.
  - Evidence anchors:
    - `docs/adr/0157-headless-dnd-v1-contract-surface.md`
    - `ecosystem/fret-dnd/src/modifier.rs`
    - `ecosystem/fret-dnd/src/scroll.rs`
    - `ecosystem/fret-dnd/src/frame.rs`
    - `ecosystem/fret-dnd/src/engine.rs`
    - `ecosystem/fret-workspace/src/tab_strip/kernel.rs`
    - `ecosystem/fret-docking/src/dock/space.rs`

## M5 — Follow-on parity targets (do not block the main refactor)

- [x] Keyboard sensor design note:
  - ownership
  - focus semantics
  - interaction with auto-scroll
  - Evidence anchors:
    - `docs/workstreams/headless-dnd-fearless-refactor-v1/KEYBOARD_SENSOR_DESIGN_NOTE.md`
    - `docs/adr/0157-headless-dnd-v1-contract-surface.md`
    - `crates/fret-ui/src/elements/cx.rs`
    - `crates/fret-ui/src/focus_visible.rs`
    - `crates/fret-ui/src/tree/commands.rs`
    - `repo-ref/dnd-kit/packages/dom/src/core/sensors/keyboard/KeyboardSensor.ts`
- [x] Continuous auto-scroll driver semantics in the integration layer.
  - Decision: document the target driver contract now, but defer shared extraction until at least
    two consumers converge on the same `DndUpdate.autoscroll`-driven interface.
  - Evidence anchors:
    - `docs/workstreams/headless-dnd-fearless-refactor-v1/AUTO_SCROLL_DRIVER_DESIGN_NOTE.md`
    - `ecosystem/fret-ui-kit/src/dnd/types.rs`
    - `ecosystem/fret-workspace/src/tab_strip/kernel.rs`
    - `ecosystem/fret-workspace/src/tab_strip/mod.rs`
    - `ecosystem/fret-docking/src/dock/space.rs`
    - `ecosystem/fret-docking/src/dock/tests/drag.rs`
- [x] Decide whether multi-rect droppables belong in the core snapshot now:
  - Decision: defer from v1 for now; keep the core snapshot single-rect-per-droppable
  - Reason: current shared consumers remain single-rect, while richer hit-region cases still use
    product-local geometry rather than a shared registry-driven contract
  - Follow-up trigger: revisit once at least two registry-driven consumers need one semantic
    droppable to span multiple rects
  - Evidence anchors:
    - `docs/workstreams/headless-dnd-fearless-refactor-v1/MULTI_RECT_DROPPABLES_DECISION.md`
    - `ecosystem/fret-dnd/src/registry.rs`
    - `ecosystem/fret-ui-kit/src/dnd/registry.rs`
    - `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`
    - `ecosystem/fret-ui-shadcn/src/extras/kanban.rs`
    - `ecosystem/fret-workspace/src/tab_strip/mod.rs`
    - `ecosystem/fret-docking/src/dock/space.rs`
- [ ] Sortable group semantics beyond the current minimal insertion helpers.

## Product adoption and gates

- [x] Sortable recipe gate added or strengthened.
- [x] Kanban gate added or strengthened.
- [x] Docking tab hover/insert path verified against the new engine/controller shape.
- [x] Carousel-vs-DnD arbitration gate remains green after adapter changes.
- [x] Workspace tab-strip activation gate covers the activation-only seam.
- [x] Node retained-canvas activation gates rerun after `compat-retained-canvas` build recovery.

## Docs and contract follow-up

- [ ] If the refactor changes the public contract shape of `fret-dnd`, update:
  - `docs/adr/0157-headless-dnd-v1-contract-surface.md`
  - `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- [ ] If the refactor changes first-party teaching surfaces, update:
  - `docs/crate-usage-guide.md`
  - relevant UI Gallery examples

## Minimum completion bar

- [x] Known panic fixed and guarded.
- [x] `DndUpdate` collision output made truthful.
- [x] Headless engine/operation introduced.
- [x] UI-kit controller reduced to a thinner adapter.
- [x] At least one real first-party integration migrated to the preferred seam.
