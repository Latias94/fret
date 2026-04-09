# Open-source onboarding (fearless refactor v1) — TODO

## Examples surface

- [x] Verify the “boring ladder” ordering still matches the fastest-to-understand path:
  templates → cookbook → gallery → labs.
- [x] Ensure onboarding docs use GitHub-clickable links (avoid bare relative paths).
- [x] Audit `README.md` “Quick Start” commands to ensure they still run on `main`.

## Cookbook curation

- [x] Maintain an explicit “Official vs Lab” split (Official should compile fast and avoid optional subsystems).
- [x] For each Lab example, ensure:
  - a `required-features` gate exists, and
  - `fretboard-dev list cookbook-examples --all` shows the required feature name(s).
- [x] Add a diagnostics walkthrough for `hello` (stable `test_id` + one script).
- [x] Add one additional diagnostics walkthrough after `hello` (`simple_todo` smoke).

## UI gallery gating (no heavy refactor)

- [x] Keep default native build “lite” and fast.
- [x] Gate unfinished/debug/dev pages behind `gallery-dev`.
- [x] Gate `material3` behind `gallery-material3`.
- [x] Provide a “gallery-full” umbrella feature for contributors.

## Default feature surfaces (dependency audit)

- [x] Audit `ecosystem/fret` default features vs “just build an app” expectation:
  - [x] Keep `default = ["desktop", "app"]` (desktop + shadcn).
  - [x] Keep `diagnostics` opt-in (`app` excludes it; `batteries` includes it).
  - [x] Keep selector/query helpers opt-in (`state` is separate from `app`).
  - [x] Treat `shadcn` as the minimum “pleasant” baseline for first-time apps.
- [x] Keep `fretboard-dev new` templates boring: do not enable `fret/diagnostics` by default.
- [x] Audit `apps/` runnable targets and make sure the recommended ones do not pull in heavy optional stacks.
  - Onboarding: `fret-cookbook` examples (no optional features by default).
  - Catalog: `fret-ui-gallery` (lite by default; dev/material3 behind features).
  - Maintainer bins: `fret-demo` (discover via `fretboard-dev list native-demos --all`).

## README code samples (staleness audit)

- [x] Ensure the README “Todo App API Taste” still matches current APIs and conventions:
  - `Model<T>` + `ViewCx` patterns
  - typed action macro usage
  - shadcn authoring surface (constructors + builders)
