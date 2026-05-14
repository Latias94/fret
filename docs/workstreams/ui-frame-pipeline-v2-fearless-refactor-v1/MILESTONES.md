# Milestones

Status: Code-editor vertical slice complete; broader ADR 0327 lane active
Last updated: 2026-05-14

First-open progress ledger:

- `PROGRESS.md` summarizes the global refactor state. It is the quickest way to distinguish the
  closed code-editor vertical slice from the still-active ADR 0327 follow-on work.
- `PROGRESS.md#completion-contract` is the final acceptance contract for this global refactor.

## M0: Contract Lock

Exit criteria:

- ADR 0327 is reviewed and accepted, or replaced by a better accepted ADR.
- Target interface state is updated to match the accepted ADR.
- The old-path inventory exists.
- The progress ledger explains the current global state and next follow-on without reopening the
  closed vertical slice.
- The first code-editor repro and gate commands are current.

Status on 2026-05-13:

- Baseline/source inventory exists in `M0_BASELINE_AUDIT_2026-05-13.md`.
- First repro and gate commands are current in `EVIDENCE_AND_GATES.md`.
- Historical state at the time: ADR 0327 still needed review/acceptance or a superseding accepted
  ADR before broad migration. This is now closed by `M0_CONTRACT_FREEZE_2026-05-14.md`.

Status on 2026-05-14:

- `PROGRESS.md` now acts as the compact progress ledger for the global refactor. It records that
  the code-editor vertical slice is complete while global ADR 0327 implementation remains active
  follow-on work.
- `M0_CONTRACT_FREEZE_2026-05-14.md` accepts ADR 0327 as the target contract while keeping
  implementation status partial globally.
- M0 is complete as a contract lock. The workstream remains active for the M6 global completion
  contract.

## M1: Code Editor Boundary Pilot

Exit criteria:

- Code-editor UI Gallery content root has boundary-level diagnostics.
- The first runtime boundary state exists in code or a narrow transitional equivalent exists with a
  deletion plan.
- `ui-code-editor-resize-probes` still passes.
- `code_editor.paint_perf` remains non-zero and is correlated with boundary diagnostics.

Status on 2026-05-13:

- Historical note: the first boundary-diagnostics pilot used transitional
  `debug.cache_roots[].boundary`.
- The first internal `ViewBoundaryState` store now exists in `crates/fret-ui`, with `BoundaryId`
  keyed to the retained node identity for this migration slice.
- Deletion plan for that transitional path is recorded in
  `M1_BOUNDARY_DIAGNOSTICS_SLICE_2026-05-13.md`.
- Perf gate and worst-bundle attribution were rerun for the diagnostic slice. The result confirms
  this slice is attribution-only: `paint.widget` remains dominant and is the M2/M3 target.

Status on 2026-05-14:

- M4B retires the transitional `debug.cache_roots[].boundary` field. Boundary-level diagnostics are
  now emitted through first-class `debug.boundaries[]`, and `fret-diag` derives any cache-root
  report summary from that canonical top-level list.

## M2: Prepaint Ownership

Exit criteria:

- Code-editor frame-derived state moves out of broad paint work and into shared prepaint ownership
  or an explicitly compatible boundary prepaint layer.
- Paint consumes prepaint state for the migrated path.
- Tests prove stale prepaint state cannot be replayed across dependency changes.

Status on 2026-05-13:

- The first M2 vertical slice landed through `Canvas` prepaint + `windowed_rows_surface`
  prepaint ownership.
- The latest canvas-output slice now carries the row-scene replay plan through node-scoped
  `PrepaintOutputs`, so the prepaint phase owns a concrete output carrier instead of only the
  scheduling hook.
- The follow-up boundary-prepaint slice in
  `M2B_VIEW_BOUNDARY_PREPAINT_STATE_SLICE_2026-05-13.md` moves `PrepaintOutputs` out of `Node` and
  into `ViewBoundaryState::prepaint`, so canvas prepaint output is now boundary-owned rather than
  node-owned.
- The same slice adds `debug.boundaries[]` as the first top-level boundary diagnostics list. It is
  directly enumerated from `ViewBoundaryState`, joins matching cache-root outcome fields, and
  reports `prepaint_owner=view_boundary_prepaint_state`.
- `ecosystem/fret-code-editor` now schedules frame-derived prefetch/bookkeeping in prepaint.
- A focused helper test locks prepaint-before-paint ordering and output visibility for the windowed
  rows surface.
