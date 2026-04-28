# ImUi Identity Browser Fixture v1 - Milestones

Status: closed
Last updated: 2026-04-28

## M0 - Tracking

Result:

- Created as a narrow follow-on of the closed offline HTML visual gate lane.
- Scope is limited to one committed sample bundle and fixture-backed query coverage.

## M1 - Sample Bundle

Result:

- Added `crates/fret-diag/tests/fixtures/identity_warnings/bundle.schema2.json`.
- The identity warning query tests now reuse the committed sample instead of keeping a second bundle
  shape embedded in Rust test code.

## M2 - Query/HTML Gate

Result:

- Added fixture coverage for grouped browser JSON, offline HTML output, and the
  `check.identity_browser_html` smoke sidecar.

## M3 - Closeout

Result:

- Lane closed on 2026-04-28.
- Browser screenshot and dashboard gates remain separate follow-on candidates.
