# Ecosystem Integration Traits v1

Status: Active working plan (pre-release fearless refactor)
Last updated: 2026-03-11

Related:

- `docs/adr/0016-plugin-and-panel-boundaries.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/adr/0111-ecosystem-integration-contracts.md`
- `docs/adr/0148-component-ecosystem-authoring-conventions-v1.md`
- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/component-ecosystem-state-integration-v1/component-ecosystem-state-integration-v1.md`
- `docs/workstreams/router-v1/router-v1.md`
- `docs/workstreams/router-ui-v1/router-ui-v1.md`
- `docs/workstreams/query-lifecycle-v1/query-lifecycle-v1.md`
- `docs/workstreams/state-management-v1/state-management-v1-extension-contract.md`

This workstream defines a **trait budget** for ecosystem integration before the first public
release.

The goal is not to add more abstraction for its own sake. The goal is to make it obvious:

- which ecosystem seams should stay as free functions or registries,
- which seams deserve a small trait,
- where each trait belongs,
- and which old postures we should delete instead of preserving.

Because Fret is still pre-release, this document assumes **fearless cleanup**:

- old public-looking helpers may be removed,
- mixed naming can be normalized,
- temporary adapters are allowed only as short-lived migration scaffolding.

This document is not an ADR. If implementation of these targets changes stable panel persistence,
command routing, router semantics, or the `fret-ui` runtime contract, the relevant ADRs must be
updated separately.

## 1. Problem Statement

Fret already has the right high-level layering:

- `crates/*` provide mechanisms and hard contracts,
- `ecosystem/*` provides policy, recipes, and reusable product surfaces,
- `apps/*` provides app-owned shells, tooling, and demos.

What is still uneven is the **integration vocabulary** across ecosystem crates.

Today we already have good pieces:

- explicit `install(...)` / `install_app(...)` conventions,
- app-owned plugin and panel guidance,
- route/query/state workstreams,
- a cleaned-up shadcn facade with explicit `app`, `themes`, and `raw` seams.

But the repo still lacks one place that answers these practical questions:

1. When should an ecosystem crate expose a free function vs a registry vs a trait?
2. Which traits should be shared across multiple ecosystems?
3. Which traits must stay out of `crates/fret-ui`?
4. How do shadcn, docking, router, query, and future third-party kits fit the same model?
5. How do we avoid drifting into a giant `Plugin` or universal `Component` abstraction?

Without a trait budget, maintainers will tend to make one of two mistakes:

- over-abstract too early with a monolithic plugin framework, or
- let every ecosystem crate invent a different extension story.

We should do neither.

## 2. Goals

1. Define a small, explicit, layer-correct integration vocabulary for first-party and third-party
   ecosystem crates.
2. Keep the default app authoring path ergonomic:
   - `FretApp::setup(...)`,
   - curated facades,
   - explicit advanced seams.
3. Preserve the app/component/advanced split from the authoring-surface reset.
4. Keep `crates/fret-ui` mechanism-only.
5. Make old mixed integration postures easy to delete before open source release.

## 3. Non-goals

- A universal trait that every UI component must implement.
- A giant ecosystem `Plugin` trait that covers commands, routing, panels, queries, themes, and
  menus at once.
- Moving ecosystem policy into `crates/fret-ui`.
- Auto-discovery / hidden registration magic.
- Replacing existing data-first registries with trait objects when a struct or function is enough.

## 4. Design Rules

### 4.1 Traits are the last step, not the first step

Use the smallest surface that matches the ownership problem:

| Need | Preferred surface |
| --- | --- |
| one-off app setup | free function (`install`, `install_app`) |
| pure configuration/data contribution | struct/enum + registry |
| stable identity to runtime instance mapping | factory |
| bidirectional serialization / canonicalization | codec |
| host-driven polymorphism across multiple ecosystems | small trait |

Rule:

- if a function or registry solves the problem cleanly, do not invent a trait.

### 4.2 Trait scope must be narrow and one-directional

A good ecosystem trait should answer exactly one question:

- "How do I install this into an app?"
- "How do I encode/decode this route?"
- "How do I contribute dock panels?"
- "How do I contribute command catalog entries?"

A bad trait tries to answer all of them at once.

### 4.3 Traits live with the owning ecosystem layer

Hard rule:

- `crates/fret-ui` does not own shadcn, docking, router, query, or selector integration traits.

Preferred ownership:

- app-integration traits live in `ecosystem/fret` or another ecosystem-level integration module,
- command-catalog traits live in `ecosystem/fret-ui-kit`,
- router traits live in `ecosystem/fret-router`,
- docking traits live in `ecosystem/fret-docking`,
- query traits live in `ecosystem/fret-query`.

### 4.4 Free functions remain the default teaching surface

Even when a small trait exists, docs should still teach the boring path first.

Example:

- app docs teach `FretApp::setup(shadcn::app::install)`,
- not "first create a trait object bundle".

Traits exist so ecosystems compose cleanly, not so ordinary app authors must think in trait
objects.

### 4.5 Registry/factory/codec beats "universal plugin"

The extension seams in this repo are heterogeneous on purpose:

- dock panels are keyed by stable `PanelKind`,
- routes are canonical URL/state translations,
- query is async resource lifecycle,
- command palettes are read models over registered commands plus optional dynamic entries.

Trying to force all of that into one `Plugin` contract will either:

- make the trait huge, or
- make it so abstract that it stops being useful.

### 4.6 Short checklist for ecosystem authors

Before adding a new ecosystem contract, answer these questions in order:

1. Which tier is this crate actually targeting?
   - app integration,
   - component/policy surface,
   - or advanced/manual assembly.
2. Can this stay a free `install(...)` / `install_app(...)` function, registry, factory, or codec?
3. If a trait is still needed, does it answer exactly one question?
4. Does the trait live in the owning ecosystem layer rather than `crates/fret-ui`?
5. Are ordinary app docs still teaching the boring path first?
6. Are we accidentally widening `fret-app::Plugin` or inventing a universal `Component` trait?

If any answer is unclear, stop and resolve the ownership model before expanding the API surface.

## 5. Trait Budget (v1)

This is the recommended v1 budget. The names can still shift slightly during implementation; the
budget itself is the main contract.

### 5.1 `InstallIntoApp`

Purpose:

- give ecosystem bundles a shared, app-facing integration seam,
- compose multiple installers into one `FretApp::setup(...)` story,
- keep installation app-thread-only and idempotent.

Target shape:

```rust
use fret::app::App;

pub trait InstallIntoApp {
    fn install_into_app(self, app: &mut App);
}

impl<F> InstallIntoApp for F
where
    F: FnOnce(&mut App),
{
    fn install_into_app(self, app: &mut App) {
        (self)(app);
    }
}
```

Rules:

- this is an app-layer integration trait, not a component trait,
- it does not belong in `crates/fret-ui`,
- it wraps existing installer functions rather than replacing them,
- the broad `FnOnce(&mut App)` implementation is an ergonomics accommodation for Rust fn-item /
  trait-bound coercion limits, not a license to teach `.setup(|app| ...)` as the default path,
- it must remain safe to call once per app lifetime,
- advanced builder-only install hooks can stay on explicit builder extension traits until a shared
  need appears.

Teaching rule:

- ordinary app docs keep `.setup(...)` on named installers, tuples, or named bundle types,
- inline closures stay on `UiAppBuilder::setup_with(...)`,
- first-party source gates should reject `.setup(|app| ...)` on the default app-author path.

Use cases:

- compose shadcn + router + docking + icons into one bundle,
- let third-party crates publish "app integration packs" without inventing custom setup APIs.

### 5.2 `CommandCatalog`

Purpose:

- contribute command palette / menu catalog entries that are richer than a flat `CommandId ->
  CommandMeta` listing,
- support dynamic sections such as recent workspaces, project-local actions, or domain-specific
  command groups,
- keep execution routed back to typed actions or `CommandId`.

Target shape:

```rust
pub trait CommandCatalog: Send + Sync + 'static {
    fn collect(
        &self,
        cx: &mut CommandCatalogCx<'_>,
        out: &mut Vec<CommandCatalogEntry>,
    );
}
```

Rules:

- catalog collection is read-only; it does not execute commands itself,
- command entries must ultimately route through typed actions or registered commands,
- the trait should live in a component/policy layer such as `fret-ui-kit::command`,
- recipe crates like `fret-ui-shadcn` can consume it, but should not own the canonical contract.

Use cases:

- command palette surfaces,
- action search popovers,
- app-wide menus that merge static registry data with dynamic entries.

### 5.3 `RouteCodec`

Purpose:

- make typed route <-> canonical `RouteLocation` conversion explicit,
- let router UI, deep-linking, query integration, and persistence share the same route encoding
  contract,
- avoid leaking string parsing logic throughout apps.

Target shape:

```rust
use fret_router::RouteLocation;

pub trait RouteCodec: Send + Sync + 'static {
    type Route: Clone + Eq + 'static;
    type Error;

    fn encode(&self, route: &Self::Route) -> RouteLocation;
    fn decode(&self, location: &RouteLocation) -> Result<Self::Route, Self::Error>;
}
```

Rules:

- the codec lives in router ecosystem space, not in `fret-ui`,
- canonical URL policy still belongs to `fret-router`,
- a route table may implement or generate this contract, but the trait is the shared seam.

Use cases:

- typed app routes,
- route-aware query keys,
- route-based command/menu affordances,
- route persistence in multi-window products.

### 5.4 `DockPanelFactory`

Purpose:

- make "stable `PanelKind` -> panel contribution" explicit,
- let reusable panel packs register into a dock registry without owning the full dock runtime,
- converge plugin/panel contributions around stable identities from ADR 0013 / ADR 0016.

Target shape:

```rust
use fret_core::{PanelKey, PanelKind};
use fret_ui::{NodeId, UiHost};

pub trait DockPanelFactory<H: UiHost>: Send + Sync + 'static {
    fn panel_kind(&self) -> PanelKind;
    fn build_panel(&self, key: &PanelKey, cx: &mut DockPanelFactoryCx<'_, H>) -> Option<NodeId>;
}

let mut registry = DockPanelRegistryBuilder::<App>::new();
registry.register(MyPanelFactory);
```

Rules:

- the app still owns the final dock registry/service,
- this trait represents a single panel contribution, not the entire workspace shell,
- it should aggregate into the existing docking registry story instead of bypassing it,
- registration belongs to the builder rather than a trait-side `register(...)` method.

Use cases:

- editor panel packs,
- domain kits that contribute dockable explorers or inspectors,
- future plugin systems that need stable panel registration without renderer coupling.

### 5.5 `QueryAdapter`

Purpose:

- let higher-level ecosystem crates integrate async resource state without forcing a hard dependency
  on one concrete query surface in their core API,
- keep primitive/component layers data-oriented while still allowing optional query-aware adapters.

Target shape:

```rust
pub trait QueryAdapter: Send + Sync + 'static {
    type Key;
    type Handle<T>;

    fn query<T>(&self, cx: &mut AppQueryCx<'_, '_>, key: Self::Key) -> Self::Handle<T>;
    fn invalidate_namespace(&self, namespace: &str);
}
```

Rules:

- this trait is optional and belongs in `fret-query` integration space,
- primitive contracts should still prefer plain values + callbacks,
- app-facing docs should continue to teach `cx.data().query(...)` first,
- only higher-level reusable libraries should need this abstraction.

Use cases:

- a reusable data table, chart, or markdown kit that wants optional query integration,
- future third-party app kits that support multiple host state stacks.

## 6. Deliberately Deferred or Rejected Traits

### 6.1 No universal `Component` trait

Rejected.

Reason:

- component authoring already has the right ecosystem vocabulary:
  - `UiIntoElement`,
  - `UiPatchTarget`,
  - `UiBuilder`,
  - recipe/headless composition in `fret-ui-kit`.
- forcing shadcn, docking, charts, markdown, and future kits behind one "component" trait would
  erase important layer distinctions.

### 6.2 No giant ecosystem `Plugin` trait

Rejected.

Reason:

- `fret-app::Plugin` is an app-owned registry boundary today,
- it is useful for commands, globals, and basic app contributions,
- but it should not become the universal integration model for router, docking, query, selector,
  shadcn, menus, and themes.

Target posture:

- keep `fret-app::Plugin` app-owned and minimal,
- add smaller domain traits instead of expanding it.

### 6.3 No shared `SelectorAdapter` in v1

Deferred.

Reason:

- `DepsBuilder`, `DepsSignature`, and `cx.data().selector(...)` already provide a strong default
  app-facing seam,
- the component-ecosystem state guidance already says primitives should remain selector-agnostic,
- we should not add another trait until there is a concrete multi-crate pressure for it.

### 6.4 No theme or icon-pack trait in v1

Rejected for now.

Reason:

- theme presets are better expressed as data + apply functions,
- icon packs are already registry/data-first,
- neither needs host-driven polymorphism yet.

## 7. Ecosystem-by-Ecosystem Target Shape

### 7.1 `fret-ui-shadcn`

Target posture:

- root/facade: curated recipe surface,
- `app`: app integration entry points,
- `themes`: theme presets,
- `raw`: explicit escape hatch for non-curated primitives/helpers.

Rules:

- do not define a parallel app runtime,
- do not require a shared component trait,
- do not leak the raw crate root onto the default app path.

Potential trait usage:

- consume `CommandCatalog`,
- optionally implement `InstallIntoApp` for shadcn app bundles.

### 7.2 `fret-docking`

Target posture:

- core docking model stays in `fret-core`,
- docking UI/policy stays in `fret-docking`,
- reusable panel packs contribute through `DockPanelFactory`,
- app-owned registry/service remains the final aggregation point.

Rules:

- no docking semantics in `crates/fret-ui`,
- no renderer-owned panel APIs,
- panel identity stays stable via `PanelKind` / `PanelKey`.

### 7.3 `fret-router`

Target posture:

- router core owns canonical `RouteLocation`,
- typed route translation is expressed via `RouteCodec`,
- UI adoption stays in app/ecosystem layers such as `fret-router-ui`,
- query integration continues to use canonical locations rather than hidden UI hooks.

### 7.4 `fret-query` / `fret-selector`

Target posture:

- app authors continue to use grouped `cx.data().selector(...)` / `cx.data().query(...)`,
- primitives stay state-stack agnostic,
- optional query adapters exist only for higher-level reusable libraries,
- selector remains data-first without a shared trait in v1.

### 7.5 Future third-party design systems / app kits

Required decision:

- choose one tier explicitly:
  - app integration,
  - component surface,
  - advanced/manual assembly,
  - or a combination with clear module boundaries.

Recommended module shape:

- `crate::app`
- `crate::themes`
- `crate::raw`
- `crate::headless` or `crate::core` where applicable

## 8. Example Target Authoring

The important idea is that ordinary apps still use the boring path.

```rust
use fret::app::prelude::*;

fn install_workspace(app: &mut App) {
    fret::shadcn::app::install(app);
    fret::router::app::install(app);
    workspace_panels::install(app);
}

fn main() -> fret::Result<()> {
    FretApp::new("workspace")
        .setup(install_workspace)
        .window("Workspace", (1280.0, 800.0))
        .view::<WorkspaceView>()?
        .run()
}
```

The trait layer exists for composition, not for first-contact docs.

```rust
use fret::app::App;
use fret::integration::InstallIntoApp;

struct WorkspaceBundle {
    routes: WorkspaceRouteCodec,
    panels: WorkspacePanels,
}

impl InstallIntoApp for WorkspaceBundle {
    fn install_into_app(self, app: &mut App) {
        fret::shadcn::app::install(app);
        fret::router::app::install(app);
        self.panels.install_into_app(app);
    }
}
```

Typed routes should stop leaking strings through app code:

```rust
enum WorkspaceRoute {
    Home,
    Doc { id: Arc<str> },
}

struct WorkspaceRouteCodec;

impl RouteCodec for WorkspaceRouteCodec {
    type Route = WorkspaceRoute;
    type Error = RouteDecodeError;

    fn encode(&self, route: &Self::Route) -> RouteLocation {
        match route {
            WorkspaceRoute::Home => RouteLocation::from_path("/"),
            WorkspaceRoute::Doc { id } => RouteLocation::from_path(format!("/docs/{id}")),
        }
    }

    fn decode(&self, location: &RouteLocation) -> Result<Self::Route, Self::Error> {
        // target shape only
        todo!()
    }
}
```

Docking should aggregate contributions by stable panel identity instead of app-specific string
switches spread through the codebase:

```rust
struct InspectorPanelFactory;

impl DockPanelFactory<App> for InspectorPanelFactory {
    fn panel_kind(&self) -> PanelKind {
        PanelKind::new("workspace.inspector")
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
registry.register(InspectorPanelFactory);
```

## 9. Migration Strategy

Recommended order:

1. lock the trait budget in docs,
2. normalize first-party module layout (`app`, `themes`, `raw`, `core`, `ui`) where missing,
3. add thin adapters around existing installer/registry/codec patterns,
4. migrate first-party crates to the shared seams,
5. add guardrails,
6. delete legacy mixed postures.

Migration rule:

- if an old surface is only preserved for in-repo transition, track it explicitly and delete it as
  soon as official docs/examples stop teaching it.

## 10. ADR Trigger Rules

An ADR update is required when this workstream causes any of the following:

- `PanelKind` / panel persistence contract changes,
- app/plugin registry semantics change materially,
- router canonicalization or navigation semantics change materially,
- `fret-ui` runtime surface grows to accommodate an ecosystem trait,
- command routing semantics change beyond catalog presentation.

Until then, this workstream note is the correct place to plan and track the fearless cleanup.
