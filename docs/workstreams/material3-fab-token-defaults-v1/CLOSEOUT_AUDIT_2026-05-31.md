# Material3 FAB Token Defaults v1 - Closeout Audit

Status: Closed
Date: 2026-05-31

## Summary

This fearless-refactor lane moved Material3 FAB and extended-FAB visual default matrices out of
`tokens::fab` resolver functions and into a private helper. The runtime token APIs and FAB recipe
behavior remain unchanged; the module boundary is now cleaner and easier to audit.

## Source-Backed Outcome

- The latest Material3 inventory showed `fab` as the highest magic-constant component token module:
  17 fallback sites and 36 magic visual constants.
- Most of that pressure came from stable FAB default matrices for size, shape, spacing, disabled
  opacity, and state-layer opacity.
- `tokens::fab_common` now owns those defaults.
- The inventory tooling now recognizes `fab_common` as token helper policy instead of treating it
  as an unmapped component token module.

## Shipped

- Added `ecosystem/fret-ui-material3/src/tokens/fab_common.rs`.
- Migrated FAB and extended-FAB default matrices out of `fab.rs`.
- Kept existing `fab_tokens::*` function names stable for the recipe.
- Updated `tools/parity-discovery/material3_token_inventory.py` to treat `fab_common.rs` as helper
  policy.
- Generated a v1 inventory artifact for this lane.

## Evidence

- `ecosystem/fret-ui-material3/src/tokens/fab_common.rs`
- `ecosystem/fret-ui-material3/src/tokens/fab.rs`
- `tools/parity-discovery/material3_token_inventory.py`
- `docs/workstreams/material3-fab-token-defaults-v1/artifacts/material3_token_inventory_report_v1.json`

## Inventory Results

FAB module counts after the refactor:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `fab` | 17 | 0 |

Helper counts:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `fab_common` | 0 | 29 |

## Gates

- Passed: `cargo fmt --package fret-ui-material3 --check`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --lib tokens::fab_common`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test fab_state`
- Passed: `cargo check -p fret-ui-material3 --features diagnostics --tests`
- Passed: `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- Passed: `python tools/parity-discovery/material3_token_inventory.py --generated-date 2026-05-31 --output docs/workstreams/material3-fab-token-defaults-v1/artifacts/material3_token_inventory_report_v1.json`
- Passed: `python -m json.tool docs/workstreams/material3-fab-token-defaults-v1/WORKSTREAM.json | Out-Null`
- Passed: `python -m json.tool docs/workstreams/material3-fab-token-defaults-v1/artifacts/material3_token_inventory_report_v1.json | Out-Null`
- Passed: `python tools/check_workstream_catalog.py`
- Passed: `python tools/check_layering.py`
- Passed: `git diff --check`

## Residual Follow-Ons

- `slider`, `autocomplete`, and `list` still carry high fallback density in the current inventory.
- A future inventory-tooling lane should rename the "shared token helper" bucket to a more accurate
  "token helper module" bucket now that it includes both shared family helpers and component default
  helper modules.
