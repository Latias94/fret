# Diag Environment Predicate Contract v1 - Milestones

Status: Active

## M0: Baseline taxonomy freeze

Exit criteria:

- The repo explicitly records the current environment-lane split:
  - per-window reactive UI environment,
  - renderer/resource-loading provenance,
  - diagnostics-run environment fingerprint.
- The lane locks the rule that `requires_capabilities` remains capabilities-only.
- The lane locks the no-erased-runtime-family verdict for these surfaces.

Primary evidence:

- `docs/workstreams/diag-environment-predicate-contract-v1/BASELINE_AUDIT_2026-04-20.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/DESIGN.md`

Status:

- Completed on 2026-04-20.

## M4: Transport-session query foundation

Exit criteria:

- Existing DevTools sessions can publish admitted environment sources via an explicit request/ack
  surface rather than static hello/session descriptors.
- The first transport-session query publishes `host.monitor_topology` truthfully as
  `preflight_transport_session` for attach-to-existing-session flows.
- `crates/fret-diag` can acquire that session-published catalog without touching manifest grammar.
- The repo keeps launch-time filesystem publication and transport-session query as parallel
  acquisition lanes rather than collapsing them into capabilities.

Primary evidence:

- `docs/workstreams/diag-environment-predicate-contract-v1/M4_TRANSPORT_SESSION_ENVIRONMENT_SOURCE_QUERY_FOUNDATION_2026-04-20.md`
- `crates/fret-diag-protocol/src/lib.rs`
- `crates/fret-diag/src/devtools.rs`
- `crates/fret-diag/src/lib.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/ui_diagnostics_devtools_ws.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics_ws_bridge.rs`

Status:

- Completed on 2026-04-20.

## M5: Requires-environment host-monitor-topology admission

Exit criteria:

- Campaign manifests support a separate `requires_environment` field.
- The first shipped grammar stays source-scoped and source-specific.
- The first admitted source/predicate pair is:
  - `source_id: "host.monitor_topology"`
  - `predicate.kind: "host_monitor_topology"`
- The first shipped thresholds are:
  - `monitor_count_ge`
  - `distinct_scale_factor_count_ge`
- Campaign admission can resolve that requirement through:
  - existing filesystem publication,
  - preflight transport/session query,
  - or a launch-time probe.
- Unsatisfied requirements emit deterministic skip artifacts separate from capability artifacts.

Primary evidence:

- `docs/workstreams/diag-environment-predicate-contract-v1/M5_REQUIRES_ENVIRONMENT_HOST_MONITOR_TOPOLOGY_ADMISSION_2026-04-20.md`
- `crates/fret-diag/src/registry/campaigns.rs`
- `crates/fret-diag/src/diag_campaign.rs`
- `tools/diag-campaigns/README.md`

Status:

- Completed on 2026-04-20.

## M1: Diagnostics predicate owner split

Exit criteria:

- The repo names `crates/fret-diag` as the owner for any future orchestration predicate contract.
- The repo records the admission rule for when a lower-level environment source may participate in
  preflight.
- Living diagnostics docs point at this lane instead of implying that debug snapshots are
  preflight-ready.

Primary evidence:

- `docs/workstreams/diag-environment-predicate-contract-v1/DESIGN.md`
- `docs/workstreams/diag-extensibility-and-capabilities-v1/determinism.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/M1_FIRST_SOURCE_AND_TIMING_DECISION_2026-04-20.md`

Status:

- Completed on 2026-04-20.

## M2: First additive implementation slice

Exit criteria:

- One concrete source qualifies for predicate-capable use under the admission rules.
- The repo has a separate environment-source provenance/timing path that does not overload
  `capabilities.json`.
- The repo chooses the smallest additive manifest/summary contract for that source.
- Diagnostics preflight can emit deterministic provenance for the new decision.

Primary evidence:

- `docs/workstreams/diag-environment-predicate-contract-v1/M2_ENVIRONMENT_SOURCE_PROVENANCE_AND_AVAILABILITY_CONTRACT_2026-04-20.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/M2_ENVIRONMENT_SOURCE_CATALOG_FOUNDATION_2026-04-20.md`
- `crates/fret-diag-protocol/src/lib.rs`
- `crates/fret-diag/src/lib.rs`
- `crates/fret-diag/src/registry/campaigns.rs`

Status:

- Completed on 2026-04-20 as the catalog/provenance foundation.

## M3: Launch-time publication and campaign provenance

Exit criteria:

- Diagnostics runtime publishes `environment.sources.json` at the diagnostics `out_dir` root.
- `host.monitor_topology` gains a source-local payload file,
  `environment.source.host.monitor_topology.json`, when the runner exposes that inventory.
- `host.monitor_topology` is reclassified from the earlier bundle-only fallback to truthful
  `launch_time` availability.
- Campaign summary/result/aggregate artifacts expose:
  - `environment_sources_path`
  - `environment_source_catalog_provenance`
  - `environment_sources`
- Campaign preflight still stays capabilities-only and does not freeze manifest syntax.

Primary evidence:

- `docs/workstreams/diag-environment-predicate-contract-v1/M3_HOST_MONITOR_TOPOLOGY_LAUNCH_TIME_PUBLICATION_AND_CAMPAIGN_PROVENANCE_2026-04-20.md`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/fs_triggers.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`
- `crates/fret-diag/src/lib.rs`
- `crates/fret-diag/src/diag_campaign.rs`
- `docs/ui-diagnostics-and-scripted-tests.md`

Status:

- Completed on 2026-04-20.
