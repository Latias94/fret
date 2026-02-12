---
title: "ADR 0149: Headless Drag-and-Drop Toolbox (`fret-dnd`) and UI Integration Registry"
---

# ADR 0149: Headless Drag-and-Drop Toolbox (`fret-dnd`) and UI Integration Registry

Status: Proposed

Scope: component ecosystem (`ecosystem/`), with constrained integration touchpoints into `fret-ui` and the runner.

Related:

- ADR 0041: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- ADR 0053: `docs/adr/0053-external-drag-payload-portability.md`
- ADR 0072: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- ADR 0011: `docs/adr/0011-overlays-and-multi-root.md`
- ADR 0017: `docs/adr/0017-multi-window-display-and-dpi.md`
- ADR 0150: `docs/adr/0150-pointer-identity-and-multi-pointer-capture.md`
- ADR 0151: `docs/adr/0151-multi-pointer-drag-sessions-and-routing-keys.md`

## Context

Fret already defines hard-to-change *mechanisms* for editor-grade drag-and-drop:

- a portable platform boundary for external OS drag-and-drop (token-based; ADR 0053),
- an app-scoped internal drag session concept (ADR 0041),
- cross-window internal-drag routing performed by the runner (ImGui-style “hovered viewport/window”),
- deterministic arbitration between docking drags, overlays, and viewport capture (ADR 0072).

However, many ecosystem surfaces need a reusable *policy/toolbox* layer similar to `dnd-kit`:

- activation constraints (delay/distance/tolerance),
- collision detection strategies (pointer-within, rect intersection, closest center/corners),
- modifiers (axis locks, snapping, bounds clamping),
- auto-scroll requests (scroll containers during drag),
- reusable “sortable/reorder” patterns.

Today these policies tend to be re-implemented per subsystem (docking, canvas/node graph, overlay gestures),
which increases drift risk and makes future multi-input targets (touch/pen/mobile) expensive to integrate.

We want to keep `crates/fret-ui` as a mechanism layer (ADR 0066) and avoid moving policy-heavy DnD logic into the
runtime or runner, while still enabling multi-window and multi-viewport editor workflows.

## Goals

1. Provide a headless, reusable DnD toolbox that can be shared across ecosystem crates (node graph, canvas,
   sortable lists/tables, docking-adjacent interactions).
2. Keep the runtime contracts stable: internal vs external DnD separation remains (ADR 0041 / ADR 0053).
3. Support multi-window and docking cross-window hover/drop without introducing runner-level widget special-cases.
4. Be multi-pointer ready (mouse/touch/pen), without forcing immediate, breaking changes across all existing widgets.
5. Make the integration model “use-case driven”: start with a small set of real integrations and extend only when
   those integrations demand it.

## Non-Goals

- Defining a UI component library for draggable/droppable widgets in `crates/fret-ui`.
- Merging internal drag sessions and external OS drag-and-drop into one model.
- Encoding editor/app policy decisions (asset import, docking graph mutation) in the toolbox.
- Solving accessibility narration/keyboard sensors in the first iteration (these can be layered later).

## Decision

### 1) Add a new headless crate: `ecosystem/fret-dnd`

`fret-dnd` provides reusable, policy-heavy primitives:

- sensors + activation constraints,
- collision detection strategies and sorting,
- modifiers/constraints on translations,
- auto-scroll request computation (outputs requests; does not perform scrolling).

`fret-dnd` is headless and should depend only on `fret-core` geometry/types (or a minimal shared geometry surface).
It must not depend on `fret-ui` or runner crates.

### 2) Add a UI integration surface in `ecosystem/fret-ui-kit`

`fret-ui-kit` provides the glue between the runtime and the headless engine:

- a per-window, per-frame registry for droppable/draggable geometry (`DndRegistryService`),
- a controller that converts Fret events to sensor inputs and produces per-frame DnD outputs (`DndController`),
- optional “recipes” that build higher-level patterns (sortable/reorder, drop zones, drag overlays) on top.

This keeps the runtime clean while making the common integrations easy.

### 2.1) Lock a canonical coordinate space for the registry snapshot

The integration registry (whether stored app-scoped or component-scoped) uses **window-local logical pixels**
as its canonical space (ADR 0017):

- droppable/draggable rects are expressed in window-local logical px,
- pointer positions fed into sensors/collision are window-local logical px,
- higher-level surfaces (canvas, 3D viewports) must perform their own mapping to/from their internal spaces.

Rationale:

- window-local logical px is already the stable cross-window routing coordinate for editor-grade input,
- it keeps the toolbox independent from any particular scene graph / transform stack,
- it avoids “hidden” DPI coupling in policy code.

