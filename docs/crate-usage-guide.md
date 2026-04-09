# Crate Usage Guide (What to Depend On, When)

This document is a **practical map** of the crates in this repository:

- what each crate is for,
- when you should depend on it,
- and how to keep dependencies portable as your ecosystem grows.

If you are a component author, start with:

- `docs/component-author-guide.md`
- `docs/component-authoring-contracts.md`
- If you intentionally consume the `fret` facade for reusable in-repo component/scaffold code,
  prefer `use fret::component::prelude::*;` over `fret::app::prelude::*;`.

## Ecosystem author checklist

If you are publishing a reusable Fret ecosystem crate or introducing a new first-party ecosystem
surface, lock these decisions before adding public API:

- pick one tier first: app integration, component/policy surface, or advanced/manual assembly
- prefer a free function, registry, factory, or codec before adding a trait
- use `fret::integration::InstallIntoApp` only for reusable app-install bundles; keep ordinary app
  docs/examples on plain installer functions
- if you really need an inline closure with captured runtime values, keep it on
  `UiAppBuilder::setup_with(...)` instead of teaching `.setup(|app| ...)`
- keep typed routes on `RouteCodec`, dockable panel contributions on `DockPanelFactory`, and host
  command catalog ownership in `fret-ui-kit::command`
- use plain `CommandMeta` when a command only needs normal registration, keybindings, menus, and
  command identity; reach for `CommandCatalog` only when a discovery surface needs grouped or
  enriched catalog entries beyond flat registered metadata
- keep selector/query integration optional for reusable kits; do not add a universal `Component`
  trait or widen `fret-app::Plugin` into the default ecosystem model
- keep `fret-app::Plugin` app-owned; domain-local plugin traits such as `GizmoPlugin` are fine,
  but they are not precedent for a repo-wide ecosystem extension template
- treat `QueryAdapter` as deferred in v1 unless a second real reusable consumer appears with a
  materially shared adapter contract
- keep reusable docs/examples aligned with the current conversion-surface target:
  app-facing teaching helpers use `Ui` / `UiChild`, pure app-facing page shells should avoid
  carrying `UiCx` unless they really need runtime/context access, reusable generic helpers should
  move toward the unified component conversion trait tracked in
  `docs/workstreams/into-element-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`, and raw
  `AnyElement` stays explicit
- keep shipped resource ownership explicit:
  - app-owned resources live under `AssetBundleId::app(...)`
  - ecosystem/package-owned images, SVGs, fonts, and similar bytes live under
    `AssetBundleId::package(...)`
  - reusable crates should publish installer/setup surfaces instead of making apps reproduce
    internal asset mounts by hand
- keep icon requirements explicit:
  - reusable component crates should prefer semantic `IconId` / `ui.*` ids over hard-wiring one
    vendor pack into their public contract
  - icon packs currently install through explicit `crate::app::install` seams backed by the
    global `IconRegistry`, so apps compose installers instead of wiring loose icon bytes manually
- for the full trait budget, target state, and migration posture, see
  `docs/workstreams/ecosystem-integration-traits-v1/DESIGN.md`,
  `docs/workstreams/ecosystem-integration-traits-v1/TARGET_INTERFACE_STATE.md`, and
  `docs/workstreams/ecosystem-integration-traits-v1/MIGRATION_MATRIX.md`
- for the follow-on conversion cleanup that collapses legacy split `into_element` vocabulary, see
  `docs/workstreams/into-element-surface-fearless-refactor-v1/DESIGN.md` and
  `docs/workstreams/into-element-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`

Rust note:

- `InstallIntoApp` stays broad in implementation because a trait-bound-only `fn(&mut App)` impl
  would force explicit casts for plain function items at call sites.
- That implementation detail should not change the teaching surface: first-party docs/examples
  should still avoid `.setup(|app| ...)`.

## Quick rules of thumb

### The 5 crate names to remember

- `fret`: desktop-first batteries-included app entry point (recommended).
- `fret-ui-shadcn`: default component surface (apps).
- `fret-ui-kit`: component authoring glue (ecosystem libraries).
- `fret-framework`: framework facade for advanced/manual assembly.
- `fretboard`: public CLI for asset manifests, project-local config helpers, and starter app scaffolds.

In this repo, maintainer-only workflows such as repo-local templates, demo runners, and diagnostics stay on
the non-published `fretboard-dev` package.

1) If you are writing a reusable ecosystem library, avoid backend crates (`fret-launch`, `winit`, `wgpu`).

2) Prefer this dependency ladder (low → high):

- `fret-core` / `fret-runtime` (portable contracts and data types)
- `fret-ui` (portable retained/declarative UI runtime)
- `ecosystem/*` (policy layers, component libraries, domain ecosystems)
- `fret-app` / `fret-bootstrap` (app-level integration and golden-path defaults)
- `fret-launch` (runner glue, effect draining, presentation)

Desktop-first quick start:

- If you want a single dependency for a native desktop app, use `fret` (ecosystem-level batteries-included wrapper).
- If you are onboarding a new app author, pair it with the default ladder: `hello` → `simple-todo` → `todo`.
  Treat `todo` as the richer third-rung product baseline, not as a replacement for the first two starters.

Web/wasm quick start (tooling):

- `cargo run -p fretboard-dev -- dev web --demo ui_gallery`

3) Only depend on `fret-app` if you need app-owned integration surfaces:

- command registration / command palette integration,
- default keybindings installation,
- file-backed settings/keymap loading helpers.

### Background work (portable)

Keep the UI/runtime deterministic by treating `App`/`ModelStore` as main-thread only (ADR 0008).

Recommended patterns:

- **Portable default**: background producers send **data-only** messages into an inbox; the UI thread drains the inbox at a driver boundary and schedules redraw (ADR 0110, ADR 0175).
- **Heavy apps**: run an external runtime (e.g. Tokio) on a dedicated thread, send results into an inbox, and `wake()` the runner to reach the next driver boundary promptly (ADR 0175).

Portability notes (native vs wasm):

- On native backends, background work is typically available (`exec.background_work=threads`).
- On wasm backends, "background" work is best-effort and may run cooperatively on the same thread (`exec.background_work=cooperative`).
  - Do not assume CPU-heavy work will not block UI; keep tasks short or move heavy work to a non-portable adapter (worker/thread/runtime) at the app boundary.
