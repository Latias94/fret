# Closeout Audit - 2026-05-25

Status: Closed

## Shipped State

- Added `overlay_adapter.rs` as the cached edge selected/hovered overlay route contract.
- Added `overlay_retained_cx.rs` as the retained `PaintCx` binding that owns the direct
  `paint_edge_overlays_selected_hovered` call.
- Updated `single_rect.rs` and `tile_path.rs` so cached edge routes call
  `overlay_adapter::paint_root_cached_edge_overlays_selected_hovered`.
- Added focused source-policy coverage in `ecosystem/fret-node/src/lib.rs`.

## Scope Kept Out

- Anchor target resolution.
- Fallback uncached edge rendering.
- Cached edge replay scene sinks.
- Cache key semantics.
- Route-input host/services/scale adapters.
- Build-state temporary scene and clip-op helpers.

## Evidence Anchors

- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/overlay_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/overlay_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/single_rect.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/tile_path.rs`
- `ecosystem/fret-node/src/lib.rs`

## Fresh Validation

- `cargo fmt --package fret-node` - passed.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_overlay_adapter` - passed.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `python3 tools/check_layering.py` - passed.
- `git diff --check` - passed.

## Follow-Ons

Start a new narrow lane for any of:

- edge anchor target routing,
- fallback retained route inputs,
- cache-key cleanup,
- broader overlay internals beyond cached path routing.
