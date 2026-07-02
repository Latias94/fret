# ADR 0327: Frame Pipeline v2 and View Boundaries

Status: Accepted (contract freeze; implementation aligned with known follow-ons)

Acceptance note (2026-05-14): this ADR is accepted as the target contract for the Frame Pipeline v2
global refactor after the code-editor vertical slice proved the direction with boundary-owned
prepaint/scene-fragment state, first-class boundary diagnostics, deletion of replaced private paths,
and perf evidence. Acceptance does not mean the global migration is complete; implementation status
is tracked in `docs/adr/IMPLEMENTATION_ALIGNMENT.md` and
`docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/PROGRESS.md`. The global workstream
closeout is recorded in
`docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/FINAL_CLOSEOUT_AUDIT_2026-05-14.md`.

Convergence note (2026-06-30): new runtime work should not reopen the closed broad Frame Pipeline
v2 lane. The active convergence plan
`docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md` uses this ADR as the
contract anchor for narrower follow-ons: ViewId-first dirty ownership, boundary-owned frame
products, retained scene chunks, renderer dirty uploads, and text/glyph budget gates.

Phase 2 convergence note (2026-07-02): the active deletion-biased follow-on is
`docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md`. It preserves this ADR as the
frame-pipeline contract while freezing stricter migration rules:

- `ViewId` and `BoundaryId` are independent window-scoped/runtime identities, not durable aliases
  for retained `NodeId` placement. The old `ViewId(pub NodeId)` wrapper is deleted. `BoundaryId(NodeId)`
  and `iter_boundary_nodes_v1` / `mark_boundary_node_v1` / `clear_boundary_node_v1` style helpers are
  remaining migration bridges with deletion gates.
- `ViewBoundaryState` and the next `ViewBoundaryStore` should be keyed by `ViewId` and/or
  `BoundaryId`, then resolve the current live node through explicit liveness metadata. A detached
  or retained boundary may exist without a live node.
- Boundary-owned products are valid only when their correctness is local to that boundary. Layout
  dirty bits, boundary prepaint products, cache-root interaction replay entries, reusable boundary
  semantics subtrees, boundary scene fragments, and boundary paint-cache entry metadata remain the
  primary candidates.
- Window/layer-forest products stay window-owned unless a later ADR proves a narrower owner:
  dispatch snapshots, command routing and availability, final semantics snapshots, hit-test path
  routing, focus/capture state, active layer roots, modal barrier state, and tree-wide paint
  recording.
- Renderer migration bridges (`Scene` as the normal semantic render input, chunk replay through
  temporary flat scenes, full-blob text resource helpers, and stream classes without chunk closure)
  must carry parity or resource-closure gates before deletion.
- Public/source-policy compatibility exceptions are not frame products. They may be referenced by
  this plan because they protect adoption during the same breaking window, but they belong to the
  app facade/source-policy contract and must carry owner, reason, and retirement criteria there.

## Context

Fret currently combines several valid but partially overlapping runtime ideas:

- a retained `UiTree` that owns layout, hit testing, focus, paint state, and invalidation,
- declarative element trees mounted into that retained runtime,
- view-cache and paint-cache roots that can replay previously recorded work,
- targeted containment knobs historically exposed as `ViewCacheProps::contained_layout`,
- and editor-specific hot paths such as code-editor row-scene replay.

This has worked well enough to build real demos and diagnostics, but recent editor-grade perf work
exposed a structural problem:

- a code-editor resize operation that should be local to the page content root can accidentally
  pull the surrounding gallery shell and scroll wrappers into layout solve work,
- paint hot paths still mix state derivation, resource touch, cache-key comparison, and scene
  emission in one phase,
- prepaint-like concepts exist in several places, but they are not yet the canonical runtime phase,
- and cache roots are still an optimization surface rather than the primary execution boundary.

The macOS `ui-code-editor-resize-probes` evidence on 2026-05-13 showed the shape of the issue:

