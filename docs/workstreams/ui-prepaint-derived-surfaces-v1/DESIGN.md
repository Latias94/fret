# UI Prepaint Derived Surfaces v1

Date: 2026-05-14
Status: Active design lane

## Problem

Frame Pipeline v2 closed the first boundary-owner migration for the accepted ADR 0327 contract, but
its required proof surfaces were intentionally narrow: code-editor resize/paint and one
non-code-editor view-cache toggle surface.

The next performance risk is not whether `ViewBoundaryState` can own runtime boundary state in the
abstract. The risk is whether boundary-owned prepaint outputs and scene fragments stay correct and
valuable across component-shaped surfaces that are not the code editor:

- retained virtual lists with window shifts, keep-alive rows, and known-height shortcuts;
- retained data tables with filter/sort/pinning/menu interactions and view-cache reuse;
- text-heavy or markdown/code-view surfaces that generate expensive derived paint data without the
  full code-editor row pipeline;
- later docking or node-graph surfaces where resize and interaction churn can invalidate large
  regions unless the boundary model stays local.

This lane exists to turn Frame Pipeline v2 from a successful vertical slice into a reusable
mechanism pattern.

## Relationship To Frame Pipeline v2

This is a narrow follow-on to `ui-frame-pipeline-v2-fearless-refactor-v1`.

The closed lane remains historical and should not be reopened for new surfaces. Its final closeout
keeps three mechanisms intentionally retained:

- `ViewCacheBuildBoundaryStore` in `WindowElementState`, keyed by `GlobalElementId`;
- `UiTree::retained_paint_cache_entries` for plain non-boundary paint-cache entries;
- `PreviousFramePaintRecording` in `PaintCacheState` while `Scene` remains a tree-wide linear
  recording source.

This lane must respect those retention decisions unless a slice produces evidence that a retained
mechanism is now the actual blocker. If that happens, the change must be promoted to a separate
ADR-backed renderer or identity follow-on instead of silently widening this lane.

## Target Model

The target model is:

```text
component policy / recipe
  -> stable boundary identity and hints
  -> boundary-owned derived prepaint output
  -> boundary-owned scene fragment or paint-cache metadata
  -> paint consumes only validated current boundary state
  -> diagnostics explain reuse, rejection, and dirty cause through debug.boundaries[]
```

Mechanism expectations:

- `ViewBoundaryState` remains the canonical retained-node runtime owner for boundary dirty state,
  layout dependency metadata, prepaint outputs, scene fragments, paint-cache metadata, and boundary
  diagnostics.
- Component or app surfaces may define policy-level recipes, but they must not own private
  previous-frame caches when an equivalent boundary-owned derived state is available.
- The runtime must be able to explain each reuse decision through stable diagnostics rather than
  through surface-specific counters only.
- Old local carrier paths should be deleted, narrowed, or explicitly retained with evidence.

## Scope

In scope:

- `crates/fret-ui`
  - boundary prepaint and scene-fragment APIs,
  - virtual-list and retained-window reconciliation hooks,
  - diagnostics for boundary reuse/rejection and virtual-list window shifts,
  - tests for boundary-owned derived state.
- `ecosystem/fret-ui-kit`
  - retained table/data-grid authoring patterns that exercise virtual-list/view-cache reuse,
  - recipe-level boundary hints when they are policy, not runtime mechanism.
- `apps/fret-ui-gallery`
  - proof surfaces and diag scripts for virtual-list and retained data-table stress.
- `crates/fret-diag`
  - stats/reporting only when needed to expose reusable proof metrics.

Out of scope:

- Rewriting the renderer `Scene` contract or introducing per-boundary recording sources.
- Linux-specific perf closure.
- Moving component interaction policy into `crates/fret-ui`.
- Replacing the code-editor row pipeline unless a proof surface shows it is the wrong abstraction.

## First Proof Surfaces

### M1: Retained Virtual List Derived Surface

Primary repro:

- `tools/diag-scripts/ui-gallery/perf/ui-gallery-virtual-list-torture-steady.json`

Why this first:

- It is already a formal perf contract on Windows.
- It exercises retained virtual-list reconciliation, window shifts, known heights, keep-alive rows,
  scroll handling, and view-cache membership.
- It is close enough to the code-editor row case to reuse the boundary model, but different enough
  to catch code-editor-only assumptions.

### M2: Retained Data Table / View-Cache Torture

Primary repro candidates:

- `tools/diag-scripts/suites/ui-gallery-data-table-retained/suite.json`
- `tools/diag-scripts/suites/ui-gallery-data-table-view-cache-torture/suite.json`
- `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change.json`

Why second:

- Data table mixes virtual-list behavior with policy-heavy component work.
- It can expose identity, view-cache membership, overlay/menu, and layout-dependency mistakes that
  a simpler virtual list will not.

### Later Surfaces

Later slices may add markdown/code-view, node graph canvas, or docking resize only after M1 and M2
show which mechanism actually limits reuse.

## ADR Position

No new ADR is required to start this lane.

ADR 0327 already owns the hard contract that `ViewBoundaryState` is the retained runtime boundary
owner for dirty/layout/prepaint/paint reuse and diagnostics. This lane applies that accepted
contract to additional proof surfaces.

Add a new ADR, or update ADR 0327 plus `docs/adr/IMPLEMENTATION_ALIGNMENT.md`, only if a slice:

- changes the renderer `Scene` recording contract;
- moves `PreviousFramePaintRecording` into a per-boundary owner;
- changes public boundary hint APIs;
- changes the accepted ownership of `ViewCacheBuildBoundaryStore` or
  `UiTree::retained_paint_cache_entries`;
- changes diagnostics schema in a way external tooling must treat as a new contract.

Current decision (2026-05-14): do not add an ADR for the current M1/M2 setup. The lane is still
applying ADR 0327's accepted boundary ownership model to more proof surfaces. A new ADR becomes
necessary only if the retained-table fix changes a public runtime contract, diagnostics schema, or
renderer recording ownership rather than fixing model propagation, cache ownership, or local
diagnostics within the existing contract.

## Current Measurement Snapshot

The first retained virtual-list pass is currently attribution-only evidence, not a new contract.
On the current machine profile, the representative bundle at
`target/fret-diag/ui-prepaint-derived-surfaces-v1-virtual-list-attrib/1778755322233/bundle.schema2.json`
shows:

- time sum: total/layout/prepaint/paint = `14396/12011/252/2133us`
- time p50/p95: total `3465/5515us`, layout `2994/4698us`, prepaint `56/76us`, paint `415/745us`
- hot p50/p95: `layout.engine_solve=845/1556us`, `paint.widget=179/355us`

The immediate conclusion is that layout invalidation breadth is still the dominant cost on this
surface. The boundary-owned virtual-list prepaint output is therefore a structural migration step
first, and a raw prepaint micro-optimization second.

## Completion Criteria

This lane is complete when:

- retained virtual-list and retained data-table proof surfaces have correctness gates, perf gates,
  and worst-bundle attribution recorded in this lane;
- derived prepaint/scene-fragment state used by those surfaces is boundary-owned or explicitly
  documented as retained with current evidence;
- diagnostics can explain boundary reuse/rejection and dirty causes for the proof surfaces without
  relying only on surface-specific counters;
- old local duplicate caches or transitional carriers touched by the lane are deleted or retained
  with a written reason;
- `python3 tools/check_layering.py`, `python3 tools/check_workstream_catalog.py`, focused
  `cargo nextest`, and the selected diag/perf gates pass.
