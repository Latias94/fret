# ADR 0105: Dev Hotpatch (Subsecond) Integration and Hot-Reload Safety Rules

Status: Accepted

## Context

Fret's UI runtime is trending toward a GPUI-style declarative model:

- authoring rebuilds an element tree each frame (ADR 0028),
- identity and cross-frame state are preserved via stable IDs and externalized state stores (ADR 0028),
- component ergonomics rely on `Render` / `RenderOnce` + `IntoElement` (ADR 0039),
- interaction policy is component-owned and expressed via runtime *action hooks* (ADR 0074, `docs/action-hooks.md`).

In development, we want a faster "edit -> observe" loop for native desktop apps. One promising direction is
**hotpatching**: patching function bodies in a running process without restarting it.

`repo-ref/dioxus/packages/subsecond` is a reference implementation of a Rust hotpatch engine:

- it detours calls through a jump table containing "latest" function pointers,
- it requires an external tool to build and deliver patches,
- it has meaningful safety/compatibility constraints (notably around struct layout and long-lived code pointers).

Fret has additional risk factors compared to short-lived request/response servers:

- UI runtimes often store closures and callbacks in retained structures (e.g. action hooks stored on nodes),
- timers/raf callbacks may retain closures across frames,
- window states and per-window UI runtimes are long-lived and often contain user-defined types.

If hotpatching changes code or capture layouts, any retained closure/function pointer from the "old world" may:

- continue calling stale code (logical correctness failure), or
- become invalid/ABI-incompatible (crash/UB risk).

We need a **dev-only** hotpatch strategy that:

- does not change core framework contracts,
- keeps `crates/fret-ui` mechanism-only (ADR 0074),
- gives the runner a well-defined "clean re-entry boundary" after patches,
- is conservative by default (safety over preserving state).

## Goals

- Provide an *optional*, dev-focused hotpatch integration point aligned with Subsecond-style patching.
- Define "hot-reload safety rules" that prevent invoking stale retained callbacks after a patch.
- Provide a runner-level authoring surface that is *hotpatch-friendly* (function-pointer based entry points).
- Keep core contracts stable and portable across native and wasm backends.

## Non-goals

- Shipping/production hotpatch support.
- Automated state migration across patches (ABI/layout compatibility is not assumed).
- Rewriting the UI runtime to be "hotpatch-native"; we prefer bounded, conservative integration at the runner boundary.
- Defining the external compiler/protocol toolchain (we only define how Fret *consumes* patches).

## Constraints (Alignment With Existing ADRs)

- **Mechanism-only runtime**: interaction policy remains in component layers (ADR 0074); this ADR only specifies
  how to safely discard/rebuild runtime state after a patch.
- **Declarative render contract**: declarative paths must still call `render_root(...)` once per frame before
  layout/paint (ADR 0028); hotpatch must not introduce "skip render" behavior.
- **Resource handles**: UI code continues to use stable IDs (`ImageId`, `SvgId`, `TextBlobId`, etc.) and effects-based
  registration at flush points (ADR 0004). Hot reload does not change the resource boundary; it only schedules redraws
  and rebuilds UI runtime state.
- **Crate layering**: hotpatch support may live in `crates/fret-launch` (non-kernel glue per ADR 0092), but must not
  leak dev-only dependencies into kernel crates (`fret-core`, `fret-runtime`, `fret-ui`, `fret-app`).

## Decision

### 1) Hotpatch integration lives in the runner glue layer and is dev-only

We introduce an **optional** hotpatch integration feature in the launcher/runner layer:

- Location: `crates/fret-launch` (desktop runner integration), behind a Cargo feature (e.g. `hotpatch-subsecond`).
- Default: off.
- Scope: native desktop first. wasm/devserver reload remains the default for web.

Rationale:

- `fret-launch` is explicitly "glue" and not part of the portable kernel (ADR 0092).
- Keeping hotpatch dependencies out of kernel crates preserves portability and reduces long-term maintenance risk.

### 2) Provide a hotpatch-friendly runner authoring surface (function-pointer driver)

We add a **function-pointer based** driver entry point in `fret-launch` in addition to the existing
`WinitAppDriver` trait-based integration.

API shape (implemented in this workspace):

- `FnDriver<S>` contains function pointers for:
  - `create_window_state: fn(&mut App, AppWindowId) -> S`
  - `handle_event: fn(WinitEventContext<'_, S>, &Event)`
  - `render: fn(WinitRenderContext<'_, S>)`
  - optional: `window_create_spec`, `window_created`, accessibility hooks, etc.

