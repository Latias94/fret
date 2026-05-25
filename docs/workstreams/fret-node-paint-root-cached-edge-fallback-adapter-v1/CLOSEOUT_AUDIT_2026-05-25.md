# Closeout Audit - 2026-05-25

Status: Closed

## Shipped State

- Added `fallback_adapter.rs` as the cached edge fallback host/paint dispatch contract.
- Added `fallback_retained_cx.rs` as the retained `PaintCx` binding that owns retained host access
  and direct edge paint dispatch.
- Updated `cached_edges/fallback.rs` and `cached_edges/edges/fallback.rs` so fallback helpers call
  the adapter instead of reading `cx.app` or calling `canvas.paint_edges` directly.
- Added focused source-policy coverage in `ecosystem/fret-node/src/lib.rs`.

## Scope Kept Out

- Cache key semantics.
- Cached edge replay scene sinks.
- Selected/hovered overlay routing.
- Anchor target routing.
- Build-state temporary scene and clip-op helpers.
- Deeper `paint_edges` internals.

## Evidence Anchors

- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/fallback_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/fallback_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/fallback.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/edges/fallback.rs`
- `ecosystem/fret-node/src/lib.rs`

## Fresh Validation

- `cargo fmt --package fret-node` - passed.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_fallback_adapter` - passed.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `python3 tools/check_layering.py` - passed.
- `git diff --check` - passed.

## Follow-Ons

Start a new narrow lane for any of:

- cache-key cleanup,
- deeper `paint_edges` retained inputs.
