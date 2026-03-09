# Orchestrated Output Evidence Path Contract V1

Status: Draft

Tracking context:

- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/TODO.md`

## Purpose

This note defines the canonical evidence-path vocabulary for orchestrated outputs.

Scope is intentionally limited to:

- `regression.summary.json`,
- aggregate campaign/batch result payloads,
- consumer-facing path fields that CLI, GUI, CI, and MCP are expected to share.

It does **not** try to rename every path-like field in every diagnostics payload.

## Decision

Treat orchestrated output paths as three distinct surfaces:

1. item-level canonical evidence fields under `items[].evidence`,
2. top-level navigation fields under summary/result aggregate artifacts,
3. additive extras for lane- or workflow-specific details.

The practical rule is:

- if a path answers "what artifact should I open for this failing item?", it belongs in
  `items[].evidence`,
- if a path answers "what file should I open for this whole run/campaign/batch?", it belongs in
  aggregate summary/result navigation fields,
- if a path is useful only for one workflow family or preflight case, keep it additive under
  `evidence.extra` or aggregate-specific extra fields instead of widening the canonical root
  vocabulary casually.

## Canonical item evidence vocabulary

For orchestrated outputs, the canonical shared item evidence fields are:

- `bundle_artifact`
- `bundle_dir`
- `triage_artifact`
- `script_result`
- `share_artifact`
- `packed_report`
- `screenshots_manifest`

Projection-only additive item evidence fields may also exist:

- `perf_summary_json`
- `compare_json`

Why this split exists:

- the canonical set is expected to be meaningful across most lanes,
- the projection-only set is useful for perf/matrix drill-down but is not the baseline cross-surface
  vocabulary that every generic consumer should start from.

Key anchors:

- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md:265`
- `crates/fret-diag/src/regression_summary.rs:343`

## Aggregate navigation vocabulary

For orchestrated outputs, aggregate navigation fields are distinct from item evidence.

Recommended aggregate fields:

- `summary_path`
- `index_path`
- `share_manifest_path`
- `capabilities_check_path`

Interpretation:

- `summary_path` and `index_path` are the canonical first-open aggregate artifacts,
- `share_manifest_path` is an aggregate packaging/handoff path, not an item evidence path,
- `capabilities_check_path` is an aggregate policy/preflight path, not a generic item evidence
  field.

Key anchors:

- `crates/fret-diag/src/diag_campaign.rs:3065`
- `crates/fret-diag/src/diag_campaign.rs:3086`

## Where capability-preflight evidence belongs

Capability-preflight failures are special:

- they may not have a bundle artifact,
- they still need evidence,
- the evidence is policy-oriented rather than run-output-oriented.

Contract rule:

- keep the aggregate `capabilities_check_path` at the campaign/batch result level,
- for the corresponding failed item, store workflow-specific detail under `items[].evidence.extra`,
- do not promote `capabilities_check_path` into the canonical item evidence root set.

Key anchor:

- `crates/fret-diag/src/diag_campaign.rs:1605`

## Compatibility rules

### Canonical write-side names

Writers of orchestrated outputs should prefer:

- `triage_artifact` over `triage_json`,
- `script_result` over `script_result_json`,
- `share_artifact` over `ai_packet_dir`,
- `packed_report` over `pack_path`,
- `bundle_artifact` over ad hoc `bundle_json` item evidence names.

Key anchor:

- `crates/fret-diag/src/regression_summary.rs:357`

### Reader compatibility

Readers may continue to accept legacy aliases where the schema already declares them.

Current compatibility window:

- `triage_json` → `triage_artifact`
- `script_result_json` → `script_result`
- `ai_packet_dir` → `share_artifact`
- `pack_path` → `packed_report`

Contract rule:

- compatibility is additive,
- new docs and new emitters should use canonical names,
- readers should ignore unknown extra fields.

## Non-goals

- This note does not rename local `diag_repro`-specific fields such as
  `selected_bundle_artifact` or `packed_bundle_artifact`.
- This note does not rename Layer A run-manifest chunk-index fields.
- This note does not require every non-summary payload in `crates/fret-diag` to use identical field
  names immediately.
- This note does not define the compact pack file format itself.

## Fields that are intentionally out of the canonical orchestrated root

The following may remain valid, but they are not the canonical shared root vocabulary for
orchestrated outputs:

- `selected_bundle_artifact`
- `packed_bundle_artifact`
- `selected_bundle_json`
- `packed_bundle_json`
- `capabilities_check_path` at item root
- `share_manifest_path` at item root

Reason:

- some are local repro/report workflow fields,
- some are aggregate-only paths,
- some are compatibility aliases that should not spread into new summary emitters.

## Practical consequence for the workstream

This resolves the open "evidence bundle/artifact paths" TODO as a contract question:

- orchestrated outputs now have a documented canonical item evidence vocabulary,
- aggregate navigation paths are explicitly separated from item evidence,
- capability-preflight paths are explicitly kept additive,
- Layer B canonical `bundle_artifact` cleanup can continue without widening summary roots
  unnecessarily.
