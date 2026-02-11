# Layout Engine Refactor Roadmap (P1-P3)

This document tracks the incremental refactor from container-owned "Taffy islands" to a
window-scoped layout engine with multiple independent viewport roots (multi-viewport docking).

This is a living roadmap (not an ADR). Contracts are defined by ADRs; this file tracks staged
deliverables and acceptance checks.

Primary ADRs:

- AvailableSpace + non-reentrant intrinsic measurement: `docs/adr/0113-available-space-and-non-reentrant-measurement.md`
- Window-scoped engine + viewport roots: `docs/adr/0114-window-scoped-layout-engine-and-viewport-roots.md`
- Migration inventory (living checklist): `docs/layout-engine-v2-migration-inventory.md`

Related constraints and boundaries:

- Hybrid layout boundary + explicit barriers: `docs/adr/0035-layout-constraints-and-optional-taffy-integration.md`
- Declarative Flex semantics and defaults: `docs/adr/0057-declarative-layout-style-and-flex-semantics.md`
- Container-owned persistent Taffy trees (historical; superseded by the window-scoped engine): `docs/adr/0076-declarative-layout-performance-hardening.md`
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
- P2: window-scoped engine skeleton (layout engine v2; enabled by default in-repo). (**In progress**; iterating in `wt-layout-engine2`.)
- P3: multi-viewport roots + engine-backed flow migration. (**In progress**; viewport-root plumbing + conformance tests landed; Flex/Grid v2 root solves are centralized and redundant precompute is guarded when subtrees are already engine-backed; viewport-root coverage now locks wrapper + region nodes (Pressable/Semantics/FocusScope/Opacity/VisualTransform/InteractivityGate/PointerRegion/WheelRegion), including absolute-only children that must still fill the region; host widget v2 "engine fast path" checks are deduped via `try_layout_children_from_engine_or_manual_absolute`.)

Update this section by editing this file (avoid scattering progress notes across ADRs).

## P1: Constraint-Correct Intrinsic Measurement (ADR 0113)

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

## P2: Window-Scoped Layout Engine Skeleton + Two-Phase Protocol (ADR 0114)

Goal: introduce a per-window `TaffyLayoutEngine` and enforce the separation between "build/request"
and "compute/apply" (layout engine v2 is the default layout engine in `fret-ui`).

Deliverables:

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

- Tests pass and deep trees remain stacksafe.

## P3: Multi-Viewport Roots + Flow Migration (End-State Convergence)

Goal: evolve "taffy islands" so that layout-style-only descendants become nodes in the same Taffy
tree, while docking-defined viewports are independent roots and explicit barriers remain outside.

Deliverables:

- Docking provides definite viewport rects and registers viewport roots.
- Engine computes each viewport root independently (no cross-viewport percent/free-space coupling).
- Apply composes viewport-local rects into window-local bounds.
- Flex/Grid/Stack/Container wrappers become nodes in the same Taffy tree when possible; `measure_in`
  is reserved for true leaves.
- Add stacksafe execution around solves + measure callbacks; enable Taffy rounding to reduce drift.

Acceptance:

- Conformance test: no cross-viewport coupling for percent/flex distribution.
- Docking demos behave consistently across DPI scales (bounds stable within rounding policy).
- Dogfood demo (manual): `cargo run -p fret-demo --bin todo_demo` matches the shadcn-style composition (Card + Input + Tabs + ScrollArea + hover-only actions) without re-entrant layout or stack growth.

## Open Decisions (Track Here)

