# M3 Host Monitor Topology Launch-Time Publication And Campaign Provenance - 2026-04-20

Status: active implementation note

Related:

- `WORKSTREAM.json`
- `DESIGN.md`
- `M2_ENVIRONMENT_SOURCE_PROVENANCE_AND_AVAILABILITY_CONTRACT_2026-04-20.md`
- `M2_ENVIRONMENT_SOURCE_CATALOG_FOUNDATION_2026-04-20.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `crates/fret-diag-protocol/src/lib.rs`
- `crates/fret-diag/src/lib.rs`
- `crates/fret-diag/src/diag_campaign.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/fs_triggers.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `apps/fret-examples/src/lib.rs`

## Purpose

This note records the next additive slice after the catalog/provenance foundation landed.

The goal of this slice was still narrow:

1. publish the first admitted source from diagnostics runtime rather than only from bundle export,
2. keep the publication owned by diagnostics runtime instead of `crates/fret-diag`,
3. surface the published catalog/provenance in campaign summary/result/aggregate artifacts,
4. and stop before manifest syntax or environment predicate execution.

## Landed result

### 1) Diagnostics runtime now publishes a launch-time filesystem catalog

`ecosystem/fret-bootstrap` now publishes `environment.sources.json` at the diagnostics `out_dir`
root.

When host monitor topology is available, the runtime also publishes
`environment.source.host.monitor_topology.json`.

That publication is owned by diagnostics runtime, not reconstructed later from bundle exports.

### 2) `host.monitor_topology` is now truthfully `launch_time`

The earlier M2 decision note kept an explicit fallback:

- if `host.monitor_topology` still only reached diagnostics through bundle export,
  its truthful availability would remain `post_run_only`.

That branch is now superseded for the runtime-published filesystem path.

Because diagnostics runtime publishes the source catalog and payload as part of launch-time
diagnostics setup, `host.monitor_topology` is now truthfully classified as `launch_time` for this
publication path.

### 3) Campaign artifacts now expose environment-source provenance explicitly

`crates/fret-diag` now resolves the published catalog plus source-local payload and surfaces:

- `environment_sources_path`
- `environment_source_catalog_provenance`
- `environment_sources`

These fields now flow through campaign summary/result/aggregate artifacts without reusing
`capability_source`.

### 4) The first source-local payload schema is now explicit

The first source-local payload file is:

- `environment.source.host.monitor_topology.json`

The first payload schema is:

- `HostMonitorTopologyEnvironmentPayloadV1`

This keeps the catalog small while allowing source-specific payload evolution outside the catalog
envelope itself.

## Important non-results

This slice intentionally did not:

- add `requires_environment`,
- execute environment predicates during campaign preflight,
- add environment predicates to campaign manifests,
- merge environment-source provenance into `capability_source`,
- or add a transport/session environment-source handshake.

Campaign preflight still only evaluates `requires_capabilities` before launch.

## Why this is the correct shape

### Runtime publication belongs to diagnostics runtime

`environment.sources.json` is a diagnostics runtime publication artifact, just like
`capabilities.json`.

That makes `ecosystem/fret-bootstrap` the correct owner for launch-time publication.

`crates/fret-diag` should consume and report that publication, not synthesize it after the fact.

### Launch-time is not the same as preflight

This slice upgrades `host.monitor_topology` from the old bundle-only fallback to truthful
`launch_time`.

It does not make the source preflight-ready.

Current campaign preflight still runs before launch, so a truthful preflight contract would still
need either:

- a persisted preflight sidecar,
- or a transport/session handshake.

### Campaign provenance is the minimal consumer surface

The correct first consumer was not a new manifest key.

The correct first consumer was campaign output plumbing, because that:

- proves the runtime publication can be found reliably,
- proves `fret-diag` can report source-specific payload paths separately from the catalog path,
- and keeps the new surface additive while syntax stays deferred.

## Evidence

- Runtime publication:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/fs_triggers.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs`
- Protocol additions:
  - `crates/fret-diag-protocol/src/lib.rs`
- Diagnostics consumer/provenance:
  - `crates/fret-diag/src/lib.rs`
  - `crates/fret-diag/src/diag_campaign.rs`
- Living doc + source-policy gate:
  - `docs/ui-diagnostics-and-scripted-tests.md`
  - `apps/fret-examples/src/lib.rs`

## Verification

- `cargo nextest run -p fret-bootstrap --features "ui-app-driver diagnostics" --lib refresh_environment_source_files_publishes_launch_time_monitor_topology_sidecars --no-fail-fast`
- `cargo nextest run -p fret-diag --lib environment_source --no-fail-fast`
- `cargo nextest run -p fret-diag --lib capability_preflight_writes_check_summary_and_result_artifacts --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_diag_environment_predicate_taxonomy --no-fail-fast`
- `git diff --check`
