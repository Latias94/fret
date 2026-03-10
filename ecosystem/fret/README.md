# fret

> [!WARNING]
> **Experimental — under heavy development.**
>
> This project is an experiment in AI-driven software development. The vast majority of the code, tests, and documentation were written by AI (Codex). Humans direct architecture, priorities, and design decisions, but have not reviewed most of the code line-by-line. Treat this accordingly — there will be bugs, rough edges, and things that don't work. Use at your own risk.

Desktop-first, batteries-included entry points for building UI apps with Fret.

This is an **ecosystem-level** crate. It intentionally provides a small, ergonomic surface for
applications while keeping the framework/kernel crates (`crates/*`) policy-light.

## Boundary note

`fret` is the golden-path authoring facade for application code. It is intentionally **not** the
repo's canonical example host.

- Use `docs/examples/README.md` for the canonical learning/index path.
- Use `examples/README.md` as the GitHub-friendly portal.
- Keep runnable lessons in `apps/fret-cookbook/examples/`, component coverage in
  `apps/fret-ui-gallery`, and heavier platform/app demos in their owning app crates.

This keeps the facade teachable while leaving example/tooling ownership outside the crate.

For repository overview / architecture docs, see the monorepo README:
https://github.com/Latias94/fret

## Quick start (in this repo)

If you are learning the repo's default path, follow this ladder in order:

1. `hello`
2. `simple-todo`
3. `todo`

- Index: `docs/examples/README.md`
- The generated template READMEs repeat the same ladder and explain where each rung fits.
- Use `fretboard new todo` when you want the richer third-rung baseline, not as a replacement for
  the first two rungs.

Generate a runnable starter (minimal baseline first):

```bash
cargo run -p fretboard -- new simple-todo --name my-simple-todo
cargo run --manifest-path local/my-simple-todo/Cargo.toml
```

Then move to the richer third rung when you actually want selectors + queries:

```bash
cargo run -p fretboard -- new todo --name my-todo
cargo run --manifest-path local/my-todo/Cargo.toml
```

## Quick start (Cargo)

With defaults (desktop + app):

```toml
[dependencies]
fret = { path = "../fret" } # path is relative to your Cargo.toml
```

Enable selector/query helpers (optional):

```toml
[dependencies]
fret = { path = "../fret", features = ["state"] }
```

If your crate lives under `apps/` in this repository:

```toml
[dependencies]
fret = { path = "../../ecosystem/fret" }
```

Or explicitly opt into a smaller surface:

```toml
[dependencies]
fret = { path = "../fret", default-features = false, features = ["desktop", "shadcn"] }
```

## Minimal app skeleton

```rust,ignore
use fret::app::prelude::*;

struct HelloView;

impl View for HelloView {
    fn render(&mut self, _ui: &mut AppUi<'_, '_, KernelApp>) -> Ui {
        shadcn::Label::new("Hello from Fret!").into()
    }
}

fn main() -> fret::Result<()> {
    FretApp::new("hello")
        .window("Hello", (560.0, 360.0))
        .view::<HelloView>()?
        .run()
}
```

## Features

- `desktop`: enable the native desktop stack (winit + wgpu) via `fret-framework/native-wgpu`.
- `app`: recommended baseline for apps (shadcn).
- `state`: enable selector/query helpers for `AppUi` (`data().selector(...)`, `data().query(...)`; currently backed by `ViewCx`).
- `batteries`: “works out of the box” opt-in bundle (config files + UI assets + icons + preloading + diagnostics).
- `config-files`: load layered config files from `.fret/` (settings/keymap/menubar).
- `diagnostics`: enable default diagnostics wiring (tracing + panic hook; plus extra dev tooling).
- `ui-assets`: enable UI render-asset caches (images/SVG) and install default budgets.
- `icons`: install the default built-in icon pack (Lucide).
- `preload-icon-svgs`: pre-register SVG icons on GPU ready.
- `command-palette`: enable the command palette wiring in the golden-path driver.

## Web / wasm

`fret` is desktop-first. For web demos in this repository, use tooling:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cargo run -p fretboard -- dev web --demo ui_gallery
```

This runs `apps/fret-demo-web` via `trunk serve`.

Related workstream: `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/`

## Choosing a native entry path

- App authors (default recommendation): `fret::FretApp::new(...).window(...).view::<V>()?`
- App authors with driver hooks: `fret::FretApp::new(...).window(...).view_with_hooks::<V>(...)?`
- Advanced integration with `fret` defaults: `fret::run_native_with_fn_driver(...)`
- Advanced integration with `FnDriver` hooks preserved: `fret::run_native_with_fn_driver_with_hooks(...)`
- Advanced integration with a preconfigured `FnDriver`: `fret::run_native_with_configured_fn_driver(...)`
- Advanced low-level interop driver path (compat seam, non-default): `fret::run_native_with_compat_driver(...)`

## What remains first-class on `fret`

Advanced users do **not** need to drop to `fret-launch` immediately. The `fret` facade keeps the
following seams first-class:

- `App::{view::<V>, view_with_hooks::<V>}`
- `UiAppBuilder::configure(...)` for launch/window config
- `UiAppBuilder::on_gpu_ready(...)`
- `UiAppBuilder::install_custom_effects(...)`
- `UiAppDriver::{window_create_spec, window_created, before_close_window}`
- `UiAppDriver::{record_engine_frame, viewport_input, handle_global_command}`

The builder chain is now the only app-author entry story on `fret`. Advanced users still keep
real extension seams without dropping to `fret-launch` immediately.

That makes `fret` suitable for both general-purpose desktop apps and many editor-style customizations
before you need to depend on `fret-bootstrap` or `fret-launch` directly.

## When to drop down to `fret-framework` + `fret-bootstrap`

`fret` is designed to keep the “first app” and “small app” story simple. Prefer dropping down
to manual assembly when you need:

- a custom runner/event loop integration (`fret-launch`),
- non-default settings/keymap/config file layering,
- different icon/asset wiring policies than the kit defaults,
- experimenting with alternate component surfaces without the kit defaults.

Mapping (rough):

- `fret::UiAppBuilder` -> `fret_bootstrap::UiAppBootstrapBuilder`
- `fret::UiAppDriver` -> `fret_bootstrap::ui_app_driver::UiAppDriver`
- `fret::run_native_with_fn_driver(...)` -> `fret_bootstrap::BootstrapBuilder::new_fn(...)`
- `fret::run_native_with_fn_driver_with_hooks(...)` -> `fret_bootstrap::BootstrapBuilder::new_fn_with_hooks(...)`
- `fret::run_native_with_configured_fn_driver(...)` -> `fret_bootstrap::BootstrapBuilder::new(...)` with a preconfigured `FnDriver`
- `fret::run_native_with_compat_driver(...)` -> `fret_bootstrap::BootstrapBuilder::new(...)` for advanced low-level interop / retained driver cases

The recommended manual-assembly entry point remains `fret-bootstrap`, keeping the underlying driver
hotpatch-friendly (function-pointer `FnDriver` surface, per ADR 0105 / 0110).
