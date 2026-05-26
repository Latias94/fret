# Fret Node Paint Root Cached Edge Key Helper v1

Status: Closed
Last updated: 2026-05-25

Status note (2026-05-25): this lane shipped the cached edge key shared field helper and is closed.
Future key input API, invalidation, or cache lifetime work should start as separate narrow
follow-ons.

## Why This Lane Exists

`fret-node-paint-root-cached-edge-fallback-adapter-v1` closed cached fallback retained route inputs.
The remaining cached edge key surface is pure data policy: `cached_edges/keys.rs` repeats the same
base key-field writes across edge, edge-label, single-rect, and tiled cache keys.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-cached-edge-fallback-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/keys.rs`

## Problem

`keys.rs` duplicates the same base key-field sequence in four functions. That makes future key input
changes easy to apply to only one edge/label or single/tiled path by mistake.

This lane is not a cache invalidation or cache lifetime change.

## Target State

- Cached edge key shared field writes live behind one helper in `keys.rs`.
- Existing scope strings, inputs, and output key shapes are preserved.
- Source-policy coverage locks the shared key-field helper so the duplicated field sequence does
  not come back.

## In Scope

- `cached_edges/keys.rs`
- focused source-policy coverage in `ecosystem/fret-node/src/lib.rs`
- workstream evidence and gates

## Out Of Scope

- cache invalidation semantics,
- cache lifetime and eviction,
- cache scope string renames,
- route input adapters,
- replay, fallback, overlay, anchor target, or build-state behavior.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The four key functions share identical base fields. | Confident | `keys.rs` repeats graph/zoom/origin/draw-order/presenter/edge-type/override/style/tile-size writes. | Do not merge any field that differs. |
| Single-rect keys must keep rect-origin fields local. | Confident | Tiled base keys do not include rect origin. | Keep rect fields outside the shared helper. |
| Scope strings are behavior and must not change. | Confident | Cache entries depend on those namespace strings. | Preserve all strings byte-for-byte. |

## Architecture Direction

Prefer a narrow helper:

- `cached_edge_key_builder(scope, base_key, style_key, edges_cache_tile_size_canvas)`

Public key functions keep their existing names and inputs, call the helper, and add only the
path-specific fields before `finish()`.

## Closeout Condition

This lane can close when `keys.rs` has one shared base key-field helper, existing key functions keep
their public shape and scope strings, focused source-policy coverage proves the field sequence is not
duplicated, and validation gates pass.

Closeout result (2026-05-25): complete. `keys.rs` now has one shared base key-field helper while
preserving all four existing key functions and scope strings.
