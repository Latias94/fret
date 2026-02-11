---
title: "ADR 0157: Headless DnD v1 Contract Surface (Sensors, Registry, Collisions)"
---

# ADR 0157: Headless DnD v1 Contract Surface (Sensors, Registry, Collisions)

Status: Proposed

Scope: public contract surface of `ecosystem/fret-dnd` (headless policy toolbox) and its narrow expectations
for UI integration layers (typically `ecosystem/fret-ui-kit`) and ecosystem consumers (docking, workspace tabs,
canvas/node graphs).

Related:

- ADR 0041: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- ADR 0072: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- ADR 0149: `docs/adr/0149-headless-dnd-toolbox-and-ui-integration.md`
- ADR 0150: `docs/adr/0150-pointer-identity-and-multi-pointer-capture.md`
- ADR 0151: `docs/adr/0151-multi-pointer-drag-sessions-and-routing-keys.md`
- ADR 0154: `docs/adr/0154-ecosystem-crate-taxonomy-glue-and-ui-kit-split-v1.md`
- ADR 0155: `docs/adr/0155-docking-tab-dnd-contract.md`

## Context

Fret already defines “hard to change later” *mechanisms* for editor-grade drag-and-drop:

- internal, app-scoped drag sessions and cross-window routing (ADR 0041),
- multi-window and DPI/coordinate semantics (ADR 0017),
- deterministic arbitration between docking drags, overlays, and viewport capture (ADR 0072).

What remains expensive to maintain without an explicit contract is the reusable *policy layer*:

- activation constraints (delay, distance, tolerance),
- collision strategies (pointer-within, closest-center, rect intersection),
- deterministic ordering and tie-breakers,
- modifiers (axis lock, clamping),
- auto-scroll request computation.

This is the “dnd-kit” class of abstraction: headless, reusable, but shared across multiple subsystems.
Without a stabilized surface, each subsystem tends to re-invent slightly different semantics and drift
over time.

ADR 0149 established the architectural decision (“a headless toolbox crate + UI integration glue”).
This ADR locks the v1 **contract surface and invariants** to avoid future, large refactors as more
use-cases adopt the toolbox.

## Goals

1. Provide a stable, reusable headless DnD policy surface that can be shared across ecosystem crates.
2. Preserve determinism: identical event streams produce identical outputs (including tie-breaks).
3. Be multi-pointer ready: no global single-pointer assumptions.
4. Keep dependency and layering boundaries clean (no `fret-ui` or runner deps in the toolbox).
5. Keep the surface use-case driven: only add features once at least two consumers need them.

## Non-Goals

- Defining a widget/component API in `crates/fret-ui`.
- Unifying internal drag sessions with external OS drag-and-drop.
- Supporting arbitrary shapes/paths as first-class droppables in v1 (rects only).
- Defining keyboard-driven DnD sensors in v1 (can be layered later for accessibility).

## Decision

### 1) `fret-dnd` remains a headless “primitives” crate

`ecosystem/fret-dnd`:

- depends only on `fret-core` geometry/types and IDs,
- MUST NOT depend on `fret-ui`, runner crates, winit/wgpu, or platform-specific code,
- exports policy primitives and data-only snapshots; it does not mutate app state or perform side effects.

This matches the “primitives vs kit” taxonomy (ADR 0154):

- `fret-dnd`: headless primitives
- `fret-ui-kit`: integration glue + ergonomic recipes

### 2) Canonical coordinate contract for UI integration: window-local logical pixels

When used with `fret-ui` input and layout, the canonical coordinate space for:

- pointer positions fed into sensors/collision, and
- droppable/draggable rects stored in registry snapshots

is **window-local logical pixels** (ADR 0017).

Note: domain surfaces (e.g. a 2D canvas inside a viewport) may maintain their own local coordinate
spaces, but they must perform their own mapping and treat `fret-dnd` as operating in that explicit space.

### 3) Registry snapshot contract (rect droppables with deterministic ordering)

The v1 registry snapshot is an immutable, data-only structure:

- `DndItemId(u64)` is the stable identifier for draggables/droppables.
- `Droppable` is rect-based and contains:
  - `id: DndItemId`
  - `rect: Rect`
  - `disabled: bool`
  - `z_index: i32`

Invariants:

- **Deterministic tie-breakers**: when two droppables are equally eligible, collisions MUST be ordered by:
  1) higher `z_index` first, then
  2) lower `DndItemId` first.
