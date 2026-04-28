# ImUi ID Stack Diagnostics v1 - Evidence and Gates

Status: active
Last updated: 2026-04-28

## Smallest Repro

- `cargo nextest run -p fret-ui --features diagnostics identity_diagnostics --no-fail-fast`
- `cargo nextest run -p fret-imui --features diagnostics identity_diagnostics --no-fail-fast`
- `cargo nextest run -p fret-diag query_identity_warnings --no-fail-fast`

## Current Evidence

- `docs/workstreams/imui-label-identity-ergonomics-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/imui-table-header-label-policy-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `docs/workstreams/imui-id-stack-diagnostics-v1/DESIGN.md`
- `docs/workstreams/imui-id-stack-diagnostics-v1/M1_STRUCTURED_IDENTITY_DIAGNOSTICS_2026-04-28.md`
- `crates/fret-ui/src/elements/cx.rs`
- `crates/fret-ui/src/elements/runtime.rs`
- `crates/fret-ui/src/declarative/tests/identity.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- `ecosystem/fret-imui/src/frontend.rs`
- `ecosystem/fret-imui/src/tests/identity_diagnostics.rs`
- `docs/workstreams/imui-id-stack-diagnostics-v1/M2_IDENTITY_WARNINGS_QUERY_2026-04-28.md`
- `crates/fret-diag/src/commands/query.rs`
- `crates/fret-diag/src/cli/contracts/commands/query.rs`
- `crates/fret-diag/src/cli/cutover.rs`
- `crates/fret-diag/src/cli/contracts/mod.rs`
- `crates/fret-diag/src/diag_campaign.rs`

## Gate Set

- `cargo nextest run -p fret-ui --features diagnostics identity_diagnostics --no-fail-fast`
- `cargo nextest run -p fret-imui --features diagnostics identity_diagnostics --no-fail-fast`
- `cargo check -p fret-imui --jobs 1`
- `cargo check -p fret-bootstrap --features ui-app-driver --jobs 1`
- `cargo nextest run -p fret-diag query_identity_warnings --no-fail-fast`
- `cargo check -p fret-diag --jobs 1`
- `cargo clippy -p fret-diag --all-targets -- -D warnings`
- `cargo fmt --package fret-ui --package fret-imui --package fret-bootstrap --check`
- `cargo fmt --package fret-diag --check`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-id-stack-diagnostics-v1/WORKSTREAM.json`
- `git diff --check`

## Non-Gates

- No public `render_pass_id` or evaluation-token authoring API.
- No `test_id` inference.
- No table column identity contract.
- No full interactive ID-stack browser in this slice.
- No new diagnostics script capability; the query reads already captured schema2 bundle snapshots.
