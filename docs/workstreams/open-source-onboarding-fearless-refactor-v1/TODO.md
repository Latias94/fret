# Open-source onboarding (fearless refactor v1) — TODO

## Examples surface

- [ ] Verify the “boring ladder” ordering still matches the fastest-to-understand path:
  templates → cookbook → gallery → labs.
- [x] Ensure onboarding docs use GitHub-clickable links (avoid bare relative paths).
- [x] Audit `README.md` “Quick Start” commands to ensure they still run on `main`.

## Cookbook curation

- [x] Maintain an explicit “Official vs Lab” split (Official should compile fast and avoid optional subsystems).
- [ ] For each Lab example, ensure:
  - a `required-features` gate exists, and
  - `fretboard list cookbook-examples --all` shows the required feature name(s).
- [x] Add a diagnostics walkthrough for `hello` (stable `test_id` + one script).
- [ ] Add one additional diagnostics walkthrough after `hello` (candidate: `simple_todo` smoke).

## UI gallery gating (no heavy refactor)

- [x] Keep default native build “lite” and fast.
- [x] Gate unfinished/debug/dev pages behind `gallery-dev`.
- [x] Gate `material3` behind `gallery-material3`.
- [x] Provide a “gallery-full” umbrella feature for contributors.

## Default feature surfaces (dependency audit)

- [ ] Audit `ecosystem/fret` default features vs “just build an app” expectation:
  - Confirm `default = ["desktop", "app"]` is the right story.
  - [x] Keep `diagnostics` opt-in (`app` excludes it; `batteries` includes it).
  - [x] Keep selector/query helpers opt-in (`state` is separate from `app`).
  - Confirm `shadcn` is the minimum “pleasant” baseline for first-time apps.
- [x] Keep `fretboard new` templates boring: do not enable `fret/diagnostics` by default.
- [ ] Audit `apps/` runnable targets and make sure the recommended ones do not pull in heavy optional stacks.

## README code samples (staleness audit)

- [x] Ensure the README “Todo App API Taste” still matches current APIs and conventions:
  - `Model<T>` + `ViewCx` patterns
  - typed action macro usage
  - shadcn authoring surface (constructors + builders)
