# WGPU Text Measure Dead Branch Prune v1 - Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This narrow follow-on is complete. WGPU text measurement methods are thin facades over
`fret_render_text::TextMeasureCaches`, and the unreachable inline duplicate implementations have
been removed.

## Important Invariant

Do not add text measurement cache policy back into `fret-render-wgpu` unless the renderer owns a
new backend-specific measurement behavior. Shared measurement, wrapping, and shaping-cache policy
belong in `fret-render-text`.

## Future Work

The larger text cleanup remains separate: wasm/native cfg consolidation in atlas/runtime diagnostics
and text dump code should use a dedicated follow-on with native and wasm checks.
