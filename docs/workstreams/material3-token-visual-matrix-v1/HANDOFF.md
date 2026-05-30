# Material3 Token Visual Matrix v1 - Handoff

Status: Active
Last updated: 2026-05-30

## Current State

M3TVM-010 and M3TVM-020 are complete. The workstream exists, the matrix covers all 39 M3PV2
components, and the generated inventory report maps all 38 component token modules to matrix rows.
The report classifies generated Material Web keys, v30 manual writes, fallback chains, and magic
visual constants without refactoring component recipes.

## Decisions

- This lane is about exhaustive token visual evidence, not reopening component behavior parity.
- Material Web v30 generated tokens are the token inventory source; Compose Material3 is the
  supporting toolkit source for state naming and component-specific expectations.
- Exact token correctness should be proven through fixture/scene assertions; goldens remain
  representative visual signatures.
- Unsupported future API breadth belongs in separate workstreams, not in this token matrix.

## Next Recommended Action

Start M3TVM-030: build the fixture-driven visual-token harness. Use
`material3_token_inventory_report_v1.json` to prioritize the heaviest fallback modules first:
`text_field`, `autocomplete`, `slider`, `select`, `time_picker`, `list`, chips, `fab`, `checkbox`,
`time_input`, and `icon_button`.

## Useful Gates

```powershell
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_visual_matrix_v1.json | Out-Null
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json | Out-Null
python tools/parity-discovery/material3_token_inventory.py --generated-date 2026-05-30 --output docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json
python tools/check_workstream_catalog.py
cargo nextest run -p fret-ui-material3 --lib tokens::v30
```
