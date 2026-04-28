# ImUi Identity Browser Fixture v1

Status: closed
Last updated: 2026-04-28

## Why This Lane Exists

`imui-identity-browser-visual-gate-v1` closed with a deterministic HTML smoke check for captured
identity warning bundles. That leaves one usability gap: maintainers can verify the browser/query
surface only by running an app or reading tests that synthesize a bundle in code.

This follow-on adds a checked-in schema2 bundle sample that carries the same duplicate-key and
unkeyed-reorder warning shapes used by the query tests. The sample is intentionally tiny and
diagnostic-focused; it is not a full app capture corpus.

## Starting Assumptions

- Area: lane status
  - Assumption: `imui-identity-browser-visual-gate-v1` stays closed.
  - Evidence: `docs/workstreams/imui-identity-browser-visual-gate-v1/CLOSEOUT_AUDIT_2026-04-28.md`.
  - Confidence: Confident.
  - Consequence if wrong: fixture work would blur the structural smoke gate closeout.

- Area: fixture ownership
  - Assumption: the fixture belongs under `crates/fret-diag/tests/fixtures` because it is a
    `fret-diag` query input and should travel with the crate tests.
  - Evidence: `crates/fret-diag/src/commands/query.rs` owns `diag query identity-warnings`.
  - Confidence: Likely.
  - Consequence if wrong: move the fixture into a future shared diagnostics fixture catalog.

- Area: scope boundary
  - Assumption: this lane should not introduce a larger bundle corpus or screenshot harness.
  - Evidence: prior closeouts defer browser screenshots, dashboard integration, and live devtools.
  - Confidence: Confident.
  - Consequence if wrong: split the broader sample gallery into a separate follow-on.

## Target Surface

- Committed fixture: `crates/fret-diag/tests/fixtures/identity_warnings/bundle.schema2.json`.
- Unit/query tests read the committed fixture instead of duplicating the bundle shape in Rust code.
- Example commands can generate:
  - grouped JSON with `--browser --json`,
  - self-contained HTML with `--html-out`,
  - structural smoke JSON with `--html-check-out`.

## Non-Goals

- No browser screenshot dependency.
- No live connected devtools panel.
- No dashboard integration.
- No public runtime identity API.
- No localization or `test_id` inference policy.

## Exit Criteria

- The committed fixture covers duplicate keyed-list hashes and unkeyed reorder warnings.
- `query_identity_warnings` tests prove the fixture can drive grouped JSON and HTML/check sidecars.
- Workstream docs name the repro command and stay closed after the fixture lands.
