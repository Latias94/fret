# ImUi ID Stack Browser v1 - Milestones

Status: active
Last updated: 2026-04-28

## M0 - Tracking

Exit criteria:

- the lane is documented as a narrow follow-on of the closed structured diagnostics lane,
- repo-level indexes point to the new workstream,
- and the first slice is limited to browsing/querying captured diagnostics, not widening runtime
  authoring identity APIs.

## M1 - Source Model Audit

Exit criteria:

- current identity warning bundle fields are mapped to a browser-ready row model,
- duplicate-key and unkeyed-reorder fixtures prove the model shape,
- and any capture-side gaps are listed with evidence instead of guessed.

Result:

- Landed in `crates/fret-diag/src/identity_browser.rs`.
- Existing fields are enough for first-use post-run browsing; no capture-side blocker found.
- Focused tests cover duplicate keyed-list rows, unkeyed reorder dedup, and timeline behavior.

## M2 - Browser Query Surface

Exit criteria:

- a bounded `fret-diag` surface exposes grouped identity diagnostics without raw bundle JSON,
- JSON output is stable enough for automation and future UI reuse,
- and focused tests cover filters/grouping/timeline behavior.

Result:

- Landed as `diag query identity-warnings --browser`.
- Default row query output remains compatible; browser summary/groups are opt-in.
- Contract, cutover, grouping, filter, dedup, and timeline tests cover the public surface.

## M3 - Interactive Experience

Exit criteria:

- a first interactive or browser-ready review workflow exists,
- maintainers can inspect warning groups and selected warning details efficiently,
- and live devtools work is either explicitly deferred or separately owned.

## M4 - Follow-on Decision

Exit criteria:

- final gates are recorded,
- `WORKSTREAM.json` reflects the current lane state,
- and broader ideas are split into narrower lanes instead of widening this one by default.

Current deferred candidates:

- live connected devtools identity panel,
- label-to-`test_id` inference,
- localization policy,
- sortable/resizable table column identity,
- public runtime identity APIs.
