# Material3 Token Resolver Fallback v1 Handoff

Status: Active
Last updated: 2026-05-31

## Current State

This lane is open as a narrow follow-on from `material3-foundation-deepening-v1`.

M3TRF-020 is implemented: pure color composition helpers (`alpha_mul`, `blend_over`) now live in
`foundation::token_resolver`, and local copies were removed from component token modules.

M3TRF-030 is implemented: `MaterialTokenResolver` now owns Material state-layer interaction opacity
fallbacks and disabled state-layer opacity fallback, and Checkbox/Radio/Switch/Slider use it for
their migrated state-layer color/opacity paths.

The next executable task is M3TRF-040: migrate the heaviest field-family fallback modules
(TextField, Select, Autocomplete) onto the shared resolver vocabulary without visual drift.

## Guardrails

- Keep Material-specific fallback policy in `fret-ui-material3`.
- Keep generated v30 token injection unchanged unless a later task explicitly scopes it.
- Preserve token visual fixture outcomes before widening to fallback-chain helpers.

## Suggested First Gate

For M3TRF-040, start with:

`cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`
