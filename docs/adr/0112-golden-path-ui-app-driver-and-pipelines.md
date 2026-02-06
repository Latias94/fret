# ADR 0112: Golden-Path UI App Driver and Pipelines

Status: Accepted

## Context

Fret’s kernel/backends/apps boundaries are locked (ADR 0093) and we already have an ecosystem bootstrap story (ADR 0108).
In practice, however, first-time application code still contains recurring boilerplate:

- `UiTree` wiring (`set_root`, `dispatch_event`, `dispatch_command`, `propagate_*_changes`),
- icon pack setup and optional SVG pre-registration,
- UI render-asset cache wiring (image/SVG) and budgeting,
- dev-only hotpatch toggles and safe hot reload hooks (ADR 0107),
- multi-window glue and per-window state initialization.

We want to keep the kernel portable and mechanism-only, while providing a **user-facing “golden path”** that:

- makes small apps easy to write,
- scales to editor-grade apps (multi-window, docking, heavy models),
- remains compatible with conservative Subsecond-style hotpatching (ADR 0107),
- does not force a single runtime (sync vs async, single-thread vs multi-thread).

## Goals

- Provide a **single recommended application authoring surface** that eliminates common boilerplate without hiding core boundaries.
- Standardize the **three-pipeline mental model** (Event / Command / Effect) in user-facing docs and APIs.
- Preserve Subsecond hotpatch compatibility by default (function-pointer driver surface, bounded reset hooks).
- Offer clear patterns for:
  - async/background work,
  - multi-threaded “heavy editor” architectures,
  - multi-window apps,
  - custom resource systems above ADR 0004 (UI render assets) without drifting into ADR 0026 (project assets).

## Non-goals

- Introducing an editor/project asset database/import pipeline into framework crates (ADR 0026 remains app-owned).
- Forcing a particular async runtime (Tokio/async-std) or thread model.
- Replacing `fret-launch` with a new runner abstraction.

## Decision

### 1) We standardize three pipelines as the user-facing mental model

Fret applications should be explained and structured as three unidirectional pipelines:

1. **Event pipeline (platform → UI/runtime)**:
   - platform events become `fret_core::Event`,
   - the UI tree routes them (`UiTree::dispatch_event`),
   - components may emit commands/effects as a result.

2. **Command pipeline (UI → app logic)**:
   - UI triggers commands (`CommandId`),
   - the driver dispatches them (UI-first, app-second),
   - app logic updates models/globals and/or pushes effects.

3. **Effect pipeline (app → platform/renderer → event backflow)**:
   - app logic pushes `fret_runtime::Effect`,
   - the runner drains effects at a flush point (ADR 0004),
   - platform completion produces `Event` back into the event pipeline.

This matches GPUI/Zed’s “flush point + stable IDs” direction while remaining explicit and debuggable.

### 2) We provide an ecosystem-level “golden path driver” that wraps `FnDriver` wiring

We introduce a small, opinionated wrapper in `ecosystem/fret-bootstrap` (or a dedicated ecosystem crate if needed)
that builds a `fret_launch::FnDriver` with standard glue.

Name (working): `UiAppDriver` / `UiAppBuilder` (exact naming is non-binding).

Key properties:

- **Backend glue stays out of the kernel** (ADR 0093): this is ecosystem-level and may depend on `fret-launch`.
- The authoring surface should be **hotpatch-friendly** by construction:
  - prefer `fn` pointers for top-level entry points (not captured closures or trait objects),
  - keep long-lived callback registries disposable on hot reload (ADR 0107).

### 3) Minimal user API (conceptual)

The golden path driver should allow users to write apps by providing:

- `init_window(app, window) -> WindowState`
- `view(cx, &mut WindowState) -> Vec<AnyElement>` (build root element children)
- `on_command(app, services, window, &mut WindowState, CommandId)` (app commands)

And optionally:

- `on_model_changes(app, window, &mut WindowState, changed_models)`
- `on_global_changes(app, window, &mut WindowState, changed_globals)`
- `on_hot_reload_window(app, services, window, &mut WindowState)` (close overlays, reset UI caches)

The wrapper owns the boilerplate:

- create/set root,
- dispatch event/command into `UiTree`,
- propagate model/global changes into `UiTree`,
- call overlay controllers and a11y snapshot scheduling where appropriate,
- integrate common ecosystem helpers (icons, UI assets) when opted in.

### 4) Subsecond/hotpatch compatibility rules

When users enable hotpatch (ADR 0107):

- **Recommended driver**: function-pointer based (`FnDriver`) with `hot_reload_*` hooks.
- **No kernel contamination**: Subsecond integration remains feature-gated in `fret-launch`.
- **Reset boundary**: on patch applied, the runner performs a conservative reset:
  - call `hot_reload_global`,
  - call `hot_reload_window`,
  - discard/rebuild long-lived callback registries (action hooks, overlay registries),
  - keep safety-first default behavior for window state drop (ADR 0107).

