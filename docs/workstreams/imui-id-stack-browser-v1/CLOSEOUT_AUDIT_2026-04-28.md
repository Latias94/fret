# ImUi ID Stack Browser v1 Closeout Audit - 2026-04-28

Status: closed

## Verdict

This lane shipped the first browser-ready diagnostics surface for captured IMUI/runtime identity
warnings and should now stay closed.

The final surface is intentionally post-run and bundle-oriented:

- source model: `crates/fret-diag/src/identity_browser.rs`,
- public query entry point: `diag query identity-warnings --browser`,
- automation output: opt-in `summary` and `groups` fields with existing `--json` / `--out` support,
- compatibility: default `identity-warnings` row output is unchanged when `--browser` is absent.

## Shipped Scope

- M1 mapped schema2 `debug.element_runtime.identity_warnings` into owned browser rows.
- M1 proved duplicate-key and unkeyed-reorder fixtures.
- M2 exposed grouped query output over warning kind, window, frame id, source file, list id, key
  hash, and element path.
- M2 kept live devtools, `test_id` inference, localization, table column identity, and public
  runtime identity APIs out of scope.

## M3 Decision

The first review workflow is the browser-ready query sidecar, not a dashboard or live devtools
panel.

Reasoning:

- the source data is captured in deterministic schema2 bundles,
- `--browser --json --out` gives future dashboard/HTML work a stable input shape,
- keeping this lane post-run avoids coupling identity debugging to live transport readiness,
- richer interaction can be added later without reopening the source model contract.

## Gates

Passed:

- `cargo nextest run -p fret-diag identity_browser --no-fail-fast`
- `cargo nextest run -p fret-diag query_identity_warnings --no-fail-fast`
- `cargo check -p fret-diag --jobs 1`
- `cargo fmt --package fret-diag --check`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-id-stack-browser-v1/WORKSTREAM.json`
- `git diff --check`

## Follow-On Boundaries

Start narrow follow-ons instead of reopening this lane for:

- live connected devtools identity panel,
- dashboard or HTML identity browser UI,
- label-to-`test_id` inference,
- localization policy,
- sortable/resizable table column identity,
- public runtime identity APIs.
