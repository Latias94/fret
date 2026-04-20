# Diag Monitor Topology Environment v1 — Milestones

Status: Closed

## M0: Ownership and contract freeze

Exit criteria:

- The lane explicitly records that `docking-multiwindow-imgui-parity` stays focused on runner
  behavior proof, not diagnostics contract growth.
- The repo names a runner-owned runtime snapshot as the only acceptable bridge between
  `fret-launch` and diagnostics.
- The lane freezes the rule that `scale_factors_seen` remains run-observed evidence.

Primary evidence:

- `docs/workstreams/diag-monitor-topology-environment-v1/DESIGN.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M3_MIXED_DPI_AUTOMATION_DECISION_2026-04-20.md`

Status:

- Completed on 2026-04-20.

## M1: Runtime publication and bundle export

Exit criteria:

- `crates/fret-runtime` exposes a runner-owned host monitor-topology snapshot store.
- `crates/fret-launch` publishes the latest desktop monitor inventory into that store.
- `ecosystem/fret-bootstrap` exports additive `bundle.json.env.monitor_topology` data.
- Tests prove the new field exists without changing `scale_factors_seen` semantics.

Primary gates:

- `cargo nextest run -p fret-runtime --lib --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --lib env_fingerprint_exports_host_monitor_topology_without_reclassifying_scale_factors_seen --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p3_mixed_dpi_monitor_topology_follow_on --no-fail-fast`

Status:

- Completed on 2026-04-20.

## M2: Closeout boundary

Exit criteria:

- The lane closes with an explicit rule that future environment predicates or mixed-DPI-only
  campaign selection must start as a different narrow follow-on.
- Repo entry docs point at this lane as the shipped diagnostics follow-on for the M3 decision.

Primary evidence:

- `docs/workstreams/diag-monitor-topology-environment-v1/CLOSEOUT_AUDIT_2026-04-20.md`
- `docs/workstreams/diag-monitor-topology-environment-v1/WORKSTREAM.json`
- `docs/roadmap.md`
- `docs/workstreams/README.md`
- `docs/todo-tracker.md`

Status:

- Completed on 2026-04-20.
