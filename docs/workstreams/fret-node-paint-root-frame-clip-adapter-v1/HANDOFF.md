# Fret Node Paint Root Frame Clip Adapter v1 - Handoff

Status: Closed
Last updated: 2026-05-25

## Current State

This lane is closed. It is a narrow follow-on from
`fret-node-paint-root-frame-setup-adapter-v1`. The parent lane closed after proving
bounds/viewport/render-cull route inputs behind a frame viewport adapter and split root frame clip
scene emission as the smallest next candidate.

## Final State

- Task ID: FCA-030
- Owner: codex
- Files: `CLOSEOUT_AUDIT_2026-05-25.md`, workstream status docs
- Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
- Status: DONE
- Evidence: `docs/workstreams/fret-node-paint-root-frame-clip-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`

## Decisions Since Open

- This lane should not reopen the closed frame setup adapter lane.
- The first slice owns only the root frame `SceneOp::PushClipRect` emission.
- FCA-020 introduced `frame_clip_adapter.rs` plus a retained `PaintCx` binding in
  `frame_clip_retained_cx.rs`.
- Tail `SceneOp::PopClip` remains outside this slice.
- Background paint, grid paint, path-cache diagnostics, and cached/immediate pass clip emission are
  explicit non-goals.
- FCA-030 closed this lane and routes future work to separate operation-family follow-ons.

## Blockers

- None known.

## Next Recommended Action

- Do not reopen this lane for more implementation.
- Start a narrow follow-on for path-cache diagnostics or background paint next, unless fresh
  evidence shows grid paint or tail cleanup is the smaller honest seam.
