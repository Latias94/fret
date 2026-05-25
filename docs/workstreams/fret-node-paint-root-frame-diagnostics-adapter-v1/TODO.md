# Fret Node Paint Root Frame Diagnostics Adapter v1 - TODO

Status: Active
Last updated: 2026-05-25

## FDA-M0 - Scope Freeze

- [x] FDA-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-frame-diagnostics-adapter-v1]
  Goal: Open the narrow follow-on for path-cache diagnostics recording only.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-diagnostics-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Do not reopen the closed frame clip lane.

## FDA-M1 - Path-Cache Diagnostics Seam

- [x] FDA-020 [owner=codex] [deps=FDA-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root]
  Goal: Move retained path-cache diagnostics recording behind a minimal diagnostics adapter seam.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_frame_diagnostics_adapter`
  Evidence: frame diagnostics adapter modules, `paint_root/frame/cache.rs`, source-policy test in
  `ecosystem/fret-node/src/lib.rs`
  Handoff: Keep cache begin, viewport, clip, background paint, grid paint, tail cleanup, and
  cached/immediate passes out of scope. Complete; path-cache diagnostics snapshot collection stays
  in `frame/cache.rs`, while retained window/node/frame-id/registry recording lives in
  `frame_diagnostics_retained_cx.rs`.
