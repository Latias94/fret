# Fret Node Paint Root Frame Diagnostics Adapter v1 - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

This lane is a narrow follow-on from `fret-node-paint-root-frame-clip-adapter-v1`. The parent lane
closed after proving root frame clip scene emission behind a frame clip adapter and split
path-cache diagnostics as a next candidate.

## Active Task

- Task ID: FDA-020
- Owner: codex
- Files: `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame/cache.rs`, new frame
  diagnostics adapter modules, `ecosystem/fret-node/src/lib.rs`
- Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_frame_diagnostics_adapter`
- Status: DONE
- Review: final gates passed; ready for commit
- Evidence: `docs/workstreams/fret-node-paint-root-frame-diagnostics-adapter-v1/EVIDENCE_AND_GATES.md`

## Decisions Since Open

- This lane should not reopen the closed frame clip lane.
- The first slice owns only path-cache diagnostics recording for `fret-node.canvas.paths`.
- FDA-020 introduced `frame_diagnostics_adapter.rs` plus a retained `PaintCx` binding in
  `frame_diagnostics_retained_cx.rs`.
- Cache begin remains direct canvas cache bookkeeping.
- Grid tile diagnostics and edge label budget diagnostics remain outside this lane.
- Background paint, grid paint, viewport, clip, tail cleanup, and cached/immediate passes are
  explicit non-goals.

## Blockers

- None known.

## Next Recommended Action

- Commit FDA-020.
- After commit, close this lane or split a narrow follow-on for background paint or grid paint.
