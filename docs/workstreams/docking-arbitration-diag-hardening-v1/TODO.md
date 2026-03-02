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

## Regression gates (candidate)

- A small “hardening smoke” suite for docking that includes:
  - tear-off creation,
  - hover routing across windows,
  - drag-back merge (tab restored into main),
  - no stuck drag sessions after release/cancel.