### 2.2) Activation delay is expressed in deterministic ticks

Activation delay is defined in terms of **monotonic ticks** (`TickId`, runner-owned), not wall-clock time:

- `ActivationConstraint::DelayTicks { ticks: u64 }` is evaluated against `TickId` deltas,
- distance thresholds are evaluated in window-local logical px.

This keeps drag activation deterministic and testable across runners/platforms.

Note: component-owned pointer hooks must expose `pointer_id` and `tick_id` so policy code can be written
without coupling to a concrete `UiHost` implementation.

### 2.3) Registry geometry is based on the last completed layout (1-frame lag is allowed)

The `fret-ui-kit` integration registry is populated from **last-known layout geometry** (the runtime's
"prev-bounds" snapshot), not from same-frame layout execution order:

- the registry may be empty on the first frame a subtree mounts,
- on interactive frames, the registry is expected to be available after at most one frame of layout+paint,
- integrations must tolerate a one-frame lag between a layout change and collision geometry updates.

Rationale:

- keeps the mechanism layer (`fret-ui`) small (ADR 0066) by avoiding same-frame geometry queries/hooks,
- keeps DnD policy deterministic and testable (no dependency on layout scheduling),
- matches the repository philosophy: pay small latency early to avoid a later, hard-to-reverse contract expansion.

If a future use-case requires same-frame geometry (e.g. drag overlays tightly coupled to layout), it must be
introduced via an explicit ADR that scopes the added `fret-ui` contract surface.

### 2.4) Sortable/reorder semantics are defined as `array_move(from, to)`

The canonical sortable/reorder outcome is an **index move**:

- `active`: the item being dragged
- `over`: the collision-selected target item
- on drop: reorder is performed as `array_move(active_index, over_index)`

This is the default semantic for `fret-ui-kit` recipes. More specialized semantics (e.g. before/after insertion
lines, sectioned lists, pinned columns) may be layered as higher-level recipes but must remain compatible with
this base interpretation.

### 2.5) DnD registries are scoped within a window (`DndScopeId`)

Within a single window, multiple independent DnD workflows may coexist (e.g. a canvas with ports, a tree view,
and a sortable list). To prevent unrelated droppable sets from influencing collision results, `fret-ui-kit`'s
registry is partitioned by an explicit scope ID:

- `DndScopeId` is an opaque `u64` identifier
- the default scope is `DndScopeId(0)`
- recipes and component integrations should compute a stable scope per component instance (e.g. based on the
  declarative root element id)

This keeps the mechanism layer (`fret-ui`) unchanged while avoiding a later, hard-to-reverse expansion to support
same-frame geometry or cross-component coordination.

### 3) Treat internal drag routing as mechanism-only; toolbox owns policy

The runtime continues to route `Event::InternalDrag` using hit-testing and the existing internal-drag anchor override
mechanism (`fret_ui::internal_drag`) when needed (e.g. docking tear-off).

The toolbox does not change routing rules; it consumes a geometry registry snapshot and computes “over/collisions”.

### 3.1) Only cross-window workflows require `DragSession`

`DragSession` (ADR 0041 / ADR 0151) is used only when an interaction requires **runner-assisted cross-window
hover/drop routing** or other internal-drag semantics (tear-off docking-class flows).

Window-local reorder/sortable interactions should remain component-local (pointer capture + headless sensor),
and must not require entering the app-scoped internal drag session mechanism.

### 4) Multi-pointer readiness is a first-class requirement

The toolbox engine is defined in terms of a `DndPointerKey` (opaque, comparable), so that multiple concurrent
pointers can be supported in the future without redesigning the engine APIs.

The core pointer identity contract is defined by ADR 0150. Initial integrations may remain “single active drag
session” while we validate the crate boundaries and registry shape; however, no `fret-dnd` API should assume
“only one pointer exists”.

### 5) Avoid future dead-ends: drag kind routing must be extensible

Cross-window internal-drag routing sometimes requires a stable anchor node per “drag kind” (docking today).
The routing key must be extensible beyond a closed `enum` to avoid future rewrites when ecosystem DnD flows
need the same mechanism.

Direction (implemented by ADR 0151):

- Replace `DragKind` with an extensible ID newtype `DragKindId(u64)`, with reserved constants for built-ins
  (e.g. docking).

### 6) Determinism requirements (collision + tie-breaks)

To avoid "jitter" and hard-to-debug divergence across platforms:

- collision outputs must be **stably ordered** (explicit `z_index` groups first, then stable ids),
- activation constraints must be deterministic (ticks, not wall-clock),
- per-pointer state must never "jump" across pointers (all ownership is keyed by `PointerId`).

