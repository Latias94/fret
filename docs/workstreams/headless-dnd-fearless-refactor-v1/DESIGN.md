# Headless DnD (dnd-kit Alignment) — Fearless Refactor v1

## Context

Fret already has the right high-level split for drag-and-drop:

- `ecosystem/fret-dnd` is the headless policy toolbox.
- `ecosystem/fret-ui-kit::dnd` is the runtime/UI integration layer.
- cross-window internal drag routing remains owned by the runtime/runner contract.

That split is consistent with:

- `docs/adr/0149-headless-dnd-toolbox-and-ui-integration.md`
- `docs/adr/0157-headless-dnd-v1-contract-surface.md`
- `docs/adr/0155-docking-tab-dnd-contract.md`
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`

The problem is not the top-level crate boundary. The problem is that the current implementation is
still closer to "shared math helpers + a large UI-kit controller + per-recipe local state" than to
a mature, reusable DnD architecture.

## Why a fearless refactor is justified

The current stack already shows the usual drift signals:

- `ecosystem/fret-ui-kit/src/dnd/controller/` is still the main runtime adapter surface and had
  grown into a large orchestration module that owned too much
  event translation logic.
- `DndUpdate.collisions` exists in the public integration surface but is currently returned as an
  empty vector by the main controller paths.
- `compute_dnd_frame(...)` and `DndFrameOutput` exist in `fret-dnd`, but the integration layer does
  not use them as the source of truth.
- recipes and first-party adopters still carry local drag state that duplicates central DnD state:
  - sortable recipe local pointer state
  - Kanban pointer state
  - hand-written event forwarding in multiple places
- `DndPointerForwarders` exists, but first-party usage is still inconsistent.

If we keep extending the current structure, more subsystems will duplicate state and policy, and
future parity work against `dnd-kit` will become more expensive.

## User-facing invariant

The refactor must preserve these invariants:

1. `ecosystem/fret-dnd` remains a headless crate depending only on `fret-core`.
2. `crates/fret-ui` remains mechanism-only; DnD interaction policy does not move there.
3. Cross-window/internal drag session routing remains a runtime/runner concern, not a `fret-dnd`
   concern.
4. Window-local interactions must become easier to reuse across sortable lists, tab strips, Kanban,
   and node/canvas surfaces.
5. The design should leave room to align more capabilities with `dnd-kit` over time without another
   rewrite.

## Sources of truth

### Fret contract references

- `docs/adr/0149-headless-dnd-toolbox-and-ui-integration.md`
- `docs/adr/0157-headless-dnd-v1-contract-surface.md`
- `docs/adr/0155-docking-tab-dnd-contract.md`
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`

### Fret implementation anchors

- `ecosystem/fret-dnd/src/activation.rs`
- `ecosystem/fret-dnd/src/collision.rs`
- `ecosystem/fret-dnd/src/frame.rs`
- `ecosystem/fret-dnd/src/modifier.rs`
- `ecosystem/fret-dnd/src/registry.rs`
- `ecosystem/fret-dnd/src/scroll.rs`
- `ecosystem/fret-dnd/src/sortable.rs`
- `ecosystem/fret-ui-kit/src/dnd/controller/`
- `ecosystem/fret-ui-kit/src/dnd/registry.rs`
- `ecosystem/fret-ui-kit/src/dnd/forwarders.rs`
- `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`
- `ecosystem/fret-ui-shadcn/src/extras/kanban.rs`
- `ecosystem/fret-docking/src/dock/space.rs`

### Upstream reference

- `repo-ref/dnd-kit/packages/abstract/src/core/manager/manager.ts`
- `repo-ref/dnd-kit/packages/abstract/src/core/manager/operation.ts`
- `repo-ref/dnd-kit/packages/abstract/src/core/manager/registry.ts`
- `repo-ref/dnd-kit/packages/dom/src/core/sensors/pointer/PointerSensor.ts`
- `repo-ref/dnd-kit/packages/dom/src/core/sensors/keyboard/KeyboardSensor.ts`
- `repo-ref/dnd-kit/packages/dom/src/core/plugins/scrolling/AutoScroller.ts`
- `repo-ref/dnd-kit/packages/abstract/src/core/entities/droppable/droppable.ts`