- The final stale-state replay guard and scene-fragment owner are still pending, so M2 remains a
  partial migration rather than a closeout.

## M3: Scene Fragment Replay

Exit criteria:

- The migrated boundary can replay a scene fragment with the required text/resource side indexes.
- Reuse/reject diagnostics explain fragment decisions.
- Perf evidence shows the selected paint/widget bottleneck improves by at least 20-30%.

Status on 2026-05-13:

- A transitional replay-plan slice landed in
  `M3_ROW_SCENE_PREPAINT_REPLAY_PLAN_SLICE_2026-05-13.md`.
- A follow-up carrier slice moved the replay-plan payload out of `CodeEditorState` and into
  node-scoped canvas prepaint output in
  `M3B_ROW_SCENE_PREPAINT_OUTPUT_CARRIER_SLICE_2026-05-13.md`.
- Prepaint now validates cached row scene replay candidates and paint consumes matching plan
  entries from prepaint output.
- Diagnostics expose planned vs used replay entries plus prepaint planning cost.
- The latest evidence shows the expected phase move: paint-side `us_row_text` is `0/5us`
  p50/p95 in the latest canvas-output bundle, while prepaint planning remains visible as
  `55/77us` p50/p95.
- The follow-up row-rect slice in
  `M3A_WINDOWED_ROWS_CANONICAL_ROW_RECT_SLICE_2026-05-13.md` removes the code-editor-local
  fixed-row rect reconstruction from replay planning; `WindowedRowsPaintFrame::row_rect(...)` now
  owns that surface geometry.
- The boundary fragment-carrier slice in
  `M3C_BOUNDARY_SCENE_FRAGMENT_CARRIER_SLICE_2026-05-14.md` moves the replay-plan carrier out of
  generic prepaint output and into `ViewBoundaryState::scene_fragment`, using a fragment-shaped
  `CanvasSceneFragment<RowSceneFragmentPayload>` for ops, hosted-resource side indexes, local
  bounds, and origin.
Status on 2026-05-14:

- Boundary fragment diagnostics now report owner, slot count, total entries, used entries,
  rejected entries, and reject reason through `debug.boundaries[]`.
- The latest closeout perf run shows `paint.widget` p95 at `650us` versus the M1 selected
  bottleneck evidence at `1494us`, exceeding the required 20-30% improvement for the paint-side
  bottleneck. Total p95 improved from `1811us` to `1396us`, also exceeding
  the 20% threshold.
- M3 is complete for the code-editor vertical slice. Broader renderer-side fragment/replay
  unification remains future work outside this vertical closeout.

## M4: Runtime Consolidation

Exit criteria:

- Layout containment is represented as boundary dependency metadata, not only as an ad hoc flag.
- View-cache and paint-cache paths are consolidated where the boundary model covers both.
- Old private paths replaced by the v2 path are deleted or marked migration-only with a date.

Status on 2026-05-13:

- Minimal boundary layout dependency metadata exists in `ViewBoundaryState`; current
  `should_reuse_view_cache_node(...)` uses it for contained-relayout eligibility.
- `debug.boundaries[]` exists as a first-class bundle field and is directly enumerated from
  `ViewBoundaryState`, with matching cache-root outcome fields joined in when present.

Status on 2026-05-14:

- M4A moved contained-relayout dirty reasons into `ViewBoundaryState::dirty` and replaced the
  old `dirty_cache_roots` / `dirty_cache_root_reasons` owner with a `dirty_boundaries` fast index.
- `debug.boundaries[]` now reports boundary layout dirty state through `layout_dirty`,
  `layout_dirty_source`, and `layout_dirty_detail`.
- M4B removed the nested `debug.cache_roots[].boundary` schema and changed `fret-diag stats` to join
  cache-root report summaries from canonical `debug.boundaries[]`.
- The final closeout audit is recorded below; broader view-cache/paint-cache consolidation remains
  follow-on ADR 0327 work.

Status after M4C on 2026-05-14:

- `M4C_BOUNDARY_HINT_API_SLICE_2026-05-14.md` replaces first-party direct `contained_layout`
  authoring with `ViewBoundaryHints` and `contain_layout_when_bounds_known(...)`.
- Public/ecosystem boundary-hint naming is no longer open for this lane. Remaining
  `contained_layout` names are internal low-level flags, diagnostic fields, or historical notes
  until broader view-cache/build-boundary consolidation can remove or rename them.

