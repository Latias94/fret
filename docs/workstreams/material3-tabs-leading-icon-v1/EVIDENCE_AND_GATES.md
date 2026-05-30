# Material3 Tabs Leading Icon v1 - Evidence and Gates

Status: Closed
Last updated: 2026-05-30

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/tabs.rs`
- `ecosystem/fret-ui-material3/src/tokens/tabs.rs`
- `ecosystem/fret-ui-material3/src/tokens/v30.rs`
- `ecosystem/fret-ui-material3/tests/tabs_state.rs`
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Tab.kt`
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/PrimaryNavigationTabTokens.kt`
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SecondaryNavigationTabTokens.kt`

## Gates

```powershell
cargo fmt -p fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state
cargo nextest run -p fret-ui-material3 --lib tokens::v30
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-tabs-leading-icon-v1/WORKSTREAM.json | Out-Null
python tools/check_workstream_catalog.py
```

## Residual Risk

The generic stacked icon + label tab is still missing. It should not be folded into
`TabItem::leading_icon` because it uses a different 72dp layout contract in Compose.
