# Progress Ledger

Status: closed; global Frame Pipeline v2 completion contract satisfied
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
| ADR 0327 contract | Accepted; closeout aligned with known future follow-ons | `docs/adr/0327-frame-pipeline-v2-and-view-boundaries.md` is accepted as the target contract; `M0_CONTRACT_FREEZE_2026-05-14.md` records the contract freeze; `FINAL_CLOSEOUT_AUDIT_2026-05-14.md` records the global closeout. | Start narrow follow-ons for new scope. |
| Code-editor vertical slice | Complete | `CLOSEOUT_AUDIT_2026-05-14.md` closes the slice with correctness, layering, diagnostics, deletion, and perf evidence. | Keep as the first proof surface and regression gate. |
| Boundary runtime core | Complete for the accepted workstream contract | `ViewBoundaryState` owns dirty, prepaint, scene-fragment state, boundary paint-cache entry metadata, and boundary diagnostics. `ViewCacheBuildBoundaryStore`, `UiTree::retained_paint_cache_entries`, and `PreviousFramePaintRecording` are retained intentionally with current workstream/ADR reasons in M4M, M4L, M4K, and `FINAL_CLOSEOUT_AUDIT_2026-05-14.md`. | Start narrow follow-ons for identity or renderer-contract changes. |
| Boundary diagnostics | Complete | `debug.boundaries[]` is canonical; nested `debug.cache_roots[].boundary` is retired. Cache-root summaries use `layout_dependency`, and final proof bundles report `contained_layout_count=0`. | Keep as a regression gate. |
| Prepaint ownership | Complete for selected proof surfaces | Code-editor row-derived state moved out of paint into boundary-owned prepaint/scene-fragment state; future geometry-derived surfaces need their own proof lanes. | Start follow-ons only with a new proof surface. |
| Scene-fragment replay | Complete for selected proof surfaces with retained scene recording | `CanvasSceneFragment<RowSceneFragmentPayload>` is boundary-owned for the code-editor row path. Boundary `PaintCacheEntry` replay metadata is owned by `ViewBoundaryState::paint_cache`; `PreviousFramePaintRecording` remains the accepted per-tree linear scene source. | Renderer/display-list evolution is outside this lane. |
| Layout containment | Authoring API replaced; diagnostics vocabulary canonicalized; retained runtime flag and live schema compatibility field removed; second proof surface validated | `M4C_BOUNDARY_HINT_API_SLICE_2026-05-14.md` introduces `ViewBoundaryHints` and first-party `contain_layout_when_bounds_known(...)` authoring. `M4D_VIEW_CACHE_BUILD_BOUNDARY_STORE_SLICE_2026-05-14.md` removes flat element-runtime view-cache rendered/next side maps, but runtime still maps boundary hints into low-level view-cache flags. M4O makes cache-root diagnostics report `layout_dependency` as the primary containment explanation. M4P replaces the retained `ViewCacheFlags::contained_layout` boolean with `ViewCacheParentLayoutDependency`, and M4Q deletes `contained_layout` from new cache-root bundle/report schemas plus fixture vocabulary. M4R validates `contained_when_bounds_known` and `parent_dependent` layout dependencies on the non-code-editor view-cache surface. | Preserve in final closeout audit. |
| Old-path deletion | Complete for replaced slice paths | Closeout audit records deleted node-owned prepaint storage, row replay carriers, dirty cache-root maps, and nested boundary diagnostics. | Keep deleting only when a replacement path has gates and evidence. |
| Perf gate | Complete | Final closeout reran both proof surfaces. Code-editor gate reports `failures=[]`, worst total `1601us`, row scene replay hit rate `99%`. View-cache toggle gate reports `failures=[]`, worst total `593us`, stable view-cache reuse `2/2`. | Keep baselines as regression gates. |
| Env knob cleanup | Complete for current known runtime default-path knobs | `M4I_PAINT_CACHE_RELAX_VIEW_CACHE_GATING_DELETION_SLICE_2026-05-14.md` deletes the live `FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING` runtime branch now that view-cache-active paint-cache ownership is boundary-gated. `M4J_HIT_TEST_ONLY_PAINT_CACHE_REPLAY_DEFAULT_SLICE_2026-05-14.md` promotes local hit-test-only paint-cache replay to canonical behavior and deletes `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY`; descendant-originated hit-test-only dirtiness still repaints ancestors. `M4N_LAYOUT_ENV_KNOB_CANONICALIZATION_SLICE_2026-05-14.md` deletes the live layout default-path env branches and promotes subtree dirty aggregation, on-demand layout-engine sweep, translation-only request/build skip, and clean barrier-child retention to canonical behavior. | Keep validation/debug knobs only when they verify the canonical path. |

