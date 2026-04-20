# Diag Environment Predicate Contract v1 - Evidence and Gates

Status: Active

## Smallest current repro

Use this sequence to reopen the taxonomy, acquisition lanes, shipped grammar, and campaign
admission seam:

```bash
rg -n "ElementEnvironmentSnapshotV1|RendererFontEnvironmentSnapshot|UiDiagnosticsEnvFingerprintV1|requires_capabilities|diag-environment-predicate-contract-v1" ecosystem/fret-bootstrap/src/ui_diagnostics/element_runtime_diagnostics.rs ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs crates/fret-runtime/src/font_catalog.rs crates/fret-diag/src/registry/campaigns.rs docs/workstreams/diag-environment-predicate-contract-v1 docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md docs/workstreams/resource-loading-fearless-refactor-v1/README.md
rg -n "refresh_environment_source_files|environment_source_catalog_provenance|environment_sources_path|HOST_MONITOR_TOPOLOGY_ENVIRONMENT_SOURCE_ID_V1|FILESYSTEM_HOST_MONITOR_TOPOLOGY_ENVIRONMENT_PAYLOAD_FILE_NAME_V1" ecosystem/fret-bootstrap/src/ui_diagnostics/fs_triggers.rs ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs crates/fret-diag/src/lib.rs crates/fret-diag/src/diag_campaign.rs crates/fret-diag-protocol/src/lib.rs docs/workstreams/diag-environment-predicate-contract-v1/M3_HOST_MONITOR_TOPOLOGY_LAUNCH_TIME_PUBLICATION_AND_CAMPAIGN_PROVENANCE_2026-04-20.md apps/fret-examples/src/lib.rs
rg -n "environment.sources.get|environment.sources.get_ack|devtools.environment_sources|PreflightTransportSession|read_transport_published_environment_sources|wait_for_environment_sources_get_ack" crates/fret-diag-protocol/src/lib.rs crates/fret-diag/src/devtools.rs crates/fret-diag/src/lib.rs ecosystem/fret-bootstrap/src/ui_diagnostics/ui_diagnostics_devtools_ws.rs ecosystem/fret-bootstrap/src/ui_diagnostics_ws_bridge.rs docs/workstreams/diag-environment-predicate-contract-v1/M4_TRANSPORT_SESSION_ENVIRONMENT_SOURCE_QUERY_FOUNDATION_2026-04-20.md apps/fret-examples/src/lib.rs
rg -n "requires_environment|HostMonitorTopologyPredicateDefinition|maybe_execute_campaign_environment_admission|check.environment.json|LaunchTimeProbe|distinct_scale_factor_count_ge" crates/fret-diag/src/registry/campaigns.rs crates/fret-diag/src/diag_campaign.rs tools/diag-campaigns/README.md docs/workstreams/diag-environment-predicate-contract-v1/M5_REQUIRES_ENVIRONMENT_HOST_MONITOR_TOPOLOGY_ADMISSION_2026-04-20.md apps/fret-examples/src/lib.rs
cargo nextest run -p fret-diag-protocol --lib environment_sources_get --no-fail-fast
cargo nextest run -p fret-bootstrap --features "ui-app-driver diagnostics-ws" --lib environment_sources_get --no-fail-fast
cargo nextest run -p fret-bootstrap --features "ui-app-driver diagnostics" --lib refresh_environment_source_files_publishes_launch_time_monitor_topology_sidecars --no-fail-fast
cargo nextest run -p fret-diag --lib environment_admission --no-fail-fast
cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_diag_environment_predicate_taxonomy --no-fail-fast
```

What this proves:

- the repo still has separate environment lanes,
- diagnostics orchestration now keeps `requires_capabilities` and `requires_environment` on
  separate policy lanes,
- the new lane keeps the no-erased-runtime-family verdict explicit,
- and the first shipped contract is now frozen as a separate `environment.sources.json` catalog
  plus a source-scoped `requires_environment` grammar rather than a premature generic snapshot or
  expression language.
- `host.monitor_topology` now has a launch-time filesystem publication lane rather than relying
  only on bundle export.
- existing DevTools sessions now have a separate `environment.sources.get` /
  `environment.sources.get_ack` publication lane.
- campaign summary/result/aggregate artifacts now expose environment-source provenance separately
  from capability provenance.
- campaign admission now writes `check.environment.json` when the first admitted environment
  requirement is unsatisfied.

## Gate set

### Source-policy gate

```bash
cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_diag_environment_predicate_taxonomy --no-fail-fast
```

### Environment-foundation gate

```bash
cargo nextest run -p fret-diag --lib environment_source --no-fail-fast
```

### Environment-protocol gate

```bash
cargo nextest run -p fret-diag-protocol --lib environment_sources_get --no-fail-fast
```

### Transport-session query gate

