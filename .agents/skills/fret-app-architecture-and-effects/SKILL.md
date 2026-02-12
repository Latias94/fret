---
name: fret-app-architecture-and-effects
description: "Build Fret apps with a predictable data flow (Models + Commands + Effects) and runner-owned concurrency (Dispatcher + Inbox). Use when adding persistence, background work, or app-level side effects without blocking the UI thread."
---

# Fret app architecture & effects

This skill is about **app-level structure**: how to wire Models, Commands, Effects, and background
work so your app remains deterministic, portable (native + wasm), and easy to debug.

If you are looking for **GPU post-processing** (“blur/glass/backdrop”), that is **EffectLayer**
(`docs/effects-authoring.md`), not the runtime `Effect` pipeline described here.

## When to use

- You are building a new app (or a new feature surface) and want a stable “golden path”.
- You need persistence (load/save), file/network work, or other side effects.
- You need time-based behavior (debounce, timeouts, animation ticks) without split-brain scheduling.
- You are unsure whether something belongs in `Effect::SetTimer` vs `Dispatcher::dispatch_after`.

## Inputs to collect (ask the user)

Ask these so the architecture stays portable and deterministic:

- Target platform(s): native only vs native + wasm (filesystem/network constraints)?
- Side effects: persistence, network, filesystem, clipboard, timers, background compute?
- Concurrency needs: what can be off-main-thread, what must stay on the UI thread?
- State shape: window-local vs app-global; any derived/async state stack needs (`selector/query/router`)?
- Regression protection: what behavior needs a unit test vs a `fretboard diag` script?

Defaults if unclear:

- Use `fret-kit` golden path, keep UI state main-thread-only, and put background results through an inbox + wake.

## Smallest starting point (one command)

- `cargo run -p fretboard -- dev native --bin todo_demo`

## Quick start

- For a minimal app wiring example, start with `apps/fret-examples/src/todo_demo.rs`.
- For “background work without blocking UI”, follow the Dispatcher + Inbox pattern (ADR 0160).
- If this touches user-visible timing, prefer `Effect::SetTimer` / `Effect::RequestAnimationFrame` over ad-hoc timers.

## Workflow

Follow the “Golden path (recommended defaults)” below, then protect the behavior with at least one regression artifact
(unit/integration test, or a `fretboard diag` script if a state machine is involved).

## Definition of done (what to leave behind)

- Minimum deliverables (3-pack): Repro (smallest app surface), Gate (test/script), Evidence (command + anchors). See `fret-skills-playbook`.
- Entry point chosen and justified (`fret-kit` vs `fret-bootstrap` vs custom `fret-launch`).
- Side effects are expressed via the canonical surfaces (Effects + Dispatcher/Inbox), without blocking the UI thread.
- Any user-visible timing uses runner-owned scheduling (`Effect::SetTimer` / RAF) instead of ad-hoc timers.
- At least one regression artifact exists for the changed behavior:
  - unit/integration test for deterministic logic, and/or
  - `tools/diag-scripts/*.json` for interaction/state-machine behavior.
- Commands/ID surfaces are stable (no churn without migration plan).

## Golden path (recommended defaults)

### 1) Pick an entry point

- **Most apps**: start with `fret-kit` (`fret_kit::app_with_hooks(...)`).
- **Need more control** (config layering, runner tuning, non-default policies): drop down to
  `fret-bootstrap` or a custom `fret-launch` driver.

Reference: `ecosystem/fret-kit/README.md`.

### 2) Shape your state (window-local vs global)

Recommended split:

- **Window-local state**: a struct returned by `init_window(...)` and passed into `view(...)` /
  hooks (per-window UI tree + per-window Models).
- **App globals** (`app.set_global(...)`): cross-window caches, registries, and runner-provided
  handles (e.g. `DispatcherHandle`).

Prefer **small `Model<T>` fields** over one giant model. Keep stable ids as plain values (e.g.
`next_id: u64`) and put reactive data into Models.

Example (minimal): `apps/fret-examples/src/todo_demo.rs`.

### 2.1) Adopt the state stack defaults (v1)

Recommended default split:

