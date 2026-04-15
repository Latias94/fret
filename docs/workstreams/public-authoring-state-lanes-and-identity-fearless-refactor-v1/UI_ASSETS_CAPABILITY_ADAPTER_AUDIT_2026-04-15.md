# UI Assets Capability Adapter Audit — 2026-04-15

Status: follow-on audit for the active public authoring lane

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_DEREF_PRESSURE_CLASSIFICATION_AUDIT_2026-04-15.md`
- `ecosystem/fret-ui-assets/src/ui.rs`
- `ecosystem/fret-ui-assets/src/lib.rs`
- `apps/fret-examples/src/assets_demo.rs`
- `apps/fret-examples/src/markdown_demo.rs`
- `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`

## Assumptions First

### A1) The repeated `cx.app + cx.window` asset-helper pattern is a real helper-surface problem,
not just demo-local style drift

Confidence: Confident

Evidence:

- `apps/fret-examples/src/assets_demo.rs`
- `apps/fret-examples/src/markdown_demo.rs`
- `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`

If wrong:

- this change would be overfitting to a few example files instead of closing a repeated
  ecosystem-level seam.

### A2) The correct ownership layer is `ecosystem/fret-ui-assets`, not `ecosystem/fret`

Confidence: Confident

Evidence:

- `ecosystem/fret-ui-assets/src/ui.rs`
- `ecosystem/fret-ui-assets/src/image_asset_state.rs`
- `ecosystem/fret-ui-assets/src/ui_assets.rs`

If wrong:

- the adapter would belong in the app facade rather than next to the actual image/SVG helper
  surfaces.

### A3) This helper should live in the optional `ui` module, not the root public surface

Confidence: Confident

Evidence:

- `ecosystem/fret-ui-assets/src/lib.rs`
- `ecosystem/fret-ui-assets/src/ui.rs`

If wrong:

- the crate would be mixing `ElementContext`-dependent sugar into the non-UI root surface and
  weakening its current feature boundary.

## Question

What is the next correct framework slice for the repeated helper-local asset state/stats seam after
the `AppUi` root accessor batch closed?

## Verdict

Land capability-first UI adapters in `fret-ui-assets::ui`.

Do **not** solve this by:

- widening the `fret` facade,
- teaching more direct `cx.app` / `cx.window` access in examples,
- or reopening the broader `AppUi` `Deref` debate.

The correct slice is:

- add `_in(...)` helpers in `fret-ui-assets::ui`,
- keep them on the existing optional UI sugar surface,
- and migrate first-party proof callsites that only needed `cx.app + cx.window` to enter those
  helpers.

## Findings

### 1) The repeated asset seam was already cross-consumer, not one-file noise

Before this slice, the same pattern appeared in multiple first-party consumers:

- `assets_demo`
- `markdown_demo`
- `assets_reload_epoch_basics`

The repeated shape was:

- pass `cx.app` and `cx.window` directly to asset helpers,
- or call `UiAssets::image_stats(&mut *cx.app)` / `svg_stats(...)` from an otherwise typed render
  context.

Conclusion:

- this was real repeated helper-surface friction and justified a framework-level adapter.

### 2) The adapter belongs next to the asset helpers, not in the app facade

The underlying owner functions already live in `fret-ui-assets`:

- `image_asset_state::use_rgba8_image_state(...)`
- `UiAssets::image_stats(...)`
- `UiAssets::svg_stats(...)`

Conclusion:

- the capability-first wrappers should live in the same crate, so the owner surface stays
  discoverable and the app facade does not absorb asset-specific policy.

### 3) `fret-ui-assets::ui` is the right public lane for this sugar

`fret-ui-assets` already keeps `ElementContext`/UI-specific sugar under the optional `ui` module,
for example:

- `ImageSourceElementContextExt`
- `SvgAssetElementContextExt`

This slice extends that same lane with:

- `use_rgba8_image_state_in(...)`
- `image_stats_in(...)`
- `svg_stats_in(...)`

Conclusion:

- the new helpers follow the crate’s existing UI-only layering instead of opening a parallel
  surface.

### 4) The migrated proof surfaces now teach the narrower helper entry story

Landed migrations:

- `apps/fret-examples/src/assets_demo.rs`
- `apps/fret-examples/src/markdown_demo.rs`
- `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`

Consumer impact:

- `assets_demo` no longer spells raw `cx.app` / `cx.window` just to enter RGBA8 image state and
  stats helpers.
- `markdown_demo` no longer spells raw `cx.app` / `cx.window` for its inline raster helper.
- `assets_reload_epoch_basics` no longer reads cache stats through `&mut *cx.app` when the render
  surface already owns a typed context.

Conclusion:

- the slice improved the framework story and immediately paid off in first-party authoring
  surfaces.

## Landed Slice

This audit lands one bounded framework change plus first-party migrations:

1. `ecosystem/fret-ui-assets/src/ui.rs` now exports:
   - `use_rgba8_image_state_in(...)`
   - `image_stats_in(...)`
   - `svg_stats_in(...)`
2. `ecosystem/fret-ui-assets/src/lib.rs` now source-gates that these UI-only helpers stay under
   the optional `ui` module.
3. First-party examples/cookbook proofs are migrated onto that adapter lane.
4. `apps/fret-examples/Cargo.toml` now enables the `ui` feature on the direct `fret-ui-assets`
   dependency so the examples crate can consume the intended UI helper surface explicitly.

## Repro, Gate, Evidence

Repro targets:

- `cargo run -p fretboard -- dev native --bin assets_demo`
- `cargo run -p fretboard -- dev native --bin markdown_demo`

Primary gates:

- `cargo nextest run -p fret-ui-assets ui_context_asset_helpers_stay_under_optional_ui_module`
- `cargo nextest run -p fret-examples asset_helper_entrypoints_prefer_ui_assets_capability_adapters`
- `cargo nextest run -p fret-cookbook assets_reload_epoch_example_prefers_ui_assets_capability_adapters`
- `cargo check --all-targets -p fret-examples -p fret-cookbook`

What these prove:

- the adapter stays on the intended `fret-ui-assets::ui` lane,
- first-party consumer proofs now teach the capability-first asset helper path,
- and the relevant examples/cookbook compile after the dependency wiring and callsite migration.
