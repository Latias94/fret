# Diag Monitor Topology Environment v1

Status: Closed historical design note
Last updated: 2026-04-20

Status note (2026-04-20): this document remains useful for the owner split and target environment
contract, but the shipped verdict now lives in `CLOSEOUT_AUDIT_2026-04-20.md` and
`WORKSTREAM.json`. Read the execution framing below as the lane-opening rationale that led to the
closed implementation.

Related:

- `WORKSTREAM.json`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CLOSEOUT_AUDIT_2026-04-20.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M3_MIXED_DPI_AUTOMATION_DECISION_2026-04-20.md`
- `docs/workstreams/diag-extensibility-and-capabilities-v1/determinism.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `crates/fret-runtime/src/runner_monitor_topology_diagnostics.rs`
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs`

This workstream is a narrow diagnostics follow-on to
`docking-multiwindow-imgui-parity`.

It does not reopen the broad docking parity lane.
It does not add mixed-DPI-only campaign gates.
It does not invent host-environment predicates inside `crates/fret-diag`.

## Why this lane exists

`M3_MIXED_DPI_AUTOMATION_DECISION_2026-04-20.md` froze an important boundary:

- `bundle.json.env.scale_factors_seen` remains run-observed per-window evidence,
- `mixed_dpi_signal_observed` remains drag evidence,
- and the repo should only revisit automation after diagnostics owns a real host monitor-topology
  environment source.

The runner already knew how to enumerate monitors for placement and clamping logic, but diagnostics
did not expose that inventory as a reusable, first-class environment fingerprint.

At the same time, `crates/fret-launch` cannot depend on `ecosystem/fret-bootstrap`, so the missing
bridge had to be a runner-owned runtime snapshot contract, not a diagnostics-private callback.

## Must-be-true outcomes

1. `crates/fret-runtime` owns a runner-published host monitor-topology snapshot contract that
   remains portable and data-only.
2. `crates/fret-launch` publishes desktop monitor inventory and virtual desktop bounds through that
   runtime contract without depending on diagnostics implementation details.
3. `ecosystem/fret-bootstrap` exports an additive `bundle.json.env.monitor_topology` field that is
   clearly separated from `scale_factors_seen`.
4. `scale_factors_seen` remains the last-known per-window scale-factor evidence captured during the
   run; it is not silently reclassified as host monitor inventory.
5. No mixed-DPI-only automated gate or campaign environment predicate lands in this lane.

## In scope

- Define the runner-owned monitor-topology snapshot structs/store in `crates/fret-runtime`.
- Publish the latest desktop monitor inventory from `crates/fret-launch`.
- Sync that runtime snapshot into diagnostics and export it in the bundle environment fingerprint.
- Update living diagnostics docs and source-gates so the owner split is explicit.

## Out of scope

- Adding a `requires mixed-dpi` manifest key.
- Adding campaign/script host-environment predicates.
- Reopening docking follow behavior, hover routing, or other multi-window interaction logic.
- Reclassifying `mixed_dpi_signal_observed` or `scale_factors_seen` as preflight capability data.

## Owner split

### `crates/fret-runtime`

Owns the portable runner-published monitor-topology snapshot contract.

### `crates/fret-launch`

Owns desktop monitor enumeration and publication of the latest host topology into the runtime
store.

### `ecosystem/fret-bootstrap`

Owns diagnostics-side caching and bundle export of the environment fingerprint.

### `crates/fret-diag`

Intentionally unchanged in this lane. Tooling still only gates on `requires_capabilities`; host
environment predicates remain a future follow-on.

## Target shipped state

After this lane, the repo should be able to say:

> diagnostics bundles can report the host monitor inventory explicitly, but mixed-DPI automation is
> still not allowed to pretend that bundle evidence equals a host-environment predicate.

That final shipped state is recorded in `CLOSEOUT_AUDIT_2026-04-20.md`.
