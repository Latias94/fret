# WGPU Custom Effect V3 Raw Wanted Shape v1 - Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This narrow follow-on is complete. `CustomEffectV3Pass::raw_wanted` is part of the cross-target
render-plan shape, and the cfg attributes around V3 raw-source pass literals have been removed.

## Important Invariant

Do not make `src_raw` or `src_pyramid` lifecycle validation conditional on `raw_wanted` or
`pyramid_wanted`. Custom Effect V3 execution still prepares both views; the wanted flags describe
requested source semantics for diagnostics and summaries.

## Future Work

Any broader Custom Effect V3 source-planning changes should start a separate follow-on and prove the
executor, lifecycle validator, render-plan summaries, and wasm feature checks together.
