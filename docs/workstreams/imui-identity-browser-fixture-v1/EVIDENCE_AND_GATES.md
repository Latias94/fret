# ImUi Identity Browser Fixture v1 - Evidence and Gates

Status: closed
Last updated: 2026-04-28

## Smallest Repro

```text
cargo nextest run -p fret-diag query_identity_warnings --no-fail-fast
```

## Manual Sample Commands

```text
fretboard-dev diag query identity-warnings crates/fret-diag/tests/fixtures/identity_warnings/bundle.schema2.json --browser --json
fretboard-dev diag query identity-warnings crates/fret-diag/tests/fixtures/identity_warnings/bundle.schema2.json --html-out target/identity.html --html-check-out target/check.identity_browser_html.json
```

## Evidence

- `docs/workstreams/imui-identity-browser-visual-gate-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/imui-identity-browser-visual-gate-v1/WORKSTREAM.json`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `crates/fret-diag/tests/fixtures/identity_warnings/bundle.schema2.json`
- `crates/fret-diag/src/commands/query.rs`
- `docs/workstreams/imui-identity-browser-fixture-v1/CLOSEOUT_AUDIT_2026-04-28.md`

## Gates

- `cargo nextest run -p fret-diag query_identity_warnings --no-fail-fast`
- `cargo nextest run -p fret-diag identity_browser_html --no-fail-fast`
- `cargo check -p fret-diag --jobs 1`
- `cargo fmt --package fret-diag --check`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-identity-browser-fixture-v1/WORKSTREAM.json`
- `git diff --check`

## Non-Gates

- No browser screenshot harness in this lane.
- No dashboard integration.
- No live devtools transport.