- making the script deterministic and enabling `code_editor.paint_perf` revealed a layout-solve
  failure first,
- applying a code-editor page `contained_layout` boundary reduced p95 total by roughly 34% and
  layout solve by roughly 85%,
- the new bottleneck became paint/widget row replay and content resolution,
- and renderer encode/upload stayed out of the critical path for that diagnostic surface.

This means the current bottleneck is no longer just a local slow function. The runtime needs a
clearer execution model before more broad optimizations are attempted.

## Goals

1. Make the frame pipeline explicit enough that build, layout, prepaint, paint, and renderer work
   are independently attributable and gateable.
2. Promote view/cache boundaries into a first-class runtime contract instead of a set of ad hoc
   cache and containment knobs.
3. Make editor-grade surfaces local by default: resize, scroll, hover, and text updates should not
   accidentally dirty unrelated shell structure.
4. Require the migration to delete or retire redundant old paths after the replacement path is
   verified.
5. Keep Fret's existing layer boundaries intact: `fret-ui` owns mechanisms; ecosystem crates own
   component policy and recipes.

## Non-goals

- Replacing the Fret architecture with GPUI/Zed code.
- Moving shadcn/Radix/Base UI interaction policy into `crates/fret-ui`.
- Rewriting the renderer contract or abandoning the `Scene` display-list boundary.
- Keeping private runtime compatibility shims after the v2 migration has gates and evidence.
- Optimizing Linux-specific behavior in this workstream.

## Decision

### 1. The runtime frame pipeline has named phases

Fret's runtime pipeline should converge on these phases:

```text
schedule / dirty propagation
  -> build
  -> request layout
  -> layout
  -> prepaint
  -> paint
  -> renderer prepare / encode / upload / present
```

Each phase must have an observable contract:

- what inputs it may read,
- what outputs it may write,
- which dirty flags trigger it,
- which cache keys allow reuse,
- and which diagnostics counters attribute its cost.

The existing implementation may migrate incrementally, but new editor-grade optimization work
should be designed against this phase model.

### 2. `ViewBoundary` is the core execution boundary

The v2 model introduces a first-class `ViewBoundary` concept inside the runtime.

A boundary represents a stable subgraph that can independently decide whether it needs:

- build,
- layout,
- prepaint,
- paint,
- hit-test rebuild,
- semantics refresh,
- or renderer resource refresh.

The target boundary state includes:

```text
BoundaryId
parent boundary
stable element/view identity
dirty_build
dirty_layout
dirty_prepaint
dirty_paint
dirty_hit_test
dirty_semantics
layout_dependency_key
prepaint_dependency_key
paint_dependency_key
layout_result
prepaint_state
scene_fragment
diagnostic counters
```