- `wake()` may be coalesced on all platforms; on wasm it may also be degraded (`exec.wake=best_effort`), so treat it as a hint to reach the next driver boundary, not a precise scheduling guarantee.
- Timers may be throttled on wasm (`exec.timers=best_effort`). Use runner-owned effects (`Effect::SetTimer`, RAF) for UI-visible timing and avoid relying on precise intervals.

Dependency guidance:

- **Reusable ecosystem crates** SHOULD depend on portable surfaces only (`fret-core` / `fret-runtime` / `fret-ui`) and use the inbox/dispatcher surface (ADR 0111, ADR 0175).
- **Apps** may use `fret-bootstrap` (or `fret`) to get the golden-path wiring so they do not need to hand-roll channels + timers + wake logic.

## Features (Cargo)

Cargo features are widely used in Rust UI ecosystems to keep “small apps small” while allowing optional integrations.
We treat feature naming as **recommended convention**, not a hard requirement for third-party crates.

## Core framework crates (`crates/*`)

### `fret` (ecosystem meta crate)

**What it is:** desktop-first, batteries-included app entry points (golden path).

**Use it when:** you want the recommended “just build an app” experience without hand-assembling runners, effects draining, and default integrations.

**Boundary note:** keep editor/workspace shell composition on owning crates such as
`fret-workspace`; `fret` should stay focused on app-facing authoring, not editor-specific shell
facades.

**Shell ownership note:**

- keep **window bootstrap** on the builder/launch lane:
  `FretApp::window(...)`, `.window_min_size(...)`, `.window_position_logical(...)`,
  `.window_resize_increments(...)`, and related startup window configuration
- keep **page shell** app-owned:
  centered cards, docs scaffolds, responsive page padding, and similar interior framing helpers are
  ordinary app/example composition rather than stable framework contracts
- keep **workspace shell** on `fret-workspace`:
  editor-grade frame chrome, pane-content focus targets, and workspace command scope should stay on
  the explicit workspace owner
- keep the optional in-window menubar bridge explicit on `fret::in_window_menubar::*`; it is not a
  synonym for workspace shell ownership

**Default authoring mental model:** when you take the `fret` golden path, start with `View` + `AppUi` + typed actions, prefer `local.layout_value(cx)` / `local.paint_value(cx)` for ordinary LocalState tracked reads, and use `local.layout_read_ref(cx, |value| ...)` / `local.paint_read_ref(cx, |value| ...)` when app code only needs a borrowed projection without cloning the full slot. Keep the first-contact handler surface to `cx.actions().locals_with((...)).on::<A>(|tx, (...)| ...)`, `cx.actions().local(&local).set::<A>(...)` / `.update::<A>(...)` / `.toggle_bool::<A>()`, `cx.actions().transient::<A>(...)`, plus widget `.action(...)` / `.action_payload(...)` whenever the control already exposes a stable action slot. For ordinary initialized locals inside `locals_with((...)).on::<A>(...)`, prefer `tx.value(&local)` for reads and keep `tx.value_or(...)` / `tx.value_or_else(...)` for explicit fallback cases only. For view-owned keyed lists, bind row payloads with `.action_payload(...)` and prefer `cx.actions().local(&rows_state).payload_update_if::<A>(...)` as the default row-write path. If a widget already exposes its own `.on_activate(...)` hook, stay on that component-owned surface instead of importing the activation bridge just to attach a no-op or side effect override. Only add `use fret::app::AppActivateExt as _;` for activation-only surfaces that do not yet offer a narrower widget-owned app-facing helper, and keep the same action-first vocabulary there via `widget.action(act::Save)`, `widget.action_payload(act::Remove, payload)`, and `widget.listen(|host, acx| { ... })`. Drop down to `cx.actions().models::<A>(...)` for shared `Model<T>` graphs and `cx.actions().payload_models::<A>(...)` when the same graph needs typed payload actions without reopening the deleted payload-carrier namespace. There is one explicit advanced raw-model seam: import `use fret::advanced::AppUiRawModelExt;` and call `cx.raw_model::<T>()` only when the raw handle itself is the point. Treat lower-level payload helpers, raw `AppUi::on_action_notify*`, and low-level `.on_activate(cx.actions().listen(...))` glue as cookbook/reference-only host-side escape hatches; if you intentionally reopen that seam, keep it on `cx.actions().listen(...)` or import `AppActivateExt` explicitly for activation-only typed dispatch.

`fret::app::AppActivateSurface` / `AppActivateExt` are intentionally narrow: they cover
activation-only widgets that expose the standard `OnActivate` slot but still lack a tighter
component-owned authoring API. Typed payload/context callbacks remain component-owned surfaces even
when they eventually dispatch app actions. First-party controls such as `shadcn::Button`,
`shadcn::SidebarMenuButton`, `WorkflowControlsButton`, `MessageAction`, `ArtifactAction`,
`ArtifactClose`, `CheckpointTrigger`, `ConversationDownload`, `PromptInputButton`,
`WebPreviewNavigationButton`, `ConfirmationAction`, `Attachment`, `QueueItemAction`, `Test`,
`FileTreeAction`, `Suggestion`, `MessageBranch`, and `Badge` link overrides stay on their native `.action(...)`,
native `.action_payload(...)`, or widget-owned `.on_activate(...)` contracts instead of relying on
the activation bridge. As of 2026-03-16, the first-party default widget bridge table is
intentionally empty. This bridge stays off `fret::app::prelude::*`; default app authors should
only import it for truly activation-only custom/third-party widgets that have not yet graduated to
their own app-facing action surface.

