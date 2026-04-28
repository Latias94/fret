# ImUi ID Stack Browser v1 - Evidence and Gates

Status: active
Last updated: 2026-04-28

## Smallest Repro

- `cargo nextest run -p fret-diag identity_browser --no-fail-fast`

## Current Evidence

- `docs/workstreams/imui-id-stack-diagnostics-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/imui-id-stack-diagnostics-v1/WORKSTREAM.json`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/workstreams/imui-id-stack-browser-v1/M1_SOURCE_MODEL_2026-04-28.md`
- `docs/workstreams/imui-id-stack-browser-v1/M2_BROWSER_QUERY_2026-04-28.md`
- `crates/fret-diag/src/identity_browser.rs`
- `crates/fret-diag/src/commands/query.rs`
- `crates/fret-diag/src/cli/contracts/commands/query.rs`
- `crates/fret-diag/src/cli/contracts/mod.rs`
- `crates/fret-diag/src/cli/cutover.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- `crates/fret-ui/src/elements/cx.rs`
- `ecosystem/fret-imui/src/frontend.rs`

## Initial Gate Set

- `cargo nextest run -p fret-diag identity_browser --no-fail-fast`
- `cargo nextest run -p fret-diag query_identity_warnings --no-fail-fast`
- `cargo check -p fret-diag --jobs 1`
- `cargo fmt --package fret-diag --check`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-id-stack-browser-v1/WORKSTREAM.json`
- `git diff --check`

## Future Gates

- Focused `fret-diag` tests for browser model grouping/filtering.
- CLI contract/cutover tests if a new command or public flags land.
- Dashboard or HTML fixture checks if the first interactive surface is visual rather than query-only.

## Non-Gates

- No public render-pass or evaluation-token API.
- No label-to-`test_id` inference.
- No sortable/resizable table column identity.
- No mandatory live devtools transport in the first slice.
