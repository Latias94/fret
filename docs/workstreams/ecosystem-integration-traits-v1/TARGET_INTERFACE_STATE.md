# Ecosystem Integration Traits v1 — Target Interface State

Status: Target state for the pre-release cleanup
Last updated: 2026-03-15

This document records the intended interface state for ecosystem integration surfaces.

It answers four concrete questions:

1. which shared contracts should exist,
2. which layer owns them,
3. which ecosystems should consume them,
4. which abstractions should not exist at all.

## 1. Cross-Ecosystem Contract Inventory

| Contract | Target owner | Surface kind | Default audience | Target status |
| --- | --- | --- | --- | --- |
| `InstallIntoApp` | ecosystem-level integration module (`fret::integration`) | small trait | app integration bundles, first-party and third-party app packs | Landed / adoption in progress |
| `CommandCatalog` | `fret-ui-kit::command` (or equivalent component-policy module) | data types + collector helpers (trait deferred until needed) | command palette / menu surfaces | Landed as data contract |
| `RouteCodec` | `fret-router` | small trait | router-aware apps and router UI integrations | Migrated |
| `DockPanelFactory` | `fret-docking` | small trait + registry builder | reusable panel packs, workspace shells | Migrated |
| `QueryAdapter` | `fret-query` integration module | optional small trait | higher-level reusable libraries with optional query support | Deferred in v1 |

### Evidence snapshot (2026-03-15)

Accepted first-party seams now have an explicit evidence set:

- `InstallIntoApp`:
  `ecosystem/fret/src/integration.rs`,
  `ecosystem/fret/src/app_entry.rs`,
  `apps/fret-cookbook/examples/docking_basics.rs`
- `CommandCatalog` data contract:
  `ecosystem/fret-ui-kit/src/command.rs`,
  `ecosystem/fret-ui-shadcn/src/command.rs`,
  `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
- `RouteCodec`:
  `ecosystem/fret-router/src/codec.rs`,
  `ecosystem/fret-router-ui/src/lib.rs`,
  `apps/fret-cookbook/examples/router_basics.rs`,
  `apps/fret-ui-gallery/src/spec.rs`,
  `apps/fret-demo-web/src/wasm.rs`
- `DockPanelFactory`:
  `ecosystem/fret-docking/src/dock/panel_registry.rs`,
  `apps/fret-cookbook/examples/docking_basics.rs`,
  `apps/fret-examples/src/docking_demo.rs`,
  `apps/fret-examples/src/docking_arbitration_demo.rs`
- `QueryAdapter` defer evidence:
  there is still no in-tree implementation outside this workstream, while the current direct
  query-pressure signals remain local and non-shared:
  `ecosystem/fret-markdown/src/mermaid_svg_support.rs`,
  `ecosystem/fret-authoring/src/query.rs`,
  `ecosystem/fret-router/src/query_integration.rs`

## 2. Default Teaching Rule

The default docs posture remains:

- app authors learn free functions and curated facades first,
- traits exist so ecosystems compose cleanly,
- some seams may intentionally start as data contracts plus collector helpers before a trait is
  justified,
- traits are not the first-contact mental model for ordinary app code.

Canonical app-path example:

```rust
use fret::app::prelude::*;

fn install_app(app: &mut App) {
    fret::shadcn::app::install(app);
    fret::router::app::install(app);
}

