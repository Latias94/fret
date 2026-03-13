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

- [ ] Add a central data-only operation/engine model inside `ecosystem/fret-dnd`.
- [ ] Decide the internal module split:
  - `engine.rs` vs `operation.rs`
  - sensor modules
  - entity metadata placement
- [ ] Move more drag truth into the headless engine:
  - active pointer/source
  - translation
  - collisions
  - `over`
  - cancel/end lifecycle
- [ ] Keep `fret-dnd` free of `fret-ui`, runtime, and runner dependencies.
  - Gate: `python tools/check_layering.py`
- [ ] Add engine-level tests that cover:
  - multi-pointer independence
  - activation lifecycle
  - cancel/end cleanup
  - deterministic collision ordering

## M3 — Thin UI-kit adapter and forwarder-first adoption

- [ ] Split `ecosystem/fret-ui-kit/src/dnd/controller.rs` by responsibility.
- [ ] Recenter the UI-kit controller around the headless engine output instead of hand-assembling
  partial `DndUpdate` values.
- [ ] Make `DndPointerForwarders` the preferred path for new integrations.
- [ ] Migrate at least one first-party production integration away from hand-written forwarding:
  - sortable recipe
  - Kanban
  - or both
- [ ] Add a small inventory note for remaining manual forwarding call sites.
  - Evidence anchors:
    - `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`
    - `ecosystem/fret-ui-shadcn/src/extras/kanban.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/carousel/demo.rs`

## M4 — First `dnd-kit`-style extension seams

- [ ] Add richer pointer sensor configuration without introducing DOM assumptions:
  - activator/handle-oriented configuration
  - prevent-activation policy hook
  - pointer-kind-aware defaults if needed
- [ ] Decide whether to add lightweight draggable/droppable metadata now:
  - `type`
  - `accept`
  - collision-strategy hook
- [ ] Add a small monitor/event surface only if at least two consumers need it.
- [ ] Preserve the rule that modifiers and auto-scroll core remain pure/data-only.

## M5 — Follow-on parity targets (do not block the main refactor)

- [ ] Keyboard sensor design note:
  - ownership
  - focus semantics
  - interaction with auto-scroll
- [ ] Continuous auto-scroll driver semantics in the integration layer.
- [ ] Multi-rect droppables only if at least two consumers require them.
- [ ] Sortable group semantics beyond the current minimal insertion helpers.

## Product adoption and gates

- [ ] Sortable recipe gate added or strengthened.
- [ ] Kanban gate added or strengthened.
- [ ] Docking tab hover/insert path verified against the new engine/controller shape.
- [ ] Carousel-vs-DnD arbitration gate remains green after adapter changes.

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
- [ ] Headless engine/operation introduced.
- [ ] UI-kit controller reduced to a thinner adapter.
- [ ] At least one real first-party integration migrated to the preferred seam.
