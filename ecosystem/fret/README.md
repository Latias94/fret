# fret

> [!WARNING]
> **Experimental — under heavy development.**
>
> This project is an experiment in AI-driven software development. The vast majority of the code, tests, and documentation were written by AI (Codex). Humans direct architecture, priorities, and design decisions, but have not reviewed most of the code line-by-line. Treat this accordingly — there will be bugs, rough edges, and things that don't work. Use at your own risk.

Desktop-first, batteries-included entry points for building UI apps with Fret.

This is an **ecosystem-level** crate. It intentionally provides a small, ergonomic surface for
applications while keeping the framework/kernel crates (`crates/*`) policy-light.

For repository overview / architecture docs, see the monorepo README:
https://github.com/Latias94/fret

## Quick start (in this repo)

Generate a runnable starter:

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
use fret::prelude::*;

fn main() -> fret::Result<()> {
    fret::run("hello", |_app, _window| (), |cx, _st| {
        shadcn::Label::new("Hello from Fret!").into_element(cx).into()
    })
}
```

## Features

- `desktop`: enable the native desktop stack (winit + wgpu) via `fret-framework/native-wgpu`.
- `app`: recommended baseline for apps (shadcn + optional state helpers).
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

## When to drop down to `fret-framework` + `fret-bootstrap`

`fret` is designed to keep the “first app” and “small app” story simple. Prefer dropping down
to manual assembly when you need:

- a custom runner/event loop integration (`fret-launch`),
- non-default settings/keymap/config file layering,
- different icon/asset wiring policies than the kit defaults,
- experimenting with alternate component surfaces without the kit defaults.

Mapping (rough):

- `fret::app_with_hooks(...)` → `fret_bootstrap::ui_app_with_hooks(...)`
- `fret::UiAppBuilder` → `fret_bootstrap::UiAppBootstrapBuilder`
- `fret::UiAppDriver` → `fret_bootstrap::ui_app_driver::UiAppDriver`

The recommended manual-assembly entry point remains `fret-bootstrap`, keeping the underlying driver
hotpatch-friendly (function-pointer `FnDriver` surface, per ADR 0105 / 0110).
