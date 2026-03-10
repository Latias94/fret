# Producer-local Reason Code Naming Audit V1

Status: Draft

Tracking context:

- `docs/workstreams/diag-fearless-refactor-v2/M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/REASON_CODE_CONSUMER_ALIGNMENT_AUDIT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/TODO.md`

## Purpose

This note audits the remaining producer-local `reason_code`-adjacent naming after the shared
consumer contract has already been aligned.

The goal is not to reopen the summary/index reason-code contract. The goal is to decide whether the
remaining local fields such as `error_reason_code`, `reason_code_counts`, or `failure_kind`
represent a high-value cleanup target or an intentional local-report layer.

## Scope

Included:

- command-local report payloads in `diag_repeat`, `diag_suite`, and `diag_repro`,
- producer-local reason-code counters or wrapper fields,
- how those fields bridge into `RegressionSummaryV1` when a shared summary is emitted.

Excluded:

- `regression.summary.json` and `regression.index.json` machine fields,
- CLI/GUI/MCP aggregate consumers,
- `UiScriptResultV1.reason_code`,
- Layer A / artifact-path naming questions.

## Audit outcome

Main conclusion:

1. the remaining drift is real, but it is almost entirely producer-local,
2. current field names still describe meaningful local report roles rather than an accidental shared
   contract leak,
3. no immediate code rename is justified unless one of these producer reports is already being
   refactored for another reason,
4. if a cleanup is desired later, it should start with output payload field names, not internal
   local variables.

## Findings by surface

### `diag_repeat` — local repeat summary naming is acceptable

Key anchors:

- `crates/fret-diag/src/diag_repeat.rs:147`
- `crates/fret-diag/src/diag_repeat.rs:865`
- `crates/fret-diag/src/diag_repeat.rs:888`

Observations:

- local repeat summaries still emit `reason_code_counts` under `highlights`,
- local repeat failure wrappers still emit `error_reason_code`,
- when a fallback shared regression item is synthesized, `error_reason_code` is mapped into
  `source_reason_code` while the shared item keeps normalized
  `reason_code = "tooling.diag_repeat.failed"`.

Interpretation:

- `reason_code_counts` is a local aggregate histogram, not a contract drift,
- `error_reason_code` is a local wrapper around a deeper tooling failure code,
- the shared summary bridge is already doing the right normalization.

Recommendation:

- Defer renaming.
- If this surface is touched later, a rename should be strictly local and additive.

### `diag_suite` — `failure_kind` is intentionally more local than shared `reason_code`

Key anchors:

- `crates/fret-diag/src/diag_suite.rs:380`
- `crates/fret-diag/src/diag_suite.rs:389`
- `crates/fret-diag/src/diag_suite.rs:394`
- `crates/fret-diag/src/diag_suite.rs:1711`

Observations:

- suite-local payloads still emit `reason_code_counts`,
- suite-local failure wrappers still emit `error_reason_code`,
- suite-local payloads also emit `failure_kind`,
- when a shared fallback regression item is created, the stable shared code becomes
  `tooling.diag_suite.failed` while `failure_kind` is preserved in `source_reason_code`.

Interpretation:

- `failure_kind` is not obviously wrong; it acts as a lower-level producer discriminator,
- the suite layer is already separating stable cross-surface reason code from local source detail,
- renaming `failure_kind` purely for vocabulary symmetry would buy little right now.

Recommendation:

- Defer renaming.
- If revisited later, consider a focused local cleanup around whether `failure_kind` should become a
  suite-local alias of `source_reason_code`, but only if another suite-summary refactor is already
  justified.

### `diag_repro` — `overall_reason_code` / `error_reason_code` is a local repro summary seam

Key anchors:

- `crates/fret-diag/src/diag_repro.rs:284`
- `crates/fret-diag/src/diag_repro.rs:872`

Observations:

- repro execution tracks an internal `overall_reason_code`,
- the serialized repro summary exposes it as `error_reason_code`,
- this payload is a repro-local report rather than a shared orchestrated summary artifact.

Interpretation:

- the current naming is slightly inconsistent with the shared summary contract,
- but it is still clearly scoped to the repro summary layer,
- changing it now would be a local polish task, not a contract fix.

Recommendation:

- Defer renaming unless repro summary shaping is already being refactored.

### `diag_run` / `UiScriptResultV1` path is already aligned

Key anchor:

- `crates/fret-diag/src/diag_run.rs:101`

Observation:

- `diag_run` already forwards `UiScriptResultV1.reason_code` directly without inventing another
  producer-local wrapper vocabulary.

Recommendation:

- No action needed.

## Hazard assessment

The main hazard in touching these fields now is low-value churn:

- producer-local payload readers or docs may need additive compatibility for little user benefit,
- local summaries would become noisier to review without improving shared summary/GUI/MCP behavior,
- a rename wave could blur the difference between stable cross-surface `reason_code` and local
  wrapper/counter fields that actually serve different purposes.

## If cleanup is ever desired later

Most defensible order:

1. `diag_repeat` output keys only,
2. `diag_suite` output keys only,
3. `diag_repro` summary output keys only,
4. internal local variable renames last, if still worth doing.

Guardrails:

- do not change `RegressionSummaryV1` fields,
- do not change CLI/GUI/MCP consumer expectations,
- prefer additive aliases if any existing reader depends on the old local field names.

## Recommended next move

1. Treat producer-local reason-code naming as audited and intentionally deferred.
2. Do not start a code rename slice from this note alone.
3. Revisit only when one of the affected producer summaries is already being changed for a more
   substantive reason.
