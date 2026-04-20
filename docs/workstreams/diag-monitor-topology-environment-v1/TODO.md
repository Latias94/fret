# Diag Monitor Topology Environment v1 — TODO

Status: Closed

## Baseline / owner split

- [x] DMTE-001 Record why the docking parity lane stays focused on behavior proof while this narrow
  follow-on owns the diagnostics environment source.
- [x] DMTE-002 Freeze the rule that `scale_factors_seen` remains run-observed per-window evidence,
  not host monitor inventory.

## Runtime + runner contract

- [x] DMTE-010 Add a runner-owned monitor-topology snapshot contract in `crates/fret-runtime`.
- [x] DMTE-011 Publish the desktop monitor inventory from `crates/fret-launch` through that
  runtime contract.

## Diagnostics export

- [x] DMTE-020 Cache the runtime snapshot inside diagnostics without introducing a
  `fret-launch -> fret-bootstrap` dependency edge.
- [x] DMTE-021 Export additive `bundle.json.env.monitor_topology` data while keeping
  `scale_factors_seen` unchanged.
- [x] DMTE-022 Update living docs and source-gates to keep the boundary explicit.

## Deferred by design

- [x] DMTE-030 Explicitly defer host-environment predicates and mixed-DPI-only campaign gating to
  a future narrower follow-on.
