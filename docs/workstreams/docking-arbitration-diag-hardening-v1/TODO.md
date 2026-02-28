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

## Regression gates (candidate)

- A small “hardening smoke” suite for docking that includes:
  - tear-off creation,
  - hover routing across windows,
  - drag-back merge (tab restored into main),
  - no stuck drag sessions after release/cancel.

