# Render Text Dead Code Prune v1 - Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This lane is complete. `fret-render-text/src` no longer carries `dead_code` allowances.

## Important Invariant

Do not reintroduce wrapper-level hit-testing unless a real caller needs that interface. The current
renderer-facing hit-testing owner is prepared-line geometry via `hit_test_point_from_lines` and
`hit_test_x_from_stops`.

## Future Work

Broader text platform-shape cleanup remains separate. The next likely lane is wasm/native cfg
consolidation across WGPU text atlas/runtime diagnostics and text dump code.
