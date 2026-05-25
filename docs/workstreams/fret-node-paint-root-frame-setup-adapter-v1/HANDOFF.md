# Fret Node Paint Root Frame Setup Adapter v1 - Handoff

Status: Closed
Last updated: 2026-05-25

## Current State

This lane is a narrow follow-on from `fret-node-paint-root-cache-plan-adapter-v1`. The parent lane
proved cache-plan host/bounds/scale-factor route inputs behind an adapter seam.

This lane is closed. The frame setup operation-family audit is complete, and the first
implementation seam is complete: bounds/viewport/render-cull route inputs now go through a frame
viewport adapter.

## Final State

- Task ID: FSA-040
- Owner: codex
- Files: `CLOSEOUT_AUDIT_2026-05-25.md`, workstream status docs
- Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
- Status: DONE
- Evidence: `docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`

## Decisions Since Open

- Frame setup should be audited before adding an adapter.
- Bounds/viewport route inputs are the likely smallest implementation candidate.
- Clip/background/grid scene emission should not be folded into the first frame seam by default.
- FSA-010 froze the audit-first scope.
- FSA-020 selected bounds/viewport/render-cull route inputs as the first frame setup seam.
- FSA-030 introduced `frame_viewport_adapter.rs` plus a retained `PaintCx` binding in
  `frame_viewport_retained_cx.rs`.
- Cache stats diagnostics, clip emission, background paint, and grid paint deliberately remain in
  `frame.rs`.
- FSA-040 closed this lane and routes future work to separate operation-family follow-ons.

## Blockers

- None known.

## Next Recommended Action

- Do not reopen this lane for more implementation.
- Start a narrow follow-on for clip scene emission first, unless fresh evidence shows diagnostics,
  background paint, or grid paint is the smaller honest seam.
