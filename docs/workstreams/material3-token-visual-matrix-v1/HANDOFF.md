# Material3 Token Visual Matrix v1 - Handoff

Status: Active
Last updated: 2026-05-30

## Current State

M3TVM-010 is complete: the workstream exists, the initial matrix covers all 39 M3PV2 components,
and the schema/source map defines the token visual dimensions, source precedence, owner layers,
and first family packets.

## Decisions

- This lane is about exhaustive token visual evidence, not reopening component behavior parity.
- Material Web v30 generated tokens are the token inventory source; Compose Material3 is the
  supporting toolkit source for state naming and component-specific expectations.
- Exact token correctness should be proven through fixture/scene assertions; goldens remain
  representative visual signatures.
- Unsupported future API breadth belongs in separate workstreams, not in this token matrix.

## Next Recommended Action

Start M3TVM-020: build the token inventory/fallback audit before touching component recipe code.
The report should identify duplicated fallback chains, magic visual constants, token modules, and
candidate typed outcome boundaries.

## Useful Gates

```powershell
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_visual_matrix_v1.json | Out-Null
python tools/check_workstream_catalog.py
cargo nextest run -p fret-ui-material3 --lib tokens::v30
```
