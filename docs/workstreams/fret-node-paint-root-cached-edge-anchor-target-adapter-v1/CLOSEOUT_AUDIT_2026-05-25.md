# Closeout Audit - 2026-05-25

Status: Closed

## Shipped State

- Added `anchor_target_adapter.rs` as the cached edge anchor target route contract.
- Added `anchor_target_retained_cx.rs` as the retained `PaintCx` binding that owns direct shared
  edge-anchor helper calls.
- Updated `anchor_target.rs` so cached edge anchor target routing calls
  `anchor_target_adapter::resolve_paint_root_cached_edge_anchor_target`.
- Added focused source-policy coverage in `ecosystem/fret-node/src/lib.rs`.

## Scope Kept Out

- Fallback uncached edge rendering.
- Selected/hovered overlay routing.
- Cached edge replay scene sinks.
- Cache key semantics.
- Build-state temporary scene and clip-op helpers.
- Deeper shared `paint_root/edge_anchor/*` internals.

## Evidence Anchors

- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/anchor_target_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/anchor_target_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/anchor_target.rs`
- `ecosystem/fret-node/src/lib.rs`

## Fresh Validation

- `cargo fmt --package fret-node` - passed.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_anchor_target_adapter` - passed.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `python3 tools/check_layering.py` - passed.
- `git diff --check` - passed.

## Follow-Ons

Start a new narrow lane for any of:

- fallback retained route inputs,
- cache-key cleanup,
- deeper shared edge-anchor internals.
