# M0 Contract Freeze - 2026-05-14

Status: complete for ADR contract acceptance; global implementation remains active

## Purpose

This note closes the M0 contract-lock gap for the Frame Pipeline v2 workstream.

ADR 0327 is now accepted as the target contract for the global refactor. This is a contract freeze,
not a global implementation closeout.

## Review Result

The accepted contract remains the same target shape already used by the code-editor vertical slice:

- named frame phases: schedule/dirty propagation, build, request layout, layout, prepaint, paint,
  renderer prepare/encode/upload/present;
- `ViewBoundary` or final equivalent as the canonical execution boundary;
- boundary-owned dependency keys, dirty state, prepaint state, scene fragments, and diagnostics;
- layout containment as dependency metadata rather than only a local optimization flag;
- prepaint as the owner of geometry-derived state;
- scene-fragment replay as the paint-cache direction;
- deletion or explicit retention of replaced old runtime paths.

## Evidence

The contract is backed by the completed code-editor vertical slice:

- `CLOSEOUT_AUDIT_2026-05-14.md`
- `M2B_VIEW_BOUNDARY_PREPAINT_STATE_SLICE_2026-05-13.md`
- `M3C_BOUNDARY_SCENE_FRAGMENT_CARRIER_SLICE_2026-05-14.md`
- `M4A_BOUNDARY_DIRTY_SET_SLICE_2026-05-14.md`
- `M4B_BOUNDARY_DIAGNOSTICS_CANONICALIZATION_SLICE_2026-05-14.md`
- `EVIDENCE_AND_GATES.md`

The slice proves:

- minimal `ViewBoundaryState` / `BoundaryId` ownership exists for the migrated path;
- prepaint outputs and row scene fragments are boundary-owned for code-editor row replay;
- contained relayout dirty state lives under boundary state for the migrated path;
- `debug.boundaries[]` is the canonical boundary diagnostics list;
- the nested `debug.cache_roots[].boundary` compatibility schema is retired;
- the selected paint-side bottleneck improved beyond the required 20-30% threshold.

## What This Does Not Close

The global refactor remains incomplete while:

- broader view-cache rendered/next maps still have separate ownership;
- broader paint-cache replay stores still have separate ownership;
- direct page-specific `contained_layout` remains the practical authoring hint;
- older paint-cache/layout env knobs still need owner-specific deletion or retention decisions;
- a second non-code-editor proof surface has not yet validated the final model.

## Files Updated

- `docs/adr/0327-frame-pipeline-v2-and-view-boundaries.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/PROGRESS.md`
- `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/TODO.md`
- `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json`

## Verification

This is a documentation and contract-state slice. It does not change runtime code.

Required checks:

```bash
python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json > /dev/null
python3 tools/check_workstream_catalog.py
git diff --check
```

## Next Work

Continue with the next global-contract follow-ons:

- design the boundary-hint API that can replace direct `contained_layout`;
- migrate a broader view-cache or paint-cache ownership path into boundary state;
- choose and gate a second non-code-editor proof surface;
- record deletion or retention decisions for old runtime paths as each replacement lands.
