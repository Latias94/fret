# Progress Ledger

Status: active follow-on; code-editor vertical slice complete; global refactor not complete
Last updated: 2026-05-14

This document is the first-open progress ledger for the Frame Pipeline v2 global refactor.

Use it to answer three questions quickly:

- why the refactor exists,
- what the target architecture is,
- and which parts are complete versus still open.

Detailed slice evidence remains in the dated implementation notes and `EVIDENCE_AND_GATES.md`.
This ledger should stay short and should link out instead of duplicating every gate result.

## Why This Refactor Exists

Fret's current UI runtime is already capable of real apps, diagnostics, and targeted performance
work. The remaining problem is structural rather than a single slow function:

- retained `UiTree` state, declarative mounting, view-cache reuse, layout containment, prepaint-like
  staging, paint-cache replay, and code-editor row replay still read as overlapping mechanisms
  instead of one direct execution model;
- local changes such as a code-editor resize can still become broader layout or paint work unless
  the correct containment/cache path is manually aligned;
- paint can still mix geometry-derived state, resource touch, cache decisions, and scene emission;
- diagnostics are useful, but the long-term story needs phase and boundary attribution as the
  canonical explanation for reuse, rejection, and cost;
- old private paths must be deleted once the replacement path is proven, otherwise local
  optimizations will turn into permanent compatibility layers.

The goal is not to copy Zed/GPUI internals. The goal is to give Fret an equally direct,
phase-attributable execution model while preserving Fret's mechanism-vs-policy layering.

## Target Architecture

The target frame pipeline is:

```text
schedule / dirty propagation
  -> build
  -> request layout
  -> layout
  -> prepaint
  -> paint
  -> renderer prepare / encode / upload / present
```

The target runtime unit is a `ViewBoundary` or final equivalent. A boundary should own enough state
to answer:

- did build/layout/prepaint/paint need to run?
- which dependency key changed?
- which dirty flag forced work?
- can geometry-derived prepaint state be reused?
- can a scene fragment be replayed?
- did hit testing, semantics, or renderer resources need refresh?
- why was reuse rejected?

The current target is documented in:

- `docs/adr/0327-frame-pipeline-v2-and-view-boundaries.md`
- `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`

## Scope Guardrails

In scope:

- `crates/fret-ui` runtime internals for boundary state, dirty propagation, layout/prepaint/paint
  ownership, cache/replay mechanics, and diagnostics.
- Editor-grade proof surfaces that reveal frame-pipeline pressure, starting with UI Gallery code
  editor resize/paint.
- Deleting or retiring old private paths once a v2 replacement has correctness and perf evidence.

Out of scope:

- Linux-specific performance closure.
- Replacing Fret with Zed/GPUI code.
- Moving shadcn/Radix/Base UI interaction policy into `crates/fret-ui`.
- Rewriting the renderer display-list contract as part of this lane.

## Progress Summary

