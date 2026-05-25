# Fret Node Paint Prepaint Adapter v1 - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

This is a follow-on to `fret-node-low-level-adapter-v1`. The low-level adapter lane proved common
host operations and command dispatch seams. Paint/prepaint is now split into this dedicated lane
because the retained paint tree is broad and should migrate one operation family at a time.

NPA-020 completed the first slice by moving prepaint cull-window route preparation behind
`prepaint_cull_window_adapter.rs`. `retained_widget_cull_window.rs` now only binds `PrepaintCx` to
the adapter and forwards; `retained_widget_cull_window_shift.rs` records debug output through the
adapter seam.

NPA-030 audited paint root scene emission and decided not to introduce a broad paint-root adapter in
this lane. The paint root has multiple retained-context operation families. The recommended
follow-on is a narrower cache-plan adapter lane: `fret-node-paint-root-cache-plan-adapter-v1`.

## Active Task

- Task ID: NPA-040
- Owner: planner
- Files: `docs/workstreams/fret-node-paint-prepaint-adapter-v1/EVIDENCE_AND_GATES.md`,
  `docs/workstreams/fret-node-paint-prepaint-adapter-v1/HANDOFF.md`,
  optional closeout audit
- Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
- Status: NEEDS_CONTEXT
- Review: not started
- Evidence: `docs/workstreams/fret-node-paint-prepaint-adapter-v1/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Paint/prepaint is split from event routing and command dispatch.
- The first proof should target prepaint cull-window operations, not the full paint tree.
- NPA-020 proved the prepaint cull-window seam without migrating paint root scene emission.
- NPA-030 found that paint root scene emission is not a single operation family and should split
  into a narrower cache-plan adapter follow-on before frame or scene-emission adapters.

## Blockers

- None known.

## Next Recommended Action

- Execute NPA-040: close this lane or scaffold the recommended
  `fret-node-paint-root-cache-plan-adapter-v1` follow-on.
