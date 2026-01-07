# ADR 0107: Dev Hotpatch (Subsecond) Integration and Hot-Reload Safety Rules

Status: Proposed

## Context

Fret’s UI runtime is trending toward a GPUI-style declarative model:

- authoring rebuilds an element tree each frame (ADR 0028),
- identity and cross-frame state are preserved via stable IDs and externalized state stores (ADR 0028),
- component ergonomics rely on `Render` / `RenderOnce` + `IntoElement` (ADR 0039),
- interaction policy is component-owned and expressed via runtime *action hooks* (ADR 0074, `docs/action-hooks.md`).

In development, we want a faster “edit → observe” loop for native desktop apps. One promising direction is
**hotpatching**: patching function bodies in a running process without restarting it.

`repo-ref/dioxus/packages/subsecond` is a reference implementation of a Rust hotpatch engine:

- it detours calls through a jump table containing “latest” function pointers,
- it requires an external tool to build and deliver patches,
- it has meaningful safety/compatibility constraints (notably around struct layout and long-lived code pointers).

Fret has additional risk factors compared to short-lived request/response servers:

- UI runtimes often store closures and callbacks in retained structures (e.g. action hooks stored on nodes),
- timers/raf callbacks may retain closures across frames,
- window states and per-window UI runtimes are long-lived and often contain user-defined types.

If hotpatching changes code or capture layouts, any retained closure/function pointer from the “old world” may:

- continue calling stale code (logical correctness failure), or
- become invalid/ABI-incompatible (crash/UB risk).

We need a **dev-only** hotpatch strategy that:

- does not change core framework contracts,
- keeps `crates/fret-ui` mechanism-only (ADR 0074),
- gives the runner a well-defined “clean re-entry boundary” after patches,
- is conservative by default (safety over preserving state).

## Goals

- Provide an *optional*, dev-focused hotpatch integration point aligned with Subsecond-style patching.
- Define “hot-reload safety rules” that prevent invoking stale retained callbacks after a patch.
- Provide a runner-level authoring surface that is *hotpatch-friendly* (function-pointer based entry points).
- Keep core contracts stable and portable across native and wasm backends.

## Non-goals

- Shipping/production hotpatch support.
- Automated state migration across patches (ABI/layout compatibility is not assumed).
- Rewriting the UI runtime to be “hotpatch-native”; we prefer bounded, conservative integration at the runner boundary.
- Defining the external compiler/protocol toolchain (we only define how Fret *consumes* patches).

## Constraints (Alignment With Existing ADRs)

- **Mechanism-only runtime**: interaction policy remains in component layers (ADR 0074); this ADR only specifies
  how to safely discard/rebuild runtime state after a patch.
- **Declarative render contract**: declarative paths must still call `render_root(...)` once per frame before
  layout/paint (ADR 0028); hotpatch must not introduce “skip render” behavior.
- **Resource handles**: UI code continues to use stable IDs (`ImageId`, `SvgId`, `TextBlobId`, etc.) and effects-based
  registration at flush points (ADR 0004). Hot reload does not change the resource boundary; it only schedules redraws
  and rebuilds UI runtime state.
- **Crate layering**: hotpatch support may live in `crates/fret-launch` (non-kernel glue per ADR 0093), but must not
  leak dev-only dependencies into kernel crates (`fret-core`, `fret-runtime`, `fret-ui`, `fret-app`).

## Decision

### 1) Hotpatch integration lives in the runner glue layer and is dev-only

We introduce an **optional** hotpatch integration feature in the launcher/runner layer:

- Location: `crates/fret-launch` (desktop runner integration), behind a Cargo feature (e.g. `hotpatch-subsecond`).
- Default: off.
- Scope: native desktop first. wasm/devserver reload remains the default for web.

Rationale:

- `fret-launch` is explicitly “glue” and not part of the portable kernel (ADR 0093).
- Keeping hotpatch dependencies out of kernel crates preserves portability and reduces long-term maintenance risk.

### 2) Provide a hotpatch-friendly runner authoring surface (function-pointer driver)

We add a **function-pointer based** driver entry point in `fret-launch` in addition to the existing
`WinitAppDriver` trait-based integration.

Proposed API shape (names subject to bikeshedding):

- `FnDriver<S>` contains function pointers for:
  - `create_window_state: fn(&mut App, AppWindowId) -> S`
  - `handle_event: fn(WinitEventContext<'_, S>, &Event)`
  - `render: fn(WinitRenderContext<'_, S>)`
  - optional: `window_create_spec`, `window_created`, accessibility hooks, etc.

`FnDriver<S>` is runner-level only; it does not change `fret-ui` authoring model (ADR 0039).

Rationale:

