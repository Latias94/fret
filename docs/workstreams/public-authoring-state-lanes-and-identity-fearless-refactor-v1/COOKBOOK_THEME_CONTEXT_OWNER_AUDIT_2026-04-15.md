# Cookbook Theme Context Owner Audit — 2026-04-15

Status: landed follow-on audit
Last updated: 2026-04-15

Related:

- `TODO.md`
- `MILESTONES.md`
- `apps/fret-cookbook/src/lib.rs`
- `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`
- `apps/fret-cookbook/examples/undo_basics.rs`
- `apps/fret-cookbook/examples/drag_basics.rs`
- `apps/fret-cookbook/examples/icons_and_assets_basics.rs`
- `apps/fret-cookbook/examples/effects_layer_basics.rs`
- `apps/fret-cookbook/examples/drop_shadow_basics.rs`
- `apps/fret-cookbook/examples/canvas_pan_zoom_basics.rs`
- `apps/fret-cookbook/examples/customv1_basics.rs`
- `apps/fret-cookbook/examples/app_owned_bundle_assets_basics.rs`
- `apps/fret-cookbook/examples/virtual_list_basics.rs`
- `apps/fret-cookbook/examples/embedded_viewport_basics.rs`
- `apps/fret-cookbook/examples/external_texture_import_basics.rs`
- `apps/fret-cookbook/examples/chart_interactions_basics.rs`

## Why this note exists

After the earlier examples/cookbook theme cleanup work, `apps/fret-cookbook/examples` still had a
mixed tail of `Theme::global(&*cx.app).snapshot()` reads.

Those callsites were not one owner class:

- some were ordinary `AppUi` roots,
- some were `UiCx` helpers,
- some were explicit `ElementContext` direct-leaf interop roots.

Without classifying them, the repo would keep treating all remaining cookbook theme reads as the
same kind of debt.

## Finding 1: ordinary cookbook token reads should stay on context-owned helpers

The following cookbook examples only needed theme tokens for ordinary UI chrome:

- `assets_reload_epoch_basics`
- `undo_basics`
- `drag_basics`
- `icons_and_assets_basics`
- `effects_layer_basics`
- `drop_shadow_basics`
- `canvas_pan_zoom_basics`
- `customv1_basics`
- `app_owned_bundle_assets_basics`
- `virtual_list_basics`

Those surfaces already render through `AppUi` roots or extracted `UiCx` helpers.

Conclusion:

- ordinary cookbook `AppUi` / `UiCx` theme reads should use `cx.theme_snapshot()`,
- not host-global `Theme::global(&*cx.app).snapshot()`.

## Finding 2: direct-leaf interop roots should read theme from `ElementContext`

The remaining explicit interop / retained roots were:

- `embedded_viewport_basics`
- `external_texture_import_basics`
- `chart_interactions_basics`

These are not app-facing `AppUi` sugar proofs.
They are direct `ElementContext` roots that already own retained/interop boundaries.

Conclusion:

- these examples should read theme from `cx.theme().snapshot()`,
- which keeps the context-owned direct-leaf lane explicit without reopening host-global access.

## Landed result

This audit lands:

- `cx.theme_snapshot()` on cookbook `AppUi` / `UiCx` ordinary theme reads,
- `cx.theme().snapshot()` on cookbook direct-leaf `ElementContext` interop roots,
- cookbook source-policy gates that freeze the split:
  - `cookbook_app_ui_examples_prefer_context_owned_theme_snapshot_helpers`
  - `cookbook_direct_leaf_interop_examples_prefer_element_context_theme_snapshot`

## Decision from this audit

Treat remaining cookbook theme reads by context owner, not by one flat grep bucket:

- `AppUi` / `UiCx` -> `cx.theme_snapshot()`
- `ElementContext` direct-leaf interop -> `cx.theme().snapshot()`

Do not use cookbook `Theme::global(&*cx.app).snapshot()` as a fallback default story anymore.
