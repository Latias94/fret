# Fret Node Paint Root Frame Setup Adapter v1 - TODO

Status: Active
Last updated: 2026-05-25

## FSA-M0 - Scope And Evidence Freeze

- [x] FSA-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1]
  Goal: Freeze frame setup audit scope, non-goals, and gates.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Complete. Do not implement a broad frame adapter.

## FSA-M1 - Frame Setup Operation-Family Audit

- [x] FSA-020 [owner=codex] [deps=FSA-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root]
  Goal: Audit `paint_root/frame.rs`, `paint_root/frame/cache.rs`, and `paint_root/frame/background.rs`
  to select the first narrow frame setup adapter seam or split again.
  Validation: source audit plus `cargo check -p fret-node --features compat-retained-canvas`
  Evidence: frame setup audit note, `paint_root/frame.rs`, `paint_root/frame/cache.rs`,
  `paint_root/frame/background.rs`
  Handoff: Complete. Next seam should target bounds/viewport route inputs only.

## FSA-M2 - First Frame Seam Or Closeout

- [x] FSA-030 [owner=codex] [deps=FSA-020] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root]
  Goal: Implement the first narrow frame seam for bounds/viewport route inputs.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_frame_viewport_adapter`
  Evidence: frame viewport adapter modules, `paint_root/frame.rs`, source-policy test in
  `ecosystem/fret-node/src/lib.rs`
  Handoff: Keep cache stats diagnostics, clip emission, background paint, and grid paint out of
  scope. Complete; frame setup now delegates bounds/viewport/render-cull preparation through the
  frame viewport adapter while retaining diagnostics and scene emission in `frame.rs`.