- Subsecond-style hotpatching is fundamentally about detouring function pointers.
- Trait methods and captured closures tend to end up behind vtables or in struct layouts that are difficult to
  patch safely and predictably.
- A function-pointer surface makes “hot anchors” explicit and reduces accidental retention of stale code pointers.

### 3) Define explicit hot anchors

When `hotpatch-subsecond` is enabled, the runner treats the following as **hot anchors**:

- `render` (per-frame; primary clean boundary)
- `handle_event` (per input event; secondary boundary)
- optionally `create_window_state` (window creation; coarse boundary)

The runner is responsible for:

- detecting when any anchor has been patched (e.g. via `HotFn::changed()`),
- invoking a hot-reload safety procedure before continuing execution in the new code.

### 4) Hot-reload safety rules (required behavior)

When a hot patch is detected or applied, the runner MUST ensure that no retained callback/closure from the “old world”
remains callable.

We define a runner-level procedure:

#### `HotReload` procedure (default: safety-first)

1) Increment a global `hot_reload_generation` counter (debug/observability).
2) For each window:
   - request an immediate redraw (`Effect::Redraw` or `Effect::RequestAnimationFrame`) to reach the next `render` anchor quickly.
3) For each window, rebuild or reset all runtime structures that may retain code pointers:
   - discard/recreate the per-window declarative runtime state (element-id → node-id mapping, per-node hook tables),
   - discard/recreate `UiTree` (or an equivalent “hard reset” operation that guarantees all registered action hooks are dropped),
   - discard/recreate overlay controllers, outside-press observers, and any other long-lived policy surfaces that store callbacks.
4) Window state handling (critical for safety):
   - Default: create a new window state via `create_window_state` and replace the old state.
   - Default drop policy: **do not drop** the old window state (leak) to avoid running potentially incompatible drop code.
   - Allow an opt-in “drop old state” mode for advanced users who accept the risk and enforce ABI stability themselves.

Notes:

- This is intentionally conservative. In dev mode, correctness and process safety outweigh preserving ephemeral state.
- This rule is compatible with ADR 0028: it forces the system back to the frame boundary where the element tree is rebuilt,
  which is the natural “clean” re-entry point in a declarative system.

### 5) Action hooks are retained as a mechanism; hot reload treats registrations as disposable

Action hooks remain the mechanism for component-owned policy (ADR 0074).

Hot reload introduces a dev-only contract:

- Action hook registrations are **runtime caches**. On hot reload, all existing registrations must be discarded.
- The next frame’s render will re-register hooks from the patched code.

Future optimization (not required for this ADR):

- replace “store closures on nodes” with `ActionHookId` + a registry so hot reload can swap registries without rebuilding
  the whole UI tree. This is an optimization, not a correctness requirement.

### 6) wasm development workflow remains devserver reload first

For `wasm32`, the default “fast iteration” path remains “rebuild + reload” via a devserver (e.g. `trunk serve`).
This ADR does not require Subsecond integration on wasm.

## Alternatives Considered

### A) Remove action hooks entirely

Rejected.

Action hooks are central to keeping `fret-ui` mechanism-only while enabling component-owned policy (ADR 0074).
Removing them would either:

- push policy back into the runtime (contract bloat), or
- force each component to reimplement bespoke event routing/policy logic.

### B) Hotpatch without rebuilding the UI runtime

Rejected for P0.

Without an explicit “drop all retained callbacks” step, stale closures can survive indefinitely and remain callable.
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

## Implementation Plan (Sketch)

1) Add `FnDriver<S>` and a runner adapter (no Subsecond dependency).
2) Add feature-gated Subsecond integration in `fret-launch`:
   - wrap anchors with hot-function indirection,
   - detect changes and trigger `HotReload`.
3) Implement `HotReload`:
   - per-window UI runtime reset,
   - state replacement strategy + leak/drop policy,
   - force redraw.
4) Add a small demo harness in `apps/` that exercises hot reload boundaries (render + event) and validates
   that action hooks do not survive a reload.

## Open Questions

- What is the minimal “hard reset” API on `UiTree` that guarantees all hooks are dropped without forcing full allocation?
- Should the default policy leak old window state, or attempt a best-effort drop behind a feature flag?
- Which long-lived subsystems need explicit reset hooks (timers, drag sessions, platform completion handlers)?

## References

- Declarative execution model: `docs/adr/0028-declarative-elements-and-element-state.md`
- Component authoring model: `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`
- Action hooks (policy in components): `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`
- Resource handles + flush point: `docs/adr/0004-resource-handles.md`
- Crate layering: `docs/adr/0093-crate-structure-core-backends-apps.md`
- Subsecond reference: `repo-ref/dioxus/packages/subsecond`

