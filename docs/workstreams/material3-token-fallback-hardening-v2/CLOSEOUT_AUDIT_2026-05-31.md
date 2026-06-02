# Material3 Token Fallback Hardening v2 - Closeout Audit

Status: Closed
Date: 2026-05-31

## Summary

This fearless-refactor lane removed a real source of token fallback duplication in the Material3
chip family. The repeated disabled-color, shape, height, icon-size, state-layer, elevation, and
outline fallback logic now lives in a shared chip token helper, while the four chip token modules
keep their existing public crate-local APIs.

## Source-Backed Outcome

- Chip token fallback logic was duplicated across `chip`, `filter_chip`, `input_chip`, and
  `suggestion_chip`.
- A new shared helper, `tokens::chip_common`, now owns the repeated fallback policy.
- The inventory tooling now recognizes `chip_common` as a shared token helper instead of an
  unmapped component token module.

## Shipped

- Added `ecosystem/fret-ui-material3/src/tokens/chip_common.rs`.
- Migrated the repeated chip-family fallback code into that helper.
- Kept the existing chip token module function names stable for recipes and fixtures.
- Updated `tools/parity-discovery/material3_token_inventory.py` to treat `chip_common.rs` as a
  shared helper and to skip `visual_fixture_model.rs` as test-only inventory noise.
- Generated a v2 inventory artifact for this lane.

## Evidence

- `ecosystem/fret-ui-material3/src/tokens/chip_common.rs`
- `ecosystem/fret-ui-material3/src/tokens/chip.rs`
- `ecosystem/fret-ui-material3/src/tokens/filter_chip.rs`
- `ecosystem/fret-ui-material3/src/tokens/input_chip.rs`
- `ecosystem/fret-ui-material3/src/tokens/suggestion_chip.rs`
- `tools/parity-discovery/material3_token_inventory.py`
- `docs/workstreams/material3-token-fallback-hardening-v2/artifacts/material3_token_inventory_report_v2.json`

## Inventory Results

Chip-family component module counts after the refactor:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `chip` | 4 | 0 |
| `filter_chip` | 5 | 0 |
| `input_chip` | 4 | 0 |
| `suggestion_chip` | 3 | 0 |

Shared helper counts:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `chip_common` | 11 | 12 |

## Gates

- Passed: `cargo fmt --package fret-ui-material3 --check`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --lib tokens::chip_common`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test chip_state`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_headless_goldens material3_headless_controls_suite_goldens_v1`
- Passed: `cargo check -p fret-ui-material3 --features diagnostics --tests`
- Passed: `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- Passed: `python tools/parity-discovery/material3_token_inventory.py --generated-date 2026-05-31 --output docs/workstreams/material3-token-fallback-hardening-v2/artifacts/material3_token_inventory_report_v2.json`
- Passed: `python -m json.tool docs/workstreams/material3-token-fallback-hardening-v2/WORKSTREAM.json | Out-Null`
- Passed: `python -m json.tool docs/workstreams/material3-token-fallback-hardening-v2/artifacts/material3_token_inventory_report_v2.json | Out-Null`
- Passed: `python tools/check_workstream_catalog.py`
- Passed: `python tools/check_layering.py`
- Passed: `git diff --check`

## Residual Follow-Ons

- If chip-family fallback pressure grows again, the next slice should split one more shared helper
  only if two or more chip modules need the same policy.
- Larger Material3 fallback hardening should continue from the highest-density remaining families
  in the inventory, not by widening this lane.