## Design (Illustrative)

### A) `fret-dnd` vocabulary

The headless crate defines minimal, engine-agnostic concepts:

- `DndPointerKey`: opaque pointer identity (allows multi-pointer).
- `DndItemId`: stable identifier for draggables/droppables (u64 or newtype).
- `DndRect`: a rectangle in a caller-defined coordinate space (typically window-local logical px or canvas space).
- `DndCollision`: `(id, score)` where `score` is algorithm-specific (distance or intersection ratio).
- `CollisionDetection`: `fn(snapshot) -> Vec<DndCollision>` (sorted).
- `ActivationConstraint`: delay/distance/tolerance constraints.
- `Modifier`: `fn(translation, ctx) -> translation`.

The toolbox must not bake in “UI node ids” or “layout tree” assumptions.

### B) Registry snapshot model (produced by `fret-ui-kit`)

To support collision detection beyond the runtime hit-test (e.g. canvas ports that are not discrete UI nodes),
widgets can register explicit droppable regions:

- `draggable`: id, initial rect, optional drag overlay rect, optional metadata for policy layers
- `droppable`: id, rect(s), disabled flag, z-order group (overlay-first semantics), optional metadata

The registry is window-scoped and frame-scoped:

- it is rebuilt every frame (or logically treated as rebuilt) to reflect layout/scroll/transform changes,
  - it can be stored as a service and reused via a `revision` key to avoid allocations, but consumers must treat it
  as an immutable snapshot for collision computation.

Timing note:

- when the registry is derived from declarative element geometry, it is expected to be sourced from the runtime's
  last completed layout snapshot ("prev-bounds") rather than same-frame layout hooks (see §2.3).

### C) Controller responsibilities (in `fret-ui-kit`)

`DndController`:

1. Consumes `Event::Pointer` and/or `Event::InternalDrag` to feed sensor inputs.
2. Uses the registry snapshot and selected collision strategies to compute `over` and `collisions`.
3. Emits outputs suitable for policy layers:
   - `DndFrame { active, over, collisions, phase }`
   - optional `AutoScrollRequest { axis, speed, window/container }`
4. Does not mutate app state directly; instead it returns outputs that a component can translate into models/effects
   (e.g. reorder ops, canvas edge insertion, docking ops).

### D) Multi-window and cross-window hover/drop

Cross-window drag is still achieved by runner routing (ADR 0041), which already synthesizes internal drag events to
windows under the cursor.

The toolbox does not need to know about OS window rectangles; it operates on the per-window registry snapshot and
the per-window internal drag stream.

### E) Arbitration alignment

DnD integrations must honor existing arbitration rules:

- Docking interaction precedence and overlay behavior remains locked by ADR 0072.
- Modal barriers (ADR 0011) intentionally block underlying drop targets.

The toolbox does not change arbitration rules; it only provides computed state to whichever subsystem owns the
interaction at that moment.

## Migration / Rollout Plan (Use-Case Driven)

1. Land `ecosystem/fret-dnd` with a minimal engine surface:
   - activation constraints,
   - at least two collision strategies (pointer-within + closest-center),
   - one modifier (axis lock) and one utility (clamp-to-rect).
2. Add `fret-ui-kit` integration as an internal module behind an “unstable” feature until at least two real
   integrations exist.
3. Validate with two ecosystem use-cases:
   - canvas/node graph port hit-testing or edge insertion drag,
   - sortable/reorder (lists or table headers/rows).
4. Only then consider:
   - richer modifiers,
   - auto-scroll implementation hooks,
   - exposing stable public surfaces in `fret-ui-kit`.

## Current Implementation (MVP)

- Headless toolbox: `ecosystem/fret-dnd/` (activation constraints, collision strategies, modifiers, auto-scroll).
- UI integration glue (initial): `ecosystem/fret-ui-kit/src/dnd.rs` (window-scoped registry snapshot + per-kind sensors).
- Use-case A (canvas/node graph): insert-node drag wiring in
  `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag.rs`.
- Use-case B (sortable/reorder): minimal declarative recipe in
  `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`.

## Open Questions

- (Resolved by ADR 0150) Pointer identity and per-pointer capture/dispatch is a prerequisite for true multi-pointer
  DnD and gesture coexistence.
- How do we represent multiple rects per droppable (e.g. complex shapes, ports) without over-complicating the core
  snapshot format?
- Do we want a shared “sortable preset” crate in the ecosystem, or keep it as a `fret-ui-kit` recipe layer?
- When do we need a dedicated "before/after insertion line" semantic beyond `array_move`, and what additional
  inputs does it require (pointer half, drag direction, list axis, virtualization window)?
