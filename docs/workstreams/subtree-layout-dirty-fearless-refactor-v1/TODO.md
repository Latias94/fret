# TODO

This tracker is **workstream-local** and focuses on landing the aggregation mechanism safely.

Last updated: **2026-03-01**.

## Phase A — Baseline + repro

- [ ] Write a short repro note (UI Gallery page + section + expected behavior).
- [ ] Ensure there is at least one unit test that fails without the fix and passes with it.
- [ ] Add a `fretboard diag` script for the UI Gallery repro (optional but preferred).

## Phase B — Data structure + bookkeeping

- [ ] Add `subtree_layout_dirty_count` to `UiTree` node storage.
- [ ] Update the counter when `invalidation.layout` toggles during invalidation marking.
- [ ] Update the counter when layout clears `invalidation.layout`.
- [ ] Add debug validation (assertions/diagnostic) to detect drift early.
- [ ] Gate the mechanism behind a runtime flag so we can measure overhead and compare behavior.

## Phase C — Adopt in scroll (remove workarounds)

- [ ] Replace scroll “edge correctness” logic to consult `subtree_layout_dirty(child_root)` instead
      of scanning descendants.
- [ ] Keep the existing behavior behind a debug flag for one iteration to compare outcomes.
- [ ] Remove the workaround once parity is proven and gated by tests.

## Phase D — Propagation strategy + perf

- [ ] Measure update overhead in a representative app (UI Gallery + editor-like demo).
- [ ] If needed, implement deferred propagation across view-cache boundaries.
- [ ] Add minimal telemetry to `UiTree` debug stats (counts per frame).
