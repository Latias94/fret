# Release Surface Fearless Refactor v1

This workstream is a pre-release fearless refactor of Fret's **publishable crate surface**.

It is intentionally willing to delete, move, or narrow pre-release facade APIs when they blur the
release story or drag internal-only crates into the publish closure.

This is **not** an ADR by itself. If a change crosses into hard runtime contracts, it must still go
through the ADR workflow.

## Problem statement

The current repo already has a good architectural story, but the release surface is still too
porous:

- `fret` is documented as the app-facing golden path, but it still carried editor/workspace-shell
  surface that belongs to `fret-workspace`.
- `fret` referenced crates marked `publish = false`, which blocks a real crates.io release even
  when the surface is optional or niche.
- some ecosystem crates pull obviously non-minimal feature stacks into their default closure,
  especially where recipe crates unconditionally depend on heavier domain crates.
- tooling and diagnostics are well-designed, but they should not be mistaken for "core app
  dependencies".

## Goals

- Define a release story with a small number of memorable entry crates.
- Make publishable crates depend only on other publishable crates.
- Keep app-facing facades app-facing.
- Keep editor/workspace policy and shell composition off the default `fret` lane unless we are
  willing to publish and support that surface as a first-class contract.
- Create a wave-based refactor plan that can land incrementally without waiting for one giant cut.

## Non-goals

- Publishing every crate in `ecosystem/`.
- Preserving all current `fret` root-level explicit lanes.
- Reworking router, docking, charts, or editor APIs purely for symmetry.

## Release layers

We should release in three distinct layers:

1. **User-facing entry crates**
   - `fret`
   - `fret-framework`
   - `fret-bootstrap`
   - `fret-ui-kit`
   - `fret-ui-shadcn`
2. **Required support crates**
   - kernel/runtime/UI/platform/runner/render dependencies required by those entry crates
3. **Deferred or tooling-only crates**
   - `fretboard`
   - `fret-diag`
   - editor/workspace-only crates and repo-owned harness shells

The user should only need to remember layer 1. crates.io still needs layer 2. layer 3 should not
accidentally leak into layer 1.

## First-wave decisions

### A) `fret` stops owning workspace-shell facade surface

`workspace_shell` is editor/workspace-specific shell composition. It is not part of the minimal
app-author golden path and should not force `fret-workspace` into the `fret` release closure.

Decision:

- keep `workspace_menu` on `fret` for now because it is a generic app-facing menubar helper,
- remove `workspace_shell` from the `fret` public root surface,
- remove the direct `fret-workspace` dependency from `fret`.

Consequence:

- editor/workspace shell composition must stay on `fret-workspace` or repo-owned app crates until
  we intentionally design a publishable editor shell surface.

### B) `fret-router-ui` becomes a publishable thin adoption layer

`fret::router` is already an explicit optional surface and the docs already treat
`fret-router-ui` as the thin UI adoption layer next to `fret-router`.

Decision:

- make `fret-router-ui` publishable instead of leaving `fret/router` blocked behind an
  internal-only crate,
- keep it thin and app-owned; do not let it turn into a second default app runtime.

### C) Future heavy-closure shrink work should happen by feature surgery, not facade inflation

The next heavy closures are mostly caused by:

- `fret-bootstrap` optional integrations,
- `fret-ui-shadcn` unconditional heavy recipe/domain dependencies,
- optional editor/material/AI surfaces hanging off `fret`.

Decision:

- prefer feature-gating first, especially on `fret-bootstrap` where the crate itself represents the
  recommended best-practice bootstrap bundle,
- only extract a new crate when the integration is a genuinely separate authoring surface that we
  expect users to depend on and learn independently,
- keep `fret` root features limited to app-facing lanes that we are willing to teach; maintainer
  conveniences and niche design-system/domain crates should stay on direct owning crates, though
  compatibility aliases may remain when the feature name itself aids discoverability,
- do not hide them behind more root-level facade shortcuts.

## Wave plan

### Wave 1: publish blockers

- remove `publish = false` dependencies from publishable crates,
- delete obviously misplaced root facade surface,
- make `cargo package` / release preflight mechanically plausible.

### Wave 2: closure slimming

- split heavy `fret-ui-shadcn` dependencies behind focused features,
- shrink `fret-bootstrap` default/near-default fan-out,
- reassess which optional surfaces belong on `fret` vs direct crate use.

### Wave 3: release candidate set

- lock the first public crate set,
- verify docs teach only those surfaces,
- run release preflight against the selected set instead of the whole workspace.

## Success metrics

- `fret` no longer references `publish = false` crates.
- the default `fret` root surface stays app-facing and excludes editor shell helpers.
- optional extension lanes that remain on `fret` map to publishable owning crates.
- release planning can talk about a small entry set instead of the full workspace inventory.

## Evidence anchors

- `ecosystem/fret/Cargo.toml`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret-router-ui/Cargo.toml`
- `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
- `docs/crate-usage-guide.md`
