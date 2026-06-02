# Material3 Tabs API Breadth v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-30

## Smallest Current Repro

The current repro is the focused Tabs diagnostics test target:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state
```

It should prove:

- tablist/tab semantics still export correctly,
- primary fixed tabs keep the 24 px content-sized active indicator,
- secondary fixed tabs use a full-tab-width indicator,
- primary and secondary scrollable tabs keep 52 px edge padding and 90 px minimum tab width.

## Gate Set

### Workstream State

```powershell
python -m json.tool docs/workstreams/material3-tabs-api-breadth-v1/WORKSTREAM.json | Out-Null
python tools/check_workstream_catalog.py
```

### Material3 Tabs Slice

```powershell
cargo fmt -p fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state
cargo nextest run -p fret-ui-material3 --lib tokens::v30
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

### Optional Token Fixture Gate

Use this only if the lane adds new fixture rows:

```powershell
cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures
```

## Evidence Anchors

- `docs/workstreams/material3-tabs-api-breadth-v1/DESIGN.md`
- `docs/workstreams/material3-tabs-api-breadth-v1/TODO.md`
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TabRow.kt`
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/PrimaryNavigationTabTokens.kt`
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SecondaryNavigationTabTokens.kt`
- `ecosystem/fret-ui-material3/src/tabs.rs`
- `ecosystem/fret-ui-material3/src/tokens/tabs.rs`
- `ecosystem/fret-ui-material3/src/tokens/v30.rs`
- `ecosystem/fret-ui-material3/tests/tabs_state.rs`

## Fresh Evidence Log

- 2026-05-30: Opened the lane from the closed token visual matrix residual note. Source audit found
  that secondary tabs require Compose-backed API and token alias work because the repo's Material Web
  v30 generated tokens only include primary navigation tabs.
- 2026-05-30: Completed M3TAB-020.
  - `cargo fmt -p fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state`
  - `cargo nextest run -p fret-ui-material3 --lib tokens::v30`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures`
  - `python -m json.tool docs/workstreams/material3-tabs-api-breadth-v1/WORKSTREAM.json | Out-Null`
  - `python tools/check_workstream_catalog.py`
- 2026-05-30: Closed M3TAB-090.
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state`
  - `cargo nextest run -p fret-ui-material3 --lib tokens::v30`
  - `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures`
  - `python -m json.tool docs/workstreams/material3-tabs-api-breadth-v1/WORKSTREAM.json | Out-Null`
  - `python tools/check_workstream_catalog.py`
