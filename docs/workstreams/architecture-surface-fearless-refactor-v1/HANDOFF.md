# Architecture Surface Fearless Refactor v1 — Handoff

Status: Active
Last updated: 2026-05-17

## Current State

The workstream has been opened from an architecture surface audit. ASF-020 is complete: the `fret`
backend-free app-authoring profiles no longer pull the native launch/render/backend stack, and the
consumption profile script now guards that dependency shape.

`fret-bootstrap --no-default-features` still pulls launch/render/backend dependencies; that is now
the next architectural split rather than part of the completed `fret` facade fix.

The user explicitly approved fearless refactoring with no compatibility burden: redundant old code,
aliases, and wrappers may be deleted when first-party callers are migrated.

## Active Task

- Task ID: ASF-021
- Owner: unassigned
- Files:
  - `ecosystem/fret/Cargo.toml`
  - `ecosystem/fret/src/lib.rs`
  - `ecosystem/fret/src/app_entry.rs`
  - related docs/tests
- Validation:
  - `cargo check -p fret --no-default-features --features app`
  - targeted template/doc gate if backend-running methods move

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

## Blockers

- None.

## Next Recommended Action

- Start ASF-021: split backend-running `FretApp` methods from backend-free app-authoring types so
  the public facade shape matches the dependency boundary proven by ASF-020.
