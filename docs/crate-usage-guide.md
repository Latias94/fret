# Crate Usage Guide (What to Depend On, When)

This document is a **practical map** of the crates in this repository:

- what each crate is for,
- when you should depend on it,
- and how to keep dependencies portable as your ecosystem grows.

If you are a component author, start with:

- `docs/component-author-guide.md`
- `docs/component-authoring-contracts.md`

## Quick rules of thumb

### The 5 crate names to remember

- `fret-kit`: desktop-first batteries-included app entry point.
- `fret-ui-shadcn`: default component surface (apps).
- `fret-ui-kit`: component authoring glue (ecosystem libraries).
- `fret`: framework facade for advanced/manual assembly.
- `fretboard`: dev tooling (templates + native/web runner).

1) If you are writing a reusable ecosystem library, avoid backend crates (`fret-launch`, `winit`, `wgpu`).

2) Prefer this dependency ladder (low → high):

- `fret-core` / `fret-runtime` (portable contracts and data types)
- `fret-ui` (portable retained/declarative UI runtime)
- `ecosystem/*` (policy layers, component libraries, domain ecosystems)
- `fret-app` / `fret-bootstrap` (app-level integration and golden-path defaults)
- `fret-launch` (runner glue, effect draining, presentation)

Desktop-first quick start:

- If you want a single dependency for a native desktop app, use `fret-kit` (ecosystem-level batteries-included wrapper).

Web/wasm quick start (tooling):

- `cargo run -p fretboard -- dev web --demo ui_gallery`

3) Only depend on `fret-app` if you need app-owned integration surfaces:

- command registration / command palette integration,
- default keybindings installation,
- file-backed settings/keymap loading helpers.

### Background work (portable)

Keep the UI/runtime deterministic by treating `App`/`ModelStore` as main-thread only (ADR 0008).

Recommended patterns:

- **Portable default**: background producers send **data-only** messages into an inbox; the UI thread drains the inbox at a driver boundary and schedules redraw (ADR 0112, ADR 0190).
- **Heavy apps**: run an external runtime (e.g. Tokio) on a dedicated thread, send results into an inbox, and `wake()` the runner to reach the next driver boundary promptly (ADR 0190).

Portability notes (native vs wasm):

- On native backends, background work is typically available (`exec.background_work=threads`).
- On wasm backends, "background" work is best-effort and may run cooperatively on the same thread (`exec.background_work=cooperative`).
  - Do not assume CPU-heavy work will not block UI; keep tasks short or move heavy work to a non-portable adapter (worker/thread/runtime) at the app boundary.
- `wake()` may be coalesced on all platforms; on wasm it may also be degraded (`exec.wake=best_effort`), so treat it as a hint to reach the next driver boundary, not a precise scheduling guarantee.
- Timers may be throttled on wasm (`exec.timers=best_effort`). Use runner-owned effects (`Effect::SetTimer`, RAF) for UI-visible timing and avoid relying on precise intervals.

Dependency guidance:

- **Reusable ecosystem crates** SHOULD depend on portable surfaces only (`fret-core` / `fret-runtime` / `fret-ui`) and use the inbox/dispatcher surface (ADR 0113, ADR 0190).
- **Apps** may use `fret-bootstrap` (or `fret-kit`) to get the golden-path wiring so they do not need to hand-roll channels + timers + wake logic.

## Features (Cargo)

Cargo features are widely used in Rust UI ecosystems to keep “small apps small” while allowing optional integrations.
We treat feature naming as **recommended convention**, not a hard requirement for third-party crates.

## Core framework crates (`crates/*`)

### `fret`

**What it is:** the public facade (re-exports + convenience feature bundles).

**Use it when:** you want a single dependency for “core + app + ui” (and optional bundles like `native-wgpu` / `web`).

### `fret-core`

**What it is:** the minimal portable contract crate (IDs, geometry, input/event types, scene/display-list primitives).

**Use it when:** you are building a portable library (headless engines, domain models, shared contracts).

### `fret-runtime`

**What it is:** portable runtime-facing types (effects, commands, menus, keymap parsing, when-expressions, host traits).

**Use it when:** you need `Effect`, `CommandId`, keymap parsing, or other runtime-value contracts without pulling in UI/app.

### `fret-ui`

**What it is:** the UI runtime (declarative elements, layout, hit-testing, focus routing, overlays substrate, theme).

**Use it when:** you are rendering UI (apps or portable component libraries).

**Notes:** feature flags include `unstable-retained-bridge` (compat retained widgets). The window-scoped layout engine v2 is enabled by default in this repository; `layout-engine-v2` is retained as a compatibility feature for downstream crates that explicitly enable it.

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

**What it is:** the wgpu renderer building blocks.