Additionally, both driver styles expose dev-only hot reload hooks:

- `WinitAppDriver::{hot_reload_global, hot_reload_window}`
- `FnDriverHooks::{hot_reload_global, hot_reload_window}`
  - `hot_reload_window` receives a `WinitHotReloadContext<'_, S>` for convenient per-window resets.

`FnDriver<S>` is runner-level only; it does not change `fret-ui` authoring model (ADR 0039).

Implementation note:

- Prefer `FnDriver` as the primary authoring surface for hotpatch-enabled apps.
- Keep the trait-based driver only as a compatibility layer until in-tree apps migrate.

Rationale:

- Subsecond-style hotpatching is fundamentally about detouring function pointers.
- Trait methods and captured closures tend to end up behind vtables or in struct layouts that are difficult to
  patch safely and predictably.
- A function-pointer surface makes "hot anchors" explicit and reduces accidental retention of stale code pointers.

### 3) Define explicit hot anchors

When `hotpatch-subsecond` is enabled, the runner treats the following as **hot anchors**:

- `render` (per-frame; primary clean boundary)
- `handle_event` (per input event; secondary boundary)
- optionally `create_window_state` (window creation; coarse boundary)

The runner is responsible for:

- detecting when any anchor has been patched (e.g. via `HotFn::changed()`),
- invoking a hot-reload safety procedure before continuing execution in the new code.

### 4) Hot-reload safety rules (required behavior)

When a hot patch is detected or applied, the runner MUST ensure that no retained callback/closure from the "old world"
remains callable.

We define a runner-level procedure:

#### `HotReload` procedure (default: safety-first)

1) Increment a global `hot_reload_generation` counter (debug/observability).
2) For each window:
   - request an immediate redraw (`Effect::Redraw` or `Effect::RequestAnimationFrame`) to reach the next `render` anchor quickly.
3) For each window, rebuild or reset all runtime structures that may retain code pointers:
   - discard/recreate the per-window declarative runtime state (element-id -> node-id mapping, per-node hook tables),
   - discard/recreate `UiTree` (or an equivalent "hard reset" operation that guarantees all registered action hooks are dropped),
   - discard/recreate overlay controllers, outside-press observers, and any other long-lived policy surfaces that store callbacks.
4) Window state handling (critical for safety):
   - Default: **do not rebuild window state**. Instead, call a dev-only hot reload hook to reset retained UI runtime
     state in place (e.g. discard/recreate `UiTree`, clear cached node IDs, close overlays) while preserving app models.
   - Default drop policy: **do not drop** the old UI runtime state (leak) to avoid running potentially incompatible drop
     code after a patch.
   - Allow an opt-in "drop old state" mode for advanced users who accept the risk and enforce ABI stability themselves.

Notes:

- This is intentionally conservative. In dev mode, correctness and process safety outweigh preserving ephemeral state.
- This rule is compatible with ADR 0028: it forces the system back to the frame boundary where the element tree is rebuilt,
  which is the natural "clean" re-entry point in a declarative system.

### 5) Action hooks are retained as a mechanism; hot reload treats registrations as disposable

Action hooks remain the mechanism for component-owned policy (ADR 0074).

Hot reload introduces a dev-only contract:

- Action hook registrations are **runtime caches**. On hot reload, all existing registrations must be discarded.
- The next frame's render will re-register hooks from the patched code.

This explicitly answers the common question "should we remove action hooks to support hot reload?":

- No. Action hooks are the mechanism that keeps `crates/fret-ui` policy-free (ADR 0074).
- Hot reload safety comes from discarding the old hook registry (and other retained callback surfaces) at a runner-level
  reset boundary, not from removing the mechanism.

Future optimization (not required for this ADR):

- replace "store closures on nodes" with `ActionHookId` + a registry so hot reload can swap registries without rebuilding
  the whole UI tree. This is an optimization, not a correctness requirement.

### 6) wasm development workflow remains devserver reload first

For `wasm32`, the default "fast iteration" path remains "rebuild + reload" via a devserver (e.g. `trunk serve`).
This ADR does not require Subsecond integration on wasm.

### 7) Practical dev-loop policy: prefer safe restart over view-level Subsecond on Windows

