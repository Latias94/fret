# Architecture Surface Fearless Refactor v1 — Handoff

Status: Active
Last updated: 2026-05-17

## Current State

The workstream has been opened from an architecture surface audit. ASF-020, ASF-021, and ASF-030
are complete: the `fret` backend-free app-authoring profiles no longer pull the native
launch/render/backend stack, `FretApp` is now a backend-free authoring spec with desktop-only
execution methods, and `fret-bootstrap --no-default-features` now exposes bootstrap
planning/default policy without pulling the concrete launch/render/backend stack.

The user explicitly approved fearless refactoring with no compatibility burden: redundant old code,
aliases, and wrappers may be deleted when first-party callers are migrated.

## Active Task

- Task ID: ASF-031
- Owner: unassigned
- Files:
  - `ecosystem/fret-bootstrap`
  - `ecosystem/fret`
  - `apps/fretboard`
  - related docs/templates
  - related docs/tests
- Validation:
  - focused `cargo check` for affected packages
  - template/scaffold checks if call sites move
  - `python tools/check_consumption_profiles.py`

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

## Blockers

- None.

## Next Recommended Action

- Start ASF-031: migrate first-party callers/templates onto the new bootstrap/launch split and
  delete displaced helper aliases where the target surface is now clear.
