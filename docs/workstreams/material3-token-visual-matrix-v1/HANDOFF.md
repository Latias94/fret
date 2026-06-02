# Material3 Token Visual Matrix v1 - Handoff

Status: Closed
Last updated: 2026-05-30

## Current State

M3TVM-010 through M3TVM-090 are complete. The workstream is closed, the matrix covers all 39 M3PV2
components, the generated inventory report maps all 38 component token modules to matrix rows, and
the fixture-driven token visual harness covers Button, the full field-family packet,
Checkbox/Radio/Switch/Slider/SegmentedButton/IconButton, the chip family, the navigation/app
chrome family, and the overlay/surface/data-display family. M3TVM-080 also split shared token
helpers (`shape`, `typography`) out of the component-module inventory and consolidated repeated
shape/typography fallback code into those helper surfaces.

## Decisions

- This lane is about exhaustive token visual evidence, not reopening component behavior parity.
- Material Web v30 generated tokens are the token inventory source; Compose Material3 is the
  supporting toolkit source for state naming and component-specific expectations.
- Exact token correctness should be proven through fixture/scene assertions; goldens remain
  representative visual signatures.
- Unsupported future API breadth belongs in separate workstreams, not in this token matrix.

## Next Recommended Action

No next action is required for this lane. Future work should open a new workstream only when it
targets API breadth outside this matrix, such as secondary tabs, richer dropdown/menu policy, or a
separate fallback-hardening pass beyond the fixture-backed consolidation done here.

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
