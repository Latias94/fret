---
title: Campaign Metadata Execution Adoption Audit
status: draft
date: 2026-03-09
scope: diagnostics, campaign, metadata, execution, audit
---

# Campaign Metadata Execution Adoption Audit

Status: Draft

Tracking docs:

- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_CAMPAIGN_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_EXECUTION_ENTRY_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/M3_VOCABULARY_ADOPTION_AUDIT.md`
- `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_CAPABILITY_PREFLIGHT_V1.md`

## 0) Why this audit exists

The repo now persists two additional campaign metadata fields:

- `requires_capabilities`
- `flake_policy`

That closes the schema gap, but it does **not** answer the next question:

- should those fields remain descriptive metadata,
- or should they already affect campaign selection and execution behavior?

This note answers that question in a bounded way.

## 1) Audit scope

Included:

- campaign selection and execution behavior in `crates/fret-diag`,
- current CLI selectors and machine-readable run selection payloads,
- the first safe adoption order if metadata starts affecting execution.

Not in scope:

- changing status or reason-code vocabularies,
- adding new runtime capability probes,
- redesigning `diag suite`, `diag repeat`, or lower-level script execution contracts.

## 2) Current implementation state

### 2.1 The registry persists the new fields, but does not filter on them

Current `CampaignDefinition` now carries:

- `requires_capabilities`
- `flake_policy`

But `CampaignFilterOptions` and `CampaignDefinition::matches_filter(...)` still only evaluate:

- `lane`
- `tier`
- `tags`
- `platforms`

Evidence:

- `crates/fret-diag/src/registry/campaigns.rs`

Audit judgment:

- the new fields are currently catalog metadata, not selection metadata.

### 2.2 `diag campaign run` does not accept capability or flake-policy selectors

Current `CampaignRunOptions` only contains:

- explicit `campaign_ids`
- `CampaignFilterOptions`

And `parse_campaign_run_options(...)` only accepts:

- `--lane`
- `--tier`
- `--tag`
- `--platform`

Evidence:

- `crates/fret-diag/src/diag_campaign.rs`

Audit judgment:

- the command surface does not yet treat `requires_capabilities` or `flake_policy` as run
  selectors.

### 2.3 Execution does not branch on the new metadata

`execute_campaign_run_selection(...)` currently:

1. selects campaigns,
2. loops over them,
3. calls `execute_campaign(...)`,
4. optionally writes batch artifacts.

There is no campaign-level:

- capability preflight,
- retry-budget policy branch,
- flake-classification branch,
- policy-derived skip path.

Evidence:

- `crates/fret-diag/src/diag_campaign.rs`

Audit judgment:

- the new metadata does not yet influence execution scheduling or outcome classification.

### 2.4 Machine-readable run selection payloads also ignore the new metadata

`campaign_run_selection_json(...)` currently persists only:

- `campaign_ids`
- `filters`

It does not persist a resolved capability gate or retry policy decision for the run entry itself.

Evidence:

- `crates/fret-diag/src/diag_campaign.rs`

Audit judgment:

- there is no shared machine-readable execution contract yet for metadata-driven decisions.

### 2.5 The new metadata is currently visible only through descriptive surfaces

Today the fields are surfaced through:

- `campaign_to_json(...)`
- `diag campaign list`
- `diag campaign show`

That means they are inspectable and reviewable, but still passive.

Evidence:

- `crates/fret-diag/src/registry/campaigns.rs`
- `crates/fret-diag/src/diag_campaign.rs`

Audit judgment:

- this is the correct shape for the current slice.

## 3) What the current docs already imply

The workstream docs already point toward a staged adoption model:

- `requires_capabilities` exists to make environment needs machine-readable,
- `flake_policy` exists to describe retry/classification behavior,
- the first campaign slice should stay small and reuse existing `diag run` / `diag suite` /
  `diag summarize` flows,
- richer retry/classification behavior was explicitly deferred from the earliest landing.

This means the implementation is currently behind the metadata **behavior** idea, but not behind the
documented first-slice scope.

## 4) Audit conclusion

Current recommendation:

- keep `requires_capabilities` and `flake_policy` as **descriptive metadata only** for now,
- do **not** wire them into campaign execution yet,
- and do **not** add selectors such as `--require-capability` or `--flake-policy` until there is a
  concrete orchestration need.

Why this is the right stop point:

1. the schema is now stable enough for review and curation,
2. the execution layer does not yet have a dedicated preflight stage,
3. flake handling today still lives primarily in lower-level run/repeat/suite flows,
4. adding campaign-level behavior now would force a second contract decision about status and
   reason-code mapping before there is a clear consumer need.

## 5) When behavior adoption should start

Behavior adoption should start only when one of these becomes true:

1. campaign execution needs to refuse or skip runs based on environment capability mismatch,
2. CI wants campaign-level retry/classification semantics rather than only lower-level retry tools,
3. batch orchestration needs deterministic policy reporting for why one campaign ran differently
   from another.

Until then, metadata-only persistence is sufficient.

## 6) Recommended adoption order for future behavior

If the repo later wants execution semantics, the safest order is:

### Step 1. Capability preflight before item execution

Add one campaign-level preflight stage before `execute_campaign(...)` begins item execution.

That stage should:

- evaluate `requires_capabilities`,
- produce one explicit machine-readable decision,
- avoid silently dropping campaigns from the selected set.

Recommended outcome shape:

- either emit a synthetic campaign-step result with normalized `status`,
- or persist an explicit campaign-level decision field in `campaign.result.json`,
- but do not leave capability mismatch as human-only CLI text.

### Step 2. Reuse existing capability reason-code vocabulary

If capability mismatch becomes a campaign-level execution outcome:

- reuse the existing capability-missing reason-code family where possible,
- keep `reason_code` normalized,
- use `source_reason_code` only for lower-level producer detail when needed.

This avoids creating a second “campaign capability failure” taxonomy that drifts from existing
script/runtime checks.

### Step 3. Add only the smallest flake-policy behavior

If `flake_policy` becomes active, start with:

- `fail_fast`
- `retry_once`

Defer:

- `retry_three`
- `classify_only`
- quarantine policy automation

Reason:

- the first safe step is only a small retry-budget branch,
- not a full campaign-level flake engine.

### Step 4. Keep policy names separate from statuses

If `flake_policy` is consumed:

- policy names must remain retry/classification inputs,
- result statuses must remain normalized outputs such as:
  - `failed_flaky`
  - `failed_timeout`
  - `skipped_policy`
  - `quarantined`

The repo should not overload policy names into final result states.

## 7) Recommended non-goals for the next slice

The next slice should **not** do the following:

- add broad new campaign selectors,
- add parallel capability probing just for campaigns,
- make campaign metadata override lower-level script/suite evidence contracts,
- add another human-only retry vocabulary in CLI output.

## 8) What should happen next

Recommended next step:

- keep this execution adoption explicitly deferred in code,
- record the decision in workstream priority docs,
- and reopen it only when a concrete CI or batch-orchestration consumer needs campaign-level policy
  behavior.

If that need appears, the first implementation target should be:

- campaign capability preflight with explicit machine-readable outcome,
- not flake-policy automation.

That first implementation target is now also sketched in:

- `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_CAPABILITY_PREFLIGHT_V1.md`

## 9) Exit criteria for this audit

This audit has done its job when:

- contributors can see that the new metadata is intentionally passive today,
- no one assumes the current runner already honors `requires_capabilities` or `flake_policy`,
- and the next behavior adoption starts from a documented order rather than from an ad hoc CLI flag
  addition.
