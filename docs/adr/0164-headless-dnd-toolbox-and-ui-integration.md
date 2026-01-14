---
title: "ADR 0164: Headless Drag-and-Drop Toolbox (`fret-dnd`) and UI Integration Registry"
---

# ADR 0164: Headless Drag-and-Drop Toolbox (`fret-dnd`) and UI Integration Registry

Status: Proposed

Scope: component ecosystem (`ecosystem/`), with constrained integration touchpoints into `fret-ui` and the runner.

Related:

- ADR 0041: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- ADR 0053: `docs/adr/0053-external-drag-payload-portability.md`
- ADR 0072: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- ADR 0011: `docs/adr/0011-overlays-and-multi-root.md`
- ADR 0017: `docs/adr/0017-multi-window-display-and-dpi.md`
- ADR 0165: `docs/adr/0165-pointer-identity-and-multi-pointer-capture.md`
- ADR 0166: `docs/adr/0166-multi-pointer-drag-sessions-and-routing-keys.md`

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

### 3) Treat internal drag routing as mechanism-only; toolbox owns policy

The runtime continues to route `Event::InternalDrag` using hit-testing and the existing internal-drag anchor override
mechanism (`InternalDragRouteService`) when needed (e.g. docking tear-off).

The toolbox does not change routing rules; it consumes a geometry registry snapshot and computes “over/collisions”.

### 4) Multi-pointer readiness is a first-class requirement

The toolbox engine is defined in terms of a `DndPointerKey` (opaque, comparable), so that multiple concurrent
pointers can be supported in the future without redesigning the engine APIs.

The core pointer identity contract is defined by ADR 0165. Initial integrations may remain “single active drag
session” while we validate the crate boundaries and registry shape; however, no `fret-dnd` API should assume
“only one pointer exists”.

### 5) Avoid future dead-ends: drag kind routing must be extensible

Cross-window internal-drag routing sometimes requires a stable anchor node per “drag kind” (docking today).
The routing key must be extensible beyond a closed `enum` to avoid future rewrites when ecosystem DnD flows
need the same mechanism.

Proposed direction (final shape TBD during implementation):

- replace `DragKind` with an extensible ID newtype (e.g. `DragKindId(u64)`), with reserved constants for built-ins
  (e.g. docking),
- or keep `DragKind` but add an ID-carrying variant (e.g. `Custom(u64)`).

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

## Open Questions

- (Resolved by ADR 0165) Pointer identity and per-pointer capture/dispatch is a prerequisite for true multi-pointer
  DnD and gesture coexistence.
- How do we represent multiple rects per droppable (e.g. complex shapes, ports) without over-complicating the core
  snapshot format?
- Do we want a shared “sortable preset” crate in the ecosystem, or keep it as a `fret-ui-kit` recipe layer?
