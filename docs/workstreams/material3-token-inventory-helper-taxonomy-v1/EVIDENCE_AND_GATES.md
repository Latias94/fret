# Material3 Token Inventory Helper Taxonomy v1 - Evidence And Gates

## Baseline

Source inventory:

- `docs/workstreams/material3-fab-token-defaults-v1/artifacts/material3_token_inventory_report_v1.json`

Baseline helper fields:

- `shared_token_helper_modules`
- `shared_token_helper_module_count`

## Gates

```powershell
python tools/parity-discovery/material3_token_inventory.py --generated-date 2026-05-31 --output docs/workstreams/material3-token-inventory-helper-taxonomy-v1/artifacts/material3_token_inventory_report_v1.json
python -m json.tool docs/workstreams/material3-token-inventory-helper-taxonomy-v1/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-token-inventory-helper-taxonomy-v1/artifacts/material3_token_inventory_report_v1.json | Out-Null
python tools/check_workstream_catalog.py
python tools/check_layering.py
git diff --check
```

## Evidence Log

- Passed: `python tools/parity-discovery/material3_token_inventory.py --generated-date 2026-05-31 --output docs/workstreams/material3-token-inventory-helper-taxonomy-v1/artifacts/material3_token_inventory_report_v1.json`
- Passed: `python -m json.tool docs/workstreams/material3-token-inventory-helper-taxonomy-v1/WORKSTREAM.json | Out-Null`
- Passed: `python -m json.tool docs/workstreams/material3-token-inventory-helper-taxonomy-v1/artifacts/material3_token_inventory_report_v1.json | Out-Null`
- Passed: `python tools/check_workstream_catalog.py`
- Passed: `python tools/check_layering.py`
- Passed: `git diff --check`

## Final Fields

New canonical helper fields:

- `summary.token_helper_module_count`
- `summary.token_helper_modules`
- `token_helper_modules`

Legacy aliases preserved:

- `summary.shared_token_helper_module_count`
- `summary.shared_token_helper_modules`
- `shared_token_helper_modules`

Final helper module count: 5.