When app code needs explicit styling or icon nouns, keep them off the default prelude and import
them intentionally from `fret::style::{...}` and `fret::icons::{icon, IconId}`.
When extracted app helpers need hover shells or attributed text leaves, prefer
`fret_ui_kit::ui::hover_region(...)` and `fret_ui_kit::ui::rich_text(...)` over spelling
`HoverRegionProps`, `StyledTextProps`, or `cx.elements()` directly.
When app code needs explicit theme snapshot value types in extracted helper signatures, import
`fret::style::ThemeSnapshot` instead of expecting it from `fret::app::prelude::*`.
When app code needs explicit local state-handle types in validators or helper signatures, import
`fret::app::LocalState` instead of expecting it from `fret::app::prelude::*`.
When app code needs explicit command identity values, import `fret::actions::CommandId` instead of
expecting `CommandId` from the default prelude.
When app code needs explicit semantics nouns, import them intentionally from
`fret::semantics::SemanticsRole` instead of expecting them from `fret::app::prelude::*`.
When app code needs explicit selector/query nouns, keep them off the default prelude as well and
import them intentionally from `fret::selector::ui::DepsBuilder`,
`fret::selector::DepsSignature`, and `fret::query::{QueryKey, QueryPolicy, QueryState, ...}`.
Do the same for environment/responsive helpers: import them intentionally from `fret::env::{...}`
instead of treating breakpoint/media helpers as part of the default app vocabulary.
When a view intentionally opts into manual sink-style `*_build(|cx, out| ...)` composition, keep
that helper off the default prelude too and import `fret::children::UiElementSinkExt as _`
explicitly at the call site.
When app code needs explicit command-availability reads such as `cx.action_is_enabled(...)`, import
`fret::actions::ElementCommandGatingExt as _` explicitly instead of expecting command-gating traits
from `fret::app::prelude::*`.
Do the same for logical assets: import them intentionally from `fret::assets::{...}`, prefer
`AssetBundleId::app(...)` / `AssetBundleId::package(...)` plus `AssetLocator::bundle(...)` and
`register_bundle_entries(...)` as the portable default story, and keep
`AssetLocator::file(...)` / `AssetLocator::url(...)` as explicit capability-gated escape hatches.
For startup that needs one explicit development-vs-packaged switch, prefer
`AssetStartupPlan` + `AssetStartupMode` from `fret::assets::{AssetStartupPlan, AssetStartupMode}`
plus
`FretApp::asset_startup(...)` / `UiAppBuilder::with_asset_startup(...)`. Keep
`development_dir(...)` / `development_manifest(...)` for native/package-dev file-backed inputs,
and keep `packaged_entries(...)`, `packaged_bundle_entries(...)`, or
`packaged_embedded_entries(...)` for packaged/web/mobile-friendly bytes. Generated asset modules
remain the packaged lane because they already expose `ENTRIES`, `bundle_id()`, `Bundle`,
`install(app)`, and `mount(builder)`.
When you are on `fret-bootstrap` directly instead of `fret`, use the same startup contract from
`fret_bootstrap::assets::{AssetStartupPlan, AssetStartupMode}` plus
`BootstrapBuilder::with_asset_startup(...)`; keep file-backed native/package-dev inputs on
`AssetStartupPlan::development_dir(...)` / `AssetStartupPlan::development_manifest(...)`, and use
`AssetStartupPlan::packaged_entries(...)`, `AssetStartupPlan::packaged_bundle_entries(...)`, or
`AssetStartupPlan::packaged_embedded_entries(...)` for packaged/web/mobile-friendly bytes.
On native/package-dev lanes, `FileAssetManifestResolver::from_bundle_dir(...)` is the first-party
generated-manifest convenience path when you want one directory to become one logical bundle
without teaching raw repo-relative paths in app/widget code; register that resolver with
`fret::assets::register_resolver(...)` on the host path.
When a native/dev-only UI helper still needs real file reload ergonomics, keep the app/widget
surface on logical bundle locators and let
`fret-ui-assets::ui::ImageSourceElementContextExt::use_image_source_state_from_asset_request(...)`
or `fret-ui-assets::ui::SvgAssetElementContextExt::svg_source_state_from_asset_request(...)`
consume the resolver's bundle/reference bridge instead of introducing direct raw file-path widget
loading in app code. Keep `resolve_image_source_from_host_locator(...)` /
`resolve_svg_source_from_host_locator(...)` as the lower-level UI-ready source seams, and use
`fret::assets::resolve_reference(...)` / `resolve_locator_reference(...)` when a non-UI
integration truly needs the raw external reference itself.
Use `FileAssetManifestResolver::from_manifest_path(...)` plus
`fret::assets::register_resolver(...)` when tooling already emits an explicit manifest artifact
that should be reviewed, versioned, or packaged directly.
For a first-party manifest artifact command, use
`fretboard assets manifest write --dir assets --out assets.manifest.json --app-bundle my-app`.
If you are already on the `fret` builder path, keep both development and packaged startup on
`FretApp::asset_startup(...)` / `UiAppBuilder::with_asset_startup(...)` with `AssetStartupPlan` +
`AssetStartupMode`, so validation fails early during startup configuration instead of being buried
in app-local setup glue. On the builder path, asset registrations preserve call order, so later
registrations can intentionally override earlier ones for the same logical locator.
For package-owned or generated compile-time bytes, the same ordered builder surface now includes
`FretApp::{asset_entries, bundle_asset_entries, embedded_asset_entries}` and
`UiAppBuilder::{with_bundle_asset_entries, with_embedded_asset_entries}`.
On the host path, `set_primary_resolver(...)`, `register_resolver(...)`,
`register_bundle_entries(...)`, and `register_embedded_entries(...)` now participate in the same
ordered resolver stack, so later registrations override earlier ones for the same logical
locator.

**Reusable component surface:** if you intentionally use the `fret` facade for reusable
component/scaffold code, keep that code on `use fret::component::prelude::*;`. That surface now
provides `ComponentCx`, `UiBuilder`/`UiPatchTarget`/`IntoUiElement<H>`, layout/style refinements,
and semantics/overlay helpers without pulling in `FretApp`, `AppUi`, or runner-facing seams.
Overlap-heavy helper traits remain anonymous `as _` imports on this lane so method ergonomics stay
intact without widening autocomplete pressure. The conversion surface is intentionally being
collapsed to one public component conversion trait; new docs/examples should follow
`docs/workstreams/into-element-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md` instead of
teaching the legacy split conversion trait names. When reusable component code needs explicit
command identity values, import `fret::actions::CommandId` (or `fret-runtime` directly) instead
of expecting `CommandId` from `fret::component::prelude::*`. When reusable component code needs
environment/responsive helpers, import them explicitly from `fret::env::{...}` instead of
expecting breakpoint/media helpers from `fret::component::prelude::*`. When reusable component or
advanced code intentionally needs raw activation helper glue, import `fret::activate::{on_activate,
on_activate_notify, on_activate_request_redraw, on_activate_request_redraw_notify}` explicitly
instead of expecting those helper names from the component prelude.
When reusable component code needs overlay anchoring helpers or overlay stack/introspection nouns,
import them explicitly from `fret::overlay::*` instead of expecting those lower-level names from
`fret::component::prelude::*`.

