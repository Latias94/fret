# App Entry Builder v1 (TODO)

## Decision closure

- [x] Keep the builder in `ecosystem/fret`.
- [x] Use `fret::App` as the primary public type.
- [x] Provide `FretApp` in the prelude for ergonomic authoring.
- [x] Provide `AppBuilder` as a discoverability/doc alias.
- [x] Keep the underlying runtime function-pointer based by default.
- [x] Keep the older top-level helpers as compatibility shorthands instead of deleting them now.

## Builder surface

- [x] `new(root_name)`
- [x] `window(title, size)`
- [x] `defaults(...)` / `minimal_defaults()` / `config_files(...)`
- [x] `ui_assets_budgets(...)`
- [x] `install_app(...)` / `install(...)`
- [x] `register_icon_pack(...)`
- [x] `ui(init_window, view)`
- [x] `ui_with_hooks(init_window, view, configure)`
- [x] `view::<V>()`
- [x] `view_with_hooks::<V>(configure)`
- [x] `run_ui(...)` / `run_ui_with_hooks(...)`
- [x] `run_view::<V>()` / `run_view_with_hooks::<V>(...)`

## Onboarding convergence

- [x] `fretboard new hello` uses the builder chain.
- [x] `fretboard new simple-todo` uses the builder chain.
- [x] `fretboard new todo` uses the builder chain.
- [x] `docs/examples/todo-app-golden-path.md` uses the builder chain.
- [x] Representative `apps/fret-examples` demos use the builder chain for the recommended path.
- [x] `ecosystem/fret/README.md` teaches the builder chain as the primary entry story.
- [x] Cross-link this workstream from `docs/README.md`.

## Remaining work

- [ ] Decide whether compatibility shorthands should eventually be doc-deemphasized even harder or
      soft-deprecated.
- [ ] Add a compile-oriented regression gate that covers `ui_with_hooks(...)` and
      `view_with_hooks(...)` directly.
- [ ] Audit more cookbook/examples for wording consistency when describing manual assembly vs builder
      entry.
- [ ] Decide whether a closure-based entry should exist at all.
- [ ] Decide whether more convenience methods belong on `fret::App` or should stay as install-hook
      recipes.

## Validation

- [x] `cargo fmt -p fret`
- [x] `cargo check -p fret --all-targets`
- [x] `cargo check -p fret --no-default-features --features desktop`
- [x] `cargo check -p fret-examples --all-targets`
- [x] `python tools/check_layering.py`