Current `ViewCacheProps`, cache roots, paint-cache roots, and low-level contained-layout flags
should migrate toward this boundary model rather than expanding as separate one-off mechanisms.
M4C introduced `ViewBoundaryHints` as the first public authoring step away from direct
`contained_layout` knobs; the remaining migration is internal runtime consolidation.
M4D consolidated element-runtime view-cache build-time rendered/next side maps into
`ViewCacheBuildBoundaryStore`, and M4M explicitly retains that store inside `WindowElementState` as
the `GlobalElementId`-keyed declarative build-boundary mechanism. This is intentionally separate
from `ViewBoundaryState`, which is keyed by retained `NodeId` runtime boundaries; mount-time
live-node revalidation bridges recorded build membership to the current retained nodes.
M4E moved boundary-node `PaintCacheEntry` ownership into `ViewBoundaryState::paint_cache`, and M4F
deleted the remaining node-owned `PaintCacheEntry` fallback. True runtime boundaries now own their
entries through `ViewBoundaryState::paint_cache`; M4L names
`UiTree::retained_paint_cache_entries` as the explicitly retained plain-node entry store that
migrates into `ViewBoundaryState::paint_cache` if the node becomes a runtime boundary. M4G split
previous-frame scene recording storage into `PreviousFramePaintRecording`, so the previous-frame
replay carrier is now named separately from paint-cache generation and counter control. M4H moved
replay range validation, op slicing, and text blob side-index replay into that carrier.
M4I deleted the obsolete `FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING` runtime branch, so
view-cache-active paint-cache recording can no longer bypass boundary ownership for non-boundary
nodes.
M4J promoted local hit-test-only paint-cache replay to the canonical paint-cache path, prevented
descendant-originated hit-test-only dirtiness from replaying ancestors, and deleted
`FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY`.
M4K explicitly retains `PreviousFramePaintRecording` inside `PaintCacheState` as the per-tree
previous-frame linear scene recording source. Boundary `PaintCacheEntry` metadata remains
boundary-owned, but the recording source is intentionally not duplicated into every
`ViewBoundaryState` while the current `Scene` contract is one tree-wide display list.
M4N deletes the remaining known layout default-path env branches and promotes subtree dirty
aggregation, on-demand layout-engine sweep, translation-only request/build skip, and clean
barrier-child retention to canonical behavior. Validation-only subtree dirty aggregation env knobs
remain because they audit the canonical path instead of selecting a separate runtime path.
M4O canonicalizes cache-root containment diagnostics around `layout_dependency`: cache-root bundle
records now carry that boundary vocabulary, report summaries prefer `debug.boundaries[]`
`layout_dependency`.
M4P removes the live retained `ViewCacheFlags::contained_layout` runtime field and stores
`ViewCacheParentLayoutDependency` instead. Hot paths that still need a boolean use a derived
`layout_contained_when_bounds_known()` predicate, and cache-root mount/reuse/layout/paint tracing spans now
emit `layout_dependency`.
M4Q deletes the remaining live `contained_layout` cache-root compatibility field from new
diagnostics bundle/report schemas and removes `fret-diag` fallback parsing for old cache-root
`contained_layout` values. Fixture-driven invalidation scenarios now use
`layout_contained_when_bounds_known` for the low-level test input that maps into dependency
metadata.
M4R validates the second, non-code-editor proof surface through
`ui-gallery-view-cache-toggle-perf-steady`. That surface exercises shared view-cache reuse and
paint-cache replay, emits canonical `debug.boundaries[]` plus cache-root `layout_dependency`
diagnostics, and confirms the live `contained_layout` schema field was not reintroduced.

The next convergence step is to move remaining tree-wide or node-local side products under
boundary ownership where feasible:

- dirty phase bits and reuse rejection reasons,
- prepaint-derived geometry state,
- hitbox and input-handler snapshot slices,
- semantics bounds and accessibility reachability inputs,
- text-layout indexes used by editor-like surfaces,
- `SceneFragment` records that can be replayed or encoded independently.

The next convergence step MUST NOT move cross-layer products into a boundary merely to reduce
storage breadth. If a product observes active layers, modal barriers, focus/capture, command
registries, relation normalization, pointer occlusion, or multiple roots in one frame, its owner is
the window/layer forest until a separate proof shows an equivalent boundary-local contract.

### 3. Layout containment is a dependency contract

Contained layout must stop being only a local optimization flag.

In v2, a boundary declares how its layout depends on its parent:

- parent width dependent,
- parent height dependent,
- scale-factor dependent,
- theme/style dependent,
- content intrinsic-size dependent,
- or independent except for translation.

The layout solver can then choose the smallest correct solve set.

If a boundary declares that only its content rect changed and its parent shell constraints are
unchanged, live resize should not repeatedly solve unrelated shell wrappers.

### 4. Prepaint is the place for geometry-derived state

Prepaint becomes the canonical phase for state that depends on resolved geometry but should not be
recomputed inside paint:

- visible row/window computation,
- scroll extents and scrollbar geometry,
- overlay anchor geometry,
- hitbox registration inputs,
- editor frame state such as visible rows, selection/caret projection, and syntax/rich cache
  prefetch windows,
- resource lifetime touch plans that must be known before paint replay.

