# App Entry Builder v1 (TODO)

## Decision closure

- [x] Keep the builder in `ecosystem/fret`.
- [x] Use `fret::FretApp` as the primary public type.
- [x] Teach `FretApp` from `fret::app::prelude::*` as the canonical app-author path.
- [x] Delete root-level `App` / `AppBuilder` aliases before first release.
- [x] Keep the underlying runtime function-pointer based by default.
- [x] Remove the older top-level shorthand helpers from `fret` crate root so the app-author story stays builder-only.
- [x] Keep the shipped `FretApp` surface view-first; `ui*` builder entry methods are not part of the release target.

## Builder surface

- [x] `new(root_name)`
- [x] `window(title, size)`
- [x] `defaults(...)` / `minimal_defaults()` / `config_files(...)`
- [x] `ui_assets_budgets(...)`
- [x] `setup(...)`
- [x] `setup_with(...)`
- [x] icon-pack app installers via `setup(...::app::install)`
- [x] `view::<V>()`
- [x] `view_with_hooks::<V>(configure)`
- [x] Delete `run_view::<V>()` / `run_view_with_hooks::<V>(...)` from `FretApp` before release.

## Onboarding convergence

- [x] `fretboard-dev new hello` uses the builder chain.
- [x] `fretboard-dev new simple-todo` uses the builder chain.
- [x] `fretboard-dev new todo` uses the builder chain.
- [x] `docs/examples/todo-app-golden-path.md` uses the builder chain.
- [x] Representative `apps/fret-examples` demos use the builder chain for the recommended path.
- [x] `ecosystem/fret/README.md` teaches the builder chain as the primary entry story.
- [x] Cross-link this workstream from `docs/README.md`.

## Remaining work

- [ ] Sweep remaining historical docs that still teach removed helper paths or older MVU-era entry snippets.
- [x] Add compile-oriented regression coverage for the two recommended builder entry paths
      (`view`, `view_with_hooks`) and the default main-window fallback.
- [x] Add a focused surface gate that keeps `fret` crate root builder-only and keeps README
      onboarding text aligned with the builder story.
- [x] Remove stale workstream wording that still described `run_view*` as part of the live
      `FretApp` builder surface.
- [ ] Audit more cookbook/examples for wording consistency when describing manual assembly vs builder
      entry.
- [ ] Decide whether a closure-based entry should exist at all.
- [ ] Decide whether more convenience methods belong on `fret::FretApp` or should stay as install-hook
      recipes.

## Validation

- [x] `cargo fmt -p fret`
- [x] `cargo check -p fret --all-targets`
- [x] `cargo check -p fret --no-default-features --features desktop`
- [x] `cargo check -p fret-examples --all-targets`
- [x] `cargo nextest run -p fret builder_surface_tests`
- [x] `python tools/check_layering.py`
- [x] `python tools/gate_fret_builder_only_surface.py`
