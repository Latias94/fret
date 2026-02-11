# ADR 0114: Window-Scoped Layout Engine and Viewport Roots

Status: Proposed

## Context

Fret's declarative flow layout uses `taffy` as an internal algorithm (ADR 0035, ADR 0057). Today,
parts of the runtime behave like "container-owned Taffy islands" and measure some subtrees by
re-entering layout. ADR 0113 introduces `AvailableSpace` (Definite/MinContent/MaxContent) and a
non-reentrant intrinsic measurement path to remove constraint-phase semantic drift and recursion
hazards.

Fret also targets editor-class composition: multi-root layers (ADR 0011), multi-viewport docking,
explicit virtualization containers (ADR 0042), and scroll/viewports as first-class layout systems.
To scale these surfaces without repeatedly reworking the layout plumbing, we need to lock down the
end-state "window-scoped" integration shape:

- A window owns a canonical layout engine and Taffy tree for declarative flow layout.
- Docking defines multiple viewports (each with definite bounds) and each viewport is an
  independent layout root for its mounted UI subtree.
- Layout barriers (docking/scroll/virtualization/viewport surfaces) remain explicit and do not get
  forced into Taffy, but they must interoperate with the engine via stable, non-reentrant contracts.

This ADR is intentionally aligned with Zed/GPUI's approach (`repo-ref/zed/crates/gpui/src/window.rs`,
`repo-ref/zed/crates/gpui/src/taffy.rs`) while remaining compatible with Fret's existing layering
and retained UI tree contracts.

## Goals

1. **Single engine per window**: a window-scoped `TaffyLayoutEngine` owns the canonical Taffy tree
   for declarative flow layout.
2. **Multiple viewport roots**: docking-driven viewports are independent layout roots; constraints
   and "free space" must not couple across a viewport boundary.
3. **Two-phase protocol**: separate "request/build" (graph construction) from "compute/apply"
   (solver + bounds writeback) to keep measurement pure and avoid re-entrancy.
4. **Stable identity + incremental updates**: stable `NodeId -> LayoutId` mapping across frames,
   enabling `mark_dirty` updates instead of full rebuilds.
5. **Explicit barrier contracts**: docking/scroll/virtualization/viewport surfaces remain explicit
   layout systems with a clear contract for how they measure and lay out mounted subtrees.
6. **Diagnosability**: provide guardrails and debug surfaces to detect cycles, constraint-phase
   violations, and viewport coupling early.

## Non-goals

- A DOM/CSS cascade model, selectors, or runtime class parsing.
- Forcing docking, virtualization, or editor-specific layout containers into Taffy.
- A global `z-index` primitive or changing layer ordering contracts (see ADR 0062, ADR 0009).
- Full CSS parity beyond the typed Tailwind-like semantics already defined in ADR 0057/0062.

## Compatibility Notes

This ADR is intended to refine and extend existing accepted contracts, not contradict them:

- ADR 0035 (hybrid layout) remains the boundary: Taffy is still an internal algorithm for declarative
  flow layout, and docking/splits/scroll/virtualization remain explicit systems.
- ADR 0005 (layout writes bounds) remains true: the engine's "apply" step is part of layout and
  writes authoritative bounds into the retained UI tree before paint/hit-test.
- ADR 0076 (persistent container-owned Taffy trees) documents a historical performance hardening
  strategy. The repository now defaults to the window-scoped engine described by ADR 0114, carrying
  forward the same persistence/incremental update principles (stable identity + bounded measurement)
  at the window boundary rather than the container boundary.
- Terminology: "viewport" in this ADR refers to docking-defined layout roots (multi-viewport UI).
  "ViewportSurface" refers to engine integration surfaces (ADR 0007).

## Decision

1. Each window owns a `TaffyLayoutEngine` that persists across frames and maintains a canonical
   Taffy tree for declarative flow layout.
2. Docking-driven viewports are treated as independent layout roots:
   - Docking computes **definite** viewport rects.
   - The engine computes layout for each viewport root against that viewport's definite available
     space.
   - Percent/fill resolution and flex free-space distribution must not cross viewport boundaries.
3. Layout uses a strict two-phase protocol:
   - **Request/build**: construct/update the layout graph (nodes, styles, child edges, measure fns).
   - **Compute/apply**: run the solver for each root and write **definite** bounds back into the
     retained UI tree.
4. Intrinsic measurement is non-reentrant and constraint-correct per ADR 0113:
   - `AvailableSpace::{Definite, MinContent, MaxContent}` must be preserved end-to-end.
   - Measurement must not call `layout_in`, directly or indirectly.
5. The engine requires stable identity:
   - Retained `NodeId` is the stable key.
   - The engine maintains a stable mapping from `NodeId` to `LayoutId`/`TaffyNodeId`.