**Advanced/manual-assembly surface:** `use fret::advanced::prelude::*;` is now intentionally
advanced-only. It no longer forwards `fret::component::prelude::*` wholesale. If an advanced demo
or integration also authors ordinary component/UI composition (`ui::*`, `.ui()`, `.into_element`,
model watch helpers, overlay authoring helpers), add an explicit second import:

```rust
use fret::advanced::prelude::*;
use fret::component::prelude::*;
```

This keeps the tier boundary visible instead of letting advanced code rediscover component
authoring vocabulary through a hidden umbrella import.

**Surface taxonomy:** for user-facing docs, keep `fret` aligned with the same repo-wide ladder:

- **Default**: `hello`, `simple-todo`, `todo`
- **Comparison**: evidence-oriented side-by-side samples such as `simple_todo_v2_target`
- **Advanced**: manual assembly, interop, renderer/docking, maintainer harnesses

**Feature profiles (recommended):**

`fret` is designed to let apps choose between “smooth by default” and “small when you opt out”.

- `default` = `desktop` + `app` (recommended for native desktop apps).
  - Includes: shadcn integration.
  - Intentionally excludes: config files, UI asset caches, icon packs, icon preloading, command palette.
- `batteries` = a bigger opt-in bundle for app/dev convenience:
  - includes diagnostics wiring, config files, UI assets, icons, and (optional) icon SVG preloading.
  - includes `state` (selector/query helpers).

**Common feature combos (practical map):**

| Goal | Suggested `fret` features | Notes |
| --- | --- | --- |
| Small desktop app (shadcn UI only) | `["desktop","shadcn"]` | Minimal explicit profile (no config files, no diagnostics, no assets/icons). |
| Add derived + async state helpers | `["state"]` | Enables `AppUi` data helpers (`cx.data().selector_layout(...)`, raw `cx.data().selector(...)`, `cx.data().query(...)`) plus explicit `fret::selector::*` / `fret::query::*` secondary lanes. |
| Add routing integration | `["router"]` | Exposes the explicit app-level router extension surface (`fret::router::*`). |
| Add icons | `["icons"]` | Installs default icon packs (Lucide) via bootstrap wiring. |
| Add image/SVG caches | `["ui-assets"]` | Wires UI asset caches + budgets (compile/runtime cost). |
| Enable layered `.fret/*` config | `["config-files"]` | Filesystem side effects; opt-in for embed/minimal builds. |
| Opt into “everything convenient” | `["batteries"]` | Convenience bundle; may increase cold compile time. |

Minimal / explicit profile (useful for embed/minimal builds that must avoid filesystem side effects):

```toml
[dependencies]
fret = { path = "../path/to/fret/ecosystem/fret", default-features = false, features = ["desktop", "shadcn"] }
```

Enable selector/query helpers (for `cx.data().selector_layout(...)`, raw `cx.data().selector(...)`, and `cx.data().query(...)` on `AppUi`):

```toml
[dependencies]
fret = { path = "../path/to/fret/ecosystem/fret", features = ["state"] }
```

Recommended app profile (golden path; easiest):

```toml
[dependencies]
fret = { path = "../path/to/fret/ecosystem/fret" } # defaults: desktop + app
```

“Batteries included” profile (opt-in bundles):

```toml
[dependencies]
fret = { path = "../path/to/fret/ecosystem/fret", features = ["batteries"] }
```

Notes:

- `config-files` is opt-in because it reads layered `.fret/*` files (settings/keymap/menubar).
- `ui-assets` is opt-in because it wires caches/budgets and can increase compile + runtime cost.
- `icons` / `preload-icon-svgs` are opt-in (GPU-time tradeoff; apps can install custom packs).
- `devloop` and `tracing` are kept only as advanced/maintainer aliases on `fret`; prefer the
  owning crates (`fret-launch/dev-state`, `fret-bootstrap/tracing`) for new integrations.
- Docking and editor-theming ecosystems should be used from their owning crates
  (`fret-docking`, `fret-ui-editor`) instead of expecting `fret` root feature proxies.
- Design-system- or domain-specific crates that do not form a stable `fret` root story
  (for example Material 3 or AI UI ecosystems) should be used as direct crate dependencies
  instead of expecting `fret` root feature proxies.

### `fret-framework`

**What it is:** the public facade (re-exports + convenience feature bundles).

**Use it when:** you want an advanced/manual assembly surface for “core + app + ui” (and optional bundles like `native-wgpu` / `web`), without pulling ecosystem defaults into `crates/*`.

### `fret-core`

**What it is:** the minimal portable contract crate (IDs, geometry, input/event types, scene/display-list primitives).

**Use it when:** you are building a portable library (headless engines, domain models, shared contracts).

### `fret-runtime`

**What it is:** portable runtime-facing types (effects, commands, menus, keymap parsing, when-expressions, host traits).

**Use it when:** you need `Effect`, `CommandId`, keymap parsing, or other runtime-value contracts without pulling in UI/app.

### `fret-ui`

**What it is:** the UI runtime (declarative elements, layout, hit-testing, focus routing, overlays substrate, theme).

**Use it when:** you are rendering UI (apps or portable component libraries).

**Notes:** feature flags include `unstable-retained-bridge` (compat retained widgets). The window-scoped layout engine v2 is the default layout engine in `fret-ui` (no feature flag).

### `fret-app`

**What it is:** the app runtime (global store, command registry, scheduling helpers, file-backed settings/keymap helpers).

**Use it when:** you are writing an application, or when your ecosystem crate provides **optional** “app integration”.

**Component author tip:** gate app integration behind a feature (e.g. `app-integration`) so UI-only crates can stay `fret-ui`-only.

### `fret-ui-app`

**What it is:** first-party glue binding `fret-ui` to `fret-app::App` (used by demos and the bootstrap driver).

**Use it when:** you are implementing an app driver or first-party harness code that wants “App + UiTree” integration.

### `fret-launch`

**What it is:** runner glue (desktop/winit today) that owns effect draining, presentation, and frame loop wiring.

**Use it when:** you are building runnable apps (native / wasm harness shells).

**Avoid it when:** you are writing reusable component libraries.

### `fret-render`

**What it is:** the stable default facade over Fret's wgpu renderer building blocks.