The golden path driver must not store long-lived trait objects or captured closures in a way that makes patching
unpredictable. Closures used *ephemerally* during view construction are acceptable; long-lived storage should
prefer indirection via IDs/registries (ADR 0074).

### 5) Async and multi-thread patterns (no forced runtime)

Fret’s `App` and `ModelStore` are main-thread oriented; background threads must not mutate `App` directly.
We standardize two patterns:

The user-facing execution and wake surface that supports these patterns is locked in
`docs/adr/0199-execution-and-concurrency-surface-v1.md`.

**A) “Inbox + timer/RAF” (portable, minimal)**

- background work sends pure data messages into a `std::sync::mpsc`/`crossbeam_channel` sender,
- the main thread periodically drains the receiver (via `Effect::SetTimer` or `Effect::RequestAnimationFrame`) and
  applies updates to models/globals,
- this keeps boundaries explicit and works across native and wasm (with wasm using RAF instead of threads).

**B) “External runtime + wake” (heavy editor)**

- run a dedicated async runtime (Tokio) on a separate thread for I/O, indexing, LSP, etc,
- communicate to the UI thread via a message channel,
- ensure the UI thread is woken promptly (tooling/runtime may use runner facilities; details are runner-specific).

This ADR does not mandate a specific wake mechanism beyond what the current runner already provides; it documents
the invariant that **only the main thread mutates `App`**.

### 6) Resources and “assets” clarification

We explicitly distinguish:

- **UI render assets** (ADR 0004): image/SVG/icon bytes registered at a flush point and referenced via stable IDs.
  Ecosystem caches (`fret-ui-assets`) provide GPUI-style `use_asset` conveniences and budgeting/eviction/stats.
- **Project/editor assets** (ADR 0026): GUID identity, import pipeline, dependency graphs — app-owned and out of scope.

The golden path driver may optionally integrate `fret-ui-assets` by:

- creating global caches (image/SVG),
- driving them from the event pipeline (`handle_event`),
- providing a unified facade (non-binding recommendation from ADR 0108).

Apps may implement additional resource systems by following the same effect-driven flush-point design:

- UI holds stable IDs and keys,
- app logic initiates loads and registers via effects,
- runner/renderer owns actual GPU resources.

### 7) Multi-window and customization

The golden path must support:

- per-window state initialization (`init_window`),
- custom window creation policies (mapping `CreateWindowRequest` → `WindowCreateSpec`),
- multi-window command routing (commands may be window-scoped or global),
- editor-grade docking workflows (still handled by ecosystem docking crate; driver just wires the pipelines).

## Consequences

### Benefits

- Small apps become “few functions + components”, not “learn the entire runner glue”.
- Heavy apps keep explicit boundaries (models/commands/effects) and can scale to multi-threaded architectures.
- Hotpatch stays conservative and predictable: patch → reset hooks → re-render.

### Costs

- Yet another surface to document and keep stable.
- Some “escape hatches” remain necessary for advanced apps (custom runners, custom effect draining policies).

## Implementation Notes (Suggested)

- Implement the golden path as a wrapper over `fret-launch::FnDriver` in `fret-bootstrap`.
- Implement the golden path as a wrapper over `fret-launch::FnDriver` in `fret-bootstrap` (e.g. `UiAppDriver`).
- Provide a minimal `Todo`-class example as a user-facing reference (see `docs/examples/todo-app-golden-path.md`).
- Keep all Subsecond integration feature-gated and dev-focused (ADR 0107).
- Add an ecosystem integration guidance note once patterns stabilize (see ADR 0113).

## References

- Crate layering: `docs/adr/0093-crate-structure-core-backends-apps.md`
- Bootstrap/tools story: `docs/adr/0108-ecosystem-bootstrap-ui-assets-and-dev-tools.md`
- Dev hotpatch boundaries: `docs/adr/0107-dev-hotpatch-subsecond-and-hot-reload-safety.md`
- Execution and concurrency surface: `docs/adr/0199-execution-and-concurrency-surface-v1.md`
- Resource handles + flush point: `docs/adr/0004-resource-handles.md`
- Editor project assets (out of scope): `docs/adr/0026-asset-database-and-import-pipeline.md`
- Action hooks registries (policy in components): `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`
- Ecosystem integration guidance (non-binding): `docs/adr/0113-ecosystem-integration-contracts.md`
- Zed/GPUI driver-style references (non-normative):
  - app-owned effect cycle and deferred work:
    `repo-ref/zed/crates/gpui/src/app.rs` (`Effect`, `App::defer`)
  - window scheduling + draw/present and input dispatch loop:
    `repo-ref/zed/crates/gpui/src/window.rs`
  - integration glue and the “real app” surface:
    `repo-ref/zed/crates/gpui_tokio`, `repo-ref/zed/crates/zed`

