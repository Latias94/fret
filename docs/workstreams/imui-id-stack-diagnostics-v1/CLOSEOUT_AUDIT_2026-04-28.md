# Closeout Audit - 2026-04-28

Status: closed

## Verdict

`imui-id-stack-diagnostics-v1` is closed as a narrow follow-on to
`imui-label-identity-ergonomics-v1`.

The lane now owns a shipped, structured diagnostics path for the two identity footguns it set out
to make observable:

- duplicate keyed-list item key hashes,
- unkeyed list reorder drift.

The remaining ideas are broader than this lane and should start as narrower follow-ons instead of
reopening this folder.

## Landed Scope

- `fret-ui` records duplicate keyed-list and unkeyed reorder warnings as
  `IdentityDiagnosticsRecord` values.
- `fret-bootstrap` exports those warnings through schema2 bundle snapshots under
  `debug.element_runtime.identity_warnings`.
- `fret-imui` forwards diagnostics through its `diagnostics` feature and preserves author
  callsites through identity-bearing facade helpers.
- `ImUi::for_each_unkeyed` has an authoring proof for reorder warnings.
- `ImUi::for_each_keyed` delegates to `ElementContext::for_each_keyed` and has an authoring proof
  for duplicate-key warnings.
- `fretboard diag query identity-warnings` provides bounded triage over captured bundle snapshots.

## Final Gate Set

- `cargo nextest run -p fret-ui --features diagnostics identity_diagnostics --no-fail-fast`
- `cargo nextest run -p fret-imui --features diagnostics identity_diagnostics --no-fail-fast`
- `cargo check -p fret-imui --jobs 1`
- `cargo clippy -p fret-imui --all-targets --features diagnostics -- -D warnings`
- `cargo check -p fret-bootstrap --features ui-app-driver --jobs 1`
- `cargo nextest run -p fret-diag query_identity_warnings --no-fail-fast`
- `cargo check -p fret-diag --jobs 1`
- `cargo clippy -p fret-diag --all-targets -- -D warnings`
- `cargo fmt --package fret-ui --package fret-imui --package fret-bootstrap --check`
- `cargo fmt --package fret-imui --package fret-ui-kit --check`
- `cargo fmt --package fret-diag --check`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-id-stack-diagnostics-v1/WORKSTREAM.json`
- `git diff --check`

## Primary Evidence

- Runtime recorder/storage:
  - `crates/fret-ui/src/elements/cx.rs`
  - `crates/fret-ui/src/elements/runtime.rs`
  - `crates/fret-ui/src/declarative/tests/identity.rs`
- Diagnostics JSON bridge:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/element_runtime_diagnostics.rs`
- IMUI authoring proof:
  - `ecosystem/fret-imui/src/frontend.rs`
  - `ecosystem/fret-imui/src/tests/identity_diagnostics.rs`
- Diagnostics query:
  - `crates/fret-diag/src/commands/query.rs`
  - `crates/fret-diag/src/cli/contracts/commands/query.rs`
  - `crates/fret-diag/src/cli/cutover.rs`
  - `crates/fret-diag/src/cli/contracts/mod.rs`

## Deferred Follow-Ons

Start a new narrow lane for any of these:

- full interactive ID-stack browser,
- label-to-`test_id` inference,
- sortable/resizable table column identity,
- localization policy,
- public runtime identity APIs or exposed render-pass/evaluation tokens.

## Reopen Policy

Do not reopen this lane for new feature work. Use this folder as historical evidence for the
shipped structured identity diagnostics contract. If a regression affects the shipped behavior,
patch it against the existing evidence/gates; if the request widens the surface, start a follow-on.