- Local mutable state: `Model<T>` / element state helpers.
- Derived state: `fret_selector::Selector` + `use_selector(...)` for memoized counts/filters/projections.
- Async resources: `fret_query::QueryClient` + `use_query*` for loading/error/cache/retry/invalidate.
- Typed command routing:
  - `MessageRouter<M>` for per-frame dynamic actions,
  - `KeyedMessageRouter<K, M>` for view-cache-safe dynamic actions.

Reference: `docs/workstreams/state-management-v1-extension-contract.md`.

### 3) Use Commands for intent, Models for data, Effects for side effects

Typical flow:

1. UI element emits a `CommandId` (button click, submit, keybinding).
2. `on_command(...)` mutates Models (`app.models_mut().update(...)`).
3. Request UI work via Effects (`Effect::Redraw`, timers, clipboard, etc.).

Notes:

- Prefer `app.request_redraw(window)` / `Effect::Redraw(window)` for one-shot redraws.
- Prefer `Effect::RequestAnimationFrame(window)` for frame-driven progression.
- Use `Effect::SetTimer` + `Event::Timer { token }` for UI-visible timing so it participates in the
  runner’s deterministic flush points (ADR 0108 / ADR 0160).

References:

- Effect enum: `crates/fret-runtime/src/effect.rs`
- Effect enqueue semantics: `crates/fret-app/src/app.rs` (`push_effect`, `flush_effects`)

### 4) Background work: Dispatcher + Inbox (canonical pattern)

Hard rule (ADR 0160):

- UI/runtime state (`App`, `ModelStore`) is **main-thread only**.
- Background tasks must communicate results via **data-only messages**.

Runner-provided execution surface:

- `fret_runtime::DispatcherHandle` is installed by the runner as an app global (see examples).

Recommended pattern:

1. Create an `Inbox<M>` for data-only messages.
2. Use `fret_executor::Executors` to spawn background work.
3. Background task sends `M` into the inbox and calls `dispatcher.wake(window)`.
4. Main thread drains inbox at a driver boundary, applies updates, and requests redraw.

Examples:

- Manual draining in the render loop: `apps/fret-examples/src/markdown_demo.rs`
- Runner-drained inboxes via registry: `ecosystem/fret-markdown/src/mathjax_svg_support.rs`

### 4.1) Common integrations and copyable recipes

Keep the main skill lean and link out:

- Common ecosystem integrations: `references/recipes.md`
- Copyable app recipes (debounced persistence, async load): `references/recipes.md`

## Common pitfalls

- Doing file/network/CPU-heavy work on the UI thread “because it’s small” (it won’t stay small).
- Using ad-hoc timers/threads for UI-visible behavior (breaks determinism + diag visibility).
- Capturing `Model<T>` strongly in long-lived callbacks; use weak patterns where appropriate.
- Mixing “visual effects” (EffectLayer) with runtime “effects” (`Effect` enum) in terminology.

## References

- ADR 0160 (Dispatcher + inbox + wake): `docs/adr/0184-execution-and-concurrency-surface-v1.md`
- Golden path pipelines: `docs/adr/0110-golden-path-ui-app-driver-and-pipelines.md`
- Minimal todo app wiring: `apps/fret-examples/src/todo_demo.rs`
- Background inbox pattern (manual drain): `apps/fret-examples/src/markdown_demo.rs`
- Background inbox pattern (runner drain registry): `ecosystem/fret-markdown/src/mathjax_svg_support.rs`

## Evidence anchors (where to start reading code)

- Effect enum + shapes: `crates/fret-runtime/src/effect.rs`
- App effect queue + flushing: `crates/fret-app/src/app.rs`
- Config + project dirs: `crates/fret-app/src/config_files.rs`
- Executor + inbox patterns:
  - Example (manual drain): `apps/fret-examples/src/markdown_demo.rs`
  - Example (registry drain): `ecosystem/fret-markdown/src/mathjax_svg_support.rs`

## Related skills

- `fret-component-authoring` (component-level state/invalidation)
- `fret-commands-and-keymap` (commands as app intent)
- `fret-diag-workflow` (shareable bundles and scripted repros)
