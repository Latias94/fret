# Fret Node Paint Root Cached Edge Fallback Adapter v1 - TODO

Status: Closed
Last updated: 2026-05-25

## CEFA-M0 - Scope Freeze

- [x] CEFA-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-cached-edge-fallback-adapter-v1]
  Goal: Open a narrow follow-on for cached edge fallback route ownership.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-cached-edge-fallback-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Keep cache keys, replay, overlay, anchor target, build-state helpers, and `paint_edges`
  internals out of scope.

## CEFA-M1 - Cached Edge Fallback Adapter Seam

- [x] CEFA-020 [owner=codex] [deps=CEFA-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges]
  Goal: Move cached edge fallback host access and retained edge paint dispatch behind an adapter plus
  retained `PaintCx` binding.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_fallback_adapter`
  Evidence: `fallback_adapter.rs`, `fallback_retained_cx.rs`, `fallback.rs`, `edges/fallback.rs`,
  `ecosystem/fret-node/src/lib.rs`
  Handoff: Fallback helpers call the adapter, not `cx.app` or `canvas.paint_edges` directly.

## CEFA-M2 - Closeout

- [x] CEFA-030 [owner=codex] [deps=CEFA-020] [scope=docs/workstreams/fret-node-paint-root-cached-edge-fallback-adapter-v1]
  Goal: Close the lane and keep cache-key, replay, overlay, anchor target, build-state, and deeper
  edge-paint cleanup separate.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `CLOSEOUT_AUDIT_2026-05-25.md`
  Handoff: Start a separate follow-on for cache-key cleanup or deeper `paint_edges` retained inputs.
