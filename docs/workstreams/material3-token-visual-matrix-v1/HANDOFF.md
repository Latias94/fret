# Material3 Token Visual Matrix v1 - Handoff

Status: Active
Last updated: 2026-05-30

## Current State

M3TVM-010 through M3TVM-030 are complete. The workstream exists, the matrix covers all 39 M3PV2
components, the generated inventory report maps all 38 component token modules to matrix rows, and
the fixture-driven token visual harness covers Button, TextField, Select, Autocomplete, and
ExposedDropdown. M3TVM-040A completed the field-overlay subset; the broader M3TVM-040 field-family
packet still needs SearchBar/SearchView, DatePicker, and TimePicker.

## Decisions

- This lane is about exhaustive token visual evidence, not reopening component behavior parity.
- Material Web v30 generated tokens are the token inventory source; Compose Material3 is the
  supporting toolkit source for state naming and component-specific expectations.
- Exact token correctness should be proven through fixture/scene assertions; goldens remain
  representative visual signatures.
- Unsupported future API breadth belongs in separate workstreams, not in this token matrix.

## Next Recommended Action

Continue M3TVM-040 with a new narrow subset for SearchBar/SearchView, DatePicker, and TimePicker,
or switch to M3TVM-050 for controls/chips. In either path, extend
`material3_token_visual_cases_v1.json` first, let fixture failures identify route/fallback bugs,
then update the matrix and evidence after the narrow gates pass.

## Useful Gates

```powershell
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_visual_matrix_v1.json | Out-Null
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json | Out-Null
python -m json.tool ecosystem/fret-ui-material3/tests/fixtures/material3_token_visual_cases_v1.json | Out-Null
python tools/parity-discovery/material3_token_inventory.py --generated-date 2026-05-30 --output docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json
python tools/check_workstream_catalog.py
cargo nextest run -p fret-ui-material3 --lib tokens::v30
cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures
```
