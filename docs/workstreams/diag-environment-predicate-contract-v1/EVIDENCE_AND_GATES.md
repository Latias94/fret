# Diag Environment Predicate Contract v1 - Evidence and Gates

Status: Active

## Smallest current repro

Use this sequence to reopen the taxonomy and owner split:

```bash
rg -n "ElementEnvironmentSnapshotV1|RendererFontEnvironmentSnapshot|UiDiagnosticsEnvFingerprintV1|requires_capabilities|diag-environment-predicate-contract-v1" ecosystem/fret-bootstrap/src/ui_diagnostics/element_runtime_diagnostics.rs ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs crates/fret-runtime/src/font_catalog.rs crates/fret-diag/src/registry/campaigns.rs docs/workstreams/diag-environment-predicate-contract-v1 docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md docs/workstreams/resource-loading-fearless-refactor-v1/README.md
cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_diag_environment_predicate_taxonomy --no-fail-fast
```

What this proves:

- the repo still has separate environment lanes,
- diagnostics orchestration still exposes `requires_capabilities` as the only shipped preflight
  contract,
- the new lane keeps the no-erased-runtime-family verdict explicit,
- and the next contract is now frozen as a separate `environment.sources.json` catalog plus
  explicit availability classes rather than a premature manifest key.

## Gate set

### Source-policy gate

```bash
cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_diag_environment_predicate_taxonomy --no-fail-fast
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
- `crates/fret-diag` still only honors `requires_capabilities` for orchestration preflight.
- `host.monitor_topology` is now the first candidate predicate source, but manifest syntax remains
  deferred because current campaign preflight runs before fresh launch-time publication.
- the next additive contract is a separate `environment.sources.json` catalog rather than
  `capabilities.json`.
- the availability classes are now explicit:
  `preflight_filesystem_sidecar`, `preflight_transport_session`, `launch_time`,
  and `post_run_only`.
- `post_run_only` environment sources are evidence-only and must not drive preflight.
- The repo now has an explicit lane that forbids collapsing those surfaces into one generic runtime
  abstraction without stronger evidence.

## Evidence anchors

- `docs/workstreams/diag-environment-predicate-contract-v1/WORKSTREAM.json`
- `docs/workstreams/diag-environment-predicate-contract-v1/DESIGN.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/BASELINE_AUDIT_2026-04-20.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/M1_FIRST_SOURCE_AND_TIMING_DECISION_2026-04-20.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/M2_ENVIRONMENT_SOURCE_PROVENANCE_AND_AVAILABILITY_CONTRACT_2026-04-20.md`
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
- `crates/fret-runtime/src/font_catalog.rs`
- `crates/fret-diag/src/registry/campaigns.rs`
- `apps/fret-examples/src/lib.rs`
