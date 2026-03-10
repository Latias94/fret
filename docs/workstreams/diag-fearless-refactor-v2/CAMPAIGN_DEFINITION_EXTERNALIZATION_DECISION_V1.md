# Campaign Definition Externalization Decision V1

Status: Draft

Tracking context:

- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_CAMPAIGN_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/NEXT_DEVELOPMENT_PRIORITIES.md`
- `docs/workstreams/diag-fearless-refactor-v2/TODO.md`

## Purpose

This note decides what "move campaign definitions from the built-in Rust registry to external
manifests" means for the current workstream.

The repo no longer needs a speculative discussion about whether external manifests are possible.
They already exist. What still needs to be fixed is the ownership model:

- which surface is the canonical authoring source,
- which surface is fallback only,
- and which follow-up changes should remain deferred.

## Decision

For the current contract window:

1. repo-owned JSON manifests under `tools/diag-campaigns/` are the canonical authoring surface,
2. built-in Rust campaign definitions remain fallback/bootstrap data and test-friendly defaults,
3. manifest definitions override same-id built-in definitions,
4. no new campaign definition format is introduced yet,
5. full removal of built-in fallback definitions is explicitly deferred.

This means the externalization decision is no longer "move later". The decision is:

- external manifests are already the primary repo-owned definition layer,
- built-ins remain intentionally as a compatibility/bootstrap layer until a later cleanup window.

## Why this is the right decision

### 1. The repo already ships real external campaign manifests

Key anchors:

- `tools/diag-campaigns/README.md:1`
- `tools/diag-campaigns/ui-gallery-smoke.json`
- `tools/diag-campaigns/ui-gallery-correctness.json`
- `tools/diag-campaigns/docking-smoke.json`

Observations:

- the repo already contains multiple checked-in campaign manifest files,
- the authoring README already documents them as the consumed campaign source,
- maintainers already have a reviewable one-file-per-campaign workflow.

Interpretation:

- this is no longer a hypothetical externalization path,
- the missing piece was documenting the ownership rule explicitly.

### 2. The registry already implements builtin-plus-manifest overlay

Key anchors:

- `crates/fret-diag/src/registry/campaigns.rs:228`
- `crates/fret-diag/src/registry/campaigns.rs:254`
- `crates/fret-diag/src/registry/campaigns.rs:295`

Observations:

- `CampaignRegistry::load_from_workspace_root` starts from built-ins,
- it then loads manifests from `tools/diag-campaigns`,
- same-id manifest definitions replace built-in entries.

Interpretation:

- the runtime behavior already matches a staged externalization model,
- the correct next step is to name that model clearly rather than to redesign it.

### 3. Campaign command surfaces already consume the merged registry

Key anchor:

- `crates/fret-diag/src/diag_campaign.rs:571`

Observation:

- the campaign command path resolves against `CampaignRegistry::load_from_workspace_root`.

Interpretation:

- CLI behavior already treats repo manifests as first-class,
- users do not need a second externalization mechanism to start benefiting from manifest-driven
  authoring.

## Canonical source-of-truth rule

For repo-owned campaign definitions:

- author new campaigns in `tools/diag-campaigns/*.json`,
- edit existing repo campaigns there first,
- treat built-in Rust entries as fallback/defaults only.

Built-in entries should continue to exist only for:

- bootstrap behavior when no workspace manifests are available,
- narrow tests that intentionally avoid filesystem setup,
- compatibility during the current migration window.

## Format decision

For the current workstream, keep the external manifest format as:

- JSON files,
- `schema_version: 1`,
- `kind: "diag_campaign_manifest"`.

Do **not** introduce yet:

- TOML campaign manifests,
- generated registry code as the primary authoring source,
- a second higher-level DSL for campaign definitions.

Reason:

- the contract question today is ownership and precedence, not file-format expansion.

## Precedence and compatibility rules

### Precedence

- manifest definitions win over built-in same-id definitions,
- built-ins fill gaps only when a manifest is absent,
- command consumers should continue to treat the merged registry as the resolution surface.

### Compatibility

- manifest item authoring should prefer ordered `items`,
- legacy top-level `suites` / `scripts` remains accepted for now,
- removing compatibility fields is deferred until the manifest contract is more settled.

## What remains deferred

The following follow-ups are explicitly **not** part of this decision:

- removing built-in Rust fallback definitions,
- introducing TOML manifests,
- generating Rust registry code from manifest files,
- removing legacy manifest `suites` / `scripts` support,
- adding richer campaign authoring UX before current contracts need it.

These are separate cleanup or ergonomics decisions, not blockers for the current externalization
question.

## Practical consequence for the workstream

This closes the open TODO about campaign definition externalization:

- repo-owned external manifests are now the documented primary authoring surface,
- built-in Rust definitions are now explicitly classified as fallback/bootstrap only,
- no new format work is justified yet,
- later cleanup can focus on removing fallback or legacy compatibility only when that pays for
  itself.