6. Wrapper handling:
   - By default, typed wrappers with `LayoutStyle` (opacity, semantics, focus scope, interactivity
     gate, paint-only transforms) are represented as layout boxes in the Taffy tree.
   - Fret does **not** provide a general-purpose "display: contents" or Radix-style `Slot/asChild`
     prop-merging mechanism (ADR 0115). We do not attempt to arbitrarily "retarget" interaction,
     semantics, or layout props onto an unknown child element.
   - Instead, authoring should follow a GPUI-aligned shape:
     - choose a single typed root that owns layout + hit-testing + interactivity + a11y (most
       commonly `Pressable` / `Semantics` / `Container`),
     - treat "slots" as content slots, not "element substitution" slots.
   - If a real need emerges to avoid extra layout boxes in *very specific* compositions, we may
     introduce a **restricted** "layout-transparent wrapper" opt-in:
     - strictly about layout box introduction/removal (no prop merging),
     - validated constraints (single-child, `position: static`, and no geometry-affecting layout
       properties like padding/margin/size/min/max/overflow/transform/flex/grid),
     - debug/test builds must fail fast when validation is violated.
7. Measurement cycle policy:
   - Debug/test builds treat re-entrancy/cycles as a bug (panic/assert with diagnostic data).
   - Release builds must not crash; they use a safe fallback policy (exact fallback defined in ADR 0113 and implementation docs) plus rate-limited diagnostics.

## Design (Proposed)

### 1) Engine structure

The window-scoped engine owns:

- `TaffyTree<NodeContext>` where `NodeContext` references the retained `NodeId` and an optional
  measure function.
- Stable maps:
  - `node_id -> layout_id` (and reverse where useful for apply).
- Per-frame scratch buffers and caches (including measurement memoization keyed by constraints and
  environment).

The engine is invoked during the frame lifecycle (ADR 0015):

1. Declarative build/prepaint: register layout nodes for the current element tree.
2. Layout compute: solve each viewport root with its available space.
3. Apply: write computed bounds into the retained UI tree before paint/hit-test.

### 2) Two-phase protocol surface

Names illustrative:

- `LayoutEngine::begin_frame(frame_id, scale_factor, env_keys...)`
- `LayoutEngine::request_layout(node_id, style, children: &[node_id]) -> LayoutId`
- `LayoutEngine::request_measured_layout(node_id, style, measure_fn) -> LayoutId`
- `LayoutEngine::compute_root(root_layout_id, available: Size<AvailableSpace>, scale_factor)`
- `LayoutEngine::layout_rect(layout_id) -> Rect` (root-local)

Important: the request phase is allowed to observe models/globals for invalidation, but it must not
write bounds to the retained tree. Bounds writeback happens only in apply.

### 3) Viewport roots and coordinate spaces

Docking produces viewport definitions:

- `viewport_id`
- `viewport_origin` (window-local)
- `viewport_size` (definite)
- `root_node_id` mounted into that viewport

The engine computes each viewport root in a viewport-local coordinate space (origin at `(0, 0)`).
Apply/writeback composes viewport-local rects with the viewport origin and writes window-local
bounds into the retained UI tree.

This keeps multi-viewport layouts independent while preserving a single window-local coordinate
space for paint, hit-testing, and overlays.

Implementation note (barrier ordering):

- Docking/splits register viewport roots (node + definite rect) during their own layout pass.
- The runtime must ensure viewport root subtrees are laid out **after** the docking barrier has
  produced rects, but **before** overlay roots that may query those bounds for anchored placement
  (ADR 0011). In practice, this means viewport roots are flushed as independent layout passes
  between “base root” layout and subsequent overlay root layout.

### 4) Barriers and interop contracts

Barriers are layout system boundaries (ADR 0035, ADR 0042, ADR 0007):

- Docking/splits
- Scroll containers
- Virtualization containers
- Viewport surfaces

Interop rules:

- A barrier may request intrinsic measurement for content (via `measure_in`) and/or request layout
  for a mounted child subtree under a **definite rect**.
- A barrier must not rely on solver-time re-entrant layout/measurement of its descendants.
- Barriers define their own internal policy for:
  - which axes must be definite to resolve fill/percent/free-space distribution,
  - which intrinsic queries are performed (`MinContent` vs `MaxContent`),
  - how many children/items they lay out (virtualization budget).

Default policy (subject to conformance tests):

- Virtualization containers are always barriers and only request layout for visible items (+overscan).
- Scroll containers are barriers; they measure content extent using `MaxContent` constraints and lay
  out the mounted content subtree under a definite viewport rect.

### 5) Stable identity and incremental updates

The engine updates incrementally:

- If a `NodeId` disappears, its layout node is removed.
- If a node's `LayoutStyle` changes, the engine updates the Taffy style and `mark_dirty`s that node.
- If children change, the engine updates edges and marks the node dirty.
- Root-level dirtiness is tracked per viewport root to bound solve costs.

### 6) Environment keys and caching

Measurement memoization and correctness require explicit environment keys:

- `scale_factor`
- theme revision
- font stack key
- relevant model revisions for text and other intrinsic nodes

These keys participate in measurement cache keys and/or dirty invalidation rules so that changes
reliably trigger re-measure/re-layout.

### 7) Guardrails and debug surfaces

Guardrails (aligned with GPUI):

- stack-safe layout solve and measure callbacks for deep trees.
- cycle/re-entrancy guard for `measure_in`, keyed by `(node_id, constraints, env_keys)`.
- optional rounding to reduce subpixel drift.

Debug surfaces:

- layout tree dump (nodes + constraints + measured sizes + computed rects).
- per-viewport root stats (solve time, node count, max depth).
- rate-limited warnings on release-mode cycle fallback.

## Rollout Plan (Incremental)

1. Land ADR 0113 mechanism changes (`AvailableSpace`, `measure_in`, non-reentrant measurement).
2. Introduce the window-scoped engine behind an internal feature flag and migrate Flex/Grid to
   register into it (one viewport/root first).
3. Integrate docking viewports by treating each viewport as an independent root and adding
   conformance tests for non-coupling across viewports.
4. Migrate wrapper nodes into the engine as stable nodes; optionally introduce a validated
   "contents-like" mode for Radix-aligned `asChild` composition.
5. Migrate additional declarative primitives into the engine, leaving only explicit barriers and
   performance-critical containers outside Taffy.

## Alternatives Considered

1. Keep per-container "Taffy islands"
   - Rejected as an end-state: increases repeated solves and makes constraint semantics harder to
     centralize for multi-viewport and barrier interop.
2. A single global "synthetic root" for the whole window including viewports
   - Not preferred: easy to accidentally couple viewports through percent/flex semantics.
   - Might still be useful as a debug visualization tool (not the default runtime model).
3. Adopt a full CSS engine
   - Rejected: violates typed semantics and increases surface area significantly.

## Defaults (v1)

- Viewport roots are registered by docking barriers once their definite rects are known, stored in
  a per-frame root list, and consumed during compute/apply.
- Wrapper nodes: any node with layout-affecting properties is represented in the engine tree;
  skipping boxes is not supported by default; any future "layout-transparent wrapper" must be an
  explicit validated opt-in and must not imply Slot/`asChild` prop merging (ADR 0115).
- Overlays: default to window-scoped overlay roots that position via post-apply bounds queries (ADR 0011); add per-viewport overlay roots only for concrete perf/correctness needs.
- Root solve order: solve roots in window root z-order, but viewport roots never participate in a
  shared solve; each viewport root is solved independently against its own definite available space.
- Pixel rounding: optional; if enabled, snap layout outputs at the layout-engine boundary (apply/writeback)
  using the same `snap_rect` policy as ADR 0035 so hit-testing and paint share stable bounds.
  - Implementation option A (simple): solve in logical pixels, then apply `snap_rect` on writeback.
  - Implementation option B (GPUI-aligned, implementation detail): internally solve in device-pixel space
    (`* scale_factor`) with Taffy rounding enabled, then convert results back to logical pixels
    (`/ scale_factor`). This must be semantically equivalent to applying `snap_rect` on writeback and must
    not change the core coordinate space (still logical pixels).
  - Requirement: snapping must be idempotent with renderer snapping (avoid double-rounding drift).

## References

- Non-reentrant measurement + `AvailableSpace`: `docs/adr/0113-available-space-and-non-reentrant-measurement.md`
- Refactor roadmap (living doc): `docs/layout-engine-refactor-roadmap.md`
- Migration inventory (living checklist): `docs/layout-engine-v2-migration-inventory.md`
- Taffy integration boundaries: `docs/adr/0035-layout-constraints-and-optional-taffy-integration.md`
- Declarative Flex semantics: `docs/adr/0057-declarative-layout-style-and-flex-semantics.md`
- Container-owned Taffy performance hardening (near-term): `docs/adr/0076-declarative-layout-performance-hardening.md`
- Trigger composition without Slot/`asChild`: `docs/adr/0115-trigger-composition-and-no-slot-aschild.md`
- Virtualization boundaries: `docs/adr/0042-virtualization-and-large-lists.md`
- Docking viewports: `docs/adr/0013-docking-ops-and-persistence.md`
- Engine viewport surfaces: `docs/adr/0007-viewport-surfaces.md`
- Multi-root layering: `docs/adr/0011-overlays-and-multi-root.md`
- Frame lifecycle: `docs/adr/0015-frame-lifecycle-and-submission-order.md`
- Refactor progress tracker: `docs/layout-engine-refactor-roadmap.md`
- GPUI reference: `repo-ref/zed/crates/gpui/src/window.rs`, `repo-ref/zed/crates/gpui/src/taffy.rs`
  (see also `repo-ref/zed/crates/gpui/src/elements/div.rs`, `repo-ref/zed/crates/ui/src/components/tab.rs`)
