# Fret

<p align="center">
  <img src="assets/fret-icon.svg" width="128" height="128" alt="Fret icon" />
</p>

> [!WARNING]
> **Experimental — under heavy development.**
>
> This project is an experiment in AI-driven software development. The vast majority of the code, tests, and documentation were written by AI (Codex). Humans direct architecture, priorities, and design decisions, but have not reviewed most of the code line-by-line. Treat this accordingly — there will be bugs, rough edges, and things that don't work. Use at your own risk.

Fret is the precision fretboard for your Rust UI: a GPU-first framework that turns application logic into crisp, fluid interactions.

Modular by design, ecosystem included — start fast for apps today, and scale to editor-grade workflows over time.

This repo focuses on the **core framework** (`crates/`) and incubates components + tooling in-tree (`ecosystem/`, `apps/`).
Long-term, ecosystem crates may move to a separate components repository.

## What you can build with Fret

- **General-purpose desktop apps**: productivity tools, dashboards, developer utilities.
- **Editor-grade tools**: docking layouts, tear-off windows, panels, command palette, rich interaction.
- **Viewport-heavy apps**: embed one or more GPU viewports (render targets) inside a UI.
- **Web demos/apps** via wasm (WebGPU path).

## Why Fret (high-signal features)

- **Editor-grade interaction substrate (not just widgets)**: docking + tear-off windows, multi-root overlays, focus/capture arbitration, and viewport embedding as first-class contracts. *(docking, multi-window, overlays, focus/capture, viewports)*
- **Web-native ergonomics, Rust-native architecture**: declarative element tree authoring with the view runtime, `LocalState`, typed actions, and explicit advanced escape hatches. *(declarative elements, local state, actions)*
- **Ecosystem included (batteries, but modular)**: shadcn/ui v4-aligned component taxonomy + recipes, icons, docking UI, markdown, tables, node graph, charts, and more. *(fret-ui-kit, fret-ui-shadcn, icons, docking)*
- **Mechanism vs policy separation**: the core runtime stays mechanism-only; interaction policies and defaults live in ecosystem crates so apps can stay opinionated without locking the engine. *(runtime contracts, policy in components)*
- **Rendering semantics you can rely on**: ordered scene ops, clipping/rounded corners/shadows as stable semantics (implementation can evolve without breaking UI behavior). *(ordered SceneOp, compositing groups (isolated opacity), ClipPath, bounded/budgeted offscreen passes, deterministic degradation)*
- **Debuggable by design**: semantics-first inspection + shareable diagnostics artifacts so UI bugs are explainable, not “works on my machine”. *(semantics tree, inspector, shareable bundles)*
- **Performance is observable**: built-in perf attribution surfaces worst-frame regressions and layout/measure hot spots without ad-hoc instrumentation. *(worst-frame triage, attribution, layout/measure)*
- **Modular backends & integration-friendly**: portable core + pluggable platform/runner/render backends to fit both engine-hosted and editor-hosted GPU contexts; desktop-first with an explicit WebGPU/wasm path. *(pluggable backends, engine-hosted GPU, WebGPU/wasm)*

## Project Direction

Fret draws inspiration from:

- `Zed` / `GPUI` style UX and editor workflows.
- Mature web UI design systems translated into Rust-native APIs (shadcn/Radix-style patterns).

Upstream/reference links live closer to the code that uses them:

- [`ecosystem/fret-ui-shadcn/README.md`](ecosystem/fret-ui-shadcn/README.md) (shadcn/Radix/cmdk/Base UI references)
- [`ecosystem/fret-ui-headless/README.md`](ecosystem/fret-ui-headless/README.md) (behavioral ports like cmdk score + Embla)
- [`docs/reference-stack-ui-behavior.md`](./docs/reference-stack-ui-behavior.md) (APG + Radix + Floating UI + cmdk)

The goal is to provide a smooth, general-purpose application framework that scales from app UIs to editor-class products.

## Quick Start

Need help setting up your toolchain or speeding up local builds? See [docs/setup.md](./docs/setup.md).

Want the shortest onboarding path? Read [docs/first-hour.md](./docs/first-hour.md).

Need help choosing the right example entry point (templates vs cookbook vs gallery vs labs)? See [docs/examples/README.md](./docs/examples/README.md).

For new app authors, keep the default authoring model small and explicit:

- `LocalState<T>` / `LocalState<Vec<_>>` for view-owned state, including starter keyed lists,
- `cx.actions().locals(...)` for most LocalState-first typed UI actions,
- `cx.actions().transient(...)` when an action must trigger an `App`-only effect in `render()`,
- `cx.actions().models(...)` only when you intentionally coordinate shared `Model<T>` graphs,
- `on_activate*` only for local pressable/widget glue.
- Everything else (`on_action_notify`, single-model aliases, redraw-oriented `on_activate*`) is optional shorthand and should stay out of first-contact onboarding unless a demo truly needs it.
- The remaining raw `on_action_notify` examples are cookbook/reference-only host-side integrations (toasts, router availability sync, background scheduling, RAF effects).

Use the onboarding ladder on purpose:

- **Default**: `hello` → `simple-todo` → `todo`
- **Comparison**: `simple_todo_v2_target` only when you want to compare local-state/list ergonomics against the default path
- **Advanced**: gallery, interop, docking, renderer, and maintainer demos

