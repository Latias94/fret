# Component Author Guide (Fret Ecosystem)

This document is for **component and ecosystem authors** who want to build reusable libraries on
top of Fret (widgets, design systems, domain UIs like node graphs/charts/gizmos).

It focuses on *what to do* and *what to avoid* so your crate stays composable, portable, and
future-proof.

For deeper mechanism-level details, see:

- `docs/component-authoring-contracts.md`
- `docs/runtime-contract-matrix.md`

## 1) Choose the right dependency layer

Recommended layering:

- Pure data/engine tier (optional): depend on `fret-core` / `fret-runtime` only.
- UI integration tier: depend on `fret-ui` (and optionally `fret-ui-kit` for headless primitives and policy helpers).

Avoid depending on backend crates (`fret-launch`, `winit`, `wgpu`, `web-sys`) unless your crate is
explicitly a runner/tooling crate.

### Typical dependency sets (examples)

**A) Headless / engine crate**

Use this for algorithms, selection models, layout computation, and domain state machines.

```toml
[dependencies]
fret-core = { path = "../crates/fret-core" }
fret-runtime = { path = "../crates/fret-runtime" }
```

**B) UI integration crate (portable UI)**

Use this for retained UI elements and interaction policy via hooks/primitives.

```toml
[dependencies]
fret-ui = { path = "../crates/fret-ui" }
fret-ui-kit = { path = "../ecosystem/fret-ui-kit", optional = true }

[features]
default = []
kit = ["dep:fret-ui-kit"]
```

**C) Optional app integration (commands/default keybindings/config files)**

Most component libraries do *not* need `fret-app`. Only enable it when you register commands,
default keybindings, settings schemas, etc.

```toml
[dependencies]
fret-app = { path = "../crates/fret-app", optional = true }

[features]
default = []
app-integration = ["dep:fret-app"]
```

**D) Icons (semantic IDs, app-chosen packs)**

If your components render icons, depend on the registry contract (`fret-icons`) and use semantic `IconId`s.
Do **not** depend on a specific vendor pack (`fret-icons-lucide` / `fret-icons-radix`) unless your crate is
explicitly a pack or a demo.

```toml
[dependencies]
fret-icons = { path = "../ecosystem/fret-icons" }
```

## 2) Provide a single, explicit integration entry point

If your crate registers commands, tokens, settings, or asset helpers, expose:

```rust
pub fn install(app: &mut fret_app::App, services: &mut dyn fret_core::UiServices)
```

Rules:

- Idempotent: calling twice should not double-register or double-install default bindings.
- No hidden side-channels: prefer commands/effects/models instead of global singletons.

Practical note:

- Consider gating this behind a feature (e.g. `app-integration`) so pure UI crates can remain `fret-ui`-only.
  Feature names are a convention, not a requirement.

### 2.1) (Recommended) Opt into the unified `ui()` builder surface

If your component type is meant to be consumed by third-party crates and apps, prefer opting into
the ecosystem-level fluent authoring surface (ADR 0175):

- `value.ui().px_3().py_2().w_full().into_element(cx)`

This keeps styling/layout composition consistent across the ecosystem and reduces “one-off” wrapper
utilities that fragment authoring.

Minimal integration contract (ecosystem-level, stable-ish):

- Implement `fret_ui_kit::UiPatchTarget` so `.ui()` becomes available via `fret_ui_kit::UiExt`.
- Implement `fret_ui_kit::UiIntoElement` so `.into_element(cx)` is available from `UiBuilder`.
- Optionally implement `fret_ui_kit::UiSupportsChrome` / `UiSupportsLayout` to enable the full
  fluent method set (padding, sizing, radius, etc.).

Guidance:

- Keep your public constructor surface “policy-free” (no implicit theme install, no global hooks).
- Accept children as `impl IntoIterator<Item = AnyElement>` and store them as `Vec<AnyElement>`
  internally (call-site flexibility; internal stability).

## 3) Commands + shortcuts: always go through `CommandId` + keymap

If an action can be triggered by keyboard/menu/palette, it should be a `CommandId`.

Guidelines:

- Namespace command IDs: `crate.scope.action` (e.g. `node_graph.add_node`).
- Put default shortcuts into `CommandMeta.default_keybindings`.
- Use `when` expressions to guard context (e.g. disable global shortcuts when focus is in text input).

Avoid:

- Hard-coding shortcut behavior by intercepting raw key-down events inside component rendering.

## 4) Theme tokens: no hard-coded palettes

Your component’s appearance must be theme-driven:

- resolve colors/metrics via theme keys,
- document your key namespace and meaning,
- provide fallbacks.

Avoid:

- hard-coded RGB/spacing constants as the primary styling path.

## 5) Units: be explicit (logical px vs physical px)

Baseline rules:

- UI layout/input uses logical pixels (`Px`) (DPI-aware via scale factor).
- Render targets and engine buffers operate in physical pixels.

If your crate crosses the viewport/tooling boundary, expose unit-explicit APIs and carry enough
context to avoid ad-hoc conversions (see the viewport explicit-units contract).

Concrete reference:

- Gizmo + viewport integration (Tier A tooling boundary): `docs/gizmo-viewport-integration.md`

### Gizmos and viewport tooling (Tier A boundary, plugin surface)

`ecosystem/fret-gizmo` is **not** a `fret-ui` widget. It is engine/driver-owned viewport tool logic
that stays backend-agnostic and unit-explicit (ADR 0139).

If you are authoring reusable gizmo tooling:

- Prefer depending on `fret-gizmo` (and `fret-core` if you need shared IDs/types).
- Avoid `wgpu`/`winit` dependencies; rendering is owned by the engine pass.
- Use the plugin contract (`GizmoPlugin`, ADR 0155) for extensibility.
- When your plugin needs to read domain values (e.g. light radius readouts), query the host via
  `GizmoPropertySource` (read-only, ADR 0167) instead of requiring host push caches.
- Emit edits via `GizmoCustomEdit` and let the host apply validation + undo/redo (write policy is
  intentionally host-owned in v1).
- Treat gizmos as **viewport tools**, not UI widgets:
  - Share the portable tool protocol via `ecosystem/fret-viewport-tooling`.
  - Use `fret-ui-kit` host helpers (`viewport_tooling`) to arbitrate camera vs selection vs gizmo
    and to route `ViewportInputEvent` streams into stable hot/active/capture decisions.

## 6) Settings: namespaced, layered, and optional

If you support config:

- define a namespaced settings section,
- provide defaults and schema guidance,
- treat layered loading (user + project) as an app-level decision.

Avoid:

- reading files directly from arbitrary paths inside components.

## 7) Accessibility (A11y): choose a role and label

For interactive widgets:

- set an appropriate semantics role,
- provide a label/name,
- reflect state (selected/expanded/checked) where relevant,
- stamp collection metadata for list-like widgets when applicable.

## 8) Tests: add at least one conformance test for “hard contracts”

Examples:

- selection/roving focus rules,
- overlay dismissal/focus restore behavior,
- command routing invariants,
- token resolution fallback behavior.

## 9) Interactivity pseudoclasses (hover/focus/pressed): keep structure stable

Rule of thumb: treat `:hover`, `:focus`, `:focus-visible`, and `:active` as **style inputs**, not as
"different trees".

Why this matters:

- It keeps hover/focus chrome paint-only by default (better cache reuse and fewer layout invalidations).
- It avoids flicker and “feels random” regressions when view-cache reuse skips subtree execution.

Practical guidance:

- Do not add/remove nodes on hover/focus/pressed. Keep the subtree shape stable.
- Reserve space for transient chrome (toolbars, affordances) and fade it in/out via `Opacity` instead of
  inserting/removing elements.
- Use `InteractivityGate` to make closing overlays or transitioning surfaces pointer-transparent without
  unmounting them.
- If a pseudoclass edge must change intrinsic size, treat it as an explicit opt-in and document why (layout
  invalidation should be a conscious trade-off, not an accident).

Contract references:

- ADR 0181: `docs/adr/0181-interactivity-pseudoclasses-and-structural-stability.md`
- View-cache semantics: `docs/adr/1152-cache-roots-and-cached-subtree-semantics-v1.md`

## 10) View-cache boundaries: make caching explicit, keep render pure

If a subtree is expensive (panels, inspectors, code views, large lists), consider adding an explicit cache boundary.

Recommended helper (ecosystem sugar over the `fret-ui` mechanism):

- `fret_ui_kit::declarative::CachedSubtreeExt::{cached_subtree,cached_subtree_with}`

Rules of thumb:

- Keep the cached subtree's render closure as pure as possible: avoid hidden side effects that must run every frame.
- If the subtree emits per-frame requests/registries (overlays, observers, frame-local caches), ensure the contract
  is closed under cache-hit frames (either by cached synthesis or by moving the side effect outside the cached region).

## References

- Component ecosystem conventions: `docs/adr/0163-component-ecosystem-authoring-conventions-v1.md`
- Component authoring contracts (mechanism surface): `docs/component-authoring-contracts.md`
- Crate map (what to depend on): `docs/crate-usage-guide.md`
- Ecosystem integration guidance: `docs/adr/0113-ecosystem-integration-contracts.md`
