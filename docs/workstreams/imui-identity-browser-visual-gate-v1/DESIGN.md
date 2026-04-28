# ImUi Identity Browser Visual Gate v1

Status: closed
Last updated: 2026-04-28

## Why This Lane Exists

`imui-identity-browser-html-v1` shipped a self-contained offline HTML artifact for identity warning
triage. This follow-on adds the first deterministic gate for that artifact.

The gate is intentionally not a browser screenshot harness. It is a structural smoke gate that
verifies the HTML is nonblank and keeps stable visual/DOM anchors for the shell, summary, filter,
table, groups, rows, responsive CSS, and filter script.

## Starting Assumptions

- Area: lane status
  - Assumption: `imui-identity-browser-html-v1` stays closed.
  - Evidence: `docs/workstreams/imui-identity-browser-html-v1/CLOSEOUT_AUDIT_2026-04-28.md`.
  - Confidence: Confident.
  - Consequence if wrong: visual gate work would blur the shipped HTML sidecar lane.

- Area: gate type
  - Assumption: a Rust-level structural smoke gate should land before a browser-driven screenshot
    gate.
  - Evidence: the HTML renderer is pure Rust and already fixture-testable.
  - Confidence: Likely.
  - Consequence if wrong: start a screenshot harness follow-on with its own environment assumptions.

- Area: policy boundary
  - Assumption: this lane only gates presentation structure; it does not change identity semantics.
  - Evidence: `crates/fret-diag/src/identity_browser.rs` remains the source model owner.
  - Confidence: Confident.
  - Consequence if wrong: move the change into a source-model/query contract follow-on.

## Target Surface

- Stable `data-testid` anchors in the generated HTML.
- A `check.identity_browser_html` JSON smoke report.
- CLI flag: `diag query identity-warnings --html-check-out <path>`.
- Existing `--html-out`, `--browser`, `--json`, and `--out` behavior remains compatible.

## Non-Goals

- No Playwright or browser screenshot dependency.
- No dashboard integration.
- No live connected devtools panel.
- No public runtime identity API.

## Exit Criteria

- HTML renderer tests validate visual anchors.
- Query tests write the HTML check sidecar.
- CLI contract/cutover tests cover `--html-check-out`.
- Workstream is closed with gates and follow-on boundaries.
