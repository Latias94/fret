# fret

Desktop-first, batteries-included entry points for building UI apps with Fret.

This is an **ecosystem-level** crate. It intentionally provides a small, ergonomic surface for
applications while keeping the framework/kernel crates (`crates/*`) policy-light.

## Quick start (in this repo)

Generate a runnable starter:

```bash
cargo run -p fretboard -- new todo --name my-todo
cargo run --manifest-path local/my-todo/Cargo.toml
```

## Quick start (Cargo)

With defaults (desktop + diagnostics + ui-assets + lucide icons):

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
fret = { path = "../fret", default-features = false, features = ["desktop"] }
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
- `batteries`: “works out of the box” bundle (enabled by default).
- `config-files`: load layered config files from `.fret/` (settings/keymap/menubar). Enabled by default.
- `diagnostics`: enable default diagnostics (tracing + panic hook).
- `ui-assets`: enable UI render-asset caches (images/SVG) and install default budgets.
- `icons-lucide` / `icons-radix`: install a built-in icon pack (mutually exclusive).
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
