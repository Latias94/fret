# Material3 FAB Token Defaults v1 - Evidence And Gates

## Baseline

Source inventory:

- `docs/workstreams/material3-time-family-token-fallback-v1/artifacts/material3_token_inventory_report_v1.json`

Baseline FAB counts from the latest inventory:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `fab` | 17 | 36 |

## Gates

```powershell
cargo fmt --package fret-ui-material3 --check
cargo nextest run -p fret-ui-material3 --features diagnostics --lib tokens::fab_common
cargo nextest run -p fret-ui-material3 --features diagnostics --test fab_state
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python tools/parity-discovery/material3_token_inventory.py --generated-date 2026-05-31 --output docs/workstreams/material3-fab-token-defaults-v1/artifacts/material3_token_inventory_report_v1.json
python -m json.tool docs/workstreams/material3-fab-token-defaults-v1/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-fab-token-defaults-v1/artifacts/material3_token_inventory_report_v1.json | Out-Null
python tools/check_workstream_catalog.py
python tools/check_layering.py
git diff --check
```

## Evidence Log

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

## Final Inventory

FAB module counts after the refactor:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `fab` | 17 | 0 |

Helper counts:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `fab_common` | 0 | 29 |
