# Themes

Shared theme and preset JSON assets that are loaded directly from the workspace root.

This directory currently serves two roles:

- app/demo-facing theme bundles such as `fret-default-dark.json`, `godot-default-dark.json`, and
  `hardhacker-dark.json`
- shared preset data such as `node-graph-presets.v1.json`

Keep assets here when they are:

- consumed by multiple crates, demos, or tooling scripts,
- intentionally workspace-level rather than owned by a single crate,
- useful as stable reference inputs for parity or token-coverage checks.

Do not move crate-owned design-system assets here:

- shadcn registry/base-color theme assets belong under `ecosystem/fret-ui-shadcn/assets/...`
- crate-specific fixtures should stay next to the owning crate unless they are intentionally shared
  across the workspace

Current known consumers:

- `apps/fret-examples/src/gizmo3d_demo.rs`
- `ecosystem/fret-node/src/ui/presets.rs`
- `tools/check_theme_token_coverage.py`
- `tools/check_shadcn_theme_coverage.py`
