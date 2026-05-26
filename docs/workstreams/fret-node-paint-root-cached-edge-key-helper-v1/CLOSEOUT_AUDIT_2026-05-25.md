# Closeout Audit - 2026-05-25

Status: Closed

## Shipped State

- Added `cached_edge_key_builder` as the shared cached edge key-field helper.
- Added `cached_edge_base_key` and `cached_edge_single_rect_key` as small finish helpers.
- Preserved the four existing key functions and all scope strings.
- Kept single-rect rect-origin fields outside the shared base helper.
- Added focused source-policy coverage in `ecosystem/fret-node/src/lib.rs`.

## Scope Kept Out

- Cache invalidation semantics.
- Cache lifetime and eviction.
- Cache scope string renames.
- Route input adapters.
- Replay, fallback, overlay, anchor target, or build-state behavior.

## Evidence Anchors

- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/keys.rs`
- `ecosystem/fret-node/src/lib.rs`

## Fresh Validation

- `cargo fmt --package fret-node` - passed.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_key_helper` - passed.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `python3 tools/check_layering.py` - passed.
- `git diff --check` - passed.

## Follow-Ons

Start a new narrow lane for any of:

- key input API changes,
- cache invalidation work,
- cache lifetime/eviction work,
- deeper retained route-input cleanup outside cached edges.
