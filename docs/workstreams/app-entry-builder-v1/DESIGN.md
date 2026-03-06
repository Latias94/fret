# App Entry Builder v1 (Refactor Design)

## Context

Fret’s current “golden path” for small desktop-first apps is intentionally **hotpatch-friendly** and
contract-driven:

- The default runtime wiring prefers `fn` pointers (predictable dev hot reload / Subsecond-style
  patching).
- Policy-heavy behavior (dismissal rules, focus trap/restore, sizing defaults, etc.) lives in
  ecosystem crates (e.g. `fret-ui-kit`, `fret-ui-shadcn`), not `crates/fret-ui`.

This workstream proposes a **single, ergonomic, user-facing entry surface** that:

1) reads like `fret::App::new().window(...).run()` (builder chain),
2) keeps the underlying driver **fn-pointer based** by default,
3) makes ecosystem extension points explicit and easy to discover,
4) reduces feature-flag cognition for first-time users.

This is a documentation-first design. No code changes are implied until this document is approved.

## Goals

- **Onboarding ergonomics:** a first app should require minimal imports and minimal boilerplate.
- **Stable mental model:** “I depend on `fret` and I can ship a desktop UI app” should be true with
  default features, without reading a feature matrix on day 1.
- **Explicit extension seams:** users can install additional policy surfaces (router, workspace
  shell, custom icon packs, diagnostics, ui-assets) without “dropping down” too early.
- **Preserve hotpatch posture:** the default entry stays compatible with the existing `fn` pointer
  golden path and ADRs around hot reload safety.
- **No layering violations:** mechanism stays in `crates/*`, policy stays in `ecosystem/*`.

## Non-goals

- Replacing the internal driver architecture (`fret-bootstrap` / `UiAppDriver`) in v1.
- Forcing a GPUI-style closure-driven runtime everywhere.
- Collapsing or redesigning the overall crate layering (core vs ecosystem).

## Constraints / Design Rules

- Default entry must be buildable using **function pointers** (no captured closures) for core
  lifecycle hooks.
- Any “closure entry” (if offered) must be explicitly opt-in and clearly documented as **not
  hotpatch-friendly**.
- API should not require users to know about `winit`, `wgpu`, effect flushing, or runner internals
  for common apps.

## Proposed User-Facing API (Sketch)

### High-level shape

We introduce a new builder type in the `fret` facade crate (ecosystem-level):

```rust
use fret::prelude::*;

fn main() -> fret::Result<()> {
    fret::App::new("hello")
        .window("Hello", (560.0, 360.0))
        .mvu::<HelloProgram>()
        .run()
}
```

Key points:

- `fret::App` is a **builder facade**, not `fret_app::App`.
- The builder methods configure the existing golden-path wiring (`fret-bootstrap`) underneath.
- The `mvu::<P>()` variant preserves the current “typed messages” authoring posture.
- A non-MVU variant is also available for users who prefer `init_window + view` functions.

### MVU entry (default recommendation)

```rust
fret::App::new("todo")
    .window("Todo", (560.0, 520.0))
    .mvu::<TodoProgram>()
    .run()
```

### UI entry (init + view functions)

```rust
fret::App::new("hello")
    .window("Hello", (560.0, 360.0))
    .ui(init_window, view)
    .run()
```

### Ecosystem extension points (examples)

```rust
fret::App::new("my-app")
    .window("My App", (960.0, 720.0))
    .defaults(fret::Defaults::desktop_batteries()) // see “Feature Strategy”
    .install_app(|app| {
        // app-owned globals, commands, services
    })
    .install(|app, services| {
        // wiring with UiServices (assets, rendering hooks, etc.)
    })
    .mvu::<MyProgram>()
    .run()
```

Notes:

- `install_app` and `install` are intentionally ecosystem-level and should map to existing
  `BootstrapBuilder` entry points.
- The builder should expose a small set of **first-class knobs** (icons, ui-assets, diagnostics,
  config-files, command palette) and keep everything else available via `install_*` hooks.

