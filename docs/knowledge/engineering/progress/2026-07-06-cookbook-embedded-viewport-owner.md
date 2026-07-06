---
type: Work Progress
title: Cookbook embedded viewport owner helper
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/cookbook-embedded-viewport-owner
tags: fret,ui-framework,public-surface,cookbook,embedded-viewport,raw-model,owner
---

# Summary

`apps/fret-cookbook/examples/embedded_viewport_basics.rs` now routes viewport-input diagnostic
writes through a demo-local `EmbeddedViewportBasicsModelOwner`.

The example remains an advanced/manual embedded viewport cookbook surface because it owns
`EmbeddedViewportSurface`, raw embedded viewport models, and target/input diagnostics. The cleanup
only removes the copyable pattern of writing `uv_x`, `uv_y`, `target_w`, `target_h`, and `kind`
directly from `on_viewport_input(...)`.

# Decision

Keep the advanced interop classification. This example documents the embedded viewport mechanism,
not default app state authoring. The local owner helper is a source-surface cleanup so raw writes
stay named and auditable without inventing a premature public embedded-viewport app API.

# Evidence

- Red proof before implementation:
  `cargo nextest run -p fret-cookbook embedded_viewport_basics_model_writes_stay_behind_owner_helper --no-fail-fast`
  failed because `EmbeddedViewportBasicsModelOwner` did not exist.
- The same test now requires `EmbeddedViewportBasicsModelOwner::record_viewport_input(...)` and
  forbids the old direct viewport-input update calls.
- `cargo check -p fret-cookbook --all-targets` passes.

# Next

The remaining cookbook raw write candidate is `utility_window_materials_windows.rs`. Treat it the
same way only if its status write is owner-shaped and the example remains an advanced/manual
Windows materials interop surface.