fn main() -> fret::Result<()> {
    FretApp::new("demo")
        .setup(install_app)
        .window("Demo", (960.0, 640.0))
        .view::<DemoView>()?
        .run()
}
```

Composition rule:

- keep a single installer function as the first documented path,
- allow small tuple composition directly for app-local wiring,
- require named `InstallIntoApp` bundle types for reusable/published ecosystem packs,
- reserve `UiAppBuilder::setup_with(...)` for explicit one-off inline closures or captured runtime
  values,
- keep slice/vec-style dynamic composition out of v1 until a concrete use case appears.

Rust note:

- `InstallIntoApp` may stay broad in implementation (`FnOnce(&mut App)`) because a trait-bound-only
  `fn(&mut App)` impl would force explicit casts for plain function items.
- That implementation detail does not widen the default teaching surface: first-party docs/examples
  and source gates should still reject `.setup(|app| ...)` on the default app-author path.

## 3. Per-Ecosystem Target Shape

### 3.1 Design-system / recipe crates

Examples:

- `fret-ui-shadcn`
- future Material or custom design-system crates

Target export posture:

| Module / surface | Purpose | Notes |
| --- | --- | --- |
| crate root / curated facade | stable recipe taxonomy | default import path for app authors |
| `app` | app-level setup/install helpers | explicit, idempotent |
| `themes` / `tokens` | presets and token helpers | data/apply-function oriented |
| `raw` | explicit escape hatch | never implied by the curated root |

Target rule:

- no shared component trait is required for design-system crates.
- when a shared catalog/data contract already exists in a policy layer, recipe crates should map it
  into recipe-local UI entry types instead of re-owning the collector.

### 3.2 Docking ecosystems

Target export posture:

| Surface | Purpose | Notes |
| --- | --- | --- |
| core docking model (`fret-core`) | persistent graph/ops/identity | mechanism contract |
| `fret-docking` main surface | policy/UI/runtime glue | opt-in ecosystem layer |
| `DockPanelFactory` | reusable panel contribution seam | aggregates into a registry/service |
| registry/builder service | app-owned aggregation point | final registry remains app-owned |

Target rule:

- dock panel contributions are keyed by stable `PanelKind` / `PanelKey`,
- docking stays out of `crates/fret-ui`.
- registration belongs to `DockPanelRegistryBuilder`, while the app still owns the final registry
  service installation.
- first-party examples should teach the builder/factory path whenever panel identity is already
  stable; only genuinely dynamic cases should need additional refactor work.

### 3.3 Router ecosystems

Target export posture:

| Surface | Purpose | Notes |
| --- | --- | --- |
| `RouteLocation` and router core | canonical URL/history model | router-owned |
| `RouteCodec` | typed route encode/decode seam | shared integration contract |
| router UI crate/module | outlet/link/adoption helpers | app/ecosystem layer |
| app install helpers | commands / defaults | explicit and opt-in |

Target rule:

- route canonicalization and history semantics remain router-owned,
- route typing does not leak string parsing through app code,
- app-authored codec types own typed route translation, while `RouteTree` / `Router` keep route ID
  matching and history ownership independent.

Current first-party posture:

- `apps/fret-cookbook/examples/router_basics.rs` teaches the default typed-route path.
- `apps/fret-ui-gallery` uses `UiGalleryRouteCodec` for page routing and legacy URL normalization.
- `apps/fret-demo-web/src/wasm.rs` uses a codec-backed demo selection helper for the canonical web
  entry path.

### 3.4 Query / selector ecosystems

Target export posture:

| Surface | Purpose | Notes |
| --- | --- | --- |
| grouped `cx.data().selector_layout(...)` for LocalState-first selectors, raw `cx.data().selector(...)` for explicit signatures, and `cx.data().query(...)` | default app path | remains the canonical teaching surface |
| `QueryAdapter` | optional reusable-library bridge | deferred in v1 until a second real reusable consumer appears |
| selector integration | data-first, no shared trait in v1 | `DepsBuilder` / `DepsSignature` stay enough |

Target rule:

- primitives remain selector/query agnostic,
- reusable higher-level libraries may expose local adapters or wrappers without freezing a shared
  trait too early,
- app docs still teach grouped data helpers first.
- opt-in component-layer helpers may exist, but they stay quarantined to explicit seams rather
  than leaking into primitives or base recipes.

Current v1 defer rationale on 2026-03-15:

- `ecosystem/fret-markdown` currently uses direct query-context helpers for a markdown-specific
  resource-loading seam,
- `ecosystem/fret-authoring/src/query.rs` exposes an authoring-surface wrapper over query helpers,
- `ecosystem/fret-router/src/query_integration.rs` only contributes query-key helpers,
- this is still not a second reusable-consumer pair with a materially shared adapter contract.

Current guardrail posture on 2026-03-15:

- `ecosystem/fret-ui-kit/src/declarative/model_watch.rs` keeps `QueryHandleWatchExt` behind the
  `state-query` feature and off the default declarative prelude.
- `ecosystem/fret-ui-shadcn/src/state.rs` remains the explicit recipe-layer home for selector/query
  badges and alerts behind `state-selector` / `state-query`.
- `ecosystem/fret-ui-kit/src/lib.rs` source-policy tests reject direct `fret_query` /
  `fret_selector` imports across `src/primitives`, `src/recipes`, and the neighboring
  `../fret-ui-headless/src` base-layer surface.

## 4. Ownership Boundaries

### 4.1 Must not live in `crates/fret-ui`

These contracts must not be added to `crates/fret-ui`:

- `InstallIntoApp`
- `CommandCatalog`
- `RouteCodec`
- `DockPanelFactory`
- `QueryAdapter`
- any universal plugin/component abstraction for ecosystem crates

Reason:

- `crates/fret-ui` is a mechanism/contract layer,
- these are policy/integration concerns.

`CommandCatalog` specific rule:

- host command registry collection, gating interpretation, and shortcut derivation belong in
  `fret-ui-kit::command`,
- recipe crates such as `fret-ui-shadcn` may provide thin `Into<RecipeEntry>` mappings and
  recipe-local convenience wrappers,
- an actual `CommandCatalog` trait should only be added once a second non-shadcn consumer needs a
  shared source interface.

`CommandMeta` vs `CommandCatalog` rule:

- plain `CommandMeta` is sufficient when a command only needs normal registration, keybindings,
  menu identity, and command-palette discoverability via the shared host registry,
- use `CommandCatalog` data when a surface needs richer grouped or transformed discovery entries
  beyond a flat registry listing, for example grouped sections, extra aliases, or recipe-specific
  catalog shaping before rendering.

### 4.2 `fret-app::Plugin` target posture

Current role:

- app-owned plugin registry for commands, globals, and related app contributions.

Target posture:

- keep it app-owned and minimal,
- do not expand it into the universal ecosystem extension model,
- let it coexist with smaller domain traits where the ownership problem is different.

Clarification:

- domain-local plugin traits such as `GizmoPlugin` are acceptable when they solve a local subsystem
  problem,
- they should not be treated as precedent for a repo-wide "every ecosystem crate exposes a plugin"
  model.

## 5. Rejected Interface State

The following names or ideas should not exist as shared ecosystem contracts:

| Rejected surface | Why it is rejected |
| --- | --- |
| `Component` / `UiComponent` universal trait | erases important layer distinctions and duplicates existing `UiIntoElement` / `UiPatchTarget` contracts |
| giant `Plugin` trait | bundles unrelated responsibilities into one unstable abstraction |
| `SelectorAdapter` in v1 | no concrete multi-crate pressure yet |
| `ThemeInstaller` trait | theme presets are better as data + apply functions |
| `IconPack` trait | icon packs are already registry/data-first |

## 6. Target Example: Bundle-Oriented App Integration

The shared integration contract should support app-owned composition without turning free functions
into a second-class path.

```rust
use fret::app::App;
use fret::integration::InstallIntoApp;

