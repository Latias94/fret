# Architecture Surface Fearless Refactor v1 — Handoff

Status: Active
Last updated: 2026-05-17

## Current State

The workstream has been opened from an architecture surface audit. ASF-020, ASF-021, ASF-030,
ASF-031, ASF-040, ASF-041, and ASF-050 are complete: the `fret` backend-free app-authoring profiles
no longer pull the native launch/render/backend stack, `FretApp` is now a backend-free authoring
spec with desktop-only execution methods, `fret-bootstrap --no-default-features` now exposes
bootstrap planning/default policy without pulling the concrete launch/render/backend stack,
first-party scaffold/template guidance now uses the new app-spec-recording vs
desktop-builder-application split, the app prelude has a closed Golden Path budget, LocalState has
a private owner module, and the boolean-control family now proves the headless/primitives/kit
taxonomy.

The user explicitly approved fearless refactoring with no compatibility burden: redundant old code,
aliases, and wrappers may be deleted when first-party callers are migrated.

## Active Task

- Task ID: ASF-051
- Owner: unassigned
- Files:
  - `ecosystem/fret-ui-kit`
  - `ecosystem/fret-ui-shadcn`
  - `ecosystem/fret-ui-material3`
- Validation:
  - package tests for the migrated recipe crate
  - `python tools/check_layering.py`
  - no new backend deps in ecosystem crates

## Decisions Since Last Update

- Opened a new lane instead of reusing `framework-modularity-fearless-refactor-v1`, because the scope
  spans public facade narrowing, bootstrap/launch split, ecosystem taxonomy, menu/select shared
  policy, and renderer facade ownership.
- Treat compatibility as a non-goal for this lane unless a later release decision explicitly changes
  that constraint.
- First executable task is the minimal `fret` app-authoring profile because it has an objective cargo
  tree failure today.
- ASF-020 moved `fret-bootstrap`, `fret-launch`, and the native runner/render stack behind
  `desktop`. `app` is now documented as a backend-free authoring baseline.
- Desktop-bound convenience features on `fret` now opt into `desktop` explicitly:
  `diagnostics`, `tracing`, `devloop`, `ui-assets`, `icons`, `preload-icon-svgs`, and
  `command-palette`.
- ASF-021 made `FretApp` available in backend-free app-authoring profiles while keeping window,
  view-builder, asset-startup, command-palette, `UiAppBuilder`, and runner methods on the
  `desktop` lane.
- ASF-030 split `fret-bootstrap`'s backend-free planning/default policy from the concrete
  launch/render adapter surface. `fret-bootstrap --no-default-features` no longer pulls
  `fret-launch`, `fret-render`, `wgpu`, `winit`, native platform, or runner crates. The public
  backend-free asset planning surface is covered by
  `ecosystem/fret-bootstrap/tests/backend_free_bootstrap_profile.rs`.
- ASF-031 lets the `fret` `app` profile depend on backend-free `fret-bootstrap` planning types
  without pulling launch/render/backend crates. `FretApp::asset_startup(...)` and
  `FretApp::asset_reload_policy(...)` record startup specs in backend-free app-authoring profiles,
  while `UiAppBuilder::with_asset_startup(...)` remains the desktop builder application surface.
- `crates/fretboard` generated assets continue to mount via `generated_assets::mount(builder)?`;
  scaffold README guidance now says that generated mount applies the plan on the builder. The
  scaffold compile gate also flushed two stale template API uses (`cx.app` and `cx.text(...)` on
  `AppUi`), both migrated on the generated template surface.
- ASF-040 closes the `fret::app::prelude::*` Golden Path budget in source-level public-surface
  tests. Named prelude exports are limited to first-contact app authoring nouns, and anonymous
  extension traits remain explicit budget entries. `docs/crate-usage-guide.md` now records that the
  app prelude is not a staging area for new secondary surfaces.
- ASF-041 split the LocalState owner family out of the monolithic view authoring runtime. The
  private `ecosystem/fret/src/view/local_state.rs` module now owns `LocalState`, `LocalStateTxn`,
  `LocalActionCapture`, `WatchedState`, `TrackedStateExt`, and LocalState-backed component model
  adapters, while `crate::view` keeps the existing public re-export surface. Source-level tests now
  combine owner modules when checking the authoring API shape.
- ASF-050 chose the boolean-control family (`checkbox` + `switch`) as the first taxonomy proof.
  `ecosystem/fret-ui-headless/src/boolean_control.rs` now owns optional-bool transition behavior;
  `fret-ui-kit::primitives::{checkbox,switch}` keep runtime/a11y/model facades only; shadcn,
  Material3, editor, gallery, and `fret` facade call sites import the headless owner directly for
  pure state. ADR 0154 alignment now records that `fret-ui-primitives` remains deleted for v1.

## Blockers

- None.

## Next Recommended Action

- Start ASF-051: migrate one recipe surface to consume the finalized taxonomy directly rather than
  broad kit compatibility shims. The boolean-control recipe family is already migrated; a good next
  slice is a nearby recipe that still imports shared behavior through `fret-ui-kit::headless` when a
  direct `fret-ui-headless` import would be the clearer owner path.