## Feature Strategy (Reducing Cognition)

Compile-time features are inevitable, but we can **hide the matrix** behind a small set of named
presets and a sensible default.

### Principles

- New users should succeed with `fret = { ... }` default features.
- Advanced users can turn off batteries with `default-features = false` and opt back in.
- “Filesystem-touching defaults” must be individually disable-able (e.g. config files).

### Proposed crate features (conceptual)

- `desktop`: native runner stack (winit + wgpu) support.
- `batteries`: “works out of the box” bundle (diagnostics + config-files + shadcn integration +
  icons + optional asset caches).
- `config-files`: load layered `.fret/*` config files.
- `diagnostics`: tracing + panic hook + diag helpers.
- `ui-assets`: image/SVG caches + default budgets.
- `icons-*`: built-in icon pack installation + semantic aliases + optional SVG preloading.
- `shadcn`: enable the default shadcn surface.

The `fret` crate can keep `default = ["desktop", "batteries"]` (or equivalent), while still allowing
users to do:

```toml
fret = { path = "...", default-features = false, features = ["desktop"] }
```

### Runtime defaults (within enabled features)

Within the enabled compile-time features, the builder provides a runtime `Defaults` preset that
applies:

- i18n backend install (if not provided),
- diagnostics install (if enabled),
- config-files wiring (if enabled),
- shadcn app integration install (if enabled),
- ui-assets budgets (if enabled),
- icon pack install + preload (if enabled).

This should map to a single internal helper (today: `apply_desktop_defaults(...)`).

## Ecosystem + Extensibility (How it composes)

The builder must make “adding ecosystem layers” feel natural:

- **Component surface selection:** shadcn by default, but users can opt-out and build directly on
  `fret-ui-kit` primitives.
- **Router:** install router commands and optional integrations behind a feature gate.
- **Workspace shell:** editor-grade chrome as an optional addon, not part of `fret-ui`.
- **Custom icon packs:** keep `register_icon_pack(...)` available and documented.

The key is: users should not need to learn `fret-framework` unless they are doing manual assembly or
custom runners.

## Hotpatch Compatibility

### Default posture

The v1 builder chain should remain a thin wrapper around:

- `fret-bootstrap::BootstrapBuilder`
- `fret-bootstrap::ui_app_driver::UiAppDriver`
- `fret::mvu` (typed routing) when used

All lifecycle hooks used by the entry should remain `fn` pointers.

### Optional “closure entry”

If we introduce a closure-based variant, it must be:

- opt-in via an explicit feature and/or type name (e.g. `ClosureApp`),
- documented as “not guaranteed hotpatch-friendly”.

## Migration / Compatibility

- Keep existing non-MVU entry points (`fret::run`, `fret::app_with_hooks`, etc.); historical MVU entry points have since been removed in-tree.
- Update templates (`fretboard new hello/simple-todo/todo`) to the builder chain once the API is
  accepted.
- Keep docs showing both:
  - “recommended builder chain”
  - “drop down to `fret-bootstrap` for manual assembly”

## Open Questions

1) Naming: should the builder be `fret::App` or something less likely to confuse with
   `fret_app::App` (e.g. `fret::AppBuilder`, `fret::UiApp`)?
2) Where should the builder live?
   - `ecosystem/fret` only (recommended), or
   - also mirrored in `fret-bootstrap`?
3) How many “first-class knobs” are worth including before we push users to `install_*` hooks?
4) Do we want a single `Defaults::desktop_batteries()` preset, or multiple smaller presets?

## References (in-repo)

- Golden path driver contracts: `docs/adr/0110-golden-path-ui-app-driver-and-pipelines.md`
- Hotpatch safety: `docs/adr/0105-dev-hotpatch-subsecond-and-hot-reload-safety.md`
- Ecosystem bootstrap: `docs/adr/0106-ecosystem-bootstrap-ui-assets-and-dev-tools.md`
- Existing templates & onboarding ladder: `docs/examples/todo-app-golden-path.md`

