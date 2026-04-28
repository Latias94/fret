# ImUi Identity Browser Visual Gate v1 Closeout Audit - 2026-04-28

Status: closed

## Verdict

This lane shipped the first deterministic visual/structure smoke gate for the offline identity
browser HTML artifact and should stay closed.

The shipped gate is:

```text
fretboard-dev diag query identity-warnings <bundle-or-dir> \
  --html-out target/identity.html \
  --html-check-out target/check.identity_browser_html.json
```

## Shipped Scope

- Stable `data-testid` anchors for shell, group list, content, summary, filter, table, rows, and row
  JSON details.
- `check.identity_browser_html` JSON smoke report.
- CLI contract/cutover wiring for `--html-check-out`.
- Query fixture coverage for writing the smoke check.

## Gates

Passed:

- `cargo nextest run -p fret-diag identity_browser_html --no-fail-fast`
- `cargo nextest run -p fret-diag query_identity_warnings --no-fail-fast`
- `cargo check -p fret-diag --jobs 1`

## Follow-On Boundaries

Start narrow follow-ons instead of reopening this lane for:

- browser-driven screenshot checks,
- dashboard integration,
- live connected devtools identity panels,
- label-to-`test_id` inference,
- localization policy,
- sortable/resizable table column identity,
- public runtime identity APIs.
