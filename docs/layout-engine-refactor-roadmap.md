# Layout Engine Refactor Roadmap (Taffy Islands -> Window-Scoped Engine)

This document tracks the incremental refactor from container-owned "Taffy islands" to a
window-scoped layout engine with multiple viewport roots (multi-viewport docking).

This is a living roadmap (not an ADR). Contracts are defined by ADRs; this file tracks staged
deliverables and acceptance checks.

Primary ADRs:

- AvailableSpace + non-reentrant intrinsic measurement: `docs/adr/0115-available-space-and-non-reentrant-measurement.md`
- Window-scoped engine + viewport roots: `docs/adr/0116-window-scoped-layout-engine-and-viewport-roots.md`

Related constraints:

- Hybrid layout boundary + explicit barriers: `docs/adr/0035-layout-constraints-and-optional-taffy-integration.md`
- Declarative Flex semantics and defaults: `docs/adr/0057-declarative-layout-style-and-flex-semantics.md`
- Current perf hardening shape (container-owned persistent Taffy trees): `docs/adr/0076-declarative-layout-performance-hardening.md`
- Virtualization boundary: `docs/adr/0042-virtualization-and-large-lists.md`
- Viewport surfaces (RenderTargetId): `docs/adr/0007-viewport-surfaces.md`

## End-State Target ("C")

- One `TaffyLayoutEngine` per window (persistent across frames).
- Docking provides N viewports, each viewport is an independent layout root (no cross-viewport coupling).
- `AvailableSpace::{Definite, MinContent, MaxContent}` is preserved end-to-end; no "huge definite probe" approximations.
- Taffy `measure` callbacks are leaf-only and never re-enter layout; intrinsic size uses `measure_in`.
- Docking / scroll / virtualization / viewport surfaces remain explicit layout barriers.

## Invariants (Must Hold Throughout the Refactor)

1. `measure_in` is non-reentrant (must not call `layout_in`, directly or indirectly).
2. Percent/fill and `flex-grow` semantics only resolve against definite available space.
3. Barriers own their internal layout policy and never require solver-time re-entry into descendants.
4. Stable identity is `NodeId`; all engine node IDs are derived/cacheable from `NodeId`.
5. Debug builds fail fast on cycles; release builds degrade safely with rate-limited diagnostics.

## P1 - Constraint-Correct Intrinsic Measurement (ADR 0115)

Goal: stop semantic drift and recursion hazards by making `AvailableSpace` explicit and making
measurement leaf-only.

Deliverables:

- Add `AvailableSpace` + `LayoutConstraints` and plumb through internal layout plumbing.
- Add `measure_in` and implement it for required leaf primitives (text, images, svg, etc.).
- Refactor Flex/Grid Taffy measure callbacks to call `measure_in` (delete "huge probe bounds" fallback).
- Lock rules for `Length::Fill` and `flex-grow` under Min/MaxContent with tests.

Acceptance:

- `shadcn::Tabs` stack-overflow regression stays fixed without component-level "defensive defaults".
- Min/MaxContent no longer behaves like an enormous definite bound in measured layout.
- A minimal "invalid" composition does not crash; it produces a stable fallback size and diagnostic.

## P2 - Window-Scoped Engine Skeleton (ADR 0116, Single Root)

Goal: centralize the Taffy tree and incremental updates behind a window-scoped engine while keeping
behavior consistent.

Deliverables:

- Introduce `TaffyLayoutEngine` behind a feature flag (or internal switch).
- Provide request/build APIs (create/update nodes, styles, children, measured leaf hooks).
- Compute + apply protocol: build/update during frame build; compute once per root; write bounds back.
- Migrate Flex/Grid to request nodes into the engine (one root) while preserving existing writeback.

Acceptance:

- `cargo test` passes with and without the feature flag.
- Flex/Grid do not own per-container Taffy trees on the feature path.
- Measurement remains leaf-only and non-reentrant under the engine path.

## P3 - Multi-Viewport Roots + Docking Integration (Target "C")

Goal: treat docking-driven viewports as independent layout roots and prove non-coupling.

Deliverables:

- Define and plumb `ViewportRoot` descriptors (viewport id + origin + size + mounted root node).
- Compute each viewport root independently against its own definite available space.
- Apply results in window-local coordinates (viewport-local rect + origin composition).
- Add conformance tests ensuring no cross-viewport coupling (percent/fill/free-space).

Acceptance:

- Layout changes inside one viewport do not affect sibling viewport layout results.
- Viewport boundaries behave as layout barriers for percent/fill and free-space distribution.
- Performance: solve cost scales with per-viewport subtree size, not with total mounted roots.

## P4 - Widen the "Taffy Island" (Optional, Post-C)

Goal: reduce cross-boundary intrinsic measurement costs by turning more "pure layout" descendants
into real Taffy nodes.

Candidate migrations:

- `Stack`, nested `Flex`/`Grid`, and other "LayoutStyle-only" containers.
- Opt-in "contents-like" wrapper mode (validated, single-child, no geometry-affecting style).

## P5 - Diagnostics and Perf Tooling (Ongoing)

- Layout tree dump and per-root stats (node count, max depth, measure calls, solve time).
- Cycle detection counters and rate-limited warnings in release.
- Optional rounding / stack-safe guardrails aligned with GPUI.