| Track | Status | Evidence | Next action |
| --- | --- | --- | --- |
| ADR 0327 contract | Accepted; implementation in progress | `docs/adr/0327-frame-pipeline-v2-and-view-boundaries.md` is accepted as the target contract; `M0_CONTRACT_FREEZE_2026-05-14.md` records the contract freeze. | Continue implementation against the accepted contract. |
| Code-editor vertical slice | Complete | `CLOSEOUT_AUDIT_2026-05-14.md` closes the slice with correctness, layering, diagnostics, deletion, and perf evidence. | Keep as the first proof surface and regression gate. |
| Boundary runtime core | Partial global, complete for the slice | `ViewBoundaryState` owns dirty, prepaint, scene-fragment state, and boundary-node paint-cache entries. Plain retained paint-cache entries now use `UiTree::boundary_paint_cache_entries`, a boundary-shaped side store, instead of `Node`. `M4D_VIEW_CACHE_BUILD_BOUNDARY_STORE_SLICE_2026-05-14.md` consolidates view-cache build-time rendered/next maps into a single runtime store. `M4E_BOUNDARY_PAINT_CACHE_ENTRY_SLICE_2026-05-14.md` moves boundary-node `PaintCacheEntry` ownership into `ViewBoundaryState::paint_cache`; `M4F_NODE_PAINT_CACHE_FALLBACK_DELETION_SLICE_2026-05-14.md` deletes the remaining `Node::paint_cache` fallback; `M4G_PREVIOUS_FRAME_PAINT_RECORDING_SLICE_2026-05-14.md` splits previous-frame paint recording storage out of paint-cache generation/counter control. | Migrate or explicitly retain the consolidated build-boundary store under the final `ViewBoundary` model, then decide the final owner for `PreviousFramePaintRecording` and the plain paint-cache side-store shape. |
| Boundary diagnostics | Complete for the slice | `debug.boundaries[]` is canonical; nested `debug.cache_roots[].boundary` is retired. | Keep diagnostics stable while broader boundary stores are consolidated. |
| Prepaint ownership | Partial global, complete for the slice | Code-editor row-derived state moved out of paint into boundary-owned prepaint/scene-fragment state. | Audit other geometry-derived paint work and migrate only with proof surfaces. |
| Scene-fragment replay | Partial global, complete for row replay and node-fallback deletion | `CanvasSceneFragment<RowSceneFragmentPayload>` is boundary-owned for the code-editor row path. Boundary `PaintCacheEntry` replay metadata is owned by `ViewBoundaryState::paint_cache`; plain paint-cache entries are no longer node-owned and use the boundary-shaped side store. `PreviousFramePaintRecording` now names the remaining previous-frame scene recording carrier. | Decide final previous-frame recording owner and plain-entry store shape for non-code-editor surfaces. |
| Layout containment | Authoring API replaced; runtime consolidation still partial | `M4C_BOUNDARY_HINT_API_SLICE_2026-05-14.md` introduces `ViewBoundaryHints` and first-party `contain_layout_when_bounds_known(...)` authoring. `M4D_VIEW_CACHE_BUILD_BOUNDARY_STORE_SLICE_2026-05-14.md` removes flat element-runtime view-cache rendered/next side maps, but runtime still maps boundary hints into low-level view-cache flags. | Consolidate remaining internal `contained_layout` flags/debug fields when final boundary dependency ownership lands. |
| Old-path deletion | Complete for replaced slice paths | Closeout audit records deleted node-owned prepaint storage, row replay carriers, dirty cache-root maps, and nested boundary diagnostics. | Keep deleting only when a replacement path has gates and evidence. |
| Perf gate | Complete for the slice | `paint.widget` p95 improved from `1494us` to `650us`; total p95 improved from `1811us` to `1396us`. | Add a stricter code-editor paint stressor only if resize probes stop catching regressions. |
| Env knob cleanup | Open | Older paint-cache/layout knobs remain intentionally out of this slice. | Decide ownership and deletion policy in the relevant follow-on workstreams. |

## Current Done Boundary

The following statement is the current authoritative completion boundary:

> The code-editor vertical slice is complete. The global Frame Pipeline v2 refactor is not complete.

Do not reopen the closed slice unless fresh evidence shows its gates or diagnostics are wrong.
Continue this workstream for broader ADR 0327 follow-ons:

- Implementation against accepted ADR 0327.
- Broader runtime consolidation after the public/ecosystem boundary hint design landed in
  `M4C_BOUNDARY_HINT_API_SLICE_2026-05-14.md`.
- Wider view-cache rendered/next map consolidation after the M4D build-boundary store slice.
- Wider paint-cache previous-op-range and scene-fragment replay consolidation after the M4E/M4F
  paint-cache entry ownership slices and the M4G previous-frame recording split.
- Ownership and deletion decisions for older paint-cache/layout env knobs.

## Completion Contract

This workstream is complete only when the global refactor has one canonical runtime path and no
parallel old path is still required for correctness, diagnostics, or perf.

Final result:

- ADR 0327 is accepted, revised into an accepted ADR, or explicitly superseded by an accepted
  equivalent.
- The final `ViewBoundary` or renamed equivalent is the canonical runtime owner for build, layout,
  prepaint, paint, and boundary diagnostics.
