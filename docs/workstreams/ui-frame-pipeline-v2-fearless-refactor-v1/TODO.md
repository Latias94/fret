# TODO

Status: Code-editor vertical slice complete; broader ADR 0327 lane active
Last updated: 2026-05-14

Progress ledger:

- `PROGRESS.md` is the first-open status summary for the global refactor. It records why the
  refactor exists, the target execution model, what is complete, and what remains open after the
  code-editor vertical slice closeout.
- `PROGRESS.md#completion-contract` is the authoritative completion definition for the global
  refactor. Do not mark this workstream complete until that contract is satisfied by evidence.

## Global Completion Contract

- [x] ADR 0327 is accepted, revised into an accepted ADR, or superseded by an accepted equivalent.
  ADR 0327 is accepted as the contract freeze in `M0_CONTRACT_FREEZE_2026-05-14.md`.
- [ ] The final `ViewBoundary` or renamed equivalent is the canonical runtime owner for
  build/layout/prepaint/paint reuse and diagnostics.
- [ ] Broader view-cache rendered/next maps are consolidated into boundary-owned state or retained
  behind an accepted ADR/workstream reason. M4D consolidates element-runtime build-time maps into
  `ViewCacheBuildBoundaryStore`; final `ViewBoundaryState` ownership remains open.
- [ ] Broader paint-cache replay stores are consolidated into boundary-owned scene-fragment state or
  retained behind an accepted ADR/workstream reason. M4E moves boundary-node `PaintCacheEntry`
  ownership into `ViewBoundaryState::paint_cache`; M4F deletes the remaining node-owned
  `PaintCacheEntry` fallback and introduces a plain-node boundary-shaped side store; M4G splits the
  previous-frame scene recording into `PreviousFramePaintRecording`; M4H moves replay range
  validation and text side-index replay into that recording carrier; final recording ownership
  remains open.
- [x] Direct page-specific `contained_layout` authoring hints are replaced by a reviewed
  boundary-hint API. `M4C_BOUNDARY_HINT_API_SLICE_2026-05-14.md` introduces
  `ViewBoundaryHints` and first-party `contain_layout_when_bounds_known(...)` authoring.
- [ ] Replaced old runtime paths and compatibility diagnostics are deleted, with retained paths
  named in a deletion/retention audit.
- [ ] Boundary diagnostics remain the canonical bundle truth for boundary reuse/rejection; any
  cache-root summaries are derived report views.
- [ ] At least two proof surfaces validate the final model: code-editor resize/paint plus one
  broader non-code-editor view-cache or paint-cache surface.
- [ ] Correctness gates, perf gates, worst-bundle attribution, `python3 tools/check_layering.py`,
  relevant `cargo check`, and `git diff --check` pass for the final closeout batch.
- [ ] A final closeout audit states what was deleted, what intentionally remains, and why no old path
  is still required for the Frame Pipeline v2 contract.

## P0 Contract Setup

- [x] Create workstream lane for Frame Pipeline v2.
- [x] Add a first-open global progress ledger that distinguishes the completed code-editor
  vertical slice from the broader active ADR 0327 lane: `PROGRESS.md`.
- [x] Add ADR 0327 as the proposed frame-pipeline and boundary contract.
- [x] Review ADR 0327 and either accept it or revise it before broad code migration.
  ADR 0327 is accepted as the target contract; implementation remains partial globally.
- [x] Add an assumptions-first baseline audit of current `UiTree` build/layout/prepaint/paint paths:
  `M0_BASELINE_AUDIT_2026-05-13.md`.
- [x] Add a source map of old paths that are migration candidates:
  - `ViewCacheProps::contained_layout`,
  - view-cache root bookkeeping,
  - paint-cache replay bookkeeping,
  - code-editor-local frame state,
  - layout invalidation propagation,
  - prepaint diagnostics.

## P1 First Vertical Slice: Code Editor Boundary

- [x] Define the first internal `BoundaryId` / boundary-state shape or a narrower transitional
  equivalent.
- [x] Make UI Gallery code-editor content root report boundary-level reuse/reject reasons through
  first-class `debug.boundaries[]` diagnostics. The original transitional
  `debug.cache_roots[].boundary` path is retired in
  `M4B_BOUNDARY_DIAGNOSTICS_CANONICALIZATION_SLICE_2026-05-14.md`.
- [x] Move code-editor frame-derived row state toward shared prepaint ownership for the
  windowed-rows/editor prefetch slice.
- [x] Split code-editor paint attribution into transitional prepaint plan, paint replay, and renderer
  payload buckets for the row scene replay-plan slice.
- [x] Add or promote a stricter code-editor paint stressor if resize probes are no longer sensitive
  enough. Current closeout keeps `ui-code-editor-resize-probes` because it still catches and proves
  the selected paint-side bottleneck.
- [x] Prove `paint.widget` or total p95/max improves by at least 20-30% on the selected bottleneck
  after the final boundary-owned scene-fragment store replaces the transitional editor-owned plan.

## P2 Runtime Migration

