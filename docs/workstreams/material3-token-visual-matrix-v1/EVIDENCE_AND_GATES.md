# Material3 Token Visual Matrix v1 - Evidence And Gates

Status: Active
Last updated: 2026-05-30

## Smallest Current Repro

The initial repro is schema/catalog validation:

```powershell
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_visual_matrix_v1.json | Out-Null
python tools/check_workstream_catalog.py
```

## Gate Set

### Workstream State

```powershell
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_visual_matrix_v1.json | Out-Null
python tools/check_workstream_catalog.py
git diff --check
```

### Material Token Inner Loop

Use narrow gates while the inventory and fixture harness are young:

```powershell
cargo nextest run -p fret-ui-material3 --lib tokens::v30
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

## Evidence Anchors

- `docs/workstreams/material3-token-visual-matrix-v1/DESIGN.md`
- `docs/workstreams/material3-token-visual-matrix-v1/TODO.md`
- `docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_visual_matrix_v1.json`
- `docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_visual_matrix_schema_v1.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_v2_closeout_audit.md`
- `ecosystem/fret-ui-material3/src/tokens/v30.rs`
- `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs`
- `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`
- `ecosystem/fret-ui-material3/tests`
- `goldens/material3-headless/v1`
- `repo-ref/compose-multiplatform-core`
- `repo-ref/base-ui`

## Fresh Evidence Log

- 2026-05-30: M3TVM-010 opened the token visual matrix lane.
  - Sources inspected: M3PV2 closeout audit, `tokens::v30` injection surface, generated
    `material_web_v30`, component token module inventory, and workstream catalog conventions.
  - Result: the initial matrix covers all 39 M3PV2 components and records matrix dimensions,
    source precedence, first family packet ownership, and initial `inventory_seeded` state.
  - Evidence note:
    `docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_visual_matrix_schema_v1.md`.
