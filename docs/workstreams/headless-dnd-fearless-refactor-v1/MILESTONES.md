# Headless DnD (dnd-kit Alignment) — Milestones

This workstream is staged so that each phase leaves behind a clear gate and keeps crate ownership
intact.

## M0 — Workstream and baseline audit

**Outcome**

- The repo has a dedicated workstream directory for the refactor.
- The current hazards, boundaries, and parity targets are written down before implementation churn.

**Required artifacts**

- `README.md`
- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`

## M1 — Safety and truthful output closure

**Outcome**

- Headless modifier helpers no longer panic on oversized-rect clamp scenarios.
- `fret-ui-kit::dnd::DndUpdate` becomes a truthful integration output:
  - collisions populated
  - `over` comes from the canonical headless frame result
  - auto-scroll request comes from the canonical headless frame result

**Evidence anchors**

- `ecosystem/fret-dnd/src/modifier.rs`
- `ecosystem/fret-dnd/src/frame.rs`
- `ecosystem/fret-ui-kit/src/dnd/controller.rs`
- `ecosystem/fret-ui-kit/src/dnd/types.rs`

**Gates**

- `cargo nextest run -p fret-dnd`
- focused `fret-ui-kit` DnD tests
- `python tools/check_layering.py`

## M2 — Headless engine extraction

**Outcome**

- `fret-dnd` owns a central data-only operation/engine model rather than only disconnected helper
  functions.
- Multi-pointer, activation, collision, and end/cancel lifecycle are modeled in one place.

**Constraints**

- no `fret-ui`
- no runtime or runner dependencies
- no product-state mutation in the engine

**Gates**

- engine-level unit tests for:
  - activation lifecycle
  - multi-pointer independence
  - deterministic collisions
  - cancel/end cleanup
- `python tools/check_layering.py`

## M3 — Thin UI-kit adapter

**Outcome**

- The UI-kit DnD controller becomes a thin runtime adapter over the headless engine.
- `controller.rs` is split by responsibility.
- the public integration surface stops hand-assembling partial DnD truth

**Evidence anchors**

- `ecosystem/fret-ui-kit/src/dnd/`

**Gates**

- focused nextest coverage for the adapter paths
- at least one comparison test between headless engine output and UI-kit update output

## M4 — First-party adoption

**Outcome**

- At least one real first-party integration adopts the preferred seam:
  - `DndPointerForwarders`
  - thinner shared state ownership
  - fewer local drag-truth structs

**Candidate adopters**

- `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`
- `ecosystem/fret-ui-shadcn/src/extras/kanban.rs`
- `apps/fret-ui-gallery/src/ui/snippets/carousel/demo.rs`

**Gates**

- sortable integration gate
- Kanban integration gate
- carousel arbitration gate remains green

## M5 — dnd-kit-style extension readiness

**Outcome**

- The architecture is ready for additive follow-on parity work:
  - richer pointer sensor configuration
  - optional draggable/droppable metadata
  - monitor/event stream
  - keyboard sensor
  - continuous auto-scroll driver semantics

This milestone does not require landing all of those features. It requires the refactor to make
them additive rather than forcing another core rewrite.

**Evidence anchors**

- `repo-ref/dnd-kit/packages/abstract/src/core/manager/manager.ts`
- `repo-ref/dnd-kit/packages/abstract/src/core/manager/operation.ts`
- `repo-ref/dnd-kit/packages/dom/src/core/sensors/pointer/PointerSensor.ts`
- `repo-ref/dnd-kit/packages/dom/src/core/sensors/keyboard/KeyboardSensor.ts`

## Completion bar for v1

We consider v1 ready to transition into implementation-focused work when:

- M1 through M3 are complete
- at least one first-party adopter is migrated under M4
- the design clearly preserves Fret's crate boundaries
- the follow-on parity lane is documented and does not require rethinking the crate split
