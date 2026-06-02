# Material3 Slider Token Defaults v1 - Closeout Audit

Status: Closed
Date: 2026-05-31

## Summary

This fearless-refactor lane moved Material3 Slider visual default matrices out of `tokens::slider`
resolver functions and into a private helper. The runtime token APIs and Slider recipe behavior
remain unchanged; the token module now separates key/fallback lookup from stable Material defaults.

## Source-Backed Outcome

- The latest Material3 inventory showed `slider` as the highest fallback-density component token
  module: 36 fallback sites and 16 magic visual constants.
- The inline constants represented stable defaults for state-layer size, value indicator spacing,
  tick marks, stop indicators, track height, handle size/shape/width, and disabled/selected
  opacities.
- `tokens::slider_common` now owns those defaults.
- The inventory tooling now recognizes `slider_common` as token helper policy.

## Shipped

- Added `ecosystem/fret-ui-material3/src/tokens/slider_common.rs`.
- Migrated Slider default matrices out of `slider.rs`.
- Kept existing `slider_tokens::*` function names stable for the recipe.
- Updated `tools/parity-discovery/material3_token_inventory.py` to treat `slider_common.rs` as
  token helper policy.
- Generated a v1 inventory artifact for this lane.

## Evidence

- `ecosystem/fret-ui-material3/src/tokens/slider_common.rs`
- `ecosystem/fret-ui-material3/src/tokens/slider.rs`
- `tools/parity-discovery/material3_token_inventory.py`
- `docs/workstreams/material3-slider-token-defaults-v1/artifacts/material3_token_inventory_report_v1.json`

## Inventory Results

Slider module counts after the refactor:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `slider` | 36 | 0 |

Helper counts:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `slider_common` | 0 | 11 |

## Gates

- Passed: `cargo fmt --package fret-ui-material3 --check`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --lib tokens::slider_common`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test slider_state`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_headless_goldens material3_headless_slider_suite_goldens_v1`
- Passed: `cargo check -p fret-ui-material3 --features diagnostics --tests`
- Passed: `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- Passed: `python tools/parity-discovery/material3_token_inventory.py --generated-date 2026-05-31 --output docs/workstreams/material3-slider-token-defaults-v1/artifacts/material3_token_inventory_report_v1.json`
- Passed: `python -m json.tool docs/workstreams/material3-slider-token-defaults-v1/WORKSTREAM.json | Out-Null`
- Passed: `python -m json.tool docs/workstreams/material3-slider-token-defaults-v1/artifacts/material3_token_inventory_report_v1.json | Out-Null`
- Passed: `python tools/check_workstream_catalog.py`
- Passed: `python tools/check_layering.py`
- Passed: `git diff --check`

## Residual Follow-Ons

- `autocomplete`, `list`, `time_picker`, `text_field`, and `select` remain the highest fallback
  pressure modules after this lane.
- The next component token hardening slice should choose between a low-risk `list` token-default
  lane and a higher-impact field-family audit for `autocomplete`/`text_field`/`select`.
