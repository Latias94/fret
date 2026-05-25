# Fret Node Paint Root Cached Edge Anchor Target Adapter v1 - TODO

Status: Closed
Last updated: 2026-05-25

## CEAA-M0 - Scope Freeze

- [x] CEAA-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-cached-edge-anchor-target-adapter-v1]
  Goal: Open a narrow follow-on for cached edge anchor target route ownership.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-cached-edge-anchor-target-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Keep fallback, overlay, replay, cache keys, and build-state helpers out of scope.

## CEAA-M1 - Cached Edge Anchor Target Adapter Seam

- [x] CEAA-020 [owner=codex] [deps=CEAA-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges]
  Goal: Move cached edge anchor target routing behind an adapter plus retained `PaintCx` binding.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_anchor_target_adapter`
  Evidence: `anchor_target_adapter.rs`, `anchor_target_retained_cx.rs`, `anchor_target.rs`,
  `ecosystem/fret-node/src/lib.rs`
  Handoff: Cached edge anchor route calls the adapter, not shared `resolve_edge_anchor_target_*`
  helpers directly.

## CEAA-M2 - Closeout

- [x] CEAA-030 [owner=codex] [deps=CEAA-020] [scope=docs/workstreams/fret-node-paint-root-cached-edge-anchor-target-adapter-v1]
  Goal: Close the lane and keep fallback, overlay, replay, cache-key, and deeper edge-anchor cleanup
  separate.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `CLOSEOUT_AUDIT_2026-05-25.md`
  Handoff: Start a separate follow-on for fallback retained route inputs, cache-key cleanup, or
  deeper shared edge-anchor internals.
