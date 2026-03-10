# Reason Code Consumer Alignment Audit V1

Status: Draft

Tracking context:

- `docs/workstreams/diag-fearless-refactor-v2/M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/TODO.md`

## Purpose

This note audits whether the shared reason-code contract is actually consumed consistently across
the active orchestrated-output consumers.

The question is not whether every diagnostics payload in the repo already uses identical internal
field names. The question is whether the shared consumers for summary/index artifacts already rely
on canonical `reason_code` vocabulary instead of parsing ad hoc human text or legacy labels.

## Scope

Included:

- `regression.summary.json` and `regression.index.json`,
- CLI dashboard projections,
- DevTools GUI regression dashboard and drill-down,
- MCP regression dashboard projection,
- campaign aggregate report surfaces where a repo-level reason code is exposed.

Excluded:

- local producer reports such as `diag_repeat`, `diag_suite`, or `diag_repro`,
- internal helper counters like `reason_code_counts`,
- local error wrapper fields such as `error_reason_code`,
- raw JSON text-holder naming in DevTools / MCP modules.

## Audit outcome

Main conclusion:

1. the shared orchestrated-output consumers are already aligned on canonical `reason_code`
   vocabulary,
2. aggregate consumers already reuse `top_reason_codes` directly rather than parsing labels,
3. the remaining `reason_code`-adjacent drift is mostly producer-local report naming, not
   cross-surface consumer drift,
4. the TODO item for repo-level stable reason-code adoption can now be treated as consumer-aligned.

## Findings by surface

### Aligned — summary and index artifacts

Key anchors:

- `crates/fret-diag/src/regression_summary.rs:224`
- `crates/fret-diag/src/regression_summary.rs:226`
- `crates/fret-diag/src/diag_summarize.rs:378`
- `crates/fret-diag/src/diag_summarize.rs:455`

Observations:

- `RegressionItemSummaryV1` persists `reason_code` and `source_reason_code` as first-class fields,
- aggregate index generation counts `reason_code` directly into `by_reason_code`,
- `top_reason_codes` is emitted as a normalized machine-readable aggregate projection.

Recommendation:

- Treat summary/index reason-code vocabulary as aligned.

### Aligned — CLI dashboard projection

Key anchors:

- `crates/fret-diag/src/diag_dashboard.rs:217`
- `crates/fret-diag/src/diag_dashboard.rs:283`
- `crates/fret-diag/src/diag_dashboard.rs:315`

Observations:

- dashboard projection reads `top_reason_codes` directly,
- human output is rendered from normalized reason-code rows,
- no ad hoc parsing of free-form failure prose is required.

Recommendation:

- Treat CLI dashboard consumer adoption as aligned.

### Aligned — DevTools GUI consumer

Key anchors:

- `apps/fret-devtools/src/native.rs:5084`
- `apps/fret-devtools/src/native.rs:5113`
- `apps/fret-devtools/src/native.rs:5207`

Observations:

- GUI tests already assert that dashboard human summaries include `top reason codes`,
- drill-down fixtures use normalized `reason_code` values such as `capability.missing`,
- policy-skip evidence is consumed via structured fields, not by inferring from prose.

Recommendation:

- Treat DevTools GUI reason-code consumption as aligned.

### Aligned — MCP consumer

Key anchors:

- `apps/fret-devtools-mcp/src/native.rs:1600`
- `apps/fret-devtools-mcp/src/native.rs:1634`
- `apps/fret-devtools-mcp/src/native.rs:2336`

Observations:

- MCP result types expose `reason_code` and `top_reason_codes` explicitly,
- MCP projection reads normalized dashboard rows instead of re-deriving categories from labels,
- tests already pin policy-skip and top-reason-code behavior.

Recommendation:

- Treat MCP reason-code consumption as aligned.

### Aligned enough — campaign aggregate report surface

Key anchors:

- `crates/fret-diag/src/diag_campaign.rs:3258`
- `crates/fret-diag/src/diag_campaign.rs:3283`

Observations:

- campaign aggregate report JSON exposes a normalized `reason_code`,
- policy-skip aggregate reports normalize to `capability.missing`,
- top-level `status = ok|failed|skipped_policy` here is a small aggregate report surface rather than
  the canonical regression summary schema.

Recommendation:

- Treat the aggregate campaign reason-code surface as aligned enough for the current contract
  window.
- Do not reopen the summary reason-code contract just because this report uses a smaller aggregate
  shape.

### Deferred by design — producer-local report naming

Key anchors:

- `crates/fret-diag/src/diag_repeat.rs:865`
- `crates/fret-diag/src/diag_repeat.rs:888`
- `crates/fret-diag/src/diag_suite.rs:1662`
- `crates/fret-diag/src/diag_repro.rs:872`

Observations:

- fields such as `reason_code_counts` or `error_reason_code` still exist in local command payloads,
- these payloads are producer-specific reports rather than the shared orchestrated summary/index
  contract,
- renaming them now would be a separate producer-local cleanup, not required to make active
  consumers consistent.

Recommendation:

- Explicitly defer these fields.
- If touched later, review them as local report vocabulary cleanup rather than as a summary/MCP/GUI
  contract issue.

## Recommended next move

1. Mark the repo-level stable reason-code consumer adoption review as complete.
2. Keep future reason-code work focused on producer normalization families only if they buy real
   clarity.
3. Do not spend another consumer-facing rename pass on CLI / GUI / MCP for this topic unless a new
   consumer appears.
