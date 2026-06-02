# Material3 Token Resolver Non-Field Fallback v1 Evidence And Gates

Status: Closed
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
- Surface/navigation slice:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test card_state --test carousel_item_state --test dialog_state --test list_state --test menu_state --test navigation_drawer_state --test navigation_state --test progress_indicator_state --test snackbar_state --test tooltip_state --test bottom_sheet_motion --test automation_surface`
- Selection-control slice:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test checkbox_state --test slider_state --test switch_state --test automation_surface`
- Closeout targeted state suite:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test button_state --test chip_state --test icon_button_state --test fab_state --test segmented_button_state --test tabs_state --test card_state --test carousel_item_state --test dialog_state --test list_state --test menu_state --test navigation_drawer_state --test navigation_state --test progress_indicator_state --test snackbar_state --test tooltip_state --test bottom_sheet_motion --test checkbox_state --test slider_state --test switch_state --test automation_surface`
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
- `ecosystem/fret-ui-material3/src/tokens/icon_button.rs`
- `ecosystem/fret-ui-material3/src/tokens/fab.rs`
- `ecosystem/fret-ui-material3/src/tokens/segmented_button.rs`
- `ecosystem/fret-ui-material3/src/tokens/tabs.rs`
- `ecosystem/fret-ui-material3/src/tokens/badge.rs`
- `ecosystem/fret-ui-material3/src/tokens/card.rs`
- `ecosystem/fret-ui-material3/src/tokens/carousel_item.rs`
- `ecosystem/fret-ui-material3/src/tokens/dialog.rs`
- `ecosystem/fret-ui-material3/src/tokens/divider.rs`
- `ecosystem/fret-ui-material3/src/tokens/list.rs`
- `ecosystem/fret-ui-material3/src/tokens/menu.rs`
- `ecosystem/fret-ui-material3/src/tokens/navigation_bar.rs`
- `ecosystem/fret-ui-material3/src/tokens/navigation_drawer.rs`
- `ecosystem/fret-ui-material3/src/tokens/navigation_rail.rs`
- `ecosystem/fret-ui-material3/src/tokens/progress_indicator.rs`
- `ecosystem/fret-ui-material3/src/tokens/search_bar.rs`
- `ecosystem/fret-ui-material3/src/tokens/search_view.rs`
- `ecosystem/fret-ui-material3/src/tokens/sheet_bottom.rs`
- `ecosystem/fret-ui-material3/src/tokens/snackbar.rs`
- `ecosystem/fret-ui-material3/src/tokens/tooltip.rs`
- `ecosystem/fret-ui-material3/src/tokens/checkbox.rs`
- `ecosystem/fret-ui-material3/src/tokens/slider.rs`
- `ecosystem/fret-ui-material3/src/tokens/switch.rs`
- `ecosystem/fret-ui-material3/tests/button_state.rs`
- `ecosystem/fret-ui-material3/tests/chip_state.rs`
- `ecosystem/fret-ui-material3/tests/icon_button_state.rs`
- `ecosystem/fret-ui-material3/tests/fab_state.rs`
- `ecosystem/fret-ui-material3/tests/segmented_button_state.rs`
- `ecosystem/fret-ui-material3/tests/tabs_state.rs`
- `ecosystem/fret-ui-material3/tests/card_state.rs`
- `ecosystem/fret-ui-material3/tests/carousel_item_state.rs`
- `ecosystem/fret-ui-material3/tests/dialog_state.rs`
- `ecosystem/fret-ui-material3/tests/list_state.rs`
- `ecosystem/fret-ui-material3/tests/menu_state.rs`
- `ecosystem/fret-ui-material3/tests/navigation_drawer_state.rs`
- `ecosystem/fret-ui-material3/tests/navigation_state.rs`
- `ecosystem/fret-ui-material3/tests/progress_indicator_state.rs`
- `ecosystem/fret-ui-material3/tests/snackbar_state.rs`
- `ecosystem/fret-ui-material3/tests/tooltip_state.rs`
- `ecosystem/fret-ui-material3/tests/bottom_sheet_motion.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/checkbox_state.rs`
- `ecosystem/fret-ui-material3/tests/slider_state.rs`
- `ecosystem/fret-ui-material3/tests/switch_state.rs`
- `ecosystem/fret-ui-material3/tests/fixtures/material3_token_visual_cases_v1.json`
- `docs/workstreams/material3-token-resolver-non-field-fallback-v1/CLOSEOUT_AUDIT_2026-05-31.md`

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

## M3NF-040 Evidence

- `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`: added reusable component-chain to
  sys-chain color lookup and component-chain to sys numeric lookup helpers.
- `ecosystem/fret-ui-material3/src/tokens/icon_button.rs`: migrated icon, state-layer,
  container, disabled, and outline fallback chains to `MaterialTokenResolver`.
- `ecosystem/fret-ui-material3/src/tokens/fab.rs`: migrated FAB container, shadow, icon, label,
  state-layer color, and state-layer opacity fallback chains.
- `ecosystem/fret-ui-material3/src/tokens/segmented_button.rs`: migrated selected container,
  outline, label, icon, state-layer color, and state-layer opacity fallback chains.
- `ecosystem/fret-ui-material3/src/tokens/tabs.rs`: migrated tab container, divider, active
  indicator, icon, label, state-layer color, and state-layer opacity fallback chains.
- `rg -n "theme\\.color_by_key|theme\\.color_token|theme\\.number_by_key\\(|or_else\\(\\|\\| theme\\.number_by_key" ecosystem/fret-ui-material3/src/tokens/icon_button.rs ecosystem/fret-ui-material3/src/tokens/fab.rs ecosystem/fret-ui-material3/src/tokens/segmented_button.rs ecosystem/fret-ui-material3/src/tokens/tabs.rs`: no matches.
- Residual fallback audit no longer lists `icon_button.rs`, `fab.rs`, `segmented_button.rs`, or
  `tabs.rs`.
- `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`: 1 passed.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test icon_button_state --test fab_state --test segmented_button_state --test tabs_state`: 21 passed.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`: passed.
- `cargo fmt --package fret-ui-material3 --check`: passed.
- `python -m json.tool docs/workstreams/material3-token-resolver-non-field-fallback-v1/WORKSTREAM.json | Out-Null`: passed.
- `python tools/check_workstream_catalog.py`: passed; 525 dedicated directories, 47 standalone markdown files.
- `python tools/check_layering.py`: passed.
- `git diff --check`: passed.

## M3NF-050 Evidence

- `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`: added missing sys fallback colors
  needed by the residual surface/navigation and overlay families (`primary-container`,
  `tertiary`, `surface-variant`, inverse colors, `on-error`, and `scrim`).
- `ecosystem/fret-ui-material3/src/tokens/badge.rs`, `divider.rs`, `progress_indicator.rs`,
  `search_bar.rs`, `search_view.rs`, `sheet_bottom.rs`, and `tooltip.rs`: migrated small residual
  component/system color fallback and optional opacity lookups to `MaterialTokenResolver`.
- `ecosystem/fret-ui-material3/src/tokens/card.rs` and `carousel_item.rs`: migrated surface
  container, shadow, outline, disabled opacity, and state-layer fallback paths while retaining
  variant/state key selection.
- `ecosystem/fret-ui-material3/src/tokens/dialog.rs`, `menu.rs`, and `snackbar.rs`: migrated overlay
  container, text/icon, state-layer color, and state-layer opacity fallback paths.
- `ecosystem/fret-ui-material3/src/tokens/list.rs`: migrated list text/icon, selected container,
  disabled opacity, and state-layer fallback paths.
- `ecosystem/fret-ui-material3/src/tokens/navigation_bar.rs`,
  `ecosystem/fret-ui-material3/src/tokens/navigation_rail.rs`, and
  `ecosystem/fret-ui-material3/src/tokens/navigation_drawer.rs`: migrated navigation container,
  indicator, label/icon, state-layer color, and state-layer opacity fallback paths.
- `rg -n "theme\\.color_by_key|theme\\.color_token|theme\\.number_by_key\\(|or_else\\(\\|\\| theme\\.number_by_key|or_else\\(\\|\\| theme\\.color" ecosystem/fret-ui-material3/src/tokens/badge.rs ecosystem/fret-ui-material3/src/tokens/card.rs ecosystem/fret-ui-material3/src/tokens/carousel_item.rs ecosystem/fret-ui-material3/src/tokens/dialog.rs ecosystem/fret-ui-material3/src/tokens/divider.rs ecosystem/fret-ui-material3/src/tokens/list.rs ecosystem/fret-ui-material3/src/tokens/menu.rs ecosystem/fret-ui-material3/src/tokens/navigation_bar.rs ecosystem/fret-ui-material3/src/tokens/navigation_drawer.rs ecosystem/fret-ui-material3/src/tokens/navigation_rail.rs ecosystem/fret-ui-material3/src/tokens/progress_indicator.rs ecosystem/fret-ui-material3/src/tokens/search_bar.rs ecosystem/fret-ui-material3/src/tokens/search_view.rs ecosystem/fret-ui-material3/src/tokens/sheet_bottom.rs ecosystem/fret-ui-material3/src/tokens/snackbar.rs ecosystem/fret-ui-material3/src/tokens/tooltip.rs`: no matches.
- Residual fallback audit now only lists `checkbox.rs`, `slider.rs`, and `switch.rs`; these are
  split to M3NF-055 as a focused selection-control slice.
- `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`: 1 passed.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test card_state --test carousel_item_state --test dialog_state --test list_state --test menu_state --test navigation_drawer_state --test navigation_state --test progress_indicator_state --test snackbar_state --test tooltip_state --test bottom_sheet_motion --test automation_surface`: 62 passed.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`: passed.
- `cargo fmt --package fret-ui-material3 --check`: passed.
- `python -m json.tool docs/workstreams/material3-token-resolver-non-field-fallback-v1/WORKSTREAM.json | Out-Null`: passed.
- `python tools/check_workstream_catalog.py`: passed; 525 dedicated directories, 47 standalone markdown files.
- `python tools/check_layering.py`: passed.
- `git diff --check`: passed.

## M3NF-055 Evidence

- `ecosystem/fret-ui-material3/src/tokens/checkbox.rs`: migrated selected/unselected container,
  outline, icon, disabled color, and disabled opacity fallback paths to `MaterialTokenResolver`.
- `ecosystem/fret-ui-material3/src/tokens/slider.rs`: migrated value indicator, tick mark, stop
  indicator, active/inactive track, handle, and disabled opacity fallback paths to
  `MaterialTokenResolver`.
- `ecosystem/fret-ui-material3/src/tokens/switch.rs`: migrated disabled icon, track, handle, and
  outline fallback paths to `MaterialTokenResolver`.
- `rg --count-matches "or_else\\(\\|\\| theme\\.color_by_key|unwrap_or_else\\(\\|\\| theme\\.color_token|theme\\.color_by_key\\(" ecosystem/fret-ui-material3/src/tokens -g "*.rs" -g "!v30.rs" -g "!material_web_v30.rs"`: no matches.
- `rg -n "theme\\.number_by_key\\(|or_else\\(\\|\\| theme\\.number_by_key" ecosystem/fret-ui-material3/src/tokens/checkbox.rs ecosystem/fret-ui-material3/src/tokens/slider.rs ecosystem/fret-ui-material3/src/tokens/switch.rs`: only Slider label-text weight read remains.
- `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`: 1 passed.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test checkbox_state --test slider_state --test switch_state --test automation_surface`: 36 passed.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`: passed.
- `cargo fmt --package fret-ui-material3 --check`: passed.
- `python -m json.tool docs/workstreams/material3-token-resolver-non-field-fallback-v1/WORKSTREAM.json | Out-Null`: passed.
- `python tools/check_workstream_catalog.py`: passed; 525 dedicated directories, 47 standalone markdown files.
- `python tools/check_layering.py`: passed.
- `git diff --check`: passed.

## M3NF-060 Evidence

- `cargo fmt --package fret-ui-material3 --check`: passed.
- `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`:
  1 passed, 165 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test button_state --test chip_state --test icon_button_state --test fab_state --test segmented_button_state --test tabs_state --test card_state --test carousel_item_state --test dialog_state --test list_state --test menu_state --test navigation_drawer_state --test navigation_state --test progress_indicator_state --test snackbar_state --test tooltip_state --test bottom_sheet_motion --test checkbox_state --test slider_state --test switch_state --test automation_surface`:
  102 passed, 0 skipped.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`:
  passed.
- `rg --count-matches "or_else\\(\\|\\| theme\\.color_by_key|unwrap_or_else\\(\\|\\| theme\\.color_token|theme\\.color_by_key\\(" ecosystem/fret-ui-material3/src/tokens -g "*.rs" -g "!v30.rs" -g "!material_web_v30.rs"`:
  no matches.
- `python -m json.tool docs/workstreams/material3-token-resolver-non-field-fallback-v1/WORKSTREAM.json | Out-Null`:
  passed.
- `python tools/check_workstream_catalog.py`: passed; 525 dedicated directories, 47 standalone
  markdown files.
- `python tools/check_layering.py`: passed.
- `git diff --check`: passed.
- `docs/workstreams/material3-token-resolver-non-field-fallback-v1/CLOSEOUT_AUDIT_2026-05-31.md`:
  closeout audit added.