**Use it when:** you are extending or embedding the renderer (runner/app side), or doing rendering diagnostics.

**Stable v1 default-facade buckets:**

- Core runtime/bootstrap entrypoints: `Renderer`, `RenderSceneParams`, `SurfaceState`, `WgpuContext`
- Capability and adapter snapshots: `RendererCapabilities`, `WgpuAdapterSelectionSnapshot`
- Render-target / ingest contracts: `RenderTargetDescriptor`, `RenderTargetMetadata`, and the
  `RenderTarget*` value enums
- Diagnostics/report stores for first-party runners/tooling: `RendererPerfFrameStore`,
  `RendererPerfFrameSample`, `WgpuHubReportCounts`, `WgpuHubReportFrameStore`,
  `WgpuHubReportFrameSample`, `WgpuAllocatorReportFrameStore`,
  `WgpuAllocatorReportFrameSample`
- External image/SVG upload helpers and `viewport_overlay`

Nested diagnostics detail structs stay backend-specific by default. Reach for `fret-render-wgpu`
directly if you need names like `RenderPerfSnapshot`, `IntermediatePerfSnapshot`,
`SvgPerfSnapshot`, `BlurQualitySnapshot`, `EffectDegradationSnapshot`,
`WgpuInitDiagnosticsSnapshot`, adapter sub-snapshots, allocator summary/top-allocation rows, or
per-attempt init records.

**Topology entrypoints:**

- Editor-hosted convenience path:
  create a `WgpuContext` with `WgpuContext::new()` / `WgpuContext::new_with_surface(...)`, then
  build `Renderer` and `SurfaceState` from that context.
- Engine-hosted direct path:
  keep the host-owned `wgpu::Instance` / `Adapter` / `Device` / `Queue`, derive
  `RendererCapabilities::from_adapter_device(...)`, then call `Renderer::new(...)`,
  `SurfaceState::new(...)`, and `render_scene(...)` directly.

`WgpuContext` remains a stable convenience surface for first-party runner/bootstrap stacks, demos,
and tools that want Fret to own GPU initialization. If your engine already owns the GPU topology,
skip it and use the direct path instead.

**Reach for `fret-render-wgpu` directly when:** you need backend-specific diagnostics/report stores
or helper surfaces that are intentionally not curated by the `fret-render` facade.

### `fret-platform`, `fret-platform-native`, `fret-platform-web`

**What they are:** portable platform I/O contracts and backend implementations (clipboard, drag/drop, dialogs, open-url).

**Use them when:** you are implementing a runner/backend, or need platform services at the app boundary.

### `fret-runner-winit`, `fret-runner-web`

**What they are:** platform adapters for event/input mapping (`winit`-based today).

**Use them when:** you are implementing runnable shells or platform integration.

### `fret-fonts`

**What it is:** bundled default fonts (wasm/bootstrap convenience).

**Use it when:** you want a default font set without external font management (especially for wasm demos).

### `fret-a11y-accesskit`

**What it is:** AccessKit bridge glue for accessibility.

**Use it when:** you are working on accessibility surfaces or runner integration for a11y snapshots/actions.

## Ecosystem crates (`ecosystem/*`)

These crates are “real” but **policy-heavy and fast-moving**. They should remain portable unless explicitly runner-oriented.

### `fret-ui-kit`

**What it is:** reusable component infrastructure on top of `fret-ui`:

- styling refinements (`ChromeRefinement`, `LayoutRefinement`),
- headless primitives (roving focus, typeahead, popper/tooltip primitives),
- overlay controller/policy surfaces.

**Use it when:** you are authoring reusable components and want shared policy primitives.

### `fret-ui-shadcn`

**What it is:** shadcn/ui-aligned component taxonomy and recipes built on `fret-ui-kit`.

**Use it when:** you want a ready-to-use design-system surface (buttons, inputs, popovers, command palette, etc).

**Optional heavy lanes:**

- enable `fret-ui-shadcn/chart` only when you need the shadcn-aligned chart recipe surface
- enable `fret-ui-shadcn/executor-integration` only when you need executor-backed async recipe helpers such as Sonner promise flows

**Theme integration:**

- Direct crate usage: prefer `use fret_ui_shadcn::{facade as shadcn, prelude::*};`, then call
  `shadcn::themes::apply_shadcn_new_york(...)` for an explicit one-shot preset when the app wants
  to own a fixed theme baseline directly, or enable `fret-ui-shadcn/app-integration` and call
  `shadcn::app::install(...)` for the golden-path app wiring that also opts the app into
  environment-aware host-theme syncing. Treat that curated facade import as the only first-contact
  component-family discovery lane: `shadcn::app::*` and `shadcn::themes::*` are setup lanes, not
  peer discovery lanes. For environment / `UiServices`-boundary hooks, stay explicit with
  `fret_ui_shadcn::advanced::{sync_theme_from_environment(...), install_with_ui_services(...)}`.
  `fret_ui_shadcn::advanced::*` is an implementation/debug lane, not a competing default import.
  For first-party prose/demo helpers, `shadcn::raw::typography::*` remains an explicit escape
  hatch. Only drop to `shadcn::raw::*` beyond these documented cases when you intentionally need
  the full uncurated module surface; the flat crate root is now treated as a hidden compatibility
  layer rather than a teaching lane.
- Through `fret`: use `fret::shadcn::themes::apply_shadcn_new_york(...)` for explicit
  app-owned/fixed presets, use `fret::shadcn::app::install(...)` when you want the curated app
  integration plus environment-aware host-theme syncing, and only drop to `fret::shadcn::raw::*`
  when you need the full uncurated `fret_ui_shadcn` surface. Treat
  `fret::shadcn::{Button, Card, ...}` as the only first-contact component-family lane here too:
  `fret::shadcn::app::*` and `fret::shadcn::themes::*` are setup lanes, not peer discovery lanes.
  That same raw escape hatch also carries advanced service hooks at
  `fret::shadcn::raw::advanced::*`; first-party prose/demo helpers may also use
  `fret::shadcn::raw::typography::*`.

**Tables vs grids (naming and intent):**

