# Material3 Token Resolver Fallback v1 Evidence And Gates

Status: Active
Last updated: 2026-05-31

## Repro Surface

- Duplication audit:
  - `rg -n "fn alpha_mul|fn blend_over" ecosystem/fret-ui-material3/src/tokens -g "*.rs"`
- Token outcome proof:
  - `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`

## Gates

- Format:
  - `cargo fmt --package fret-ui-material3 --check`
- Token matrix:
  - `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`
- Package checks:
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- Workstream state:
  - `python -m json.tool docs/workstreams/material3-token-resolver-fallback-v1/WORKSTREAM.json | Out-Null`
  - `python tools/check_workstream_catalog.py`
- Layering:
  - `python tools/check_layering.py`
- Diff hygiene:
  - `git diff --check`

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`
- `ecosystem/fret-ui-material3/src/tokens/text_field.rs`
- `ecosystem/fret-ui-material3/src/tokens/select.rs`
- `ecosystem/fret-ui-material3/src/tokens/autocomplete.rs`
- `ecosystem/fret-ui-material3/src/tokens/checkbox.rs`
- `ecosystem/fret-ui-material3/src/tokens/radio.rs`
- `ecosystem/fret-ui-material3/src/tokens/switch.rs`
- `ecosystem/fret-ui-material3/src/tokens/slider.rs`
- `ecosystem/fret-ui-material3/src/tokens/list.rs`
- `ecosystem/fret-ui-material3/src/tokens/visual_fixtures.rs`
- `ecosystem/fret-ui-material3/tests/fixtures/material3_token_visual_cases_v1.json`
- `docs/workstreams/material3-token-resolver-fallback-v1/TODO.md`

## M3TRF-020 Evidence

- `rg -n "fn alpha_mul|fn blend_over" ecosystem/fret-ui-material3/src/tokens ecosystem/fret-ui-material3/src/foundation/token_resolver.rs -g "*.rs"`:
  only `foundation/token_resolver.rs` defines the helpers.
- `cargo fmt --package fret-ui-material3 --check`: passed.
- `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes foundation::token_resolver::tests`:
  3 passed.

## M3TRF-030 Evidence

- `rg -n "md\\.sys\\.state\\.(hover|focus|pressed)\\.state-layer-opacity" ecosystem/fret-ui-material3/src/tokens/checkbox.rs ecosystem/fret-ui-material3/src/tokens/radio.rs ecosystem/fret-ui-material3/src/tokens/switch.rs ecosystem/fret-ui-material3/src/tokens/slider.rs ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`:
  state-layer interaction opacity system keys are centralized in
  `foundation/token_resolver.rs` for migrated control token modules.
- `cargo fmt --package fret-ui-material3 --check`: passed.
- `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes foundation::token_resolver::tests`:
  4 passed.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`:
  passed.
