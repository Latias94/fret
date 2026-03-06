# Fret Launch + App Surface (Fearless Refactor v1) 鈥?TODO

This checklist is scoped to the launch/public-surface refactor only.

Companion docs:

- Design: `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/DESIGN.md`
- Export inventory: `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/EXPORT_INVENTORY.md`
- Config inventory: `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/CONFIG_INVENTORY.md`
- Milestones: `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/MILESTONES.md`

## Setup / Inventory

- [x] Inventory every root export in `crates/fret-launch/src/lib.rs`.
- [x] Group those exports into:
  - [x] stable public contract
  - [x] transitional public surface
  - [x] compatibility/internal-plumbing path that should stop being part of the default public story
- [x] Inventory which lower-level launch types are referenced directly from `ecosystem/fret`.
- [x] Inventory which lower-level launch types are re-exported through `crates/fret-framework` bundles.

## Surface contract decisions

- [x] Decide whether `pub mod runner` should remain public in `crates/fret-launch`.
- [ ] Decide the intended long-term advanced entry recommendation:
  - [ ] `FnDriver` only
  - [x] `FnDriver` recommended, `WinitAppDriver` compatibility-only
- [x] Write down a deprecation/migration posture for `WinitAppDriver` if we choose a single-path model.
- [x] Decide which host-integration helper types are currently contract-worthy enough to keep at crate root.

## Config curation

- [x] Partition `WinitRunnerConfig` conceptually into:
  - [x] app/window defaults
  - [x] render/backend tuning
  - [x] streaming/media tuning
  - [x] platform/web specifics
- [x] Decide which config fields should be documented as advanced-only.
- [ ] Decide whether the future direction is:
  - [ ] nested config groups
  - [ ] helper builders only
  - [ ] new public wrapper type

## `fret` facade alignment

- [ ] Verify that `fret::App`, `UiAppDriver`, and `UiAppBuilder` expose the minimum lower-layer concepts needed for common apps.
- [ ] Identify where current docs/examples force users to learn runner-centric concepts too early.
- [ ] Map which advanced seams should remain first-class on `fret`:
  - [ ] `configure(...)`
  - [ ] `on_gpu_ready(...)`
  - [ ] `install_custom_effects(...)`
  - [ ] window create/close hooks
  - [ ] engine-frame customization

## Docs / Examples

- [ ] Update launch-facing docs so they describe three paths consistently:
  - [ ] `fret` for app authors
  - [ ] `fret-framework` for manual assembly
  - [ ] `fret-launch` for advanced integration
- [x] Migrate representative launch examples from direct `WinitAppDriver` impls to `FnDriver` (`chart_demo`, `bars_demo`, `error_bars_demo`, `area_demo`, `candlestick_demo`, `grouped_bars_demo`, `stacked_bars_demo`, `horizontal_bars_demo`, `histogram_demo`, `plot_demo`, `plot_image_demo`).
- [ ] Add one minimal advanced example that shows the supported launch escape hatch without exposing unnecessary internals.
- [ ] Cross-link this workstream from any relevant builder/onboarding docs if the final surface changes.

## Validation gates

- [ ] `cargo nextest run -p fret-launch`
- [ ] `cargo nextest run -p fret-framework`
- [ ] `cargo nextest run -p fret`
- [ ] `python tools/check_layering.py`

## Rollout notes

- [ ] Land documentation and export classification before removing or hiding launch exports.
- [ ] Prefer staged de-emphasis + migration docs before hard API removal.
- [ ] Keep host-integration use cases working throughout the refactor.
- [ ] Coordinate any `fret` top-level API changes with `docs/workstreams/app-entry-builder-v1/`.