## Current baseline and concrete hazards

### H1. A real panic exists in a core helper

`clamp_rect_translation(...)` currently calls `f32::clamp(min, max)` even when the dragged rect is
larger than the bounds, which can produce `min > max` and panic at runtime.

Evidence:

- `ecosystem/fret-dnd/src/modifier.rs`

### H2. The integration surface advertises collisions but does not provide them

`DndUpdate` exposes `collisions`, but the controller returns `Vec::new()` in the main update paths.
This means first-party integrations cannot treat the public UI-kit DnD surface as a truthful source
of collision state.

Evidence:

- `ecosystem/fret-ui-kit/src/dnd/types.rs`
- `ecosystem/fret-ui-kit/src/dnd/controller/`

### H3. `fret-dnd` has frame-level outputs that are not actually driving the integration layer

`compute_dnd_frame(...)` already computes:

- ordered collisions
- `over`
- auto-scroll request

But the UI-kit controller still computes `over` and auto-scroll manually and drops collisions
entirely. This is an architectural split-brain.

Evidence:

- `ecosystem/fret-dnd/src/frame.rs`
- `ecosystem/fret-ui-kit/src/dnd/controller/`

### H4. Too much central behavior still lives in recipes and product code

The sortable recipe and Kanban both keep pointer/drag state locally and manually interpret DnD
updates. This is a sign that the central engine state is not strong enough yet.

Evidence:

- `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`
- `ecosystem/fret-ui-shadcn/src/extras/kanban.rs`

### H5. The integration layer is becoming a "god file"

the `controller/` module was already large enough that future feature growth would increase review risk and
encourage copy-paste usage.

Evidence:

- `ecosystem/fret-ui-kit/src/dnd/controller/`

## Non-goals (v1)

- Moving DnD policy into `crates/fret-ui`.
- Merging window-local headless DnD with cross-window `DragSession`.
- Reproducing `dnd-kit`'s DOM renderer, feedback rendering, or framework adapter APIs.
- Shipping every `dnd-kit` capability immediately.
- Widening `fret-dnd` with platform, runner, or app-model dependencies.

## Ownership and boundaries

### Keep in `ecosystem/fret-dnd`

`fret-dnd` should own:

- headless entities/data structures
- activation constraints and sensor state machines
- drag operation state and transitions
- collision ordering and deterministic tie-breaks
- modifier pipelines
- auto-scroll request computation
- sortable/reorder semantics
- extension seams that are still data-only and portable

### Keep in `ecosystem/fret-ui-kit::dnd`

`fret-ui-kit::dnd` should own:

- window/frame/scope registry lifecycle
- translating Fret pointer events into headless sensor/operation updates
- ergonomic forwarders/helpers for declarative widgets and recipes
- optional UI-kit-level recipes that remain ecosystem policy

### Keep outside the headless stack

These stay outside the core headless refactor:

- cross-window `DragSession`
- runtime internal-drag routing
- docking tear-off/window creation policy
- app/model mutation semantics for each product surface

## Recommended architecture (Plan B)

### Summary

Keep the crate boundary, but move more of the actual orchestration truth into `fret-dnd`.

The target is:

- `fret-dnd` becomes the owner of a **data-only DnD engine/operation model**
- `fret-ui-kit::dnd` becomes a **thin adapter over that engine**
- recipes stop re-implementing local drag state that the engine could already own

### 1) Introduce a central data-only operation/engine layer in `fret-dnd`

Add an engine layer inside `fret-dnd` that becomes the canonical source of:

- pointer-tracking state
- activation phase
- active source id
- current pointer position / translation
- ordered collisions
- current `over`
- auto-scroll request
- cancel/end semantics

Suggested module split:

- `engine.rs` or `operation.rs`
- `entities.rs` or an expanded `registry.rs`
- `sensors/` for pointer + future keyboard sensors
- `modifiers/` for composition-friendly modifier pipelines

The key idea is not "big manager object because dnd-kit has one". The key idea is one truthful,
portable state machine instead of split-brain orchestration.

### 2) Make `compute_dnd_frame(...)` the canonical integration output

