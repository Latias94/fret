# ADR 0055: Frame Recording and Subtree Replay Caching


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Makepad: https://github.com/makepad/makepad
- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

Implementation status:

- Implemented (P0): paint-stream range replay in `crates/fret-ui/src/tree/mod.rs` (per-window `UiTree`).
- Cache key: `bounds.size`, `scale_factor`, `theme_revision` (locked P0 fields; origin differences are handled via replay-time translation).
- Observations: paint-time model observations are reused on cache hit by retaining the previous `ObservationIndex` entries.
- Observability: per-window hit/miss/op counters are exposed via `UiDebugFrameStats` and shown in the demo HUD.
- Recording ingestion: `UiTree::ingest_paint_cache_source(&mut Scene)` swaps the previous frame's `Scene.ops` into the
  cache buffer before the scene is cleared, avoiding per-frame copying.
- Implemented (P0): renderer-side encoding reuse keyed by a stable `SceneRecording::fingerprint()` (+ `ops_len()` and
  render-target generation) to skip CPU encoding when the recorded scene is identical.

## Context

Fret targets editor-scale UIs: deep trees, multi-window docking, overlays/modals, multiple viewports, and heavy
text/list workloads. In debug builds, CPU overhead becomes the primary iteration bottleneck.

We already have the essential correctness contracts:

- Retained tree geometry and routing (`bounds` are authoritative, invalidation drives layout/paint): ADR 0005.
- Ordered display list semantics (`Scene.ops` order is authoritative, batching is adjacency-preserving): ADR 0002 / ADR 0009.
- Model observation and invalidation propagation: ADR 0051.
- Scheduling and redraw strategy (`FrameId`, timers, continuous mode): ADR 0034.
- Observability hooks (HUD/inspector) to diagnose perf regressions: ADR 0036.

What is missing is a **closed-loop, framework-level caching contract** that:

- reduces per-frame CPU work without breaking determinism,
- remains compatible with future GPUI-style authoring (ADR 0028 / ADR 0039),
- preserves ordered composition semantics (no reordering),
- scales to future “more than SceneOps” outputs (hit regions, input handlers, semantics, etc.),
- is debuggable (cache can be disabled for inspection/picking).

References:

- GPUI view caching via recorded ranges and replay (`AnyView::cached`, `reuse_prepaint`, `reuse_paint`):
  - `repo-ref/zed/crates/gpui/src/view.rs`
  - `repo-ref/zed/crates/gpui/src/window.rs`
- Makepad drawlist rebuild skipping (`redraw_id`, `begin_maybe`):
  - `repo-ref/makepad/draw/src/draw_list_2d.rs`

## Decision

### 1) Fret introduces a per-window “Frame Recording” abstraction (multi-stream)

For each window render, Fret conceptually produces a **Frame Recording** consisting of multiple ordered streams.

P0 stream (already exists):

- **Paint stream**: `fret-core::Scene` (`Scene.ops: Vec<SceneOp>`) in strict order.

Planned follow-up streams (not required for initial implementation, but reserved by contract):

- **Interaction stream**: hit regions (if/when we move beyond bounds-only hit-test), cursor style requests, pointer
  listeners, input handlers, tooltips, tab stops, etc.
- **Semantics stream**: accessibility-ready semantics snapshot/diff (ADR 0033).

The key requirement is that each stream supports **range-based reuse**:

- each widget/subtree contributes a contiguous range to each stream,
- a subtree cache hit replays those ranges into the current frame recording without recomputing.

This is the “closed loop” that prevents caching from becoming a one-off optimization tied only to rendering.

### 2) Subtree replay caching is the primary framework-level caching strategy

Fret implements subtree caching by reusing recorded ranges from a previous rendered frame.

On a cache hit, the runtime:

- skips widget/subtree `paint` (and any future prepaint-like hooks),
- replays the previous frame’s recorded ranges into the current frame recording **at the same callsite order**.

This preserves:

- ordered composition semantics (ADR 0002),
- overlay ordering semantics (ADR 0011),
- determinism under multi-window scheduling (ADR 0034 / ADR 0015).

