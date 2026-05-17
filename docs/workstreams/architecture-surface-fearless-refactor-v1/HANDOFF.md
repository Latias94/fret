# Architecture Surface Fearless Refactor v1 — Handoff

Status: Closed
Last updated: 2026-05-17

## Current State

The workstream has been opened from an architecture surface audit. ASF-020, ASF-021, ASF-030,
ASF-031, ASF-040, ASF-041, ASF-050, ASF-051, ASF-060, ASF-061, and ASF-070 are complete: the `fret` backend-free app-authoring
profiles no longer pull the native launch/render/backend stack, `FretApp` is now a backend-free
authoring spec with desktop-only execution methods, `fret-bootstrap --no-default-features` now
exposes bootstrap planning/default policy without pulling the concrete launch/render/backend stack,
first-party scaffold/template guidance now uses the new app-spec-recording vs
desktop-builder-application split, the app prelude has a closed Golden Path budget, LocalState has
a private owner module, the boolean-control family now proves the headless/primitives/kit taxonomy,
the carousel recipe consumes headless engines directly instead of through kit shims, and menu/select
entry-focus target selection is now owned by a shared headless module. Remaining shadcn menu/select
policy cleanup has been split into a follow-on lane, and `fret-render` has been retained as the
curated default renderer facade instead of being collapsed into `fret-render-wgpu`.

The user explicitly approved fearless refactoring with no compatibility burden: redundant old code,
aliases, and wrappers may be deleted when first-party callers are migrated.

## Active Task

- Task ID: none
- Owner: none
- Files:
  - `docs/workstreams/architecture-surface-fearless-refactor-v1`
  - `docs/workstreams/README.md`
  - `docs/todo-tracker.md`
- Validation:
  - closeout complete
  - workstream catalog check

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
- ASF-051 migrated the carousel recipe off broad kit headless shims:
  `ecosystem/fret-ui-shadcn/src/carousel.rs` now imports
  `fret_ui_headless::{carousel, embla, snap_points}` directly. This preserves `fret-ui-kit` as
  runtime/design-system infrastructure rather than the owner for pure carousel engines.
- ASF-060 extracted input-modality-gated entry-focus target selection into
  `ecosystem/fret-ui-headless/src/entry_focus.rs`. `fret-ui-kit` menu/select primitives now adapt
  runtime input modality into that owner, and shadcn select consumes the shared select adapter. A
  failed `select_keyboard_navigation` integration run exposed an adjacent pointer-open ArrowDown
  expectation conflict (`"apple"` vs expected `"banana"`); keep that as select follow-on evidence,
  not as an ASF-060 pass/fail blocker.
- ASF-061 split the remaining shadcn menu/select semantics into
  `docs/workstreams/shadcn-menu-select-policy-followon-v1/`. The older
  `docs/workstreams/menu-surfaces-alignment-v1/` lane remains completed historical OS/in-window
  menubar scope and should not be silently reopened.
- ASF-070 decided not to collapse `fret-render` into `fret-render-wgpu`. The closed
  `docs/workstreams/renderer-modularity-fearless-refactor-v1/` lane already established
  `crates/fret-render` as a curated default facade with explicit v1 buckets, compile-fail guards for
  backend-only diagnostics, and host-provided GPU topology coverage. Future renderer semantic or
  capability work belongs in a renderer-specific follow-on, not in this architecture-surface lane.
- ASF-080 closed this lane. No remaining architecture-surface task is open here.

## Blockers

- None.

## Next Recommended Action

- Continue with a narrower active lane:
  `docs/workstreams/shadcn-menu-select-policy-followon-v1/` is the next recorded follow-on for
  menu/select policy cleanup. Future renderer semantic/capability work should open a separate
  renderer-specific follow-on instead of reopening this lane.