1. **"Contents-like" wrappers**: **Decision (v1)**: no general-purpose contents-like / `Slot/asChild` prop merging (ADR 0115). Prefer GPUI-aligned single-root components; if needed later, add a restricted, validated "layout-transparent wrapper" opt-in (layout-only, no prop merging).
2. **Root solve orchestration**: **Decision (v1)**: viewport roots are registered during the parent/root layout pass and flushed immediately after that root, before continuing to subsequent overlay roots. This preserves the ADR 0011 ordering expectation ("viewport content before overlays") without coupling viewports into a shared solve. (Implementation: `UiTree::layout_all` viewport flush loop.)
3. **Engine cache keys + invalidation**: **Decision (v1)**: keep intrinsic measurement memoization scoped to a single `compute_root_with_measure` call (engine-local cache keyed by `NodeId + known_dimensions + AvailableSpace`); avoid cross-frame measurement caching until we have an explicit, stable environment key. Leaf `measure_in` implementations must observe all relevant inputs (scale factor, theme revision, font stack key, model revisions) so invalidation remains correct even without long-lived caches.
4. **Rounding policy**: **Decision (v1)**: snap layout outputs at apply/writeback using ADR 0035 `snap_rect` so hit-testing and paint share stable bounds. The engine may internally solve in device-pixel space (`* scale_factor`) with Taffy rounding enabled as an implementation detail, but the results must be semantically equivalent to `snap_rect` on writeback and must be idempotent with renderer snapping (avoid double-rounding drift).
5. **"Not all containers use Taffy" boundary**: **Decision (v1)**: keep these as explicit barriers with specialized layout algorithms, but require clean interop with the engine (definite child rects, correct `AvailableSpace`, and no solver-time re-entry):
   - Docking / multi-viewport split roots (ADR 0013 + 0116): registers viewport roots and defines definite per-viewport available space.
   - Scroll extents + clipping (barrier): measures content with `MaxContent`, then lays out a single viewport-sized subtree.
   - Virtualization (`VirtualList`) (ADR 0042): measures item extents via `measure_in`, precomputes only visible item subtrees, and never demands solver-time re-entry.
   - Resizable splits (`ResizablePanelGroup`): computes definite panel rects, then registers each panel as a viewport root in v2.
   - Viewport surfaces (`RenderTargetId`) (ADR 0007): treat as rendering/clip boundary; never try to "merge" multiple surfaces into a single flow tree.

## Known Gaps / Cautions

- In v2 Final passes, **declarative element layer roots** run a request/build stage up front (stable identity + engine-backed wrapper rects), followed by a compute stage that is skipped when clean/translation-only. This is intentionally skipped for non-element/custom roots to avoid wasted work, and viewport roots are still orchestrated via the post-root flush protocol.
 - Viewport roots participate in the same request/build stage: when docking registers viewport roots, the flush loop first request/builds all newly-registered viewport roots, then computes/applies only the roots that require layout. This preserves stable identity even when a viewport root is skipped for compute/apply.
- v2 request/build orchestration is centralized in `UiTree::request_build_window_roots_if_final(...)` and the viewport flush helper `UiTree::flush_viewport_roots_after_root(...)` (`crates/fret-ui/src/tree/layout.rs`). These helpers perform request/build and compute/apply using a single `take_layout_engine()` per wave to reduce engine handoff churn.
- Wrapper overlay nodes use `1fr` tracks with `justify_items/align_items: stretch` so wrapper nodes remain layout-transparent: if an ancestor stretches the wrapper (e.g. `w-full` via flex cross-axis stretch), the single child receives the same definite box budget and does not collapse to content width.

## Engineering Guardrails (v2 Runtime Policy)

To keep solve counts stable and avoid accidental re-introduction of re-entrant layout patterns, v2
code should follow these rules:

1. Treat barrier solves as an escape hatch: do not call them from normal wrappers/flow containers.
   Only explicit barriers may call `LayoutCx::solve_barrier_child_root_if_needed(...)`.
   Direct calls into `UiTree::solve_barrier_flow_root(_if_needed)` are considered internal-only
   plumbing (useful for tests), and should not be used by general-purpose wrappers/flow containers.

   Allowed call sites:
   - Scroll / VirtualList: `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
   - ResizableSplit: `crates/fret-ui/src/resizable_split/widget.rs`
   The `_if_needed` helper skips work when the subtree is clean, and avoids engine solves for translation-only changes
   (size stable, origin shifts).
2. Keep solve stats per-call and use them to detect regressions.
3. Translation-only bounds shifts must still keep existing engine nodes "alive" for stable identity
   and incremental updates (do not let `end_frame` prune large subtrees during scrolling/panning).

Regression tests that lock these behaviors:

- Scroll translation does not trigger engine solves: `declarative::tests::layout::scroll_translation_does_not_force_layout_engine_solves`.
- Viewport root flush only lays out invalidated roots: `declarative::tests::layout::viewport_root_flush_only_lays_out_invalidated_roots`.
- Translation-only precompute gating: `declarative::tests::layout::solve_barrier_flow_root_if_needed_skips_translation_only_bounds_changes`.
