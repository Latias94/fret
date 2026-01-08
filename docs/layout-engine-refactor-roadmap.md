# Declarative Layout Engine Refactor Roadmap (P1–P5)

This document tracks the incremental refactor from “container-owned Taffy islands” toward the
end-state described in:

- `docs/adr/0115-available-space-and-non-reentrant-measurement.md`
- `docs/adr/0116-window-scoped-layout-engine-and-viewport-roots.md`

It is intentionally **implementation-facing** and may be updated as we learn, while ADRs remain the
contract source of truth.

## Status Legend

- **Done**: merged and relied upon by production code paths.
- **In progress**: implemented in a worktree/branch; subject to iteration.
- **Planned**: agreed direction, not implemented yet.

## Non-Negotiable Constraints (Locked by ADR)

1. `AvailableSpace::{Definite, MinContent, MaxContent}` must be preserved end-to-end (no `1e9` probes).
2. Taffy measure callbacks must not re-enter subtree layout (`measure_in` only; no `layout_in`).
3. Layout barriers remain explicit systems: docking/scroll/virtualization/viewport surfaces.
4. Docking-defined viewports are independent layout roots (no cross-viewport percent/free-space coupling).

## Current Progress Snapshot

- P0: `AvailableSpace` + non-reentrant `measure_in` land and stabilize. (**In progress**, see `wt-layout-engine2`.)
- P1+: window-scoped engine and viewport roots. (**Planned**.)

Update this section by editing this file (avoid scattering progress notes across ADRs).

## P0 — Semantic Grounding (AvailableSpace + Non-Reentrant Measurement)

**Goal**: make intrinsic measurement constraint-correct and non-reentrant so “Auto main axis + flex-1/fill”
does not create feedback loops or stack overflows.

**Deliverables**

- `AvailableSpace`/`LayoutConstraints` types in `crates/fret-ui`.
- `UiTree::measure_in` + cycle guard (debug panic, release fallback).
- Flex/Grid Taffy measure callback calls `measure_in`, not `layout_in`.
- Remove “huge definite probe” (`Px(1.0e9)`) from intrinsic measurement paths.

**Acceptance**

- `cargo test -p fret-ui` passes.
- `cargo test -p fret-ui-shadcn tabs_layout_regression_does_not_stack_overflow_in_auto_sized_column` passes.

## P1 — Window-Scoped Layout Engine Skeleton (Feature-Flagged)

**Goal**: introduce a per-window `TaffyLayoutEngine` without changing behavior by default.

**Deliverables**

- Feature flag: `fret-ui/layout-engine-v2` (default off).
- `TaffyLayoutEngine` type (per-window) with:
  - stable `NodeId -> LayoutId` mapping,
  - per-frame `begin_frame/end_frame`,
  - `request_layout` / `request_measured_layout` APIs,
  - basic debug stats hooks.
- Engine stored in `crates/fret-ui` window-scoped state (not in `fret-launch`).

**Acceptance**

- With feature off: no behavior change; tests pass.
- With feature on: engine can build an empty graph and no-op safely; tests pass.

## P2 — Two-Phase Protocol (Build vs Compute/Apply)

**Goal**: enforce the separation:

- **Build/request**: construct/update the layout graph (no bounds writes).
- **Compute/apply**: solve roots and write bounds to the retained tree.

**Deliverables**

- Declarative frame pipeline calls:
  - `engine.begin_frame(...)`
  - `request_*` during build/prepaint
  - `compute_root(...)` per root
  - `apply(...)` to write bounds
- Explicit environment keys in caches (scale factor, theme revision, font stack key).

**Acceptance**

- Conformance assertion: no bounds writes occur during build/request.
- Debug stats report per-root node counts and solve time.

## P3 — Migrate Declarative Flow Containers Into the Engine

**Goal**: “taffy islands” evolve so that layout-style-only descendants become Taffy nodes and
`measure_in` is reserved for true leaves.

**Deliverables**

- Flex/Grid/Row/Column/Stack/Container wrappers become nodes in the same Taffy tree when possible.
- Wrapper policy:
  - default: wrappers are layout boxes in Taffy,
  - optional validated “contents-like” mode (single child; no geometry-affecting properties).
- Lock constraint-phase semantics with tests:
  - `Length::Fill` resolves only under definite axis; under Min/MaxContent behaves as `auto`.
  - `flex-grow` distribution only under definite main-axis available space.

**Acceptance**

- New tests cover “Auto main axis + flex-1/fill” compositions without stack overflows.
- Reduce number of per-frame Taffy solves for typical shadcn recipes (measurable via stats).

## P4 — Barriers + Multi-Viewport Roots (Final Shape)

**Goal**: integrate multi-viewport docking by treating each viewport as an independent root, while
keeping explicit barrier containers outside Taffy.

**Deliverables**

- Docking provides definite viewport rects.
- Engine computes each viewport root independently.
- Apply composes viewport-local rects into window-local bounds.
- Barriers interop contract:
  - scroll uses intrinsic `MaxContent` to measure content extent,
  - virtualization only lays out visible items (+ overscan),
  - viewport surfaces remain explicit (ADR 0007).

**Acceptance**

- Conformance test: no cross-viewport percent/flex coupling.
- Docking demos behave consistently across DPI scales.

## P5 — Performance + Diagnostics + Removal of Legacy Paths

**Goal**: make the engine the default and remove legacy islands where practical.

**Deliverables**

- stacksafe execution around solve + measure callbacks (debug safety + release stability).
- Optional rounding in engine to reduce subpixel drift.
- Rate-limited diagnostics for cycle fallback in release.
- Remove old per-container `TaffyTree` storage for flow containers (where migrated).

**Acceptance**

- Feature flag defaults to on (or legacy path removed).
- `cargo clippy --workspace --all-targets -- -D warnings` passes.
- `cargo nextest run` (or `cargo test --workspace`) passes on CI targets.

