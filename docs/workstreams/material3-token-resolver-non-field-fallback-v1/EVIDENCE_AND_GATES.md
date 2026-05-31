# Material3 Token Resolver Non-Field Fallback v1 Evidence And Gates

Status: Active
Last updated: 2026-05-31

## Repro Surface

- Residual fallback audit:
  - `rg --count-matches "or_else\\(\\|\\| theme\\.color_by_key|unwrap_or_else\\(\\|\\| theme\\.color_token|theme\\.color_by_key\\(" ecosystem/fret-ui-material3/src/tokens -g "*.rs" -g "!v30.rs" -g "!material_web_v30.rs"`
- Token outcome proof:
  - `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`

## Gates

- Format:
  - `cargo fmt --package fret-ui-material3 --check`
- Token matrix:
  - `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`
- First slice:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test button_state`
- Package checks:
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- Workstream state:
  - `python -m json.tool docs/workstreams/material3-token-resolver-non-field-fallback-v1/WORKSTREAM.json | Out-Null`
  - `python tools/check_workstream_catalog.py`
- Layering:
  - `python tools/check_layering.py`
- Diff hygiene:
  - `git diff --check`

## Evidence Anchors

- `docs/workstreams/material3-token-resolver-non-field-fallback-v1/DESIGN.md`
- `docs/workstreams/material3-token-resolver-non-field-fallback-v1/TODO.md`
- `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`
- `ecosystem/fret-ui-material3/src/tokens/button.rs`
- `ecosystem/fret-ui-material3/src/tokens/chip.rs`
- `ecosystem/fret-ui-material3/src/tokens/filter_chip.rs`
- `ecosystem/fret-ui-material3/src/tokens/input_chip.rs`
- `ecosystem/fret-ui-material3/src/tokens/suggestion_chip.rs`
- `ecosystem/fret-ui-material3/tests/button_state.rs`
- `ecosystem/fret-ui-material3/tests/chip_state.rs`
- `ecosystem/fret-ui-material3/tests/fixtures/material3_token_visual_cases_v1.json`

## M3NF-010 Evidence

- `git status --short`: clean before opening the lane.
- Residual fallback audit identified non-field component families with repeated raw color fallback
  chains; Button is the first bounded target.

## M3NF-020 Evidence

- `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`: added reusable component-token
  chain and number-chain resolver helpers plus missing secondary-container fallback colors.
- `ecosystem/fret-ui-material3/src/tokens/button.rs`: migrated Button label, container, shadow,
  icon, disabled, and state-layer opacity fallback chains to `MaterialTokenResolver`.
- `rg -n "theme\\.color_by_key|theme\\.number_by_key|theme\\.color_token|or_else\\(\\|\\| theme" ecosystem/fret-ui-material3/src/tokens/button.rs`: no matches.
- `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`: 1 passed.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test button_state`: 3 passed.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`: passed.
- `cargo fmt --package fret-ui-material3 --check`: passed.
- `python -m json.tool docs/workstreams/material3-token-resolver-non-field-fallback-v1/WORKSTREAM.json | Out-Null`: passed.
- `python tools/check_workstream_catalog.py`: passed; 525 dedicated directories, 47 standalone markdown files.
- `python tools/check_layering.py`: passed.
- `git diff --check`: passed.

## M3NF-030 Evidence

- `ecosystem/fret-ui-material3/src/tokens/chip.rs`: migrated AssistChip label/icon/state-layer,
  elevated container, shadow, and flat outline color/disabled fallback chains to
  `MaterialTokenResolver`.
- `ecosystem/fret-ui-material3/src/tokens/filter_chip.rs`: migrated FilterChip selected/elevated
  containers, shadow, label, state-layer, leading/trailing icons, and outline fallback chains.
- `ecosystem/fret-ui-material3/src/tokens/input_chip.rs`: migrated InputChip selected container,
  unselected outline, label, state-layer, leading icon, and trailing icon fallback chains.
- `ecosystem/fret-ui-material3/src/tokens/suggestion_chip.rs`: migrated SuggestionChip elevated
  container, shadow, label, state-layer, leading icon, and flat outline fallback chains.
- `rg -n "theme\\.color_by_key|theme\\.color_token|or_else\\(\\|\\| theme" ecosystem/fret-ui-material3/src/tokens/chip.rs ecosystem/fret-ui-material3/src/tokens/filter_chip.rs ecosystem/fret-ui-material3/src/tokens/input_chip.rs ecosystem/fret-ui-material3/src/tokens/suggestion_chip.rs`: no matches.
- `rg -n "theme\\.number_by_key\\(|or_else\\(\\|\\| theme\\.number_by_key" ecosystem/fret-ui-material3/src/tokens/chip.rs ecosystem/fret-ui-material3/src/tokens/filter_chip.rs ecosystem/fret-ui-material3/src/tokens/input_chip.rs ecosystem/fret-ui-material3/src/tokens/suggestion_chip.rs`: only label-text weight reads remain.
- `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`: 1 passed.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test chip_state`: 6 passed.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`: passed.
- `cargo fmt --package fret-ui-material3 --check`: passed.
- `python -m json.tool docs/workstreams/material3-token-resolver-non-field-fallback-v1/WORKSTREAM.json | Out-Null`: passed.
- `python tools/check_workstream_catalog.py`: passed; 525 dedicated directories, 47 standalone markdown files.
- `python tools/check_layering.py`: passed.
- `git diff --check`: passed.
- Residual fallback audit no longer lists `chip.rs`, `filter_chip.rs`, `input_chip.rs`, or
  `suggestion_chip.rs`.
