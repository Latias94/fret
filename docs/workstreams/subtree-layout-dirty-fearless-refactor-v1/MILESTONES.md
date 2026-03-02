# Milestones: Subtree Layout Dirty Aggregation (v1)

## M0 — Baseline + evidence

Done criteria:

- Capture the motivating bug in a deterministic repro description:
  - “Switch docs Preview → Code near bottom; cannot scroll further; clipped content.”
- Identify current mitigations/workarounds (if any) and their costs.
- Add a minimal unit test demonstrating the failure mode (if not already present).

## M1 — Mechanism: aggregation data + bookkeeping

Done criteria:

- Add `subtree_layout_dirty_count: u32` (or equivalent) to `UiTree` node storage.
- Update the counter on:
  - `invalidation.layout` transitions in invalidation marking
  - `invalidation.layout` clearing after layout
- Add debug-only validation (assertions or a diagnostic) to catch counter drift early.
- Land behind a runtime flag (or staged rollout) so we can compare behavior and measure overhead
  before removing any existing workarounds.

## M2 — Consumer: scroll extent cache correctness

Done criteria:

- Replace scroll’s “deep scan / forced invalidation” style workaround with the O(1) aggregation
  query.
- Ensure the motivating unit test passes without ad-hoc subtree scans.
- Add a UI Gallery diag script that covers the original behavior.

## M3 — View-cache + perf hardening (optional)

Done criteria:

- Decide propagation strategy:
  - eager-to-root, or
  - eager-to-cache-root + deferred upward propagation
- Add lightweight perf telemetry:
  - count aggregation updates per frame
  - max parent-walk length
  - number of deferred propagations
- Validate no regression on UI Gallery navigation / scroll perf workstreams.

## Current status (implementation)

- M1: implemented (counter + bookkeeping + validation + telemetry).
- M2: implemented “no deep scan” scroll edge guardrail using the O(1) subtree query.
  - Generic layout does **not** use subtree-dirty to force ancestor relayouts; contained cache roots
    remain owned by the contained relayout pass.