Paint should consume prepaint state and emit or replay scene fragments. It should not perform broad
model reads, window derivation, or expensive geometry scans except through explicitly documented
escape hatches.

### 5. Paint replay uses explicit scene fragments

Paint caching should converge on boundary-owned `SceneFragment` reuse:

- a fragment records scene ops plus required side indexes such as text blobs and resources,
- replay can translate or transform the fragment when the boundary dependency keys allow it,
- debug diagnostics report why a fragment was replayed or rejected,
- and renderer prepare/encode counters remain separate from runtime paint counters.

This preserves Fret's `Scene` contract while making paint reuse a first-class boundary outcome.

### 6. The migration is allowed to delete old internal paths

The v2 migration is a fearless refactor:

- internal compatibility is not a goal,
- redundant env knobs and private cache paths should be removed once the v2 path has gates,
- old paths may stay only behind an explicit short-lived migration note,
- and the workstream must include a deletion audit before closeout.

Public app-facing breakage still needs a migration note when it affects published or documented
surfaces, but private runtime compatibility must not block the correct architecture.

### 7. Code editor is the first vertical slice

The first implementation slice should use the code editor and UI Gallery content boundary because
it exercises the runtime phases under real pressure:

- resize,
- text-heavy paint,
- row-scene replay,
- scroll/window derivation,
- and renderer payload counters.

The first success criteria are not "all of Frame Pipeline v2 exists"; they are:

- the code-editor resize/paint path uses explicit boundary attribution,
- the measured bottleneck moves for the expected reason,
- the old ad hoc path can be deleted or narrowed,
- and the perf gate proves at least a 20-30% p95 or max improvement on the selected bottleneck.

The global closeout criteria are stricter and are tracked by the workstream completion contract:
at least two proof surfaces must pass, including the code-editor resize/paint surface and one
broader non-code-editor view-cache or paint-cache surface. M4R provides the second proof surface,
and `FINAL_CLOSEOUT_AUDIT_2026-05-14.md` closes the workstream with the final retained/deleted
runtime path audit plus the full final gate set.

## Consequences

### Positive

- Runtime work becomes easier to attribute by phase and boundary.
- Editor-grade surfaces can become local by construction instead of by scattered knobs.
- Prepaint-driven surfaces can share one mechanism instead of each building custom frame-state
  staging.
- Old cache and containment paths can be deleted intentionally after migration.

### Negative / Costs

- This is a hard runtime contract change and will touch `crates/fret-ui` internals.
- Existing diagnostics and tests must be updated to report boundary-level reasons.
- Some current helper APIs may become migration-only or disappear.
- Short-term churn is expected in code-editor, view-cache, paint-cache, and layout invalidation
  tests.

## Validation Requirements

Each implementation slice must leave:

- a smallest repro script or demo,
- a correctness gate,
- a perf gate or baseline check,
- a worst-bundle attribution summary,
- and a deletion/migration note for any old path replaced by the slice.

The first code-editor slice should gate at minimum:

- `ui-code-editor-resize-probes` on macOS M4,
- a code-editor paint-focused stressor once introduced,
- focused `fret-ui` boundary/layout/prepaint/paint tests,
- `python3 tools/check_layering.py`.

## References

- `docs/architecture.md`
- `docs/golden-architecture.md`
- `docs/adr/0028-declarative-elements-and-element-state.md`
- `docs/adr/0055-frame-recording-and-subtree-replay-caching.md`
- `docs/adr/0165-dirty-views-and-notify-gpui-aligned.md`
- `docs/adr/0175-prepaint-windowed-virtual-surfaces.md`
- `docs/adr/0178-ephemeral-prepaint-items-v1.md`
- `docs/adr/0213-cache-roots-and-cached-subtree-semantics-v1.md`
- `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`
- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`
- `repo-ref/zed/crates/gpui/src/view.rs`
- `repo-ref/zed/crates/gpui/src/window.rs`
- `repo-ref/zed/crates/gpui/src/taffy.rs`
