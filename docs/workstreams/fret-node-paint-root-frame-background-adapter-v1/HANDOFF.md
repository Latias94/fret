# Fret Node Paint Root Frame Background Adapter v1 - Handoff

Status: Closed
Last updated: 2026-05-25

## Current State

This lane is closed. It is a narrow follow-on from
`fret-node-paint-root-frame-diagnostics-adapter-v1`. The parent lane closed after proving
path-cache diagnostics recording behind a frame diagnostics adapter and split background paint as
the next small operation-family candidate.

## Final State

- Task ID: FBA-030
- Owner: codex
- Files: `CLOSEOUT_AUDIT_2026-05-25.md`, workstream status docs
- Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
- Status: DONE
- Evidence: `docs/workstreams/fret-node-paint-root-frame-background-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`

## Decisions Since Open

- Keep chrome hint resolution in `frame/background.rs` for this lane.
- The adapter should accept viewport rect plus resolved background color, not a prebuilt `SceneOp`.
- Retained scene mutation belongs in `frame_background_retained_cx.rs`.
- Grid paint, tail cleanup, cached/immediate passes, and diagnostics remain outside this lane.
- FBA-030 closed this lane and routes future work to separate operation-family follow-ons.

## Blockers

None.

## Next Recommended Action

- Do not reopen this lane for more implementation.
- Start a narrow follow-on for grid paint next, unless fresh evidence shows tail cleanup or
  cached/immediate pass clip emission is the smaller honest seam.
