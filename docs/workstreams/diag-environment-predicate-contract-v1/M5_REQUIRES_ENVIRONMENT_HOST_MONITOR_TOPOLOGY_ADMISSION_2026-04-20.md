# M5 Requires-Environment Host-Monitor-Topology Admission - 2026-04-20

Status: completed implementation note

Related:

- `WORKSTREAM.json`
- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `M1_FIRST_SOURCE_AND_TIMING_DECISION_2026-04-20.md`
- `M2_ENVIRONMENT_SOURCE_PROVENANCE_AND_AVAILABILITY_CONTRACT_2026-04-20.md`
- `M3_HOST_MONITOR_TOPOLOGY_LAUNCH_TIME_PUBLICATION_AND_CAMPAIGN_PROVENANCE_2026-04-20.md`
- `M4_TRANSPORT_SESSION_ENVIRONMENT_SOURCE_QUERY_FOUNDATION_2026-04-20.md`
- `crates/fret-diag/src/registry/campaigns.rs`
- `crates/fret-diag/src/diag_campaign.rs`
- `tools/diag-campaigns/README.md`

## Purpose

Land the smallest correct `requires_environment` authoring and execution slice now that both
truthful acquisition lanes for the first admitted source exist:

- existing/filesystem publication at `launch_time`,
- and transport/session publication at `preflight_transport_session`.

The goal of this slice is not to invent a general predicate language.
The goal is to let one real source, `host.monitor_topology`, drive an honest campaign skip/run
decision without smuggling environment facts into `requires_capabilities`.

## Landed contract

### 1) Campaign manifests now support `requires_environment`

The new manifest field is a list of source-scoped requirements:

- each entry names a `source_id`,
- each entry carries a source-specific `predicate`,
- and v1 admits only one source/predicate pair:
  - `source_id: "host.monitor_topology"`
  - `predicate.kind: "host_monitor_topology"`

The first predicate stays intentionally small:

- `monitor_count_ge`
- `distinct_scale_factor_count_ge`

At least one threshold must be present.

This keeps the first grammar narrow, reviewable, and anchored to a real mixed-DPI scheduling
problem rather than a speculative expression language.

### 2) Environment requirements use campaign admission, not capability preflight

`requires_capabilities` remains unchanged and remains capabilities-only.

`requires_environment` is evaluated by a separate campaign admission seam that can resolve the
first admitted source through the earliest truthful acquisition lane:

1. existing filesystem publication under the selected diagnostics `out_dir`,
2. preflight transport/session query for attached DevTools WS sessions,
3. launch-time probe for tool-launched filesystem runs.

This is the key contract correction for `host.monitor_topology`:

- it is still not a truthful pre-launch filesystem preflight input,
- but it is now a truthful campaign admission input when tooling can attach to an existing
  session or run a bounded launch-time probe.

### 3) Failure artifacts stay explicit and separate

When a requirement is unsatisfied, the campaign now writes:

- `check.environment.json`
- a campaign-level skipped-policy summary item named `environment_admission`
- `campaign.result.json` / aggregate fields that include:
  - `environment_check_path`
  - `environment_sources_path`
  - `environment_source_catalog_provenance`
  - `environment_sources`

This keeps environment admission evidence parallel to capability evidence instead of collapsing the
two into one generic policy artifact.

## Rejected in this slice

This slice intentionally did not:

- add a generic boolean expression language,
- add source-agnostic predicate schemas,
- overload `requires_capabilities`,
- promote `debug.environment` into an admitted predicate source,
- or allow `post_run_only` sources to drive campaign admission.

## Why this is the smallest correct slice

`host.monitor_topology` already proved that source presence alone is not enough.
Mixed-DPI scheduling needs source-local facts such as:

- monitor count,
- and distinct scale-factor count.

At the same time, the repo still does not have evidence for a wider shared predicate algebra.
Shipping a source-scoped and source-specific v1 therefore preserves room to evolve without
pretending the repo already knows how all future environment predicates should compose.

## Evidence

- Manifest grammar and normalization:
  `crates/fret-diag/src/registry/campaigns.rs`
- Campaign admission execution and skip artifacts:
  `crates/fret-diag/src/diag_campaign.rs`
- Authoring guidance:
  `tools/diag-campaigns/README.md`
