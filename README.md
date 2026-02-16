# Fret

<p align="center">
  <img src="assets/fret-icon.svg" width="128" height="128" alt="Fret icon" />
</p>

> [!WARNING]
> **Experimental learning project (not production-ready).**
>
> Fret is used to explore architecture and interaction ideas for Rust GUI systems. LLM tooling is used heavily as a development accelerator, and non-trivial changes are manually reviewed for code correctness and architecture direction before adoption. APIs and behavior may change quickly. **Do not use Fret in production systems.**

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
- **Web-native ergonomics, Rust-native architecture**: declarative element tree authoring with typed state (`Model<T>`) and typed messages/routing. *(declarative elements, state, routing)*
- **Ecosystem included (batteries, but modular)**: shadcn/ui v4-aligned component taxonomy + recipes, icons, docking UI, markdown, tables, node graph, charts, and more. *(fret-ui-kit, fret-ui-shadcn, icons, docking)*
- **Mechanism vs policy separation**: the core runtime stays mechanism-only; interaction policies and defaults live in ecosystem crates so apps can stay opinionated without locking the engine. *(runtime contracts, policy in components)*
- **Rendering semantics you can rely on**: ordered scene ops, clipping/rounded corners/shadows as stable semantics (implementation can evolve without breaking UI behavior). *(ordered SceneOp, compositing groups (isolated opacity), ClipPath, bounded/budgeted offscreen passes, deterministic degradation, GPU conformance gates)*
- **Debuggable by design**: semantics-first inspection + shareable diagnostics artifacts so UI bugs are explainable, not “works on my machine”. *(semantics tree, inspector, shareable bundles)*
- **Performance is observable**: built-in perf attribution surfaces worst-frame regressions and layout/measure hot spots without ad-hoc instrumentation. *(worst-frame triage, attribution, layout/measure)*
- **Modular backends & integration-friendly**: portable core + pluggable platform/runner/render backends to fit both engine-hosted and editor-hosted GPU contexts; desktop-first with an explicit WebGPU/wasm path. *(pluggable backends, engine-hosted GPU, WebGPU/wasm)*

## Project Direction

Fret draws inspiration from:

- `Zed` / `GPUI` style UX and editor workflows.
- Mature web UI design systems (shadcn/Radix patterns) translated into Rust-native APIs.

The goal is to provide a smooth, general-purpose application framework that scales from app UIs to editor-class products.

## Quick Start

Create a new native app scaffold (recommended: `simple-todo` first):

```bash
cargo run -p fretboard -- new simple-todo --name my-simple-todo
```

Then try the best-practice baseline (`todo`, includes selectors + queries):

```bash
cargo run -p fretboard -- new todo --name my-todo
```

Run native demo:

```bash
cargo run -p fretboard -- dev native --bin todo_demo
```

Run web demo:

```bash
cargo run -p fretboard -- dev web --demo ui_gallery
```

## Todo App API Taste

This is the interface style we optimize for: typed state, typed messages, and shadcn-based components.

```rust
use std::sync::Arc;
use fret::prelude::*;

#[derive(Clone)]
struct TodoItem {
    id: u64,
    done: Model<bool>,
    text: Arc<str>,
}

#[derive(Debug, Clone)]
enum Msg {
    Add,
    Remove(u64),
}

struct TodoState {
    todos: Model<Vec<TodoItem>>,
    draft: Model<String>,
    router: MessageRouter<Msg>,
    next_id: u64,
}

fn main() -> anyhow::Result<()> {
    fret::app_with_hooks("todo", init_window, view, |d| d.on_command(on_command))?
        .with_main_window("todo", (560.0, 520.0))
        .run()?;
    Ok(())
}
```

```rust
fn view(cx: &mut ElementContext<'_, App>, st: &mut TodoState) -> ViewElements {
    st.router.clear();
    let add_cmd = st.router.cmd(Msg::Add);

    let input = shadcn::Input::new(st.draft.clone())
        .placeholder("Add a task")
        .submit_command(add_cmd.clone())
        .into_element(cx);

    let add_button = shadcn::Button::new("Add")
        .on_click(add_cmd)
        .into_element(cx);

    let card = shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Todo").into_element(cx),
            shadcn::CardDescription::new("State + router + shadcn components").into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([
            ui::v_flex(cx, |_cx| [input, add_button])
                .gap(Space::N3)
                .into_element(cx),
        ])
        .into_element(cx),
    ])
    .into_element(cx);

    vec![card].into()
}
```

Reference implementation:

- `apps/fret-examples/src/todo_demo.rs`
- `docs/examples/todo-app-golden-path.md`

![Fret gallery 01](screenshots/gallery-01.png)
![Fret gallery 02](screenshots/gallery-02.png)
![Fret gallery 03](screenshots/gallery-03.png)

## Ecosystem Coverage (Incubating)

Fret keeps stable boundaries in `crates/` and incubates faster-moving pieces in `ecosystem/`.

- Component systems:
  - [`fret-ui-kit`](https://github.com/Latias94/fret/tree/main/ecosystem/fret-ui-kit)
  - [`fret-ui-shadcn`](https://github.com/Latias94/fret/tree/main/ecosystem/fret-ui-shadcn)
  - [`fret-ui-material3`](https://github.com/Latias94/fret/tree/main/ecosystem/fret-ui-material3) (in progress)
- App architecture helpers:
  - [`fret-router`](https://github.com/Latias94/fret/tree/main/ecosystem/fret-router)
  - [`fret-query`](https://github.com/Latias94/fret/tree/main/ecosystem/fret-query)
  - [`fret-selector`](https://github.com/Latias94/fret/tree/main/ecosystem/fret-selector)
- Editor modules:
  - [`fret-node`](https://github.com/Latias94/fret/tree/main/ecosystem/fret-node)
  - [`fret-docking`](https://github.com/Latias94/fret/tree/main/ecosystem/fret-docking)
  - [`fret-viewport-tooling`](https://github.com/Latias94/fret/tree/main/ecosystem/fret-viewport-tooling)
- Visualization modules:
  - [`fret-chart`](https://github.com/Latias94/fret/tree/main/ecosystem/fret-chart)
  - [`fret-plot`](https://github.com/Latias94/fret/tree/main/ecosystem/fret-plot)
  - [`fret-plot3d`](https://github.com/Latias94/fret/tree/main/ecosystem/fret-plot3d)
- Icon packs and assets:
  - [`fret-icons`](https://github.com/Latias94/fret/tree/main/ecosystem/fret-icons)
  - [`fret-icons-lucide`](https://github.com/Latias94/fret/tree/main/ecosystem/fret-icons-lucide)
  - [`fret-icons-radix`](https://github.com/Latias94/fret/tree/main/ecosystem/fret-icons-radix)
  - [`fret-ui-assets`](https://github.com/Latias94/fret/tree/main/ecosystem/fret-ui-assets)

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

- MIT License
- Apache License, Version 2.0
