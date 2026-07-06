---
type: Work Progress
title: Examples wasm ui-assets feature boundary
timestamp: 2026-07-06T00:00:00Z
git_branch: fix/examples-wasm-ui-assets
tags: fret,ui-framework,features,wasm,ui-assets,examples
---

# Summary

The `fret-examples` wasm library now enables `fret/ui-assets` by default because wasm-compiled
demos import `fret::app::ui_assets`.

The root `fret/ui-assets` feature no longer enables `desktop`. It now depends directly on
`fret-bootstrap` plus `fret-ui-assets`, keeping render-asset cache wiring cross-platform and leaving
the native runner stack owned by `desktop`/default features.

# Decision

`ui-assets` is a cross-platform UI render-asset facade, not a desktop runner alias. Desktop-only
features such as icons, SVG preloading, diagnostics, and command-palette wiring can still depend on
`desktop`, but image/SVG cache wiring must stay usable from wasm without pulling native runner
dependencies.

# Evidence

- `cargo check -p fret-examples --target wasm32-unknown-unknown --lib` now passes without extra
  command-line features.
- `cargo check -p fret --no-default-features --features app,ui-assets --target wasm32-unknown-unknown`
  passes.
- `cargo check -p fret --no-default-features --features ui-assets --target wasm32-unknown-unknown`
  passes.
- `authoring_surface_policy_tests::ui_assets_feature_stays_cross_platform_and_off_desktop_runner`
  locks the root feature relation.
- `basic_plot_demos_surface::fret_examples_wasm_target_enables_fret_ui_assets_for_asset_demos`
  locks the examples wasm dependency relation.

# Next

Keep feature names capability-oriented. If a future root feature needs the native runner stack, make
that dependency visible through `desktop` rather than hiding it behind an otherwise portable facade.
