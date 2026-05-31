# Material3 Non-Color Token Governance v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-31

## Baseline Audit

- Direct non-color read audit:
  - `rg -n "theme\\.number_by_key\\(|theme\\.duration_ms_by_key\\(|theme\\.easing_by_key\\(" ecosystem/fret-ui-material3/src -g "*.rs" -g "!tokens/v30.rs" -g "!tokens/material_web_v30.rs"`
- Focused token-module audit:
  - `rg -n "number_by_key|duration_ms_by_key|easing_by_key" ecosystem/fret-ui-material3/src/foundation ecosystem/fret-ui-material3/src/tokens -g "*.rs" -g "!v30.rs" -g "!material_web_v30.rs"`

## Gates

- Format:
  - `cargo fmt --package fret-ui-material3 --check`
- Token matrix:
  - `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`
- Typography weight slice:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test chip_state --test slider_state`
- Selection numeric slice:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment radio`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test checkbox_state --test slider_state --test switch_state`
- Motion slice:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test dialog_state --test snackbar_state --test navigation_drawer_state`
- Time picker/input slice:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`
- Package checks:
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- Workstream state:
  - `python -m json.tool docs/workstreams/material3-non-color-token-governance-v1/WORKSTREAM.json | Out-Null`
  - `python tools/check_workstream_catalog.py`
- Layering:
  - `python tools/check_layering.py`
- Diff hygiene:
  - `git diff --check`

## Evidence Anchors

- `docs/workstreams/material3-non-color-token-governance-v1/DESIGN.md`
- `docs/workstreams/material3-non-color-token-governance-v1/TODO.md`
- `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`
- `ecosystem/fret-ui-material3/src/tokens/typography.rs`
- `ecosystem/fret-ui-material3/src/tokens/chip.rs`
- `ecosystem/fret-ui-material3/src/tokens/filter_chip.rs`
- `ecosystem/fret-ui-material3/src/tokens/input_chip.rs`
- `ecosystem/fret-ui-material3/src/tokens/suggestion_chip.rs`
- `ecosystem/fret-ui-material3/src/tokens/slider.rs`
- `ecosystem/fret-ui-material3/src/tokens/radio.rs`
- `ecosystem/fret-ui-material3/src/tokens/dialog.rs`
- `ecosystem/fret-ui-material3/src/tokens/snackbar.rs`
- `ecosystem/fret-ui-material3/src/tokens/time_input.rs`
- `ecosystem/fret-ui-material3/src/tokens/time_picker.rs`
- `ecosystem/fret-ui-material3/tests/chip_state.rs`
- `ecosystem/fret-ui-material3/tests/slider_state.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `ecosystem/fret-ui-material3/tests/dialog_state.rs`
- `ecosystem/fret-ui-material3/tests/snackbar_state.rs`
- `ecosystem/fret-ui-material3/tests/navigation_drawer_state.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-non-color-token-governance-v1/CLOSEOUT_AUDIT_2026-05-31.md`

## M3NC-010 Evidence

- `git status --short`: clean before opening the lane.
- Baseline audit found direct non-color reads in typography weight, Radio disabled opacity,
  Dialog/Snackbar/ModalNavigationDrawer easing, TimeInput/TimePicker numeric fallbacks, and
  foundation/runtime capability checks.
- Metric scalar defaults are explicitly out of scope unless they form repeated fallback chains.

## M3NC-020 Evidence

- `ecosystem/fret-ui-material3/src/tokens/chip.rs`,
  `ecosystem/fret-ui-material3/src/tokens/filter_chip.rs`,
  `ecosystem/fret-ui-material3/src/tokens/input_chip.rs`,
  `ecosystem/fret-ui-material3/src/tokens/suggestion_chip.rs`, and
  `ecosystem/fret-ui-material3/src/tokens/slider.rs`: label weight normalization now uses
  `tokens::typography::text_style_with_weight`.
- `rg -n "theme\\.number_by_key\\(|FontWeight\\(" ecosystem/fret-ui-material3/src/tokens/chip.rs ecosystem/fret-ui-material3/src/tokens/filter_chip.rs ecosystem/fret-ui-material3/src/tokens/input_chip.rs ecosystem/fret-ui-material3/src/tokens/suggestion_chip.rs ecosystem/fret-ui-material3/src/tokens/slider.rs`:
  no matches.
- `cargo fmt --package fret-ui-material3 --check`: passed.
- `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`:
  1 passed, 165 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test chip_state --test slider_state`:
  10 passed, 0 skipped.

## M3NC-030 Evidence

- `ecosystem/fret-ui-material3/src/tokens/radio.rs`: disabled selected/unselected icon opacity now
  uses `MaterialTokenResolver::number_optional` instead of a direct `theme.number_by_key` read.
- `rg -n "theme\\.number_by_key\\(|FontWeight\\(" ecosystem/fret-ui-material3/src/tokens/radio.rs`:
  no matches.
- `cargo fmt --package fret-ui-material3 --check`: passed.
- `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`:
  1 passed, 165 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment radio`:
  3 passed, 69 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test checkbox_state --test slider_state --test switch_state`:
  10 passed, 0 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment --test checkbox_state --test slider_state --test switch_state`:
  failed in unrelated `material3_headless_overlays_suite_goldens_v1` and
  `material3_headless_navigation_suite_goldens_v1` golden suites inside `radio_alignment`; the
  Radio-specific filtered gate above is the canonical M3NC-030 proof.

