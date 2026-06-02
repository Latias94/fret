# Material3 Tabs Divider v1 - Evidence and Gates

Status: Closed
Last updated: 2026-05-30

## Evidence Anchors

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TabRow.kt`
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Divider.kt`
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/DividerTokens.kt`
- `ecosystem/fret-ui-material3/src/tabs.rs`
- `ecosystem/fret-ui-material3/src/tokens/tabs.rs`
- `ecosystem/fret-ui-material3/src/tokens/v30.rs`
- `ecosystem/fret-ui-material3/tests/tabs_state.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`

## Gates

```powershell
cargo fmt -p fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_tabs_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --lib tokens::v30
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-tabs-divider-v1/WORKSTREAM.json | Out-Null
python tools/check_workstream_catalog.py
```

## Residual Risk

There is no public divider slot or per-instance disable flag yet. Theme token override is the
current customization path.