Subsecond remains the preferred mechanism for hotpatching **logic-level** anchors (command/event handling, small driver hooks).
However, on Windows we currently treat **view-level** Subsecond hotpatching as best-effort due to a reproducible
stack-overflow failure mode when calling a patched `ViewFn` via `subsecond::HotFn`.

Decision:

- Default behavior (Windows, dev/hotpatch): do not require `ViewFn` to be called through Subsecond.
- If a Subsecond patch is applied and the platform is in a known-bad configuration for view hotpatching, the system SHOULD
  fall back to a safe boundary (either bypass the patched view call or restart the app).
- A developer MAY opt into view-level Subsecond calling for experimentation, with the expectation that it may crash until
  the upstream issue is resolved.

Rationale:

- The primary objective of dev hotpatch is iteration speed **without destabilizing the process**.
- A fast restart (while preserving window state and local `.fret/` state) is often a better UX than a crash loop.

### 8) `fretboard` is the supervisor of the dev loop (including restart fallback)

We treat `apps/fretboard` as the orchestration layer for native dev iteration:

- It may start/stop external tooling (`dx serve --hotpatch`) when configured.
- It may detect "patch applied but unsafe to continue" situations and automatically fall back to a fast restart.
- It may preserve/replay dev ergonomics across restarts (window placement, `.fret/` state, and other safe-to-restore surfaces).

This keeps dev tooling concerns out of kernel crates while providing a consistent, framework-controlled UX for users.

### 9) UI hot-reload is not the same as Rust hotpatch: add stable non-Subsecond reload channels

To achieve "edit -> observe" for UI without relying on Subsecond patch stability, we explicitly separate:

- **Theme reload** (tokens, colors, radii, spacing, typography): reloadable at runtime in dev.
- **Asset reload** (svg/png/fonts): reloadable at runtime in dev with cache invalidation.
- **Hot literals** (developer-overridable strings/labels): reloadable at runtime in dev.

These channels are intended to provide the common "small UI tweaks" loop while Subsecond focuses on Rust logic hotpatching.
They also provide a coherent foundation for third-party component ecosystems (components read from Theme/Assets/Literals
instead of baking values into Rust code that must be hotpatched).

## Tooling Notes (Fret Workspace)

This repository ships a small dev helper CLI (`apps/fretboard`) that can run demos with hotpatch enabled.

- File trigger (no external devserver required):
  - Start: `fretboard dev native --bin todo_demo --hotpatch`
  - Trigger reload: `fretboard hotpatch poke` (updates `.fret/hotpatch.touch`)
  - Auto trigger: `fretboard hotpatch watch` (polls workspace sources and pokes on change)
- Devserver websocket (Dioxus-style):
  - Start: `fretboard dev native --bin todo_demo --hotpatch-devserver ws://127.0.0.1:8080/_dioxus`
  - Start (end-to-end patches via dioxus-cli): `fretboard dev native --bin hotpatch_smoke_demo --hotpatch-dx`

Important:

- `--hotpatch` (file-trigger) only triggers a **safe hot-reload boundary** in the runner. It does not compile or
  deliver Subsecond patches by itself.
- Subsecond patches take effect only for **function-pointer based entry points** that are called through the Subsecond
  jump table (ADR 0105). In this workspace, the recommended hotpatch-ready path is the `FnDriver`/`UiAppDriver` based
  demos (e.g. `todo_demo`, `assets_demo`). Older demos that implement `WinitAppDriver` directly are not guaranteed to
  execute patched code.

This is tooling-only; no kernel crates depend on it (ADR 0092 / ADR 0106).

Notes:

- `--hotpatch-dx` requires `dx` (dioxus-cli) to be installed and uses `dx serve --hotpatch` under the hood.
- `--hotpatch-dx-ws` configures the devserver bind address/port for `dx` (it does not change Fret's websocket protocol;
  Fret still speaks the Dioxus devserver protocol).

## Known Issues

- **Windows: Subsecond patch can crash at the first patched `ViewFn` call**
  - Symptom: after `dx` reports `Hot-patching: ...`, the app may exit with `0xc000041d` and Rust reports a stack overflow.
  - Status: patch delivery and runner hot-reload reset succeed; the overflow happens during the first `HotFn::call(...)` into the patched view.
  - Diagnostics:
    - Enable `FRET_HOTPATCH_DIAG_BYTES=1` to capture the mapped module path and prologue bytes in `.fret/hotpatch_bootstrap.log`.
    - The patched view entrypoint is mapped into `lib*-patch-*.dll` and its prologue includes a large stack frame with a ThinLink thunk that jumps into the base EXE (stack-probe style).
  - Workarounds:
    - Set `FRET_HOTPATCH_VIEW_CALL_DIRECT=1` to bypass `HotFn` for the `ViewFn` call (prevents the crash but disables view-level hotpatching).
    - Prefer full rebuild + restart (`r` in `dx serve`) for now when iterating on view code on Windows.
  - Tracking: `docs/todo-tracker.md` ("Hotpatch golden path validation loop").

