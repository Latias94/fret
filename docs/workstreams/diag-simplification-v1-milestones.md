Status: Active (workstream tracker)

# Diag simplification v1 - Milestones

## M0: Baseline documented

Exit criteria:

- [x] A written capability/behavior matrix for filesystem vs WS transports.
- [x] A small set of nextest gates in place for protocol/tooling.
- [x] A documented policy for `reason_code` and `capabilities` naming/backward-compat.

## M1: Transport abstraction (tooling)

Exit criteria:

- [x] `diag run` and `diag suite` use a single orchestration path with a pluggable transport.
  - Evidence: `crates/fret-diag/src/lib.rs` (`ConnectedToolingTransport`, `run_script_over_transport`)
- [x] A tooling-side transport trait exists with FS/WS implementations.
  - Evidence: `crates/fret-diag/src/transport/mod.rs` (`trait DiagTransport`, `ToolingDiagClient`), `crates/fret-diag/src/transport/fs.rs`, `crates/fret-diag/src/transport/ws.rs`
- [x] No behavior change for existing filesystem workflows (tooling-side contract audited).
  - Evidence: `crates/fret-diag/src/lib.rs` (baseline-race retouch + incremental writes), `crates/fret-diag/src/lib.rs` (tests: `run_script_over_transport_retouches_in_filesystem_mode_to_avoid_baseline_race`, `dump_bundle_over_transport_materializes_filesystem_latest_pointer`)

## M2: Artifact parity (WS -> local materialization)

Exit criteria:

- [x] In WS mode, tooling can materialize a **local** bundle directory containing `bundle.json` from `bundle.dumped`.
- [x] `diag pack`, `diag triage`, `diag lint` work from that local directory in both modes.
- [x] Artifact size is bounded and reported (bytes + clipped counts where applicable).

## M3: Exit parity

Exit criteria:

- [x] A transport-neutral exit request exists (filesystem touch + WS message).
- [x] In `--launch` mode, runs exit deterministically by default.
- [x] `--keep-open` preserves long-running/manual workflows.

## M4: Evidence improvements (bounded)

Exit criteria:

- [x] `script.result.json` includes a bounded per-run event log that helps explain failures without relying
  solely on "last N frames".
- [x] Reason codes remain stable; failures avoid silent timeouts when missing capabilities.

## M5: Artifact format v2 (manifest + chunks)

Exit criteria:

- A v2 artifact layout exists: `manifest.json` + chunks (snapshots/evidence/screenshots).
- Tooling can `pack/triage/lint` from either v1 (`bundle.json`) or v2 artifacts.
- WS mode can export large artifacts without relying on a single huge message (chunking policy exists).

## M6: Config consolidation (compat-first)

Exit criteria:

- A canonical config file exists and tooling can launch with it.
- Ambiguous env vars have explicit replacements (old names still supported).

## M7: Implementation split (reduce monolith risk)

Exit criteria:

- [x] Stale bundle checks are isolated from `crates/fret-diag/src/stats.rs` to reduce churn risk.
  - Evidence: `crates/fret-diag/src/stats/stale.rs`
- [ ] `crates/fret-diag/src/stats.rs` is decomposed into domain-focused modules (reduce merge conflicts; improve reviewability).
- [ ] `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` is split into a `ui_diagnostics/` module tree (service/export/script/inspect),
  while keeping existing public API and `use` paths stable for downstream crates.
