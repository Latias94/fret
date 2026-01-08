# Layout Engine Refactor Roadmap (P1-P3)

This document tracks the incremental refactor from container-owned "Taffy islands" to a
window-scoped layout engine with multiple independent viewport roots (multi-viewport docking).

This is a living roadmap (not an ADR). Contracts are defined by ADRs; this file tracks staged
deliverables and acceptance checks.

Primary ADRs:

- AvailableSpace + non-reentrant intrinsic measurement: `docs/adr/0115-available-space-and-non-reentrant-measurement.md`
- Window-scoped engine + viewport roots: `docs/adr/0116-window-scoped-layout-engine-and-viewport-roots.md`

Related constraints and boundaries:

- Hybrid layout boundary + explicit barriers: `docs/adr/0035-layout-constraints-and-optional-taffy-integration.md`
- Declarative Flex semantics and defaults: `docs/adr/0057-declarative-layout-style-and-flex-semantics.md`
- Container-owned persistent Taffy trees (current perf hardening): `docs/adr/0076-declarative-layout-performance-hardening.md`
- Virtualization boundary: `docs/adr/0042-virtualization-and-large-lists.md`
- Viewport surfaces (RenderTargetId): `docs/adr/0007-viewport-surfaces.md`

GPUI reference (implementation, not contract):

- `repo-ref/zed/crates/gpui/src/taffy.rs`

## End-State Target ("C")

- One `TaffyLayoutEngine` per window (persistent across frames).
- Docking provides N viewports, each viewport is an independent layout root (no cross-viewport coupling).
- `AvailableSpace::{Definite, MinContent, MaxContent}` is preserved end-to-end; no "huge definite probe" approximations.
- Taffy `measure` callbacks are leaf-only and never re-enter layout; intrinsic size uses `measure_in`.
- Docking / scroll / virtualization / viewport surfaces remain explicit layout barriers.

## Invariants (Must Hold Throughout the Refactor)

1. `AvailableSpace::{Definite, MinContent, MaxContent}` is preserved end-to-end (no `1e9` probes).
2. Taffy measure callbacks must not re-enter subtree layout (`measure_in` only; no `layout_in`).
3. Percent/fill and `flex-grow` semantics only resolve against definite available space.
4. Barriers own their internal layout policy and never require solver-time re-entry into descendants.
5. Stable identity is `NodeId`; engine node IDs are derived/cacheable from `NodeId`.
6. Debug builds fail fast on cycles; release builds degrade safely with rate-limited diagnostics.

## Status Legend

- **Done**: merged and relied upon by production code paths.
- **In progress**: implemented in a worktree/branch; subject to iteration.
- **Planned**: agreed direction, not implemented yet.

## Current Progress Snapshot

- P1: `AvailableSpace` + non-reentrant `measure_in`. (**Done**; merged.)
- P2: window-scoped engine skeleton behind `fret-ui/layout-engine-v2`. (**In progress**; iterating in `wt-layout-engine2`.)
- P3: multi-viewport roots + engine-backed flow migration. (**In progress**; viewport-root plumbing + conformance tests landed; more migration remains.)

Update this section by editing this file (avoid scattering progress notes across ADRs).

## P1: Constraint-Correct Intrinsic Measurement (ADR 0115)

Goal: stop semantic drift and recursion hazards by making `AvailableSpace` explicit and making
measurement leaf-only.

Deliverables:

- Add `AvailableSpace` + `LayoutConstraints` and plumb through internal layout plumbing.
- Add `measure_in` and implement it for required leaf primitives (text, images, svg, etc.).
- Refactor Flex/Grid Taffy measure callbacks to call `measure_in` (delete "huge probe bounds" fallback).
- Lock rules for `Length::Fill` and `flex-grow` under Min/MaxContent with tests.

Acceptance:

- `cargo test -p fret-ui` passes.
- A minimal regression composition for "Auto main axis + flex-1/fill" does not stack overflow.

## P2: Window-Scoped Layout Engine Skeleton + Two-Phase Protocol (ADR 0116)

Goal: introduce a per-window `TaffyLayoutEngine` without changing behavior by default, and enforce
the separation between "build/request" and "compute/apply".

Deliverables:

- Feature flag: `fret-ui/layout-engine-v2` (default off).
- One engine instance per window, persistent across frames.
- Stable `NodeId -> LayoutId` mapping and incremental updates (`mark_dirty` on invalidation).
- APIs:
  - `request_layout(style, children) -> LayoutId`
  - `request_measured_layout(style, measure_fn) -> LayoutId` (leaf intrinsic measurement only)
- Frame protocol:
  - `engine.begin_frame(frame_id)`
  - request/build during declarative build (no bounds writes)
  - `compute_root_with_measure(root, available, measure_cb)` per root
  - apply results back into retained `UiTree` bounds

Acceptance:

- With feature off: no behavior change; tests pass.
- With feature on: empty graphs and small trees solve; tests pass.

## P3: Multi-Viewport Roots + Flow Migration (End-State Convergence)

Goal: evolve "taffy islands" so that layout-style-only descendants become nodes in the same Taffy
tree, while docking-defined viewports are independent roots and explicit barriers remain outside.

Deliverables:

- Docking provides definite viewport rects and registers viewport roots.
- Engine computes each viewport root independently (no cross-viewport percent/free-space coupling).
- Apply composes viewport-local rects into window-local bounds.
- Flex/Grid/Stack/Container wrappers become nodes in the same Taffy tree when possible; `measure_in`
  is reserved for true leaves.
- Add stacksafe execution around solves + measure callbacks; enable optional rounding to reduce drift.

Acceptance:

- Conformance test: no cross-viewport coupling for percent/flex distribution.
- Docking demos behave consistently across DPI scales (bounds stable within rounding policy).

## Open Decisions (Track Here)

1. **"Contents-like" wrappers**: **Decision (v1)**: no general-purpose contents-like / `Slot/asChild` prop merging (ADR 0117). Prefer GPUI-aligned single-root components; if needed later, add a restricted, validated "layout-transparent wrapper" opt-in (layout-only, no prop merging).
2. **Root solve orchestration**: which roots we precompute (viewport only vs additional layer roots) and the required ordering relative to overlay roots (ADR 0011, ADR 0064).
3. **Engine cache keys + invalidation**: exact environment keys for measurement caching (scale factor, theme revision, font stack key, model revisions).
4. **Rounding policy**: whether/where to enable engine-level pixel rounding, and how it composes with hit-testing and paint.
5. **"Not all containers use Taffy" boundary**: list-like/virtualized surfaces and other perf-critical containers that should remain specialized algorithms, and the conformance tests that keep them interoperable.
