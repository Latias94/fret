# App Entry Builder v1 (TODO)

## Design / API

- Decide the public naming (`fret::App` vs `fret::UiApp` vs `fret::AppBuilder`).
- Finalize the minimal builder surface:
  - `new(root_name)`
  - `window(title, size)`
  - `mvu::<P>()` / `ui(init, view)`
  - `install_app(...)` / `install(...)`
  - `defaults(...)` and a small `Defaults` preset set.
- Decide whether “closure entry” exists in v1, and if so:
  - feature gating
  - documentation warnings
  - API naming that makes the tradeoff obvious.

## Feature simplification

- Propose a small set of feature aliases (e.g. `batteries`) and document them.
- Ensure filesystem-touching defaults can be disabled (`config-files`).
- Ensure icon-related ergonomics do not require users to understand `fret-ui-kit/icons` vs pack
  installers.

## Templates / Docs

- Update `fretboard new hello` to use the builder chain.
- Update `fretboard new simple-todo` and `fretboard new todo` to use the builder chain.
- Update `docs/first-hour.md` and `docs/examples/todo-app-golden-path.md` to show the new entry.
- Add a short “Drop down to `fret-bootstrap`” section with a mapping table.

## Validation gates

- Add a compile-only doc test snippet that uses the new builder chain.
- Add a minimal smoke test that ensures the builder uses fn-pointer hooks by default (no captured
  closures).
- Verify `cargo check -p fret` with:
  - defaults
  - `--no-default-features --features "desktop"`

## Rollout

- Land design doc + milestones first (this folder).
- Implement builder behind a feature if needed.
- Switch templates and docs after the API is stable enough.

