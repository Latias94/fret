# Material3 Token Resolver Fallback v1 Closeout Audit

Status: Closed
Date: 2026-05-31

## Outcome

This lane closed after deepening the Material3 token resolver around three bounded fallback
surfaces:

- Pure color composition helpers now live behind `foundation::token_resolver`.
- State-layer interaction opacity fallback is centralized for the migrated selection/control token
  modules.
- TextField, Select, and Autocomplete field-family color/opacity fallback mechanics use shared
  `MaterialTokenResolver` helpers.

## Shipped Changes

- `alpha_mul` and `blend_over` are defined in `foundation::token_resolver` and no longer copied in
  component token modules.
- `MaterialTokenResolver` now owns state-layer interaction opacity lookup and disabled state-layer
  opacity fallback.
- Checkbox, Radio, Switch, and Slider keep component role/state key selection while using resolver
  helpers for migrated state-layer paths.
- `MaterialTokenResolver` now exposes field-family primitives for component-to-system color
  fallback, multi-system color fallback chains, optional opacity lookup, and explicit fallback-color
  lookup.
- TextField, Select, and Autocomplete token modules no longer use raw
  `theme.color_by_key(...).or_else(|| theme.color_by_key(...)).unwrap_or_else(...)` fallback chains.

## Verification

- `cargo fmt --package fret-ui-material3 --check`
- `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test text_field_hover --test select_behavior --test autocomplete_motion`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- `rg -n "fn alpha_mul|fn blend_over" ecosystem/fret-ui-material3/src/tokens ecosystem/fret-ui-material3/src/foundation/token_resolver.rs -g "*.rs" -g "!v30.rs" -g "!material_web_v30.rs"`
- `rg -n "or_else\\(\\|\\| theme\\.color_by_key|unwrap_or_else\\(\\|\\| theme\\.color_token|theme\\.color_by_key\\(" ecosystem/fret-ui-material3/src/tokens/autocomplete.rs ecosystem/fret-ui-material3/src/tokens/text_field.rs ecosystem/fret-ui-material3/src/tokens/select.rs`
- `python -m json.tool docs/workstreams/material3-token-resolver-fallback-v1/WORKSTREAM.json | Out-Null`
- `python tools/check_workstream_catalog.py`
- `python tools/check_layering.py`
- `git diff --check`

## Residual Risk

- Non-field component token modules still contain raw color fallback chains. The highest-count
  residual families are Button, Chip/InputChip/FilterChip/SuggestionChip, IconButton, Slider,
  Tabs, FAB, Snackbar, Card, Dialog, List, Tooltip, and navigation/drawer surfaces.
- Some component modules still mix token role selection with layout/elevation/shadow fallback
  details. This is outside this lane because the shipped target was resolver fallback mechanics,
  not all component token cleanup.
- Generated Material Web v30 data remains unchanged by design.

## Follow-Ons

- Open a narrow `material3-token-resolver-non-field-fallback-v1` lane if continuing resolver
  hardening. Start with non-field component color fallback families that already have visual
  fixture coverage.
- Keep public recipe behavior unchanged; continue proving each migrated family with the token visual
  fixture matrix plus targeted component tests.
