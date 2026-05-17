# Architecture Surface Fearless Refactor v1 — Handoff

Status: Active
Last updated: 2026-05-17

## Current State

The workstream has been opened from an architecture surface audit. ASF-020, ASF-021, ASF-030, and ASF-031
are complete: the `fret` backend-free app-authoring profiles no longer pull the native
launch/render/backend stack, `FretApp` is now a backend-free authoring spec with desktop-only
execution methods, `fret-bootstrap --no-default-features` now exposes bootstrap planning/default
policy without pulling the concrete launch/render/backend stack, and first-party scaffold/template
guidance now uses the new app-spec-recording vs desktop-builder-application split.

The user explicitly approved fearless refactoring with no compatibility burden: redundant old code,
aliases, and wrappers may be deleted when first-party callers are migrated.

## Active Task

- Task ID: ASF-041
- Owner: unassigned
- Files:
  - `ecosystem/fret`
  - `ecosystem/fret/src/lib.rs`
  - `ecosystem/fret/src/view.rs`
  - `ecosystem/fret/src/actions.rs`
  - `ecosystem/fret/tests`
  - related docs/tests
- Validation:
  - existing `fret` public surface tests
  - targeted compile/tests for `LocalState`, typed actions, selector/query reads, and raw-model escape hatches
  - focused `cargo nextest run -p fret ...`
  - `python tools/check_consumption_profiles.py` only if feature/profile behavior changes

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

## Blockers

- None.

## Next Recommended Action

- Start ASF-041: split the large view/action authoring implementation into deeper owner modules
  without widening the public interface. Begin by mapping the current owners in `ecosystem/fret/src/view.rs`
  and `ecosystem/fret/src/actions.rs`, then move one cohesive family at a time behind stable
  re-exports guarded by the existing public-surface tests.