- `DataTable`: business-table surface (headless filtering/sorting/pagination + virtualized rows; recipes like toolbar + pagination).
- `DataGrid` (`DataGridCanvas`): performance ceiling (canvas-rendered, constant-ish UI node count; intended for spreadsheet-scale density).
- `experimental::DataGridElement`: element-based prototype for rich per-cell UI; not intended for spreadsheet-scale workloads.
- Diagnostics for virtualized tables should stay table-owned:
  prefer `DataTable::debug_ids(TableDebugIds { ... })` or explicit `TableDebugIds` on
  `fret_ui_kit::declarative::table::table_virtualized*` rather than relying on renderer-local
  `test_id` markers inside header/cell closures. Stable anchors should bind to the table's own
  header-row / header-cell / body-row layout wrappers.

### `fret-icons` + icon packs

- `fret-icons`: renderer-agnostic icon IDs + registry.
- `fret-icons-lucide`, `fret-icons-radix`: vendored SVG packs and curated aliases.

**Use them when:** you want semantic icon IDs in components without coupling to SVG rasterization or GPU caches.

**Recommended integration:**

- **Component crates:** depend on `fret-icons` (and use semantic `IconId`s). Avoid depending on a specific pack.
- **Apps using `fret`:** keep the default Lucide pack on `fret`'s `icons` feature, or install an
  explicit pack through `FretApp::setup(fret_icons_lucide::app::install)` /
  `FretApp::setup(fret_icons_radix::app::install)`. For custom packs, publish the same shape on
  your own crate and call `FretApp::setup(my_icons::app::install)`.
  Treat vendor ids such as `lucide.*` / `radix.*` as explicit pack contracts. Treat semantic
  `ui.*` ids as the reusable default for component crates. Today semantic alias registration is
  install-order-sensitive (`first installed pack wins`), so reusable crates should not assume more
  than one semantic provider unless they document that requirement.
- **Apps using `fret-bootstrap` directly:** use `BootstrapBuilder::with_lucide_icons()` /
  `BootstrapBuilder::with_radix_icons()`. For custom packs, call
  `BootstrapBuilder::register_icon_pack(...)`.
- **Direct app wiring:** when you depend on a pack directly, use the explicit `crate::app` seam
  (`fret_icons_lucide::app::install`, `fret_icons_radix::app::install`) instead of root-level
  install helpers.
- **Reusable ecosystem crates with transitive resources:** if a crate depends on a pack and also
  ships images/SVGs/fonts, publish one installer/bundle surface that composes both. App code
  should call `FretApp::setup(MyKitBundle)` (or compose named installers), not replay
  `IconRegistry` mutation plus `register_bundle_entries(...)` manually for the dependency. See
  `docs/workstreams/resource-loading-fearless-refactor-v1/ECOSYSTEM_INSTALLER_COMPOSITION.md`.

### `fret-ui-assets`

**What they are:** UI render asset caches and upload helpers (images/SVG), aligned with the “resource handles + flush point” model.

**Use them when:** your UI loads icons/images/SVG and you want key-based caching, budgeting, and ID-based rendering.

**Recommended integration:**

- **Apps using `fret`:** keep logical asset identity on `fret::assets::{AssetLocator, AssetRequest, StaticAssetEntry, register_bundle_entries, register_embedded_entries, ...}` and enable
  `fret`'s `ui-assets` feature when you want the default image/SVG caches driven from the event pipeline.
  App-owned resources should normally live under `AssetBundleId::app(...)`; ecosystem/package-owned
  shipped resources should normally live under `AssetBundleId::package(...)`.
  On native/package-dev lanes, keep the builder path on `FretApp::asset_startup(...)` /
  `UiAppBuilder::with_asset_startup(...)` with
  `AssetStartupPlan::{development_dir(...), development_manifest(...)}` before dropping to
  `FileAssetManifestResolver::{from_bundle_dir(...), from_manifest_path(...)}` plus
  `fret::assets::register_resolver(...)`.
  For compile-time owned bytes, prefer generated modules that expose `mount(builder)` or the
  builder-path helpers `with_bundle_asset_entries(...)` / `with_embedded_asset_entries(...)`
  before falling back to app-local setup glue.
  If a crate only publishes shipped bytes, the generated `Bundle` / `install(app)` /
  `mount(builder)` surface is usually enough. Once the crate also composes icon packs, commands,
  settings, theme/bootstrap wiring, or multiple generated asset modules, wrap those low-level
  generated helpers in one named installer/bundle surface and teach that wrapper instead.
  When you want to emit an explicit manifest artifact from a bundle directory, prefer
  `fretboard assets manifest write ...` over hand-authoring JSON.
  Direct host registrations preserve order across `set_primary_resolver(...)`,
  `register_resolver(...)`, `register_bundle_entries(...)`, and `register_embedded_entries(...)`,
  so later registrations intentionally override earlier ones for the same logical locator.
- **Apps using `fret-bootstrap` directly:** enable `fret-bootstrap/ui-assets` so `UiAppDriver` drives the caches from the event pipeline; optionally override
  budgets via `BootstrapBuilder::with_ui_assets_budgets(...)`. Keep logical asset identity on
  `fret_bootstrap::assets::{AssetBundleId, AssetLocator, AssetRequest, StaticAssetEntry, ...}`.
  For startup that needs one explicit development-vs-packaged switch, prefer
  `fret_bootstrap::assets::{AssetStartupPlan, AssetStartupMode}` plus
  `BootstrapBuilder::with_asset_startup(...)`. Keep native/package-dev file-backed inputs on
  `AssetStartupPlan::development_dir(...)` / `AssetStartupPlan::development_manifest(...)`, and
  keep packaged bytes on `AssetStartupPlan::packaged_entries(...)`,
  `AssetStartupPlan::packaged_bundle_entries(...)`, or
  `AssetStartupPlan::packaged_embedded_entries(...)`.
- **Direct app wiring:** use `fret_ui_assets::app::configure_caches(...)` or
  `fret_ui_assets::app::configure_caches_with_budgets(...)`; keep
  `fret_ui_assets::advanced::{configure_caches_with_ui_services(...), configure_caches_with_ui_services_and_budgets(...)}`
  as explicit advanced/bootstrap-only escape hatches.
- **Component crates:** prefer receiving handles/IDs from the app; only depend on caches directly if you truly need cache APIs,
  and gate it behind an explicit feature (e.g. `app-integration`).
  If the crate ships its own bytes, keep them package-owned and expose one installer/mount helper
  instead of asking apps to know your internal asset bundle layout.
  Prefer `BundleAsset` when the bytes are part of the crate's public lookup story (`AssetLocator::bundle(...)`,
  stable docs/examples, app overrides, or cross-crate composition). Use `Embedded` for lower-level
  owner-scoped bytes that are not the public cross-package contract.