**Use it when:** you are extending or embedding the renderer (runner/app side), or doing rendering diagnostics.

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

**Theme integration:**

- Call `fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(...)` for an explicit preset, or
- Enable `fret-ui-shadcn/app-integration` and call `fret_ui_shadcn::install_app(...)` for the golden-path default.

**Tables vs grids (naming and intent):**

- `DataTable`: business-table surface (headless filtering/sorting/pagination + virtualized rows; recipes like toolbar + pagination).
- `DataGrid` (`DataGridCanvas`): performance ceiling (canvas-rendered, constant-ish UI node count; intended for spreadsheet-scale density).
- `experimental::DataGridElement`: element-based prototype for rich per-cell UI; not intended for spreadsheet-scale workloads.

### `fret-icons` + icon packs

- `fret-icons`: renderer-agnostic icon IDs + registry.
- `fret-icons-lucide`, `fret-icons-radix`: vendored SVG packs and curated aliases.

**Use them when:** you want semantic icon IDs in components without coupling to SVG rasterization or GPU caches.

**Recommended integration:**

- **Component crates:** depend on `fret-icons` (and use semantic `IconId`s). Avoid depending on a specific pack.
- **Apps:** enable `fret-bootstrap/icons-lucide` or `fret-bootstrap/icons-radix` and call the corresponding builder helper
  (`with_lucide_icons()` / `with_radix_icons()`). For custom packs, call `BootstrapBuilder::register_icon_pack(...)`.

### `fret-ui-assets` / `fret-asset-cache`

**What they are:** UI render asset caches and upload helpers (images/SVG), aligned with the “resource handles + flush point” model.

**Use them when:** your UI loads icons/images/SVG and you want key-based caching, budgeting, and ID-based rendering.

**Recommended integration:**

- **Apps:** enable `fret-bootstrap/ui-assets` so `UiAppDriver` drives the caches from the event pipeline; optionally override
  budgets via `BootstrapBuilder::with_ui_assets_budgets(...)`.
- **Component crates:** prefer receiving handles/IDs from the app; only depend on caches directly if you truly need cache APIs,
  and gate it behind an explicit feature (e.g. `app-integration`).

### `fret-bootstrap`

**What it is:** an opinionated bootstrap layer for apps (golden-path defaults) on top of `fret-launch`.

**Use it when:** you want:

- layered settings/keymap loading,
- icon pack registration (built-in packs or custom),
- optional UI app driver wiring,
- optional command palette integration,
- optional dev hotpatch toggles.

### `fret-kit`

**What it is:** a desktop-first batteries-included convenience crate that composes:

- `fret` (with `native-wgpu`) + `fret-bootstrap` (golden path wiring) + `fret-ui-shadcn` (default component surface).

**Use it when:** you want a working native app with minimal boilerplate and a single memorable dependency.

### `fret-canvas`

**What it is:** policy-light canvas substrate helpers (pan/zoom transforms, drag phases, pixel policies, text caches).

**Use it when:** you build interactive 2D canvas UIs (node graphs, charts, editor canvases) and want shared math/state helpers.

### `fret-node`

**What it is:** a serializable node graph substrate with typed connections and editor-grade contracts.

**Use it when:** you need a node graph model (headless or UI-integrated).

**Notes:** supports a `headless` mode; UI integration is behind its `fret-ui` feature.

### `fret-plot` / `fret-chart` / `fret-plot3d`

- `fret-plot`: 2D plot/chart components (data-to-geometry + interaction) built on `fret-ui`.
- `fret-chart`: chart components built on the headless `delinea` engine and `fret-canvas`.
- `fret-plot3d`: 3D plot widgets embedded via viewport surfaces (engine-owned render targets) and viewport input forwarding.

**Use them when:** you need plotting/charting UI surfaces, and want to stay portable (no direct `wgpu`/`winit` coupling).

### `fret-gizmo`

**What it is:** editor-grade 3D gizmo logic for engine viewports (rendered by the engine; Fret composites the viewport).

**Use it when:** you need transform gizmos, pick policies, and viewport-space tool math (unit-explicit via the viewport input contract).

**Start here:** `docs/gizmo-viewport-integration.md` (end-to-end reference: `apps/fret-examples/src/gizmo3d_demo.rs`).

**Notes:** custom gizmos are supported via `GizmoPlugin` and host read-only domain values via `GizmoPropertySource` (ADR 0155/0167). For large-world picking stability, enable the optional `fret-gizmo/f64-math` feature (projection/unprojection runs in f64; public API stays f32).

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
- `docs/adr/0111-user-facing-crate-surfaces-and-golden-path.md`
- `docs/adr/0112-golden-path-ui-app-driver-and-pipelines.md`