This contract is **framework-level** and must remain editor-domain-agnostic (ADR 0027).

### 3) Cache identity and keying (retained-tree and GPUI-style compatibility)

Caching identity is derived from the UI identity model:

- retained widgets: `NodeId`
- declarative/element model: `GlobalElementId` + explicit keys (ADR 0028 / ADR 0039)

The caching layer must not depend on pointer addresses or transient allocations.

Each cached subtree stores a **Cache Key** that captures the minimal set of inputs that affect recorded output.

Locked P0 cache key fields:

- `bounds.size` (layout size; pure translations are handled via replay-time translation)
- `scale_factor` (logical→physical mapping affects text and pixel snapping)
- `theme_revision` (ADR 0032)

Reserved/expected future fields (may be added without breaking the contract):

- `content_mask` / clip context (if/when clip semantics expand beyond explicit `SceneOp` push/pop)
- `text_style_context` (if/when text style is propagated via a stack/context rather than per-op explicit style)
- `text_system_revision` (if the host text system exposes a revision counter for atlas/layout reuse)

### 4) Invalidation is the single source of truth; caching is never relied on for correctness

Caching must be **strictly subordinate** to invalidation:

- If a widget changes anything that affects layout/paint/hit-test, it must request invalidation.
- Cache hits are allowed only when the relevant invalidation flags are clear.
- Resource lifetime must remain compatible with replay:
  - cached `SceneOp` ranges may reference external handles such as `TextBlobId`,
  - therefore widgets must not release/replace those handles in `layout()` (which can run even when `paint()` is replayed),
  - releases should happen in `paint()` (when repainting) and/or `cleanup_resources()`.

Model observation (ADR 0051) is a first-class part of the closed loop:

- on a cache hit, the runtime must behave as if the widget had re-established its model observations for paint,
  either by reusing the previous observation set or by replaying a recorded “observation stream”.

This avoids the “cached subtree stops reacting” failure mode.

### 5) Debuggability: cache can be disabled for inspection/picking

When debugging UI structure (e.g. an inspector overlay, mouse picking, accessibility inspection),
the runtime may disable caching to ensure all debug/probe data is complete and up to date.

This follows the GPUI pattern: “cache is an optimization; introspection correctness comes first”.

### 6) Public API surface (future): explicit caching boundaries for GPUI-style authoring

To match GPUI ergonomics without coupling to editor semantics, Fret will eventually provide an authoring-layer
escape hatch akin to:

- `view.cached(cache_key_inputs...)` / `element.cached(...)`

This is not required for the retained-tree runtime, but it is a key part of being a reusable, third-party-friendly
framework.

The underlying mechanism remains the same: recorded ranges + replay.

## Implementation Sketch (Non-Normative)

P0 implementation can focus on the paint stream only:

1. Add `Scene::replay(range, prev_scene)` (or equivalent) to append a range of `SceneOp` from a previous scene.
2. In `fret-ui`, extend the paint traversal to record, per node:
   - scene range `(start..end)` for that node’s subtree contribution.
3. On paint:
   - if `Invalidation::Paint` is clear and cache key matches, replay the cached range and skip widget paint.
   - otherwise paint normally and update cached range + key.
4. Preserve paint-time model observations on cache hit (reuse previous observation set).
5. Add observability counters (ADR 0036):
   - nodes painted vs replayed,
   - scene ops generated vs replayed,
   - cache hit rate per window.

## Consequences

- Debug iteration becomes faster without requiring release builds.
- Caching remains deterministic and compatible with ordered composition.
- The framework has a clear path to expand caching beyond rendering to interaction and semantics outputs.
- The caching design aligns with GPUI’s “record and reuse ranges” model while remaining compatible with a retained tree.

## Open Questions

1. Should we introduce an explicit “prepaint” phase in Fret to separate interaction/semantics stream construction
   from paint?
2. Eviction and memory pressure:
   - how to cap per-window cached recordings (by frames, by ops count, by bytes)?
3. Granularity:
   - do we cache at node level only, or introduce explicit cache boundaries/components (GPUI-style) earlier?
4. Clip semantics:
   - if we expand beyond scissor-like clip, what cache key fields become mandatory?