See [docs/README.md](./docs/README.md#state-management-authoring-ergonomics) for the full authoring map.

### 1) Run a lightweight cookbook example (recommended)

```bash
cargo run -p fretboard -- dev native --example hello
cargo run -p fretboard -- dev native --example simple_todo
```

### 2) Generate a new native app scaffold

Start with `simple-todo` (minimal baseline):

```bash
cargo run -p fretboard -- new simple-todo --name my-simple-todo
cargo run --manifest-path local/my-simple-todo/Cargo.toml
```

Then try the best-practice baseline (`todo`, includes selectors + queries):

```bash
cargo run -p fretboard -- new todo --name my-todo
cargo run --manifest-path local/my-todo/Cargo.toml
```

### 3) Explore runnable demos (workspace)

Discover runnable targets:

```bash
cargo run -p fretboard -- list cookbook-examples
cargo run -p fretboard -- list web-demos
```

Run a cookbook example (this runner auto-enables known feature-gated Lab examples):

```bash
cargo run -p fretboard -- dev native --example query_basics
```

Maintainer native demo bins (from `apps/fret-demo`, not the onboarding path):

```bash
cargo run -p fretboard -- list native-demos --all
```

Run the UI gallery (optional; heavier than cookbook):

```bash
cargo run -p fret-ui-gallery
```

Run a web demo (optional):

```bash
cargo run -p fretboard -- dev web --demo ui_gallery
```

### 4) Optional: diagnostics walkthrough (advanced)

Fret includes an optional diagnostics + scripted UI automation toolchain (`fretboard diag`).
If you are new to it, start with the cookbook walkthrough:

- [apps/fret-cookbook/README.md#diagnostics-optional](./apps/fret-cookbook/README.md#diagnostics-optional)

## Todo App API Taste

This is the interface style we optimize for: typed state, typed actions, and shadcn-based components.

```rust
use fret::app::prelude::*;

mod act {
    fret::actions!([Add = "app.todo.add.v1"]);
}

struct TodoView;

fn install_app(app: &mut App) {
    shadcn::shadcn_themes::apply_shadcn_new_york(
        app,
        shadcn::shadcn_themes::ShadcnBaseColor::Slate,
        shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}

impl View for TodoView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let draft = cx.state().local::<String>();
        let enabled = !cx.state().watch(&draft).layout().value_or_default().trim().is_empty();

        cx.actions().local_set::<act::Add, String>(&draft, String::new());

        let input = shadcn::Input::new(&draft)
            .a11y_label("New task")
            .placeholder("Add a task…")
            .submit_command(act::Add.into());

        let add_btn = shadcn::Button::new("Add").disabled(!enabled).action(act::Add);

        ui::h_flex(|cx| ui::children![cx; input, add_btn])
            .gap(Space::N2)
            .items_center()
            .into_element(cx)
            .into()
    }
}

fn main() -> fret::Result<()> {
    FretApp::new("todo")
        .window("todo", (560.0, 520.0))
        .config_files(false)
        .setup(install_app)
        .run_view::<TodoView>()
}
```

Reference implementation:

- Cookbook: [`apps/fret-cookbook/examples/simple_todo.rs`](./apps/fret-cookbook/examples/simple_todo.rs)
- Template guide: [`docs/first-hour.md`](./docs/first-hour.md)
- Example taxonomy: [`docs/examples/README.md`](./docs/examples/README.md)

![Fret gallery 01](screenshots/gallery-01.png)
![Fret gallery 02](screenshots/gallery-02.png)
![Fret gallery 03](screenshots/gallery-03.png)

## Ecosystem Coverage (Incubating)

Fret keeps stable boundaries in `crates/` and incubates faster-moving pieces in `ecosystem/`.

- Component systems:
  - [`fret-ui-kit`](./ecosystem/fret-ui-kit)
  - [`fret-ui-shadcn`](./ecosystem/fret-ui-shadcn)
  - [`fret-ui-material3`](./ecosystem/fret-ui-material3) (in progress)
- App architecture helpers:
  - [`fret-router`](./ecosystem/fret-router)
  - [`fret-query`](./ecosystem/fret-query)
  - [`fret-selector`](./ecosystem/fret-selector)
- Editor modules:
  - [`fret-node`](./ecosystem/fret-node)
  - [`fret-docking`](./ecosystem/fret-docking)
  - [`fret-viewport-tooling`](./ecosystem/fret-viewport-tooling)
- Visualization modules:
  - [`fret-chart`](./ecosystem/fret-chart)
  - [`fret-plot`](./ecosystem/fret-plot)
  - [`fret-plot3d`](./ecosystem/fret-plot3d)
- Icon packs and assets:
  - [`fret-icons`](./ecosystem/fret-icons)
  - [`fret-icons-lucide`](./ecosystem/fret-icons-lucide)
  - [`fret-icons-radix`](./ecosystem/fret-icons-radix)
  - [`fret-ui-assets`](./ecosystem/fret-ui-assets)

## Public Crate Surfaces (v0.1)

- `fret`: framework facade and stable entry point.
- `fret-ui-kit`: component authoring glue and policy helpers.
- `fret-ui-shadcn`: shadcn-inspired component recipes.
- `fret-node`: node-graph foundation for editor workflows.
- `fret-router`: typed message routing for app architecture.

## References

- Zed editor: https://github.com/zed-industries/zed
- GPUI crate in Zed: https://github.com/zed-industries/zed/tree/main/crates/gpui

## MSRV and Toolchain

- MSRV is `1.92` (`workspace.package.rust-version`) aligned with current `wgpu` minimum requirements.
- Development toolchain is pinned in `rust-toolchain.toml` for reproducible local and CI behavior.

## License

Licensed under either of:

- MIT License (`LICENSE-MIT`)
- Apache License, Version 2.0 (`LICENSE-APACHE`)
