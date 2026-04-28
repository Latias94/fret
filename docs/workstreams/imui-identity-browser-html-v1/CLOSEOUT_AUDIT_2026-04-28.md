# ImUi Identity Browser HTML v1 Closeout Audit - 2026-04-28

Status: closed

## Verdict

This lane shipped the offline HTML identity warning browser and should now stay closed.

The final surface is:

```text
fretboard-dev diag query identity-warnings <bundle-or-dir> --html-out target/identity.html
```

`--html-out` writes a self-contained HTML sidecar generated from the shared identity browser model.
It does not require live devtools, dashboard state, or raw JSON reading by the author.

## Shipped Scope

- Added `crates/fret-diag/src/identity_browser_html.rs`.
- Reused `crates/fret-diag/src/identity_browser.rs` instead of parsing bundle JSON again.
- Added safe escaping for bundle paths, source files, element paths, and JSON details.
- Added the `--html-out` query flag and migrated CLI contract/cutover wiring.
- Preserved existing JSON/human query behavior unless HTML output is explicitly requested.

## Gates

Passed:

- `cargo nextest run -p fret-diag identity_browser_html --no-fail-fast`
- `cargo nextest run -p fret-diag query_identity_warnings --no-fail-fast`
- `cargo check -p fret-diag --jobs 1`
- `cargo fmt --package fret-diag --check`

## Follow-On Boundaries

Start narrow follow-ons instead of reopening this lane for:

- live connected devtools identity panels,
- dashboard integration,
- richer visual regression checks for the HTML artifact,
- label-to-`test_id` inference,
- localization policy,
- sortable/resizable table column identity,
- public runtime identity APIs.
