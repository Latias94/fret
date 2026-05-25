# Fret Node Paint Root Frame Clip Adapter v1 - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

This lane is a narrow follow-on from `fret-node-paint-root-frame-setup-adapter-v1`. The parent lane
closed after proving bounds/viewport/render-cull route inputs behind a frame viewport adapter and
split root frame clip scene emission as the smallest next candidate.

## Active Task

- Task ID: FCA-020
- Owner: codex
- Files: `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame.rs`, new frame clip adapter
  modules, `ecosystem/fret-node/src/lib.rs`
- Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_frame_clip_adapter`
- Status: DONE
- Review: final gates passed; ready for commit
- Evidence: `docs/workstreams/fret-node-paint-root-frame-clip-adapter-v1/EVIDENCE_AND_GATES.md`

## Decisions Since Open

- This lane should not reopen the closed frame setup adapter lane.
- The first slice owns only the root frame `SceneOp::PushClipRect` emission.
- FCA-020 introduced `frame_clip_adapter.rs` plus a retained `PaintCx` binding in
  `frame_clip_retained_cx.rs`.
- Tail `SceneOp::PopClip` remains outside this slice.
- Background paint, grid paint, path-cache diagnostics, and cached/immediate pass clip emission are
  explicit non-goals.

## Blockers

- None known.

## Next Recommended Action

- Commit FCA-020.
- After commit, close this lane or split a narrow follow-on for path-cache diagnostics, background
  paint, grid paint, or tail cleanup.