## M3NC-040 Evidence

- `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`: added Material duration/easing
  resolver helpers for sys durations, optional easing, easing chains, and linear fallback.
- `ecosystem/fret-ui-material3/src/tokens/dialog.rs`: default open/close durations and easing now
  route through `MaterialTokenResolver`.
- `ecosystem/fret-ui-material3/src/tokens/snackbar.rs`: open/close durations and emphasized-to-
  standard easing chain now route through `MaterialTokenResolver`.
- `ecosystem/fret-ui-material3/src/modal_navigation_drawer.rs`: open/close duration fallback and
  easing fallback now route through `MaterialTokenResolver`.
- `rg -n "theme\\.duration_ms_by_key\\(|theme\\.easing_by_key\\(" ecosystem/fret-ui-material3/src/tokens/dialog.rs ecosystem/fret-ui-material3/src/tokens/snackbar.rs ecosystem/fret-ui-material3/src/modal_navigation_drawer.rs`:
  no matches.
- `cargo fmt --package fret-ui-material3`: passed.
- `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`:
  1 passed, 165 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test dialog_state --test snackbar_state --test navigation_drawer_state`:
  9 passed, 0 skipped.

## M3NC-050 Evidence

- `ecosystem/fret-ui-material3/src/tokens/time_input.rs` and
  `ecosystem/fret-ui-material3/src/tokens/time_picker.rs`: state-layer opacity component-to-system
  number fallback chains now use `MaterialTokenResolver::number_comp_or_sys`.
- `ecosystem/fret-ui-material3/src/time_picker.rs` and
  `ecosystem/fret-ui-material3/src/date_picker.rs`: modal duration/easing fallback reads now use
  `MaterialTokenResolver` motion helpers; TimePicker scrim opacity uses resolver number access.
- `ecosystem/fret-ui-material3/src/tokens/date_picker.rs`: outside-month label opacity uses
  resolver number access.
- `ecosystem/fret-ui-material3/src/tokens/dropdown_menu.rs` and
  `ecosystem/fret-ui-material3/src/tokens/tooltip.rs`: close durations use resolver duration
  access.
- `ecosystem/fret-ui-material3/src/foundation/indication.rs`: ripple/state duration and easing
  token reads use resolver motion helpers.
- `rg -n "theme\\.number_by_key\\(|theme\\.duration_ms_by_key\\(|theme\\.easing_by_key\\(" ecosystem/fret-ui-material3/src -g "*.rs" -g "!tokens/v30.rs" -g "!tokens/material_web_v30.rs"`:
  remaining matches are limited to `lib.rs` token-key registration checks, `foundation/token_resolver.rs`,
  and centralized `tokens/typography.rs`.
- `rg -n "number_by_key|duration_ms_by_key|easing_by_key" ecosystem/fret-ui-material3/src/foundation ecosystem/fret-ui-material3/src/tokens -g "*.rs" -g "!v30.rs" -g "!material_web_v30.rs"`:
  remaining matches are limited to context capability flags, `foundation/token_resolver.rs`,
  centralized `tokens/typography.rs`, and `tokens/visual_fixtures.rs` fixture lookup.
- `cargo fmt --package fret-ui-material3 --check`: passed.
- `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`:
  1 passed, 165 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface --test menu_state --test tooltip_state --test dialog_state --test snackbar_state --test navigation_drawer_state --test chip_state --test slider_state --test checkbox_state --test switch_state`:
  58 passed, 0 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment radio`:
  3 passed, 69 skipped.

## M3NC-060 Evidence

- `cargo fmt --package fret-ui-material3 --check`: passed.
- `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`:
  1 passed, 165 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface --test menu_state --test tooltip_state --test dialog_state --test snackbar_state --test navigation_drawer_state --test chip_state --test slider_state --test checkbox_state --test switch_state`:
  58 passed, 0 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment radio`:
  3 passed, 69 skipped.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`:
  passed.
- `rg -n "theme\\.number_by_key\\(|theme\\.duration_ms_by_key\\(|theme\\.easing_by_key\\(" ecosystem/fret-ui-material3/src -g "*.rs" -g "!tokens/v30.rs" -g "!tokens/material_web_v30.rs"`:
  remaining matches limited to `lib.rs`, `foundation/token_resolver.rs`, and `tokens/typography.rs`.
- `rg -n "number_by_key|duration_ms_by_key|easing_by_key" ecosystem/fret-ui-material3/src/foundation ecosystem/fret-ui-material3/src/tokens -g "*.rs" -g "!v30.rs" -g "!material_web_v30.rs"`:
  remaining matches limited to context flags, resolver internals, typography helpers, and fixture
  lookup.
- `python -m json.tool docs/workstreams/material3-non-color-token-governance-v1/WORKSTREAM.json | Out-Null`:
  passed.
- `python tools/check_workstream_catalog.py`: passed; 526 dedicated directories, 47 standalone
  markdown files.
- `python tools/check_layering.py`: passed.
- `git diff --check`: passed.
- `docs/workstreams/material3-non-color-token-governance-v1/CLOSEOUT_AUDIT_2026-05-31.md`:
  closeout audit added.