- **Reusable ecosystem crates that also depend on icon packs:** keep icon-pack installation plus
  package-bundle registration inside the same installer surface so apps compose one named
  dependency bundle rather than replaying low-level icon + asset registrations by hand. See
  `docs/workstreams/resource-loading-fearless-refactor-v1/ECOSYSTEM_INSTALLER_COMPOSITION.md`.

### `fret-bootstrap`

**What it is:** an opinionated bootstrap layer for apps (golden-path defaults) on top of `fret-launch`.

**Use it when:** you want:

- layered settings/keymap loading,
- the same named asset startup contract as `fret`, via
  `fret_bootstrap::assets::{AssetStartupPlan, AssetStartupMode}` and
  `BootstrapBuilder::with_asset_startup(...)`,
- icon pack registration (built-in packs or custom),
- optional UI app driver wiring,
- optional command palette capability (toggle handling + per-window state + command gating),
- optional diagnostics + tracing wiring.

**Command palette note:** `fret-bootstrap/ui-app-command-palette` keeps the app-driver capability
layer only. If you want the default shadcn `CommandDialog` presentation on top of that capability,
enable `fret-bootstrap/ui-app-command-palette-shadcn` or use `fret`'s `command-palette` feature.

**Diagnostics note:** `fret-bootstrap/diagnostics` now keeps the generic diagnostics/export path
leaner. Enable `fret-bootstrap/diagnostics-canvas` only when you need retained canvas cache stats
in diagnostics bundles. Enable `fret-bootstrap/diagnostics-ws` only when you need the devtools
WebSocket transport bridge.

Note: dev hotpatch is an internal maintainer workflow today and is not part of the user-facing
onboarding path.

### `fret-executor`

**What it is:** portable background work helpers built on the `DispatcherHandle` contract:

- spawn background tasks without assuming a specific async runtime,
- deliver results through inboxes drained at a driver boundary (ADR 0175),
- propagate cancellation via `CancellationToken`.

**Async adapters (optional):**

- install a `FutureSpawnerHandle` global (tokio/wasm) and use `spawn_future_to_inbox(...)` to
  bridge async ecosystems into the inbox + driver-boundary apply model.

**Use it when:** you need to run work off the UI thread (or cooperatively on wasm) and want a
consistent inbox + cancellation vocabulary.

### `fret-selector`

**What it is:** selector-style derived state helpers:

- memoize expensive derived values behind an explicit dependency signature (`Deps: PartialEq`),
- optional UI sugar (`ElementContext::use_selector(...)`) plus the `fret` app-surface bridge
  (`cx.data().selector_layout(...)` for LocalState-first inputs, raw `cx.data().selector(...)`
  otherwise) to keep view code readable.

**Feature note:** on the default `fret` app path, enable `fret`'s `state` feature and prefer
`cx.data().selector_layout(...)` for view-owned `LocalState<T>` inputs. Keep raw
`cx.data().selector(...)` for explicit shared `Model<T>` / global signatures. When app code needs
to spell dependency-signature helpers explicitly, import them from
`fret::selector::ui::DepsBuilder` plus `fret::selector::DepsSignature` instead of expecting them
from `fret::app::prelude::*`. Enable `fret-selector/ui` only when you are working directly with
`ElementContext` in component/advanced surfaces.

**Use it when:** you need stable derived values (counts, filtered views, projections) without
introducing “tick models” or storing every derived value in the model store.

**Default app note:** on the `fret` golden path, prefer `cx.data().selector_layout(...)` when the
inputs are view-owned `LocalState<T>` slots. Keep raw `cx.data().selector(...)` plus
`fret::selector::ui::DepsBuilder` plus `fret::selector::DepsSignature` for explicit shared
`Model<T>` / global signatures or direct component/advanced `ElementContext` work.

### `fret-query`

**What it is:** query-style async resource state (TanStack Query-like) adapted to Fret:

- cached resource state in `Model<QueryState<T>>` so UI can observe it,
- background fetch via `fret-executor`,
- completion marshaled back through `InboxDrainRegistry` (ADR 0175),
- invalidation + time-based GC.

**Use it when:** you need loading/error/cache/invalidation semantics for remote resources or expensive
computations.

**Async fetch:** install a `FutureSpawnerHandle` global and use `cx.data().query_async(...)` /
`cx.data().query_async_local(...)` on `AppUi`. See `docs/integrating-tokio-and-reqwest.md`.

**Default app read note:** after creating a handle on the `fret` app path, prefer
`handle.read_layout(cx)` for the common `QueryState::<T>::default()` fallback case. Keep
`handle.layout(cx).value_or_default()` or declarative/component `handle.layout_query(cx)` when you
are intentionally staying on the lower-level tracked-read surface.

**Default app invalidation note:** when query invalidation happens inside `AppUi` or extracted
`UiCx` helpers, prefer `cx.data().invalidate_query(...)` /
`cx.data().invalidate_query_namespace(...)` so grouped app code keeps the redraw shell in one
place. Keep `fret::query::with_query_client(...)` for pure app/driver code that does not have a
`cx.data()` surface.

**Feature note:** on the default `fret` app path, enable `fret`'s `state` feature and prefer the
grouped app data helpers (`cx.data().query*`). When app code needs explicit query nouns, import
them from `fret::query::{QueryKey, QueryPolicy, QueryState, ...}` rather than expecting them from
`fret::app::prelude::*`. Extracted `UiCx` helpers keep that same grouped surface through
`UiCxActionsExt` / `UiCxDataExt` (or explicit imports from `fret::app::{UiCxActionsExt,
UiCxDataExt}` when you are intentionally not using the prelude). Enable `fret-query/ui` only when
you are working directly with low-level `ElementContext` or generic writer extensions outside the
app-facing `fret` facades.

### `fret-router` + `fret-router-ui`

**What they are:** a small router core (`fret-router`) and a thin UI adoption layer (`fret-router-ui`).

**Use them when:** you need a lightweight “URL + history + outlet” architecture without pulling in
UI gallery-scale harnesses.

Notes:

- On the default app path, enable `fret`'s `router` feature and prefer the explicit
  `fret::router::*` seam (`fret::router::app::install(...)`, `RouterUiStore`, `RouterOutlet`,
  link/history helpers).
