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

## References

- Component ecosystem conventions: `docs/adr/0158-component-ecosystem-authoring-conventions-v1.md`
- Component authoring contracts (mechanism surface): `docs/component-authoring-contracts.md`
- Crate map (what to depend on): `docs/crate-usage-guide.md`
- Ecosystem integration guidance: `docs/adr/0113-ecosystem-integration-contracts.md`
