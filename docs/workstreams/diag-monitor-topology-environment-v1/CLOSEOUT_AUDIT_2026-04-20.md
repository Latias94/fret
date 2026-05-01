# Closeout Audit - 2026-04-20

Status: Closed

## Result

This lane landed the missing host monitor-topology environment fingerprint that the docking mixed-DPI
automation decision explicitly required before any honest preflight work could happen.

The shipped outcome is:

- `crates/fret-runtime` owns a runner-published `RunnerMonitorTopologyDiagnosticsStore`,
- `crates/fret-launch` refreshes that store from desktop monitor inventory,
- `ecosystem/fret-bootstrap` exports `bundle.json.env.monitor_topology`,
- `bundle.json.env.scale_factors_seen` remains run-observed per-window evidence,
- and `crates/fret-diag` still does not grow environment predicates or mixed-DPI-only campaign
  selection in this lane.

## Why this is the correct closeout point

The problem this lane opened to solve was narrow:

1. create a real host monitor-topology environment source,
2. do it without violating the `fret-launch` / `fret-bootstrap` dependency boundary,
3. and avoid pretending that this alone already solves campaign preflight.

That problem is now closed.

What remains for the future is a different question:

- how tooling should express host-environment predicates,
- whether those predicates should be bundle-driven, launch-time, or capability-gated,
- and how mixed-DPI-only campaign selection should be declared honestly.

Those are not implementation leftovers here; they are a different contract lane.

## Evidence

- Runtime contract:
  - `crates/fret-runtime/src/runner_monitor_topology_diagnostics.rs`
  - `crates/fret-runtime/src/lib.rs`
- Desktop publication path:
  - `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
  - `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs`
- Diagnostics export:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- Tests / gates:
  - `cargo nextest run -p fret-runtime --lib --no-fail-fast`
  - `cargo nextest run -p fret-bootstrap --lib env_fingerprint_exports_host_monitor_topology_without_reclassifying_scale_factors_seen --no-fail-fast`
  - `python tools/gate_imui_workstream_source.py`
  - `python3 tools/check_layering.py`

## Next-action rule

Keep this lane closed.

If future pressure demands:

- host-environment predicates,
- campaign selection based on monitor topology,
- or a formal diagnostics capability around environment fingerprints,

start a different narrow follow-on from this closeout and
`docs/workstreams/docking-multiwindow-imgui-parity/M3_MIXED_DPI_AUTOMATION_DECISION_2026-04-20.md`
rather than reopening this folder.
