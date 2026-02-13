# Diag simplification v1 - Milestones

## M0: Baseline documented

Exit criteria:

- A written capability/behavior matrix for filesystem vs WS transports.
- A small set of nextest gates in place for protocol/tooling.
- A documented policy for `reason_code` and `capabilities` naming/backward-compat.

## M1: Transport abstraction (tooling)

Exit criteria:

- `diag run` and `diag suite` use a single orchestration path with a pluggable transport.
- No behavior change for existing filesystem workflows.

## M2: Artifact parity (WS -> local materialization)

Exit criteria:

- In WS mode, a `capture_bundle` produces a **local** bundle directory containing `bundle.json`.
- `diag pack`, `diag triage`, `diag lint` work from that local directory in both modes.
- Artifact size is bounded and reported (bytes + clipped counts where applicable).

## M3: Exit parity

Exit criteria:

- A transport-neutral exit request exists (filesystem touch + WS message).
- In `--launch` mode, runs exit deterministically by default.
- `--keep-open` preserves long-running/manual workflows.

## M4: Evidence improvements (bounded)

Exit criteria:

- `script.result.json` includes a bounded per-run event log that helps explain failures without relying
  solely on "last N frames".
- Reason codes remain stable; failures avoid silent timeouts when missing capabilities.

## M5: Artifact format v2 (manifest + chunks)

Exit criteria:

- A v2 artifact layout exists: `manifest.json` + chunks (snapshots/evidence/screenshots).
- Tooling can `pack/triage/lint` from either v1 (`bundle.json`) or v2 artifacts.
- WS mode can export large artifacts without relying on a single huge message (chunking policy exists).

## M6: Config consolidation (compat-first)

Exit criteria:

- A canonical config file exists and tooling can launch with it.
- Ambiguous env vars have explicit replacements (old names still supported).
