# Fret Node Paint Root Frame Setup Adapter v1 - TODO

Status: Active
Last updated: 2026-05-25

## FSA-M0 - Scope And Evidence Freeze

- [ ] FSA-010 [owner=unassigned] [deps=none] [scope=docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1]
  Goal: Freeze frame setup audit scope, non-goals, and gates.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Do not implement a broad frame adapter in this task.

## FSA-M1 - Frame Setup Operation-Family Audit

- [ ] FSA-020 [owner=unassigned] [deps=FSA-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root]
  Goal: Audit `paint_root/frame.rs`, `paint_root/frame/cache.rs`, and `paint_root/frame/background.rs`
  to select the first narrow frame setup adapter seam or split again.
  Validation: source audit plus `cargo check -p fret-node --features compat-retained-canvas`
  Evidence: frame setup audit note, `paint_root/frame.rs`, `paint_root/frame/cache.rs`,
  `paint_root/frame/background.rs`
  Handoff: Keep static layer replay/store, cached/immediate passes, and tail cleanup out of scope.

## FSA-M2 - First Frame Seam Or Closeout

- [ ] FSA-030 [owner=planner] [deps=FSA-020] [scope=docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1]
  Goal: Implement the first narrow frame seam if selected, or close/split the lane.
  Validation: source-policy test if implemented; otherwise catalog/diff gates.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, optional closeout audit.
  Handoff: Candidate seams are bounds/viewport inputs, cache stats diagnostics, clip emission,
  background paint, or grid paint.