struct WorkbenchBundle;

impl InstallIntoApp for WorkbenchBundle {
    fn install_into_app(self, app: &mut App) {
        fret::shadcn::app::install(app);
        fret::router::app::install(app);
        workbench_panels::install(app);
    }
}
```

The same app should still be able to teach the non-trait version:

```rust
fn install_workbench(app: &mut App) {
    fret::shadcn::app::install(app);
    fret::router::app::install(app);
    workbench_panels::install(app);
}
```

## 7. Target Example: Typed Routes

```rust
enum Route {
    Home,
    Settings,
    Doc { id: Arc<str> },
}

struct RouteCodecV1;

impl RouteCodec for RouteCodecV1 {
    type Route = Route;
    type Error = RouteDecodeError;

    fn encode(&self, route: &Self::Route) -> RouteLocation {
        match route {
            Route::Home => RouteLocation::from_path("/"),
            Route::Settings => RouteLocation::from_path("/settings"),
            Route::Doc { id } => RouteLocation::from_path(format!("/docs/{id}")),
        }
    }

    fn decode(&self, location: &RouteLocation) -> Result<Self::Route, Self::Error> {
        let _ = location;
        todo!()
    }
}
```

## 8. Target Example: Dock Panel Contribution

```rust
struct SearchPanelFactory;

impl DockPanelFactory<App> for SearchPanelFactory {
    fn panel_kind(&self) -> PanelKind {
        PanelKind::new("workbench.search")
    }

    fn build_panel(
        &self,
        key: &PanelKey,
        cx: &mut DockPanelFactoryCx<'_, App>,
    ) -> Option<NodeId> {
        let _ = (key, cx);
        Some(todo!())
    }
}

let mut registry = DockPanelRegistryBuilder::<App>::new();
registry.register(SearchPanelFactory);
```

## 9. Completion Rule

This target state is reached when:

- each accepted contract has one clear owning layer,
- first-party ecosystems use the same small set of integration seams,
- official docs/examples no longer teach mixed legacy postures,
- rejected interfaces remain explicitly absent.
