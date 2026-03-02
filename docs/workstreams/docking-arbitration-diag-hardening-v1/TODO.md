# Docking arbitration diag hardening (v1) — TODO

Scope: stabilize the `docking-arbitration` diagnostics suite in `--launch` mode on desktop (native),
with special focus on multi-window tear-off + drag-back sequences.

## Immediate TODOs

- Make `docking-arbitration-demo-multiwindow-drag-tab-back-to-main` deterministic (no flake, no timeouts).
- Decide the contract for “scripted cross-window drag release”:
  - which subsystem owns `Drop` routing (runner vs in-app diagnostics injection),
  - which coordinate space is the source of truth (screen vs window-client),
  - what the required evidence gates are (bundle fields + assertions).
- Convert any remaining schema v1 docking scripts to schema v2.
- Reduce coupling to layout presets (prefer fingerprints / structural assertions where possible).

## Next TODOs (after diag no-hang)

- Fix chained tear-off + merge-back correctness:
  - `docking-arbitration-demo-multiwindow-chained-tearoff-two-tabs-merge` reaches the final assertion but the dock graph
    signature does not return to the pre-tearoff fingerprint (observed: missing `demo.viewport.left`).
- Ensure bundle-level evidence is sufficient without logs:
  - `debug.docking_interaction.dock_graph_signature` / `dock_graph_stats` should be present and up-to-date for all frames
    that matter to gates (either by recording every frame, or by an explicit “latest snapshot” contract).

## Hardening backlog (diag correctness + isolation)

- Runner: isolate scripted cursor overrides from physical mouse movement.
  - Goal: when diagnostics cursor override is active (and a script is running), physical mouse movement must not change
    the runner cursor position used for docking hover/drop routing.
  - Acceptance: a cross-window docking script remains deterministic even if the user moves the mouse during playback.
- Window counting: make `known_window_count_*` source-of-truth runner-owned.
  - Goal: diagnostics predicates for window count must reflect real open OS windows, not “windows that produced input”.
  - Acceptance: tear-off create/auto-close gates do not flake under occlusion / z-order churn.
- Cached `test_id` predicate evaluation: audit + evidence.
  - Goal: `exists/not_exists` by `test_id` may be evaluated off-window using cached `test_id_bounds` to avoid occlusion
    deadlocks, but must not introduce false positives from stale caches.
  - Work:
    - Define a max-age / freshness rule (e.g. require a recent snapshot for the target window).
    - Add bounded evidence in the script event log when cache-based evaluation is used (hit/miss/stale).
    - Add a focused repro script that intentionally occludes the target window and still progresses without hanging.

## Regression gates (candidate)

- A small “hardening smoke” suite for docking that includes:
  - tear-off creation,
  - hover routing across windows,
  - drag-back merge (tab restored into main),
  - no stuck drag sessions after release/cancel.
