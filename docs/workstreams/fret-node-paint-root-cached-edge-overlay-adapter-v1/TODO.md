# Fret Node Paint Root Cached Edge Overlay Adapter v1 - TODO

Status: Closed
Last updated: 2026-05-25

## CEAO-M0 - Scope Freeze

- [x] CEAO-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-cached-edge-overlay-adapter-v1]
  Goal: Open a narrow follow-on for cached edge selected/hovered overlay route ownership.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-cached-edge-overlay-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Keep anchor target, fallback uncached rendering, replay, cache keys, and build-state
  helpers out of scope.

## CEAO-M1 - Cached Edge Overlay Adapter Seam

- [x] CEAO-020 [owner=codex] [deps=CEAO-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges]
  Goal: Move cached edge selected/hovered overlay routing behind an adapter plus retained `PaintCx`
  binding.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_overlay_adapter`
  Evidence: `overlay_adapter.rs`, `overlay_retained_cx.rs`, `single_rect.rs`, `tile_path.rs`,
  `ecosystem/fret-node/src/lib.rs`
  Handoff: Cached edge routes call the adapter, not `paint_edge_overlays_selected_hovered` directly.

## CEAO-M2 - Closeout

- [x] CEAO-030 [owner=codex] [deps=CEAO-020] [scope=docs/workstreams/fret-node-paint-root-cached-edge-overlay-adapter-v1]
  Goal: Close the lane and keep anchor target, fallback, replay, and cache-key cleanup separate.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `CLOSEOUT_AUDIT_2026-05-25.md`
  Handoff: Start a separate follow-on for anchor target routing, fallback retained route inputs, or
  cache-key cleanup.
