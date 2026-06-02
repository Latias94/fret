# Material3 Token Resolver Fallback v1 Handoff

Status: Closed
Last updated: 2026-05-31

## Current State

This lane is closed as a narrow follow-on from `material3-foundation-deepening-v1`.

M3TRF-020 is implemented: pure color composition helpers (`alpha_mul`, `blend_over`) now live in
`foundation::token_resolver`, and local copies were removed from component token modules.

M3TRF-030 is implemented: `MaterialTokenResolver` now owns Material state-layer interaction opacity
fallbacks and disabled state-layer opacity fallback, and Checkbox/Radio/Switch/Slider use it for
their migrated state-layer color/opacity paths.

M3TRF-040 is implemented: TextField, Select, and Autocomplete token modules now use shared
`MaterialTokenResolver` helpers for migrated component-to-system color fallback, multi-system
fallback chains, optional opacity lookup, and explicit fallback-color lookup.

M3TRF-050 is implemented: fresh closeout gates passed, the lane is closed, and remaining raw color
fallback families in non-field token modules are recorded as a future follow-on instead of being
folded into this lane.

## Guardrails

- Keep Material-specific fallback policy in `fret-ui-material3`.
- Keep generated v30 token injection unchanged unless a later task explicitly scopes it.
- Preserve token visual fixture outcomes before widening to fallback-chain helpers.

## Verified Gates

- `cargo fmt --package fret-ui-material3 --check`
- `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- `python -m json.tool docs/workstreams/material3-token-resolver-fallback-v1/WORKSTREAM.json | Out-Null`
- `python tools/check_workstream_catalog.py`
- `python tools/check_layering.py`
- `git diff --check`

## Follow-On

Open a narrower non-field component token fallback lane if continuing resolver hardening. The first
candidate scope is Button/Chip/IconButton/FAB/Tabs and related surface/navigation token modules that
still have raw component-to-system color fallback chains.