- View-cache root, paint-cache replay, layout containment, prepaint outputs, and scene-fragment
  replay are either owned by the boundary model or explicitly documented as separate mechanisms with
  a current ADR reason.
- Direct page-specific `contained_layout` authoring hints are replaced by a reviewed boundary-hint
  API, or retained only behind an explicit accepted decision that explains why the direct hint is
  the final contract.
- Old private runtime paths replaced by the boundary model are deleted. Any retained compatibility
  path must have an owner, an ADR/workstream reason, and a gate proving it is still required.
- Boundary diagnostics are the canonical explanation for build/layout/prepaint/paint reuse or
  rejection. Derived report summaries may exist, but bundle schemas should not reintroduce parallel
  boundary truth.
- At least two proof surfaces validate the final model:
  - the current code-editor resize/paint surface,
  - and one broader non-code-editor surface that exercises shared view-cache or paint-cache reuse.
- Perf gates show no regression on the code-editor proof surface and show a measurable improvement
  or justified neutral result on the second proof surface.
- `python3 tools/check_layering.py`, focused nextest gates, relevant `cargo check` commands, and
  `git diff --check` pass for the closeout batch.
- The closeout audit states which old paths were deleted, which paths remain intentionally, and
  which future work is outside this refactor rather than unfinished Frame Pipeline v2 work.

Not complete:

- `ViewBoundaryState` does not yet canonically own all build/layout/prepaint/paint state across the
  final proof surfaces.
- view-cache build-time rendered/next maps are consolidated in `ViewCacheBuildBoundaryStore`, but
  that store is not yet the final `ViewBoundaryState` owner.
- boundary-node paint-cache entries are now owned by `ViewBoundaryState::paint_cache`, and the
  node-owned entry fallback is deleted. Plain retained paint-cache entries use
  `UiTree::boundary_paint_cache_entries`, and `PaintCacheState` still owns
  `PreviousFramePaintRecording` as the named previous-frame scene recording carrier.
- internal low-level `contained_layout` flags and diagnostic fields remain after public authoring
  moved to boundary hints.
- old env knobs or compatibility paths remain without an owner and a deletion/retention decision.
- diagnostics require reading both old cache-root boundary data and new boundary data as independent
  sources of truth.

## Progress Tracking Rules

Every future refactor batch must update the workstream before closeout:

- `PROGRESS.md`: update the progress table, the done boundary, and completion-contract status.
- `TODO.md`: mark the specific task done or add the next unresolved task.
- `MILESTONES.md`: update the milestone status and explain whether the batch is a local slice or a
  global-contract step.
- `EVIDENCE_AND_GATES.md`: add repro, correctness gates, perf gates, and worst-bundle attribution
  for any perf claim.
- `WORKSTREAM.json`: add new authoritative docs, evidence anchors, and changed continuation policy.
- `CLOSEOUT_AUDIT_YYYY-MM-DD.md` or a dated slice note: record deletion decisions for replaced old
  paths.
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`: update when the batch changes ADR 0327 implementation
  status.

Do not mark the global refactor complete from chat memory. Mark it complete only when the completion
contract above is satisfied and a final closeout audit records the evidence.

## Long-Running Goal Candidate

Recommended next goal:

> Continue the Frame Pipeline v2 global refactor from the workstream documents until the completion
> contract is satisfied: keep `PROGRESS.md`, `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`,
> `WORKSTREAM.json`, and ADR alignment current after every slice; continue implementation against
> accepted ADR 0327; make the final boundary model the canonical owner for build/layout/prepaint/
> paint diagnostics and reuse; replace direct `contained_layout` with a final boundary-hint decision;
> migrate broader view-cache and paint-cache paths to boundary-owned state; delete replaced old
> runtime paths; and close the lane only after correctness gates, perf gates, worst-bundle
> attribution, layering checks, and a final deletion audit prove the old path is no longer needed.

This is intentionally a program goal rather than a single-slice goal. Each landable batch should
still be small enough to review, test, and close with evidence.