Status after M4D on 2026-05-14:

- `M4D_VIEW_CACHE_BUILD_BOUNDARY_STORE_SLICE_2026-05-14.md` consolidates element-runtime
  view-cache build-time rendered/next maps, key mismatch roots, reuse roots, last-reused frame
  tracking, transitioned reuse roots, and the active scope stack behind
  `ViewCacheBuildBoundaryStore`.
- Existing `ElementContext` and declarative mount method calls are preserved, so this is a
  runtime ownership slice rather than an authoring/API slice.
- M4D removes the flat side maps from `WindowElementState`, but it does not yet make
  `ViewBoundaryState` the final build-owner and does not touch node-owned paint-cache replay.

Status after M4E on 2026-05-14:

- `M4E_BOUNDARY_PAINT_CACHE_ENTRY_SLICE_2026-05-14.md` moves boundary-node `PaintCacheEntry`
  ownership into `ViewBoundaryState::paint_cache`.
- Ordinary paint-cache replay still uses `PaintCacheState` for previous-frame op storage, and
  non-boundary nodes still use the node-owned fallback entry path.
- Boundary diagnostics now include `paint_cache_owner`, so bundles can distinguish boundary-owned
  paint-cache entries from missing or retained fallback ownership.

Status after M4F on 2026-05-14:

- `M4F_NODE_PAINT_CACHE_FALLBACK_DELETION_SLICE_2026-05-14.md` deletes the remaining
  `Node::paint_cache` field and routes plain paint-cache node entries through
  `UiTree::boundary_paint_cache_entries`.
- True runtime boundaries still store entries in `ViewBoundaryState::paint_cache`; if a plain cached
  node becomes a runtime boundary, the side-store entry migrates into the boundary state.
- The plain-node replay, hit-test-only replay, side-store-to-boundary migration, scroll
  invalidation, model invalidation, and view-cache gating tests now prove the node fallback is gone
  without polluting the full `view_boundaries` table.
- Ordinary `PaintCacheEntry` ownership no longer has a node fallback. `PaintCacheState` still owns
  previous-frame op storage and remains the next paint-cache replay ownership decision.

Status after closeout audit on 2026-05-14:

- `CLOSEOUT_AUDIT_2026-05-14.md` closes the code-editor vertical slice.
- Broader view-cache/paint-cache consolidation remains follow-on ADR 0327 work, not a blocker for
  the current vertical-slice closeout.

## M5: Closeout and Deletion Audit

Exit criteria:

- Deletion audit is complete.
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md` reflects the final ADR 0327 state.
- Perf gates and correctness gates are documented with final evidence paths.
- Workstream status is moved to maintenance or closed.

Status on 2026-05-14:

- Code-editor vertical slice closeout audit is complete in `CLOSEOUT_AUDIT_2026-05-14.md`.
- Deletion audit is complete for paths replaced by this vertical slice.
- Perf/correctness/layering/check evidence is documented in `EVIDENCE_AND_GATES.md` and the
  closeout audit.
- The broader ADR 0327 lane remains active for architecture follow-ons. ADR acceptance/review is
  closed by `M0_CONTRACT_FREEZE_2026-05-14.md`.

## M6: Global Frame Pipeline v2 Completion

Exit criteria:

- The completion contract in `PROGRESS.md` is satisfied.
- ADR 0327 is no longer proposed-only; it is accepted, revised into an accepted ADR, or superseded by
  an accepted equivalent.
- The final boundary model owns the runtime state needed for build/layout/prepaint/paint reuse and
  diagnostics across the selected proof surfaces.
- At least two proof surfaces pass: code-editor resize/paint and one broader non-code-editor
  view-cache or paint-cache surface.
- Replaced old private paths are deleted, and retained paths are justified by an accepted
  ADR/workstream decision.
- A final closeout audit names the final runtime path, retained exceptions, deleted paths, gates,
  perf evidence, and remaining work that is outside Frame Pipeline v2.

Status on 2026-05-14:

- Open. The code-editor vertical slice is complete, but the global contract is not complete while
  the consolidated view-cache build-boundary store still needs a final `ViewBoundaryState`
  ownership/retention decision, previous-op paint-cache storage still needs migration or retention
  decision, internal contained-layout flag cleanup, old env-knob ownership, and the second
  non-code-editor proof surface remain open.