```bash
cargo nextest run -p fret-bootstrap --features "ui-app-driver diagnostics-ws" --lib environment_sources_get --no-fail-fast
```

### Launch-time publication gate

```bash
cargo nextest run -p fret-bootstrap --features "ui-app-driver diagnostics" --lib refresh_environment_source_files_publishes_launch_time_monitor_topology_sidecars --no-fail-fast
```

### Environment-admission gate

```bash
cargo nextest run -p fret-diag --lib environment_admission --no-fail-fast
```

### Campaign aggregate provenance gate

```bash
cargo nextest run -p fret-diag --lib capability_preflight_writes_check_summary_and_result_artifacts --no-fail-fast
```

### Lane hygiene

```bash
python3 tools/check_workstream_catalog.py
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
python3 -m json.tool docs/workstreams/diag-environment-predicate-contract-v1/WORKSTREAM.json > /dev/null
git diff --check
```

## Current evidence after opening

- `ElementEnvironmentSnapshotV1` remains the committed per-window environment/debug surface.
- `RendererFontEnvironmentSnapshot` remains the renderer/resource provenance surface.
- `UiDiagnosticsEnvFingerprintV1` remains the bundle/run-level diagnostics environment fingerprint.
- `crates/fret-diag` now supports a separate `requires_environment` manifest field.
- the first shipped `requires_environment` slice is source-scoped and source-specific.
- `host.monitor_topology` is the first admitted source id.
- `host_monitor_topology` is the first shipped predicate kind.
- `host.monitor_topology` is now the first candidate predicate source and the first launch-time
  published source in the filesystem catalog family.
- the next additive contract is a separate `environment.sources.json` catalog rather than
  `capabilities.json`.
- the repo now also has an additive session-published source catalog surface via
  `environment.sources.get` / `environment.sources.get_ack`.
- the availability classes are now explicit:
  `preflight_filesystem_sidecar`, `preflight_transport_session`, `launch_time`,
  and `post_run_only`.
- the runtime now publishes `environment.source.host.monitor_topology.json` when host monitor
  topology is available.
- the runtime now advertises `devtools.environment_sources` for DevTools WS sessions that support
  the explicit source query surface.
- campaign summary/result/aggregate artifacts now expose `environment_sources_path`,
  `environment_source_catalog_provenance`, and `environment_sources`.
- unsatisfied environment admission now writes `check.environment.json`.
- `launch_time` publication still does not make `host.monitor_topology` a truthful preflight input;
  the source is instead consumed through campaign admission.
- `post_run_only` environment sources are evidence-only and must not drive preflight.
- `fret-diag-protocol` now exposes the additive environment-source catalog types.
- `crates/fret-diag` now has a parallel filesystem loader/provenance seam for
  `environment.sources.json`.
- The repo now has an explicit lane that forbids collapsing those surfaces into one generic runtime
  abstraction without stronger evidence.

## Evidence anchors

- `docs/workstreams/diag-environment-predicate-contract-v1/WORKSTREAM.json`
- `docs/workstreams/diag-environment-predicate-contract-v1/DESIGN.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/BASELINE_AUDIT_2026-04-20.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/M1_FIRST_SOURCE_AND_TIMING_DECISION_2026-04-20.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/M2_ENVIRONMENT_SOURCE_PROVENANCE_AND_AVAILABILITY_CONTRACT_2026-04-20.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/M2_ENVIRONMENT_SOURCE_CATALOG_FOUNDATION_2026-04-20.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/M3_HOST_MONITOR_TOPOLOGY_LAUNCH_TIME_PUBLICATION_AND_CAMPAIGN_PROVENANCE_2026-04-20.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/M4_TRANSPORT_SESSION_ENVIRONMENT_SOURCE_QUERY_FOUNDATION_2026-04-20.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/M5_REQUIRES_ENVIRONMENT_HOST_MONITOR_TOPOLOGY_ADMISSION_2026-04-20.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/TODO.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/MILESTONES.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/diag-monitor-topology-environment-v1/CLOSEOUT_AUDIT_2026-04-20.md`
- `docs/adr/0189-ui-diagnostics-extensibility-and-capabilities-v1.md`
- `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
- `docs/workstreams/resource-loading-fearless-refactor-v1/README.md`
- `docs/workstreams/diag-extensibility-and-capabilities-v1/determinism.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/element_runtime_diagnostics.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/fs_triggers.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`
- `crates/fret-runtime/src/font_catalog.rs`
- `crates/fret-diag-protocol/src/lib.rs`
- `crates/fret-diag/src/lib.rs`
- `crates/fret-diag/src/devtools.rs`
- `crates/fret-diag/src/diag_campaign.rs`
- `crates/fret-diag/src/registry/campaigns.rs`
- `tools/diag-campaigns/README.md`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/ui_diagnostics_devtools_ws.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics_ws_bridge.rs`
- `apps/fret-examples/src/lib.rs`
