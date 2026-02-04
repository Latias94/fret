# Examples (in-repo)

Use these as canonical references when authoring app-level code.

## Minimal app: commands + models + redraw

- `apps/fret-examples/src/todo_demo.rs`

Focus:

- `fret_kit::app_with_hooks(...)`
- `Model<T>` state shape (`draft`, `todos`, `filter`)
- command dispatch (`CommandId`) and `Effect::Redraw`
- stable `test_id` anchors

## Background work: DispatcherHandle + Inbox (manual drain)

- `apps/fret-examples/src/markdown_demo.rs`

Focus:

- runner-installed `DispatcherHandle` (`app.global::<DispatcherHandle>()`)
- `fret_executor::Executors::spawn_background_to_inbox(...)`
- `Inbox::drain()` + `app.request_redraw(window)` when inbox changes

## Background work: InboxDrainRegistry (runner-drained)

- `ecosystem/fret-markdown/src/mathjax_svg_support.rs`

Focus:

- `InboxDrainer` + `InboxDrainRegistry` registration
- requesting redraw from inbox application (`host.request_redraw(...)`)

## Contracts (source of truth)

- Runtime effects enum: `crates/fret-runtime/src/effect.rs`
- Dispatcher + inbox registry: `crates/fret-runtime/src/execution.rs`
- Effect enqueue/flush behavior: `crates/fret-app/src/app.rs`