## Alternatives Considered

### A) Remove action hooks entirely

Rejected.

Action hooks are central to keeping `fret-ui` mechanism-only while enabling component-owned policy (ADR 0074).
Removing them would either:

- push policy back into the runtime (contract bloat), or
- force each component to reimplement bespoke event routing/policy logic.

### B) Hotpatch without rebuilding the UI runtime

Rejected for P0.

Without an explicit "drop all retained callbacks" step, stale closures can survive indefinitely and remain callable.
Selective replacement is complex and easy to get wrong.

### C) Only support trait-based drivers

Rejected as the recommended path.

Trait methods and vtable-based call sites are harder to patch predictably and encourage retaining code pointers in
long-lived objects.

We keep the trait-based driver for compatibility, but recommend the function-pointer driver for hotpatch mode.

## Consequences

### Benefits

- Dev hotpatch is possible without changing kernel contracts or contaminating `fret-ui` with dev-tool dependencies.
- Hot reload behavior becomes predictable: patches take effect at well-defined anchors, and runtime caches are rebuilt.
- Action hooks remain intact and compatible with hotpatch by construction.

### Costs / Trade-offs

- Default safety policy may reset/leak per-window state after a patch.
- Rebuilding `UiTree` can be expensive for large UIs; acceptable in dev, and can be optimized later.
- Requires careful runner engineering to ensure *all* retained callback surfaces participate in the reset procedure.

## Implementation Status (Current)

Implemented:

1) Runner-level dev hooks in `crates/fret-launch`:
   - trait surface: `WinitAppDriver::{hot_reload_global, hot_reload_window}`
   - fn-driver surface: `FnDriverHooks::{hot_reload_global, hot_reload_window}`
2) Desktop trigger sources:
   - manual: `Ctrl+Shift+R`
   - Subsecond patch-complete signal (feature-gated): `--features hotpatch-subsecond` + `FRET_HOTPATCH=1`
     - runner registers `subsecond::register_handler` and schedules a safe runner-side reset at the next event-loop turn
   - optional devserver listener (feature-gated): set `FRET_HOTPATCH_DEVSERVER_WS=ws://...`
     - listens for Dioxus-style devserver messages (`DevserverMsg::HotReload`) and applies incoming `JumpTable`s via `subsecond::apply_patch`
     - optional filter: set `FRET_HOTPATCH_BUILD_ID=<u64>` and require the devserver message `for_build_id` to match
   - fallback polling trigger (feature-gated): `FRET_HOTPATCH_TRIGGER_PATH` (+ `FRET_HOTPATCH_POLL_MS`)
     - when `FRET_HOTPATCH_DEVSERVER_WS` is set, the default touch-file polling is disabled unless `FRET_HOTPATCH_TRIGGER_PATH` is explicitly provided
3) Safety-first default: demos leak old per-window UI runtime state after a reload to avoid post-patch drop ABI risk.
   - opt-in demo switch: `FRET_HOTPATCH_DROP_OLD_STATE=1`

Remaining:

- Document and standardize build-id filtering for devserver-driven patches (to avoid cross-process confusion in multi-app workflows).
- Expand the "hard reset" coverage list (timers/drag sessions/long-lived callback registries) as those subsystems land.

## Open Questions

- What is the minimal "hard reset" API on `UiTree` that guarantees all hooks are dropped without forcing full allocation?
- Should the default policy leak old window state, or attempt a best-effort drop behind a feature flag?
- Which long-lived subsystems need explicit reset hooks (timers, drag sessions, platform completion handlers)?

## References

- Declarative execution model: `docs/adr/0028-declarative-elements-and-element-state.md`
- Component authoring model: `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`
- Action hooks (policy in components): `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`
- Resource handles + flush point: `docs/adr/0004-resource-handles.md`
- Crate layering: `docs/adr/0092-crate-structure-core-backends-apps.md`
- Subsecond reference: `repo-ref/dioxus/packages/subsecond`