## Current Done Boundary

The following statement is the current authoritative completion boundary:

> The code-editor vertical slice is complete, the second non-code-editor view-cache proof surface is
> validated, and the final closeout batch plus deletion/retention audit have passed. The global
> Frame Pipeline v2 completion contract is satisfied for this workstream.

Do not reopen the closed slice unless fresh evidence shows its gates or diagnostics are wrong.
Start narrower follow-ons for future work:

- Implementation against accepted ADR 0327.
- Broader runtime consolidation after the public/ecosystem boundary hint design landed in
  `M4C_BOUNDARY_HINT_API_SLICE_2026-05-14.md`.
- Wider view-cache rendered/next map consolidation after the M4D build-boundary store slice and the
  M4M explicit retention decision.
- Wider paint-cache entry-store consolidation after the M4E/M4F paint-cache entry ownership
  slices, the M4G previous-frame recording split, the M4H replay span owner narrowing, the M4K
  explicit retention decision for the per-tree previous-frame recording source, and the M4L
  retained plain-node entry-store decision.
- Layout default-path env knob cleanup after M4I/M4J deleted the obsolete paint-cache env switches
  and M4N deleted or canonicalized the remaining known layout default-path switches.
- Cache-root containment diagnostics cleanup after M4O made `layout_dependency` the primary
  cache-root report vocabulary, M4Q deleted the derived `contained_layout` compatibility field
  from new bundle/report schemas, and M4R proved the second surface emits canonical dependency
  diagnostics without reintroducing the retired field.
- Retained runtime containment flag cleanup after M4P replaced `ViewCacheFlags::contained_layout`
  with `ViewCacheParentLayoutDependency`.
- Additional proof surfaces beyond the two required here.

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

Closeout result:

- view-cache build-time rendered/next maps are consolidated in `ViewCacheBuildBoundaryStore`, and
  M4M explicitly retains that store inside `WindowElementState` as the `GlobalElementId`-keyed
  declarative build-boundary mechanism instead of migrating it into `ViewBoundaryState`.
- boundary-node paint-cache entries are now owned by `ViewBoundaryState::paint_cache`, and the
  node-owned entry fallback is deleted. Plain retained paint-cache entries use
  `UiTree::retained_paint_cache_entries` as an explicit retained plain-node entry store. `PaintCacheState` intentionally owns
  `PreviousFramePaintRecording` as the retained per-tree previous-frame scene recording source; M4K
  records this as an explicit retention decision because ordinary paint-cache replay still indexes a
  linear tree-wide `Scene` recording.
- the live retained `ViewCacheFlags::contained_layout` runtime field is gone after M4P, and M4Q
  deletes the remaining live bundle-schema/report compatibility field plus fixture scenario
  vocabulary. Historical documentation can still mention the retired name as evidence.
- known old paint-cache/layout default-path env knobs have deletion or canonicalization decisions:
  `FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING` is deleted,
  `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY` is promoted to default behavior and deleted, and M4N
  deletes the live layout default-path branches while retaining only subtree aggregation validation
  knobs.
- boundary diagnostics are canonical for the code-editor slice and the M4R second proof surface;
  `FINAL_CLOSEOUT_AUDIT_2026-05-14.md` records final bundle/schema checks, final gates, and the
  deletion/retention audit.

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

This lane is now closed by `FINAL_CLOSEOUT_AUDIT_2026-05-14.md`. Do not reopen it from chat memory;
start a narrow follow-on if new evidence shows a fresh requirement.

## Follow-On Policy

Recommended next action:

> Treat this lane as closed. Start a narrower follow-on for any new proof surface, renderer
> display-list evolution, Linux-specific performance closure, or ecosystem policy work.

Each follow-on should still keep the same evidence discipline: repro, correctness gate, perf gate
when relevant, worst-bundle attribution for perf claims, and explicit deletion/retention notes.
