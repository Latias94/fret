---
type: Work Progress
title: Cookbook utility window owner helper
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/cookbook-utility-window-owner
tags: fret,ui-framework,public-surface,cookbook,utility-window,raw-model,owner
---

# Summary

`apps/fret-cookbook/examples/utility_window_materials_windows.rs` now routes its command status
write through a demo-local `UtilityWindowMaterialsModelOwner`.

The example remains an advanced/manual window material interop cookbook surface because it owns
utility-window style effects, platform capability diagnostics, and manual retained tree seams. The
cleanup only removes the copyable pattern of updating `status` directly from `on_command(...)`.

# Decision

Keep the advanced interop classification. This example documents platform window material behavior,
not default app state authoring. The local owner helper keeps the remaining raw status model write
named and auditable without inventing a premature public utility-window materials API.

# Evidence

- Red proof before implementation:
  `cargo nextest run -p fret-cookbook utility_window_materials_model_writes_stay_behind_owner_helper --no-fail-fast`
  failed because `UtilityWindowMaterialsModelOwner` did not exist.
- The same test now requires `UtilityWindowMaterialsModelOwner::set_status(...)` and forbids the old
  direct `on_command(...)` status update.
- `cargo check -p fret-cookbook --all-targets` passes.
- `rg -n "models_mut\\(\\)\\.update" apps/fret-cookbook/examples -g '*.rs'` now returns no matches.

# Next

The cookbook advanced/manual examples still contain raw mechanisms by design, but direct raw writes
in copyable example bodies are now routed through local owner helpers. Further cleanup should focus
on retiring whole advanced classifications only when app-facing facade contracts exist.
