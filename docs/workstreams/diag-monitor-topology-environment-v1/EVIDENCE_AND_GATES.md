# Diag Monitor Topology Environment v1 — Evidence and Gates

Status: Closed

## Smallest current repro

Use this sequence to reopen the shipped environment-fingerprint evidence:

```bash
rg -n "RunnerMonitorTopologyDiagnosticsStore|RunnerMonitorTopologySnapshotV1|monitor_topology|scale_factors_seen" crates/fret-runtime/src/runner_monitor_topology_diagnostics.rs crates/fret-launch/src/runner/desktop/runner/app_handler.rs crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs docs/workstreams/diag-monitor-topology-environment-v1 docs/workstreams/docking-multiwindow-imgui-parity/M3_MIXED_DPI_AUTOMATION_DECISION_2026-04-20.md
cargo nextest run -p fret-bootstrap --lib env_fingerprint_exports_host_monitor_topology_without_reclassifying_scale_factors_seen --no-fail-fast
python tools/gate_imui_workstream_source.py
```

What this proves:

- the runner-owned runtime contract exists,
- diagnostics exports an explicit host monitor-topology field,
- `scale_factors_seen` remains separate,
- and the follow-on remains closed on a no-env-predicate verdict.

## Gate set

### Source and unit gates

```bash
cargo nextest run -p fret-runtime --lib --no-fail-fast
cargo nextest run -p fret-bootstrap --lib env_fingerprint_exports_host_monitor_topology_without_reclassifying_scale_factors_seen --no-fail-fast
python tools/gate_imui_workstream_source.py
python3 tools/check_layering.py
```

### Lane hygiene

```bash
python3 tools/check_workstream_catalog.py
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
python3 -m json.tool docs/workstreams/diag-monitor-topology-environment-v1/WORKSTREAM.json > /dev/null
git diff --check
```

## Current evidence after landing

- `crates/fret-runtime` now exposes `RunnerMonitorTopologyDiagnosticsStore` plus the v1 snapshot
  types.
- `crates/fret-launch` refreshes that store from the desktop runner's current monitor inventory.
- `ecosystem/fret-bootstrap` caches the runtime snapshot and exports
  `bundle.json.env.monitor_topology`.
- `bundle.json.env.scale_factors_seen` remains the last-known per-window summary seen during the
  run and is not reused as host monitor inventory.
- `crates/fret-diag` campaign/environment predicate support remains intentionally unchanged.

## Evidence anchors

- `docs/workstreams/diag-monitor-topology-environment-v1/DESIGN.md`
- `docs/workstreams/diag-monitor-topology-environment-v1/TODO.md`
- `docs/workstreams/diag-monitor-topology-environment-v1/MILESTONES.md`
- `docs/workstreams/diag-monitor-topology-environment-v1/CLOSEOUT_AUDIT_2026-04-20.md`
- `docs/workstreams/diag-monitor-topology-environment-v1/WORKSTREAM.json`
- `docs/workstreams/docking-multiwindow-imgui-parity/M3_MIXED_DPI_AUTOMATION_DECISION_2026-04-20.md`
- `docs/workstreams/diag-extensibility-and-capabilities-v1/determinism.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `crates/fret-runtime/src/runner_monitor_topology_diagnostics.rs`
- `crates/fret-runtime/src/lib.rs`
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
- `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- `tools/gate_imui_workstream_source.py`