- Consumers MUST treat the snapshot as immutable for the duration of collision computation.

### 4) Sensor contract (multi-pointer, activation, and cancel semantics)

The v1 sensor surface is pointer-based:

- input events are keyed by `PointerId` (ADR 0135/0151),
- sensors maintain per-pointer state (no global single-pointer state),
- activation constraints MAY combine delay and distance:
  - distance is expressed in logical pixels (float) and MUST clamp negatives to `0`,
  - non-finite thresholds (NaN/inf) MUST be treated as `0` (immediate satisfaction).

Cancel semantics:

- `Cancel` MUST end tracking for the pointer (even if not active).
- `Up` MUST end tracking for the pointer.

This aligns with editor-grade robustness requirements (pointer capture loss, window destroy, gesture cancel).

### 5) Collision strategies contract (stable outputs)

The v1 toolbox includes at least:

- `PointerWithin`: collisions are droppables whose rect contains the pointer position.
- `ClosestCenter`: collisions are all enabled droppables sorted by squared distance to rect center.

Ordering MUST be stable and deterministic:

- `PointerWithin`: ordered by `z_index` desc, then `DndItemId` asc.
- `ClosestCenter`: ordered by distance asc, then `DndItemId` asc.

### 6) Modifiers and auto-scroll are pure outputs (no side effects)

Modifiers (axis locks, clamp-to-rect) and auto-scroll computations:

- are pure functions,
- return transformed translations or “requests” (data-only),
- MUST NOT perform scrolling, layout mutation, or model updates directly.

### 7) Sortable insertion semantics (before/after)

For sortable/reorder use-cases, the toolbox defines a stable, UI-agnostic “insertion line” semantic:

- `InsertionSide::{Before,After}` is computed by splitting the *over* droppable’s rect into halves
  along a caller-provided axis:
  - `Axis::X`: left half → `Before`, right half → `After`
  - `Axis::Y`: top half → `Before`, bottom half → `After`

This is intentionally minimal and deterministic. Higher-level layers may build richer policies
(e.g. insertion lines, ghost items, RTL support, axis locks) on top.

### 7) Multi-window composition remains owned by internal drag routing

Cross-window DnD remains owned by the runner/runtime internal drag routing (ADR 0041). The headless toolbox
operates **per window**:

- the UI integration layer supplies per-window pointer positions and per-window registry snapshots,
- the toolbox produces per-window `over/collisions` outputs,
- subsystem-specific code (docking, workspace tabs, node graph) translates those outputs into model ops/effects.

`fret-dnd` does not reason about OS window rectangles or global pointer state.

## Consequences

### Benefits

- Deterministic “policy” behavior shared across docking, workspace tabs, and canvas/node graphs.
- Multi-pointer readiness is baked into the contract early.
- Clear layering: runtime/runner keep owning cross-window routing; ecosystem owns DnD policy.

### Costs

- Rect-only droppables are limiting for some domains (ports, curved shapes); domains must approximate with rects in v1.
- Some repetition remains in UI integration until a single registry/controller is widely adopted.

## Rollout Plan (Use-Case Driven)

1. Keep docking tab dragging aligned (ADR 0155) and reuse rect-based collision selection.
2. Adopt the same primitives for workspace tab reordering and at least one canvas/node-graph interaction.
3. Only then extend:
   - multiple-rect droppables,
   - richer collision strategies and modifiers,
   - keyboard sensors / accessibility affordances.

## Implementation Notes (Evidence)

- Headless toolbox modules: `ecosystem/fret-dnd/src/{activation.rs,collision.rs,modifier.rs,registry.rs,scroll.rs,sortable.rs}`.
- Deterministic rect picking helper: `ecosystem/fret-dnd/src/rect_index.rs`.
- Sortable insertion helpers: `ecosystem/fret-dnd/src/sortable.rs`.
- Docking consumers: hover collisions and tab insertion index (`ecosystem/fret-docking/src/dock/space.rs`), end-to-end insert index assertion via `InternalDrag` (`ecosystem/fret-docking/src/dock/tests.rs`).

## Open Questions

1. Do we need “multi-rect droppables” in the core snapshot, or should that remain a domain-level abstraction?
2. When should we introduce keyboard sensors and focus semantics (a11y baseline), and where should those live?
3. When do we need insertion semantics beyond `InsertionSide` (e.g. insertion lines independent of droppable rects, RTL, virtualization windows)?
