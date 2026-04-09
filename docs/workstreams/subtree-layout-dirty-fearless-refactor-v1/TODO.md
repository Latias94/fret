# TODO

This tracker is **workstream-local** and focuses on landing the aggregation mechanism safely.

Last updated: **2026-03-02**.

## Phase A — Baseline + repro

- [ ] Write a short repro note (UI Gallery page + section + expected behavior).
- [x] Ensure there is at least one unit test that fails without the fix and passes with it.
- [x] Add a `fretboard-dev diag` script for the UI Gallery repro (optional but preferred).
      - Example: `tools/diag-scripts/ui-gallery/typography/ui-gallery-typography-inline-code-tab-scroll-range.json`

## Phase B — Data structure + bookkeeping

- [x] Add `subtree_layout_dirty_count` to `UiTree` node storage.
- [x] Update the counter when `invalidation.layout` toggles during invalidation marking.
- [x] Update the counter when layout clears `invalidation.layout`.
- [x] Update the counter on structural attach/detach (children changes + subtree removal).
- [x] Add debug validation (assertions/diagnostic) to detect drift early.
- [x] Gate the mechanism behind a runtime flag so we can measure overhead and compare behavior.

## Phase C — Adopt in scroll (remove workarounds)

- [x] Replace scroll “edge correctness” logic to consult `subtree_layout_dirty(child_root)` instead
      of scanning descendants.
- [ ] Decide whether generic layout skip logic should consult subtree-dirty.
      - Current stance: do **not** force ancestor relayouts based on subtree-dirty (contained
        view-cache roots must remain owned by the contained relayout pass).

## Phase D — Propagation strategy + perf

- [ ] Measure update overhead in a representative app (UI Gallery + editor-like demo).
- [ ] If needed, implement deferred propagation across view-cache boundaries.
- [x] Add minimal telemetry to `UiTree` debug stats (counts per frame).
