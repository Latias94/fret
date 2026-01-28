# ADR 0193: Ephemeral Prepaint Items (v1)

Status: Proposed

## Context

Fret is converging on a GPUI-aligned “closed loop”:

- state/model changes → `notify` / dirty views
- `layout` → `prepaint` → `paint`
- cache-root range reuse (paint + interaction, later semantics)
- high-frequency updates (scroll/hover/caret/drag indicators) should ideally avoid rerendering a cache root

ADR 0190 establishes the direction for **prepaint-windowed virtual surfaces**: move “what is visible” decisions toward
`prepaint` so scroll-driven window changes do not necessarily imply a cache-root rerender.

However, Fret currently lacks a crisp contract for **ephemeral prepaint items**:

- What data is allowed/expected to be produced in `prepaint`?
- How is that data cached/reused across cache-hit frames?
- How does it interact with view-cache reuse, liveness/GC (ADR 0191), and diagnostics?
- Which actions are allowed in prepaint (invalidate / redraw / animation frames) without mutating structure?

This ADR defines the minimum contract needed to scale ADR 0190/0192 without a future rewrite.

## Goals

- Define a stable “ephemeral prepaint item” model that:
  - can be produced during `prepaint` after layout bounds are known
  - can be reused on cache-hit frames (range reuse) without rerendering the cache root
  - supports scroll/window updates, hover/caret chrome, and overlay positioning as paint-only updates
- Make prepaint-driven behavior explainable from a single diagnostics bundle:
  - why did the window change?
  - why did a cache root rerender?
  - what did prepaint request (invalidate/redraw/raf)?
- Avoid coupling ephemeral items to declarative node liveness so GC under view-cache reuse remains sound (ADR 0191).

## Non-goals

- Defining the final multi-stream recording schema (paint + interaction + semantics) end-to-end.
- Replacing the retained `UiTree` structure wholesale with a GPUI-style “frame-only tree”.
- Solving composable virtualization by itself (see ADR 0192 for retained host boundaries).

## Prior Art (What we want to emulate)

### GPUI / Zed (conceptual)

- Rebuild declarative output only for dirty views; otherwise reuse cached prepaint/paint ranges.
- Prepaint is a distinct phase whose outputs can be reused when the view is clean.
- Dirty propagation is “one story” with cache boundaries: if a child view becomes dirty, ancestors that would replay old
  ranges must also become dirty.

### Flutter (conceptual)

- Pipeline: build → layout → paint; the **layer tree** is ephemeral per-frame output.
- High-frequency changes can avoid rebuilding widgets if they only affect paint/compositing (e.g., repaint boundaries,
  transforms, opacity).
- Virtualization uses retained render objects with windowed child management (sliver-style) rather than rebuilding the
  entire tree each frame.

The common best practice: keep **cross-frame state retained**, but make **frame-local output** (layers/ops/interaction
registries) cheap to update and cacheable when structure is stable.

## Definitions

- **Cache root / view**: the unit of view-cache reuse (v1: cache-root-first, ADR 0180).
- **Ephemeral prepaint item**: frame-local data derived from:
  - stable structure (render output can be reused)
  - current geometry (layout bounds/transform/clip)
  - current input state (scroll offset, hover/focus, selection/caret state, drag indicators)
  and produced during `prepaint` to drive `paint`/interaction routing without rerendering.
- **Prepaint output**: the collected ephemeral items for a cache root (or for the window) that can be cached/reused when
  the cache root is clean and reuse gates allow it.

## Decision (v1 Contract)

### 1) Prepaint can request invalidation and scheduling, but must not mutate structure

During `Widget::prepaint(PrepaintCx)`:

- Allowed:
  - `invalidate(Paint|HitTest|HitTestOnly)` for current/target nodes
  - `request_redraw()` (one-shot)
  - `request_animation_frame()` (frame-driven progression; also implies `Invalidate::Paint`)
  - reading geometry and “last known” bounds
  - updating app-owned models or retained widget state that does not change the declarative node tree shape
- Not allowed (v1 contract):
  - structural changes to the `UiTree` node graph (add/remove/reorder children)
  - changing element identity or cache-root boundaries

Rationale: keep prepaint a safe phase that cannot silently invalidate GC/liveness bookkeeping.

### 2) Ephemeral prepaint items must be cacheable and explainable

Each cache root may accumulate prepaint outputs keyed by a **prepaint cache key**. The v1 key should include the minimum
inputs that make reuse correct:

- cache-root bounds (at least size; position may be excluded if replay is translation-safe)
- scale factor
- theme revision (or style revision token)
- relevant clip/transform keys (where applicable)
- virtual surface “window key” inputs (viewport/offset/overscan/items revision) for windowed surfaces

If the key matches and the cache root is not dirty, prepaint outputs may be reused.

### 3) Liveness/GC does not depend on ephemeral items

- Ephemeral items are **not** part of the declarative liveness graph.
- They may reference nodes/elements, but they do not keep nodes alive.
- Declarative liveness continues to be governed by explicit liveness roots + retained child edges (ADR 0191).

Rationale: prevent prepaint-only fast paths from creating “liveness islands” or masking detach bugs.

### 4) Diagnostics must expose prepaint requests (v1 requirement)

Diagnostics bundles must export bounded, per-frame prepaint requests so regressions are debuggable without a debugger.

Minimum fields:

- node id (source)
- optional target node id (for invalidations)
- action kind: invalidate / request_redraw / request_animation_frame
- invalidation kind where applicable
- frame id

This enables tests and scripts to assert that a paint-only update stayed paint-only (no rerender) while remaining correct.

## Implementation Notes (Current State)

- `Widget::prepaint(PrepaintCx)` exists and is called for view-cache roots during the prepaint pass.
- Bundles export `debug.prepaint_actions` as a bounded list of prepaint requests.
- VirtualList window telemetry already exists in bundles, but window derivation is still render-driven (ADR 0190 gap).

## Rollout Plan

1. Land the diagnostics/debug surface (`debug.prepaint_actions`) as the baseline guardrail.
2. Introduce a minimal “ephemeral items” API in `UiTree` prepaint that can store/reuse prepaint outputs per cache root.
3. Move VirtualList window derivation to prepaint (ADR 0190) and validate:
   - cache+shell scroll stays transform-only until the window changes
   - window boundary frames do not force cache-root rerender when a retained host can attach/detach items (ADR 0192)
4. Expand the interaction stream vocabulary (ADR 0182) using the same contract and reuse gates.

## Alternatives Considered

- **Always rerender on any prepaint-visible change**: simplest, but defeats ADR 0190 and keeps scroll jank.
- **Allow structural mutations in prepaint**: powerful but risks breaking liveness/GC invariants and makes bundles
  inexplicable; defer to ADR 0192 retained host boundaries instead.
- **Global per-window ephemeral registry only**: too coarse; cache roots need local reuse keys and attribution for
  explainability.