- [x] Convert layout containment from a standalone flag into boundary dependency metadata.
- [x] Move contained-relayout dirty reasons from cache-root side maps into `ViewBoundaryState`.
- [x] Convert the code-editor row-scene replay carrier into boundary-owned scene-fragment state.
- [x] Promote boundary-owned scene-fragment reuse diagnostics and perf closeout evidence for the
  code-editor vertical slice.
- [x] Make prepaint diagnostics first-class per boundary.
- [x] Remove duplicated or superseded debug counters after boundary diagnostics cover them for the
  retired nested cache-root boundary path.
- [x] Replace code-editor-owned `RowSceneReplayPlan` with a boundary-owned prepaint output
  carrier.
- [x] Move the transitional replay-plan carrier into boundary-owned fragment state or delete it if a
  narrower direct replay contract replaces it.
- [x] Keep `fret-ui` mechanism-only; move any policy decisions back to ecosystem crates.

## P3 Delete Old Paths

- [x] Write a deletion audit before closeout.
- [x] Delete or retire old private paths that v2 replaces for the code-editor vertical slice. M4A removed
  `dirty_cache_roots` / `dirty_cache_root_reasons`, but broader view-cache/paint-cache compatibility
  views remain. M4B retired the nested `debug.cache_roots[].boundary` compatibility path.
- [x] Remove migration-only env knobs that no longer have a diagnostic purpose for this vertical
  slice. No migration-only env knob was introduced by the slice; retained env knobs are documented
  as out-of-scope diagnostics in `CLOSEOUT_AUDIT_2026-05-14.md`.
- [x] Update first-party examples and docs if public authoring guidance changes. No public
  authoring guidance changed in this vertical slice.
- [x] Update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` for the code-editor vertical slice state.

## Always-On Gates

- [x] `python3 tools/check_layering.py`
- [x] Focused `fret-ui` unit tests for any boundary/invalidation change.
- [x] `ui-code-editor-resize-probes` perf gate on macOS M4 for code-editor slices.
- [x] Worst-bundle `diag stats` attribution for every perf claim.

## Follow-On ADR 0327 Lane Work

- [x] Review ADR 0327 and either accept it or revise/supersede it before broad migration.
  Closed by `M0_CONTRACT_FREEZE_2026-05-14.md`.
- [x] Design and land a non-page-specific boundary hint API that replaces first-party direct
  `contained_layout` authoring hints. Landed in
  `M4C_BOUNDARY_HINT_API_SLICE_2026-05-14.md`.
- [ ] Consolidate broader view-cache rendered/next maps and paint-cache previous-op-range replay
  into final boundary-owned build/paint stores.
  - [x] Consolidate element-runtime view-cache build-time rendered/next side maps into one
    `ViewCacheBuildBoundaryStore` (`M4D_VIEW_CACHE_BUILD_BOUNDARY_STORE_SLICE_2026-05-14.md`).
  - [ ] Decide whether `ViewCacheBuildBoundaryStore` migrates into `ViewBoundaryState` directly or
    remains as an explicitly retained build-boundary mechanism.
  - [x] Move boundary-node `PaintCacheEntry` ownership into `ViewBoundaryState::paint_cache`
    (`M4E_BOUNDARY_PAINT_CACHE_ENTRY_SLICE_2026-05-14.md`).
  - [x] Delete the non-boundary node-owned `PaintCacheEntry` fallback and route plain paint-cache
    entries through `UiTree::boundary_paint_cache_entries`, with migration into
    `ViewBoundaryState::paint_cache` when the node becomes a true runtime boundary
    (`M4F_NODE_PAINT_CACHE_FALLBACK_DELETION_SLICE_2026-05-14.md`).
  - [x] Split paint-cache previous-frame recording storage out of generation/counter control into
    `PreviousFramePaintRecording`
    (`M4G_PREVIOUS_FRAME_PAINT_RECORDING_SLICE_2026-05-14.md`).
  - [x] Move previous-frame replay range validation, op slicing, and text side-index replay into
    `PreviousFramePaintRecording`
    (`M4H_PREVIOUS_FRAME_PAINT_REPLAY_SPAN_SLICE_2026-05-14.md`).
  - [ ] Decide whether `PreviousFramePaintRecording` migrates into `ViewBoundaryState`, becomes a
    boundary-owned scene-fragment source, or remains as an explicitly retained per-tree recording
    mechanism.
- [ ] Decide the future of older paint-cache/layout env knobs in their owning workstreams.
  - [x] Delete the obsolete `FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING` runtime branch
    (`M4I_PAINT_CACHE_RELAX_VIEW_CACHE_GATING_DELETION_SLICE_2026-05-14.md`).
  - [x] Promote hit-test-only paint-cache replay to default behavior and delete
    `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY`
    (`M4J_HIT_TEST_ONLY_PAINT_CACHE_REPLAY_DEFAULT_SLICE_2026-05-14.md`).
  - [ ] Decide layout aggregation/sweep env knobs in their owning workstreams.
