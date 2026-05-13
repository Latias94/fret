# Milestones

Status: Active
Last updated: 2026-05-13

## M0: Contract Lock

Exit criteria:

- ADR 0327 is reviewed and accepted, or replaced by a better accepted ADR.
- Target interface state is updated to match the accepted ADR.
- The old-path inventory exists.
- The first code-editor repro and gate commands are current.

## M1: Code Editor Boundary Pilot

Exit criteria:

- Code-editor UI Gallery content root has boundary-level diagnostics.
- The first runtime boundary state exists in code or a narrow transitional equivalent exists with a
  deletion plan.
- `ui-code-editor-resize-probes` still passes.
- `code_editor.paint_perf` remains non-zero and is correlated with boundary diagnostics.

## M2: Prepaint Ownership

Exit criteria:

- Code-editor frame-derived state moves out of broad paint work and into shared prepaint ownership
  or an explicitly compatible boundary prepaint layer.
- Paint consumes prepaint state for the migrated path.
- Tests prove stale prepaint state cannot be replayed across dependency changes.

## M3: Scene Fragment Replay

Exit criteria:

- The migrated boundary can replay a scene fragment with the required text/resource side indexes.
- Reuse/reject diagnostics explain fragment decisions.
- Perf evidence shows the selected paint/widget bottleneck improves by at least 20-30%.

## M4: Runtime Consolidation

Exit criteria:

- Layout containment is represented as boundary dependency metadata, not only as an ad hoc flag.
- View-cache and paint-cache paths are consolidated where the boundary model covers both.
- Old private paths replaced by the v2 path are deleted or marked migration-only with a date.

## M5: Closeout and Deletion Audit

Exit criteria:

- Deletion audit is complete.
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md` reflects the final ADR 0327 state.
- Perf gates and correctness gates are documented with final evidence paths.
- Workstream status is moved to maintenance or closed.
