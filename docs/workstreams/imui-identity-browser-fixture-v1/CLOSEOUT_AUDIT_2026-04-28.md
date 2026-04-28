# ImUi Identity Browser Fixture v1 Closeout Audit - 2026-04-28

Status: closed

## Verdict

This lane shipped a committed sample bundle for identity browser review and should stay closed.

The sample is:

```text
crates/fret-diag/tests/fixtures/identity_warnings/bundle.schema2.json
```

It can drive the same offline review workflow as a captured app bundle:

```text
fretboard-dev diag query identity-warnings crates/fret-diag/tests/fixtures/identity_warnings/bundle.schema2.json --browser --json
fretboard-dev diag query identity-warnings crates/fret-diag/tests/fixtures/identity_warnings/bundle.schema2.json --html-out target/identity.html --html-check-out target/check.identity_browser_html.json
```

## Shipped Scope

- Stable schema2 bundle sample with:
  - one duplicated unkeyed reorder warning across two snapshots,
  - one duplicate keyed-list item key hash warning,
  - source location metadata for query filtering and grouping.
- Query tests reuse the committed sample bundle shape.
- Fixture-backed test covers grouped browser JSON and HTML/check sidecar generation.

## Gates

Passed:

- `cargo nextest run -p fret-diag query_identity_warnings --no-fail-fast`
- `cargo nextest run -p fret-diag identity_browser_html --no-fail-fast`
- `cargo check -p fret-diag --jobs 1`

## Follow-On Boundaries

Start narrow follow-ons instead of reopening this lane for:

- a larger diagnostics fixture corpus,
- browser-driven screenshot checks,
- dashboard integration,
- live connected devtools identity panels,
- label-to-`test_id` inference,
- localization policy,
- sortable/resizable table column identity,
- public runtime identity APIs.
