# Fret Node Paint Root Pass Clip Adapter v1 - TODO

Status: Closed
Last updated: 2026-05-25

## PCA-M0 - Scope Freeze

- [x] PCA-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-pass-clip-adapter-v1]
  Goal: Open a narrow follow-on for pass-router scene sink access and record the cached-pass scope
  narrowing.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-pass-clip-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Keep deeper cached group/node/edge internals, edge paint routing, overlay layers, and
  public scene schema changes out of this lane.

## PCA-M1 - Pass Scene Adapter Seam

- [x] PCA-020 [owner=codex] [deps=PCA-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root]
  Goal: Move immediate pass static group/node scene sink access behind a pass scene adapter plus
  retained `PaintCx` binding.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_pass_scene_adapter`
  Evidence: `immediate_pass.rs`, `pass_scene_adapter.rs`, `pass_scene_retained_cx.rs`,
  `ecosystem/fret-node/src/lib.rs`
  Handoff: Adapter exposes named pass operations and the retained binding owns scene/services/scale
  reads. Complete.

## PCA-M2 - Closeout

- [x] PCA-030 [owner=codex] [deps=PCA-020] [scope=docs/workstreams/fret-node-paint-root-pass-clip-adapter-v1]
  Goal: Close the lane and split cached internals into follow-on candidates.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `CLOSEOUT_AUDIT_2026-05-25.md`
  Handoff: Start a separate follow-on for cached static layer scene replay or cached edge replay.
  Complete; see `CLOSEOUT_AUDIT_2026-05-25.md`.
