# Material3 Time Family Token Fallback v1 - Closeout Audit

Status: Closed
Date: 2026-05-31

## Summary

This fearless-refactor lane removed duplicated period-selector token fallback policy from the
Material3 time family. `time_picker` and `time_input` now keep their existing token API surface but
delegate shared shape, outline, selected container, label, and state-layer fallback logic to a
private token helper.

## Source-Backed Outcome

- The v2 Material3 inventory showed `time_picker` at 40 fallback sites / 17 magic visual constants
  and `time_input` at 26 fallback sites / 8 magic visual constants.
- Both modules repeated equivalent `period_selector_*` fallback logic.
- `tokens::time_period_common` now owns that repeated period-selector fallback policy.
- The inventory tooling now counts `time_period_common` as a shared token helper.

## Shipped

- Added `ecosystem/fret-ui-material3/src/tokens/time_period_common.rs`.
- Migrated period-selector token functions in `time_picker.rs` and `time_input.rs` to the helper.
- Preserved existing token function names consumed by recipes and visual fixtures.
- Updated `tools/parity-discovery/material3_token_inventory.py` to treat `time_period_common.rs` as
  shared token policy.
- Generated a v1 inventory artifact for this lane.

## Evidence

- `ecosystem/fret-ui-material3/src/tokens/time_period_common.rs`
- `ecosystem/fret-ui-material3/src/tokens/time_picker.rs`
- `ecosystem/fret-ui-material3/src/tokens/time_input.rs`
- `tools/parity-discovery/material3_token_inventory.py`
- `docs/workstreams/material3-time-family-token-fallback-v1/artifacts/material3_token_inventory_report_v1.json`

## Inventory Results

Time-family component module counts after the refactor:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `time_picker` | 30 | 14 |
| `time_input` | 16 | 5 |

Shared helper counts:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `time_period_common` | 10 | 7 |

## Gates

- Passed: `cargo fmt --package fret-ui-material3 --check`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --lib tokens::time_period_common`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_time_picker_interactions`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_headless_goldens material3_headless_time_picker_suite_goldens_v1`
- Passed: `cargo check -p fret-ui-material3 --features diagnostics --tests`
- Passed: `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- Passed: `python tools/parity-discovery/material3_token_inventory.py --generated-date 2026-05-31 --output docs/workstreams/material3-time-family-token-fallback-v1/artifacts/material3_token_inventory_report_v1.json`
- Passed: `python -m json.tool docs/workstreams/material3-time-family-token-fallback-v1/WORKSTREAM.json | Out-Null`
- Passed: `python -m json.tool docs/workstreams/material3-time-family-token-fallback-v1/artifacts/material3_token_inventory_report_v1.json | Out-Null`
- Passed: `python tools/check_workstream_catalog.py`
- Passed: `python tools/check_layering.py`
- Passed: `git diff --check`

## Residual Follow-Ons

- TimePicker still has non-period selector token fallback density around the clock dial and time
  selector. That should remain a separate slice because it does not have the same two-module helper
  shape as the period selector.
- `fab` remains the highest magic-constant module in the current inventory and is a good next
  single-module token-default naming slice.
