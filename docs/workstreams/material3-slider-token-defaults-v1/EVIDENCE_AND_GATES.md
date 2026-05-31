# Material3 Slider Token Defaults v1 - Evidence And Gates

## Baseline

Source inventory:

- `docs/workstreams/material3-token-inventory-helper-taxonomy-v1/artifacts/material3_token_inventory_report_v1.json`

Baseline Slider counts from the latest inventory:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `slider` | 36 | 16 |

## Gates

```powershell
cargo fmt --package fret-ui-material3 --check
cargo nextest run -p fret-ui-material3 --features diagnostics --lib tokens::slider_common
cargo nextest run -p fret-ui-material3 --features diagnostics --test slider_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_headless_goldens material3_headless_slider_suite_goldens_v1
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python tools/parity-discovery/material3_token_inventory.py --generated-date 2026-05-31 --output docs/workstreams/material3-slider-token-defaults-v1/artifacts/material3_token_inventory_report_v1.json
python -m json.tool docs/workstreams/material3-slider-token-defaults-v1/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-slider-token-defaults-v1/artifacts/material3_token_inventory_report_v1.json | Out-Null
python tools/check_workstream_catalog.py
python tools/check_layering.py
git diff --check
```

## Evidence Log

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

## Final Inventory

Slider module counts after the refactor:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `slider` | 36 | 0 |

Helper counts:

| Module | Fallback sites | Magic visual constants |
| --- | ---: | ---: |
| `slider_common` | 0 | 11 |
