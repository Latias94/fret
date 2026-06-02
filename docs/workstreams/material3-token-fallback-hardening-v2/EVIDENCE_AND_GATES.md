# Material3 Token Fallback Hardening v2 - Evidence And Gates

## Baseline

Source inventory:

- `docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json`

Baseline chip-family counts from the v1 inventory:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `chip` | 35 | 12 |
| `filter_chip` | 46 | 15 |
| `input_chip` | 37 | 12 |
| `suggestion_chip` | 35 | 12 |

Combined chip-family baseline:

- Fallback sites: 153
- Magic visual constants: 51

## Gates

```powershell
cargo fmt --package fret-ui-material3 --check
cargo nextest run -p fret-ui-material3 --features diagnostics --lib tokens::chip_common
cargo nextest run -p fret-ui-material3 --features diagnostics --test chip_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_headless_goldens material3_headless_controls_suite_goldens_v1
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python tools/parity-discovery/material3_token_inventory.py --generated-date 2026-05-31 --output docs/workstreams/material3-token-fallback-hardening-v2/artifacts/material3_token_inventory_report_v2.json
python -m json.tool docs/workstreams/material3-token-fallback-hardening-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-token-fallback-hardening-v2/artifacts/material3_token_inventory_report_v2.json | Out-Null
python tools/check_workstream_catalog.py
python tools/check_layering.py
git diff --check
```

## Evidence Log

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

## Final Inventory

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