- When depending on `fret-router-ui` directly, keep the same shape and use
  `fret_router_ui::app::install(...)` for command registration instead of inventing a parallel
  root-level app helper.
- `fret-router-ui` provides `RouterUiStore` (router + snapshot model), pressable-based link/outlet
  helpers, and history action helpers (`back_on_action()`, `forward_on_action()`,
  `navigate_history_on_action(...)`).
- Prefer wiring router history actions through
  `use fret::advanced::AppUiRawActionNotifyExt as _;` plus
  `cx.on_action_notify::<...>(store.back_on_action())` / `store.forward_on_action()` instead of
  hand-rolling window/host availability glue in app code.
- Keep routing adoption explicit at the app boundary (`FretApp::setup(...)`, app-owned models, or
  explicit view state) rather than treating router crates as a second default app runtime.
- Prefer keeping policy in apps (what pages exist, what prefetch means, what “not found” looks like).

### `fret-docking`

**What it is:** the policy-heavy docking UI/runtime adoption layer built on top of the stable
`fret-core` dock graph/contracts.

**Use it when:** you need editor-grade tab/split/tear-off workflows and are willing to opt into the
advanced retained/manual-assembly seams they require.

Notes:

- Prefer depending on `fret-docking` directly for docking adoption instead of expecting a `fret`
  root feature proxy.
- Use `fret_core::{DockNode, DockOp, PanelKey, ...}` / `fret_core::dock::*` for dock
  graph/contracts and `fret_docking::{DockManager, DockPanelRegistry, handle_dock_op, ...}` for
  the UI + runtime adoption helpers.
- Keep docking explicit at the app boundary: panel registry, docking policy, and `dock_op` driver
  wiring stay app-owned/advanced instead of becoming part of `fret::app::prelude::*`.
- Prefer teaching docking as an opt-in editor-grade capability, not as part of the small-app
  golden path.

### `fret-workspace`

**What it is:** the explicit editor/workspace shell owner for frame chrome, top/status bars,
workspace command scope, pane-content focus targets, and the default workspace menu model.

**Use it when:** you are building an editor-grade window interior rather than a small-app page or
one-off demo card layout.

Notes:

- Reach for `WorkspaceFrame`, `WorkspaceTopBar`, `WorkspaceStatusBar`,
  `WorkspaceCommandScope`, `WorkspacePaneContentFocusTarget`, and
  `workspace_default_menu_bar(...)` when assembling workspace chrome.
- Keep startup window policy out of `fret-workspace`; initial size, min/max size, startup
  position, resize increments, and other window-creation choices still belong on the
  `fret` / `fret-bootstrap` / `fret-launch` lane.
- Keep ordinary app page shells out of `fret-workspace`; centered cards, docs shells, and other
  page framing helpers remain app-owned until a future promotion audit proves a shared shape.
- Pair `fret-workspace` with `fret-docking` when you need dock graph orchestration or tear-off
  behavior; do not collapse the two owners into one surface.
- If the workspace also renders an in-window menubar, keep that renderer explicit on
  `fret::in_window_menubar::*` rather than treating it as a hidden workspace-shell alias.

### `fret-canvas`

**What it is:** policy-light canvas substrate helpers (pan/zoom transforms, drag phases, pixel policies, text caches).

**Use it when:** you build interactive 2D canvas UIs (node graphs, charts, editor canvases) and want shared math/state helpers.

### `fret-node`

**What it is:** a serializable node graph substrate with typed connections and editor-grade contracts.

**Use it when:** you need a node graph model (headless or UI-integrated).

**Notes:** supports a `headless` mode; UI integration is behind its `fret-ui` feature. If you opt
into app-owned command/keybinding wiring, use the explicit `fret_node::app::install(...)` seam
instead of root-level install helpers.

### `fret-plot` / `fret-chart` / `fret-plot3d`

- `fret-plot`: 2D plot/chart components (data-to-geometry + interaction) built on `fret-ui`.
- `fret-chart`: chart components built on the headless `delinea` engine and `fret-canvas`.
- `fret-plot3d`: 3D plot widgets embedded via viewport surfaces (engine-owned render targets) and viewport input forwarding.

**Use them when:** you need plotting/charting UI surfaces, and want to stay portable (no direct `wgpu`/`winit` coupling).

### `fret-gizmo`

**What it is:** editor-grade 3D gizmo logic for engine viewports (rendered by the engine; Fret composites the viewport).

**Use it when:** you need transform gizmos, pick policies, and viewport-space tool math (unit-explicit via the viewport input contract).

**Start here:** `docs/gizmo-viewport-integration.md` (end-to-end reference: `apps/fret-examples/src/gizmo3d_demo.rs`).

**Notes:** custom gizmos are supported via `GizmoPlugin` and host read-only domain values via `GizmoPropertySource` (ADR 0140/0152). For large-world picking stability, enable the optional `fret-gizmo/f64-math` feature (projection/unprojection runs in f64; public API stays f32).

### `fret-markdown` / `fret-code-view` / `fret-syntax`

- `fret-markdown`: Markdown renderer components (optional MathJax SVG).
- `fret-code-view`: code block UI (copy button, wrapping, syntax integration).
- `fret-syntax`: tree-sitter-based syntax infra with feature-gated language bundles.

**Use them when:** you need editor/documentation-like rich content surfaces.

### `fret-undo`

**What it is:** app-owned undo/redo infrastructure with explicit transaction boundaries and coalescing.

**Use it when:** you want a reusable history stack implementation without moving ownership into the UI runtime.

### `delinea`

**What it is:** a headless chart engine and interaction contracts used by chart/plot ecosystems.

**Use it when:** you need a portable, data-first chart engine tier that can be hosted by multiple UI surfaces.

## Apps and tooling (`apps/*`)

These are runnable harnesses, dev tools, and stress tests. Libraries should not depend on them.

- `fretboard`: dev CLI (run demos, generate templates).
- `fret-examples`: shared demo harness code.
- `fret-demo`, `fret-demo-web`: thin shells over `fret-examples`.
- `fret-renderdoc`, `fret-svg-atlas-stress`: diagnostics/stress harnesses.

## Related docs

- `docs/repo-structure.md` (core vs ecosystem vs apps)
- `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
- `docs/adr/0110-golden-path-ui-app-driver-and-pipelines.md`
