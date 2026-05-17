# Architecture Surface Fearless Refactor v1 — Handoff

Status: Active
Last updated: 2026-05-17

## Current State

The workstream has been opened from an architecture surface audit. ASF-020 and ASF-021 are
complete: the `fret` backend-free app-authoring profiles no longer pull the native
launch/render/backend stack, the consumption profile script guards that dependency shape, and
`FretApp` is now a backend-free authoring spec with desktop-only execution methods.

`fret-bootstrap --no-default-features` still pulls launch/render/backend dependencies; that is now
the next architectural split rather than part of the completed `fret` facade fix.

The user explicitly approved fearless refactoring with no compatibility burden: redundant old code,
aliases, and wrappers may be deleted when first-party callers are migrated.

## Active Task

- Task ID: ASF-030
- Owner: unassigned
- Files:
  - `ecosystem/fret-bootstrap`
  - `crates/fret-launch`
  - `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
  - related docs/tests
- Validation:
  - `cargo tree -p fret-bootstrap --no-default-features -e normal --depth 4`
  - `cargo check -p fret-bootstrap --no-default-features`
  - focused checks for callers moved between bootstrap and launch

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

## Blockers

- None.

## Next Recommended Action

- Start ASF-030: split backend-free bootstrap planning/default policy from concrete launch/render
  adapters in `fret-bootstrap`.
