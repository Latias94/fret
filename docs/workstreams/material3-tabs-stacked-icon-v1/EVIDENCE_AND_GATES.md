# Material3 Tabs Stacked Icon v1 - Evidence and Gates

Status: Closed
Last updated: 2026-05-30

## Evidence Anchors

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Tab.kt`
- `ecosystem/fret-ui-material3/src/tabs.rs`
- `ecosystem/fret-ui-material3/src/tokens/tabs.rs`
- `ecosystem/fret-ui-material3/src/tokens/v30.rs`
- `ecosystem/fret-ui-material3/tests/tabs_state.rs`

## Gates

```powershell
cargo fmt -p fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state
cargo nextest run -p fret-ui-material3 --lib tokens::v30
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-tabs-stacked-icon-v1/WORKSTREAM.json | Out-Null
python tools/check_workstream_catalog.py
```

## Residual Risk

Exact Compose baseline offsets are not modeled yet. The current regression gate covers the visible
contract available in Fret today: large row height, vertical placement, icon sizing, and indicator
geometry.
