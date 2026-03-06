# Fret Launch + App Surface (Fearless Refactor v1) — TODO

This checklist is scoped to the launch/public-surface refactor only.

## Setup / Inventory

- [ ] Inventory every root export in `crates/fret-launch/src/lib.rs`.
- [ ] Group those exports into:
  - [ ] stable public contract
  - [ ] transitional public surface
  - [ ] internal plumbing that should stop being part of the public story
- [ ] Inventory which lower-level launch types are referenced directly from `ecosystem/fret`.
- [ ] Inventory which lower-level launch types are re-exported through `crates/fret-framework` bundles.

## Surface contract decisions

- [ ] Decide whether `pub mod runner` should remain public in `crates/fret-launch`.
- [ ] Decide the intended long-term advanced entry recommendation:
  - [ ] `FnDriver` only
  - [ ] `FnDriver` recommended, `WinitAppDriver` compatibility-only
- [ ] Write down a deprecation/migration posture for `WinitAppDriver` if we choose a single-path model.
- [ ] Decide which host-integration helper types are truly contract-worthy.

## Config curation

- [ ] Partition `WinitRunnerConfig` conceptually into:
  - [ ] app/window defaults
  - [ ] render/backend tuning
  - [ ] streaming/media tuning
  - [ ] platform/web specifics
- [ ] Decide which config fields should be documented as advanced-only.
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

