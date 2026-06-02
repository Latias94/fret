# Material3 Non-Color Token Governance v1 Closeout Audit

Status: Closed
Date: 2026-05-31

## Summary

This lane closed the non-color token ownership follow-on from
`material3-token-resolver-non-field-fallback-v1`.

High-confidence non-color fallback chains now route through shared Material vocabulary:

- typography weight normalization through `tokens::typography`,
- number/duration/easing fallback access through `MaterialTokenResolver`,
- component modules still own local key selection and scalar metric defaults.

## Completed Scope

- M3NC-020: AssistChip, FilterChip, InputChip, SuggestionChip, and Slider label weight reads now use
  `tokens::typography::text_style_with_weight`.
- M3NC-030: Radio disabled icon opacity now uses `MaterialTokenResolver::number_optional`.
- M3NC-040: Dialog, Snackbar, and ModalNavigationDrawer duration/easing reads now use resolver
  motion helpers.
- M3NC-050: TimeInput/TimePicker state-layer opacity chains, DatePicker/TimePicker modal motion
  reads, DatePicker outside-month opacity, Dropdown/Tooltip durations, and indication ripple
  motion reads now use resolver helpers.
- M3NC-060: closeout verification and workstream state update.

## Final Gates

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
- `python -m json.tool docs/workstreams/material3-non-color-token-governance-v1/WORKSTREAM.json | Out-Null`:
  passed.
- `python tools/check_workstream_catalog.py`: passed; 526 dedicated directories, 47 standalone
  markdown files.
- `python tools/check_layering.py`: passed.
- `git diff --check`: passed.

## Residual Ownership

- `foundation/token_resolver.rs`: owns raw theme access for shared token fallback vocabulary.
- `tokens/typography.rs`: owns typography style lookup and weight normalization.
- `foundation/context.rs`: owns Material capability flags such as expressive mode and RTL flag.
- `lib.rs`: owns token-key registration checks across token namespaces.
- `tokens/visual_fixtures.rs`: owns fixture lookup for token visual matrix tests.
- Component-local metric scalar defaults remain intentionally local unless a future audit finds a
  repeated component-to-system fallback chain.

## Known External Drift

The unfiltered `radio_alignment` binary currently includes unrelated headless overlay/navigation
golden suites that fail independently of this lane. M3NC-030 therefore uses the filtered
`radio_alignment radio` proof plus choice-control state tests as its canonical gate.
