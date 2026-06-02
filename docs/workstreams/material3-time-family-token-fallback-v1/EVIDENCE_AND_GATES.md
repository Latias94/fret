# Material3 Time Family Token Fallback v1 - Evidence And Gates

## Baseline

Source inventory:

- `docs/workstreams/material3-token-fallback-hardening-v2/artifacts/material3_token_inventory_report_v2.json`

Baseline time-family counts from the v2 inventory:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `time_picker` | 40 | 17 |
| `time_input` | 26 | 8 |

## Gates

```powershell
cargo fmt --package fret-ui-material3 --check
cargo nextest run -p fret-ui-material3 --features diagnostics --lib tokens::time_period_common
cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_time_picker_interactions
cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_headless_goldens material3_headless_time_picker_suite_goldens_v1
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python tools/parity-discovery/material3_token_inventory.py --generated-date 2026-05-31 --output docs/workstreams/material3-time-family-token-fallback-v1/artifacts/material3_token_inventory_report_v1.json
python -m json.tool docs/workstreams/material3-time-family-token-fallback-v1/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-time-family-token-fallback-v1/artifacts/material3_token_inventory_report_v1.json | Out-Null
python tools/check_workstream_catalog.py
python tools/check_layering.py
git diff --check
```

## Evidence Log

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

## Final Inventory

Time-family component module counts after the refactor:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `time_picker` | 30 | 14 |
| `time_input` | 16 | 5 |

Shared helper counts:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `time_period_common` | 10 | 7 |
