# Fret Node Paint Root Frame Setup Adapter v1 - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

This lane is a narrow follow-on from `fret-node-paint-root-cache-plan-adapter-v1`. The parent lane
proved cache-plan host/bounds/scale-factor route inputs behind an adapter seam.

The frame setup operation-family audit is complete, and the first implementation seam is complete:
bounds/viewport/render-cull route inputs now go through a frame viewport adapter.

## Active Task

- Task ID: FSA-030
- Owner: codex
- Files: `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame.rs`,
  new frame viewport adapter modules, `ecosystem/fret-node/src/lib.rs`
- Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_frame_viewport_adapter`
- Status: DONE
- Review: final gates passed; ready for closeout or a narrow follow-on split after commit
- Evidence: `docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/EVIDENCE_AND_GATES.md`

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

## Blockers

- None known.

## Next Recommended Action

- Run final lane gates and commit FSA-030.
- After that, close this lane or split a new follow-on for one operation family only:
  cache stats diagnostics, clip emission, background paint, or grid paint.