The UI-kit layer should stop re-computing `over` and auto-scroll manually. It should call the
headless engine/frame pipeline and forward those outputs as-is.

This reduces drift and makes `DndUpdate` truthful.

### 3) Shrink `fret-ui-kit::dnd/controller/` into smaller adapters

Recommended split:

- `controller_inputs.rs`: pointer event translation
- `controller_state.rs`: per-window/per-scope engine storage
- `controller_api.rs`: public helper entry points
- `forwarders.rs`: ergonomic UI wiring

The goal is not file-count vanity. The goal is to make future changes reviewable and harder to
mis-wire.

### 4) Move recipe-local "drag truth" into shared state where possible

Recipes may still need product-local state, but they should not need to duplicate:

- whether a drag is active
- which item is currently `over`
- ordered collision results

The engine/controller should provide that centrally.

### 5) Standardize first-party adoption on `DndPointerForwarders`

`DndPointerForwarders` already exists and should become the default integration path for new
pointer-driven surfaces. Hand-written forwarding should become the exception, not the norm.

## dnd-kit parity target (architectural, not API copy)

We should align to `dnd-kit` in phases.

### P0: Must align during this refactor

- central operation/engine truth
- truthful collision outputs
- thinner UI integration adapter
- consistent forwarder usage
- safer modifier behavior

### P1: Strong near-term extension targets

- richer pointer sensor configuration:
  - activator/handle targeting
  - prevent-activation hooks
  - pointer-kind-aware defaults
- droppable metadata beyond rect + disabled + z-index:
  - optional type/accept filters
  - collision strategy selection hooks
- engine-level monitoring/event stream for diagnostics and product policy hooks

### P2: Follow-on parity targets

- keyboard sensor
- continuous auto-scroll driver semantics
- richer sortable/group semantics
- optional plugin-style extension points where the output remains data-only
- multi-rect droppables if at least two real consumers need them

## Proposed parity map

| `dnd-kit` concept | Fret current state | Refactor direction |
| --- | --- | --- |
| Manager / operation owner | split across `fret-dnd` helpers and `fret-ui-kit` controller | add headless operation/engine in `fret-dnd` |
| Registry | present, but mostly UI-kit-owned snapshot lifecycle | keep lifecycle in UI-kit, strengthen headless snapshot consumption |
| Pointer sensor config | minimal | add configuration and future pointer-kind defaults |
| Keyboard sensor | not implemented by contract v1 | add as follow-on extension once engine seam is stable |
| Collision output | headless layer computes it, integration drops it | make headless output canonical |
| Modifiers | present but minimal | keep pure, make composition safer, fix panic edge cases |
| Auto-scroll | pure request computation only | keep pure in core, add optional driver semantics in integration layer |
| Sortable | minimal insertion helpers + recipe logic | move more sortable truth into shared engine/controller |
| Monitor / event stream | effectively ad-hoc | introduce a small, data-only monitor surface later |

## Regression protection plan

### Required gates for the first implementation phase

- `cargo nextest run -p fret-dnd`
- targeted `cargo nextest run -p fret-ui-kit` DnD tests
- `python tools/check_layering.py`

### New tests to add early

- modifier panic regression for oversized dragged rect vs bounds
- controller returns collisions in deterministic order
- `compute_dnd_frame(...)` and UI-kit `DndUpdate` stay aligned
- forwarder-based integration gate for at least one real surface

### Recommended first real-product gates

- sortable recipe gate
- Kanban DnD gate
- docking tab hover/insert gate
- carousel-vs-DnD arbitration gate stays green after forwarder adoption

## Deliverables

This workstream should leave behind:

1. a stable workstream design and phased tracker
2. a smaller, more truthful headless DnD architecture
3. at least one reusable integration path that first-party code actually uses
4. clear extension seams for future `dnd-kit` parity work

## Definition of done (v1)

We consider this workstream successful when:

- `fret-dnd` remains boundary-clean and becomes the owner of central operation truth
- `fret-ui-kit::dnd` stops fabricating partial updates and forwards full headless outputs
- the known panic edge case is fixed and guarded
- at least sortable, Kanban, and one more first-party surface consume the improved integration seam
- the resulting structure makes future parity work additive instead of requiring another rewrite
