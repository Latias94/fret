# Milestones

Status: Active
Last updated: 2026-05-14

## M0: Contract Lock

Exit criteria:

- ADR 0327 is reviewed and accepted, or replaced by a better accepted ADR.
- Target interface state is updated to match the accepted ADR.
- The old-path inventory exists.
- The first code-editor repro and gate commands are current.

Status on 2026-05-13:

- Baseline/source inventory exists in `M0_BASELINE_AUDIT_2026-05-13.md`.
- First repro and gate commands are current in `EVIDENCE_AND_GATES.md`.
- ADR 0327 still needs review/acceptance or a superseding accepted ADR before broad migration.

## M1: Code Editor Boundary Pilot

Exit criteria:

- Code-editor UI Gallery content root has boundary-level diagnostics.
- The first runtime boundary state exists in code or a narrow transitional equivalent exists with a
  deletion plan.
- `ui-code-editor-resize-probes` still passes.
- `code_editor.paint_perf` remains non-zero and is correlated with boundary diagnostics.

Status on 2026-05-13:

- Transitional boundary diagnostics are implemented through `debug.cache_roots[].boundary`.
- The first internal `ViewBoundaryState` store now exists in `crates/fret-ui`, with `BoundaryId`
  keyed to the retained node identity for this migration slice.
- Deletion plan for this transitional path is recorded in
  `M1_BOUNDARY_DIAGNOSTICS_SLICE_2026-05-13.md`.
- Perf gate and worst-bundle attribution were rerun for the diagnostic slice. The result confirms
  this slice is attribution-only: `paint.widget` remains dominant and is the M2/M3 target.

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
- M3 is not complete: fragment hit/reject diagnostics and the 20-30% end-to-end bottleneck
  improvement proof are still pending.

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
- Broader dirty-set migration, view-cache/paint-cache consolidation, and old-path deletion are still
  pending.

## M5: Closeout and Deletion Audit

Exit criteria:

- Deletion audit is complete.
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md` reflects the final ADR 0327 state.
- Perf gates and correctness gates are documented with final evidence paths.
- Workstream status is moved to maintenance or closed.
