# fret

> [!WARNING]
> **Experimental — under heavy development.**
>
> This project is an experiment in AI-driven software development. The vast majority of the code, tests, and documentation were written by AI (Codex). Humans direct architecture, priorities, and design decisions, but have not reviewed most of the code line-by-line. Treat this accordingly — there will be bugs, rough edges, and things that don't work. Use at your own risk.

Desktop-first, batteries-included entry points for building UI apps with Fret.

This is an **ecosystem-level** crate. It intentionally provides a small, ergonomic surface for
applications while keeping the framework/kernel crates (`crates/*`) policy-light.

## Boundary note

`fret` is the golden-path authoring facade for application code. It is intentionally **not** the
repo's canonical example host.

- Use `docs/examples/README.md` for the canonical learning/index path.
- Use `examples/README.md` as the GitHub-friendly portal.
- Keep runnable lessons in `apps/fret-cookbook/examples/`, component coverage in
  `apps/fret-ui-gallery`, and heavier platform/app demos in their owning app crates.

This keeps the facade teachable while leaving example/tooling ownership outside the crate.

For repository overview / architecture docs, see the monorepo README:
https://github.com/Latias94/fret

## Quick start (in this repo)

If you are learning the repo's default path, follow this ladder in order:

1. `hello`
2. `simple-todo`
3. `todo`

- Index: `docs/examples/README.md`
- The generated template READMEs repeat the same ladder and explain where each rung fits.
- Use `fretboard new todo` when you want the richer third-rung baseline, not as a replacement for
  the first two rungs.

Generate a runnable starter (minimal baseline first):

```bash
cargo run -p fretboard -- new simple-todo --name my-simple-todo
cargo run --manifest-path local/my-simple-todo/Cargo.toml
```

Then move to the richer third rung when you actually want selectors + queries:

```bash
cargo run -p fretboard -- new todo --name my-todo
cargo run --manifest-path local/my-todo/Cargo.toml
```

## Quick start (Cargo)

With defaults (desktop + app):

```toml
[dependencies]
fret = { path = "../fret" } # path is relative to your Cargo.toml
```

Enable selector/query helpers (optional):

```toml
[dependencies]
fret = { path = "../fret", features = ["state"] }
```

Enable the explicit router extension surface (optional):

```toml
[dependencies]
fret = { path = "../fret", features = ["router"] }
```

Enable the explicit docking surface (optional):

```toml
[dependencies]
fret = { path = "../fret", features = ["docking"] }
```

If your crate lives under `apps/` in this repository:

```toml
[dependencies]
fret = { path = "../../ecosystem/fret" }
```

Or explicitly opt into a smaller surface:

```toml
[dependencies]
fret = { path = "../fret", default-features = false, features = ["desktop", "shadcn"] }
```

## Minimal app skeleton

```rust,ignore
use fret::app::prelude::*;

struct HelloView;

impl View for HelloView {
    fn render(&mut self, _ui: &mut AppUi<'_, '_>) -> Ui {
        shadcn::Label::new("Hello from Fret!").into()
    }
}

fn main() -> fret::Result<()> {
    FretApp::new("hello")
        .window("Hello", (560.0, 360.0))
        .view::<HelloView>()?
        .run()
}
```

If app code needs explicit style/token nouns or icon helpers/IDs beyond the default lane, import
them from `fret::style::{...}` and `fret::icons::{icon, IconId}` instead of expecting them from
`fret::app::prelude::*`.
If app code needs explicit semantic-role nouns, import them from
`fret::semantics::SemanticsRole` instead of expecting them from `fret::app::prelude::*`.
If app code needs explicit selector/query helper nouns beyond the grouped `cx.data()` story,
import them intentionally from `fret::selector::{DepsBuilder, DepsSignature}` and
`fret::query::{QueryError, QueryKey, QueryPolicy, QueryState, ...}`.
For adaptive UI helpers such as breakpoints, safe-area insets, pointer/media preferences, or
Tailwind breakpoint probes, use `fret::env::{...}` explicitly.
For logical assets, use `fret::assets::{...}` and prefer `AssetBundleId::app(...)` /
`AssetBundleId::package(...)` plus `AssetLocator::bundle(...)` / `register_bundle_entries(...)`;
keep `AssetLocator::file(...)` and `AssetLocator::url(...)` as capability-gated escape hatches.
For app-owned compile-time assets on the builder path, prefer generated modules that expose
`generated_assets::mount(builder)` or call `UiAppBuilder::with_bundle_asset_entries(...)` /
`UiAppBuilder::with_embedded_asset_entries(...)` directly; keep `FretApp::asset_dir(...)` /
`UiAppBuilder::with_asset_dir(...)` as the native/package-dev convenience lane.
On native/package-dev lanes, `fret::assets::register_file_bundle_dir(...)` is the convenience
lane that scans one directory into one logical bundle without pushing repo-relative paths into
widget code. `fret::assets::register_file_manifest(...)` is the explicit manifest-artifact lane
when tooling already emits a reviewable/packageable mapping.
On the app-facing builder path, prefer `FretApp::asset_dir(...)` /
`UiAppBuilder::with_asset_dir(...)` for the generated-manifest convenience lane, or
`FretApp::asset_manifest(...)` / `UiAppBuilder::with_asset_manifest(...)` when you already have an
explicit manifest file. On the host path, `set_primary_resolver(...)`,
`register_resolver(...)`, `register_bundle_entries(...)`, and `register_embedded_entries(...)`
participate in one ordered resolver stack, so later registrations override earlier ones for the
same logical locator.
The same ordered builder surface now also includes compile-time/static entries through
`FretApp::{asset_entries, bundle_asset_entries, embedded_asset_entries}` and
`UiAppBuilder::{with_bundle_asset_entries, with_embedded_asset_entries}`.

## Features

- `desktop`: enable the native desktop stack (winit + wgpu) via `fret-framework/native-wgpu`.
- `app`: recommended baseline for apps (shadcn).
- `state`: enable selector/query helpers on `AppUi` (`cx.data().selector(...)`, `cx.data().query(...)`) for the default app surface, plus the explicit `fret::selector::*` / `fret::query::*` secondary lanes when app code needs state helper nouns.
- `router`: enable the explicit app-level router surface (`fret::router::{app::install, RouterUiStore, RouterOutlet, ...}`).
- `docking`: enable the explicit advanced docking surface (`fret::docking::{core::*, DockManager, handle_dock_op, ...}`).
- `editor`: keep installed `fret-ui-editor` presets resilient to `FretApp` shadcn theme resets.
- `batteries`: “works out of the box” opt-in bundle (config files + UI assets + icons + preloading + diagnostics).
- `config-files`: load layered config files from `.fret/` (settings/keymap/menubar).
- `diagnostics`: enable default diagnostics wiring (tracing + panic hook; plus extra dev tooling).
- `ui-assets`: enable UI render-asset caches (images/SVG) and install default budgets.
- `icons`: install the default built-in icon pack (Lucide).
- `preload-icon-svgs`: pre-register SVG icons on GPU ready.
- `command-palette`: enable the command palette wiring in the golden-path driver.

## Web / wasm

`fret` is desktop-first. For web demos in this repository, use tooling:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cargo run -p fretboard -- dev web --demo ui_gallery
```

This runs `apps/fret-demo-web` via `trunk serve`.

Related workstream: `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/`

## Choosing a native entry path

- App authors (default recommendation): `fret::FretApp::new(...).window(...).view::<V>()?`
- App authors with driver hooks: `fret::FretApp::new(...).window(...).view_with_hooks::<V>(...)?`
- Advanced integration with `fret` defaults: `fret::advanced::run_native_with_fn_driver(...)`
- Advanced integration with `FnDriver` hooks preserved: `fret::advanced::run_native_with_fn_driver_with_hooks(...)`
- Advanced integration with a preconfigured `FnDriver`: `fret::advanced::run_native_with_configured_fn_driver(...)`
- Advanced low-level interop driver path (compat seam, non-default): `fret::advanced::interop::run_native_with_compat_driver(...)`
- Advanced low-level runtime/render/viewport seams: `fret::advanced::{kernel::*, interop::*}`

## What remains first-class on `fret`

Advanced users do **not** need to drop to `fret-launch` immediately. The `fret` facade keeps the
following seams first-class:

- `FretApp::{setup(...), view::<V>(), view_with_hooks::<V>()}`
- `FretApp::asset_dir(...)` for native/package-dev generated bundle manifests
- `FretApp::asset_manifest(...)` for native/package-dev logical bundle manifests
- `FretApp::{asset_entries(...), bundle_asset_entries(...), embedded_asset_entries(...)}`
- `UiAppBuilder::{configure(...), setup(...), setup_with(...), with_asset_dir(...), with_asset_manifest(...), with_bundle_asset_entries(...), with_embedded_asset_entries(...)}`
- `fret::advanced::FretAppAdvancedExt::install(...)`
- `fret::advanced::UiAppBuilderAdvancedExt::{install(...), on_gpu_ready(...), install_custom_effects(...)}`
- `UiAppDriver::{window_create_spec, window_created, before_close_window}`
- `UiAppDriver::{record_engine_frame, viewport_input, handle_global_command}`
- `fret::advanced::{kernel::*, interop::*}`

The default builder chain stays small and app-facing on `fret`. Advanced users still keep the same
extension seams without dropping to `fret-launch` immediately, but the GPU/effects/bootstrap hooks
now live explicitly under `fret::advanced` instead of the default inherent builder surface.

Optional ecosystems also stay explicit. For example, the router integration lives under
`fret::router`; wire it with `FretApp::setup(fret::router::app::install)` instead of expecting it
to appear in `fret::app::prelude::*`. Docking similarly lives under `fret::docking` so advanced
apps can opt into panel registries, dock ops, and retained-host wiring without turning docking into
part of the default app prelude. The default design-system surface is similarly curated under
`fret::shadcn`: keep component names at `shadcn::Button` / `shadcn::Card`, use
`shadcn::app::install(...)` for app wiring, `shadcn::themes::apply_shadcn_new_york(...)` for
explicit presets, and `shadcn::raw::*` only when you intentionally need the full underlying crate
surface. Environment / `UiServices`-boundary hooks stay off the curated lane: if you only depend
on `fret`, reach them through `fret::shadcn::raw::advanced::*`; if you depend on the recipe crate
directly, use `fret_ui_shadcn::advanced::*`. Reusable ecosystem bundles can share the same
`.setup(...)` seam by implementing
`fret::integration::InstallIntoApp`; ordinary app docs/examples should still teach plain installer
functions first. For small app-local composition, it is also acceptable to write
`.setup((install_a, install_b))`; prefer a named bundle type once that composition becomes
reusable or crate-facing API. Because Rust does not let a trait-bound-only `fn(&mut App)`
implementation accept plain function items without explicit casts, `InstallIntoApp` stays broad in
implementation. Treat that as an internal accommodation: keep `.setup(...)` on named installer
functions, tuples, or named bundles, and reserve `.setup_with(...)` for one-off inline closures or
runtime-captured values.
The same rule should apply to shipped resources: app-owned bytes normally live under
`AssetBundleId::app(...)`, ecosystem/package-owned shipped bytes normally live under
`AssetBundleId::package(...)`, and reusable crates should publish installer or mount helpers rather
than asking apps to mirror internal bundle registrations. Icon packs are still installed through
explicit `crate::app::install` seams backed by the global `IconRegistry`; reusable components
should prefer semantic `IconId` / `ui.*` ids instead of baking one vendor pack into their public
contract unless that dependency is intentionally explicit.
The same explicit-lane rule applies to optional state helpers: keep grouped `cx.data().selector(...)`
and `cx.data().query*` as the default app story, and import `DepsBuilder` / `QueryKey`-style nouns
from `fret::selector::*` / `fret::query::*` only when app code actually needs to spell them.
Editor-themed apps can also opt into `fret`'s `editor` feature to make
installed `fret-ui-editor` presets survive the default `FretApp` shadcn auto-theme replay; the
actual editor widgets and presets still live in `fret-ui-editor`.

That makes `fret` suitable for both general-purpose desktop apps and many editor-style customizations
before you need to depend on `fret-bootstrap` or `fret-launch` directly.

## When to drop down to `fret-framework` + `fret-bootstrap`

`fret` is designed to keep the “first app” and “small app” story simple. Prefer dropping down
to manual assembly when you need:

- a custom runner/event loop integration (`fret-launch`),
- non-default settings/keymap/config file layering,
- different icon/asset wiring policies than the kit defaults,
- experimenting with alternate component surfaces without the kit defaults.

Mapping (rough):

- `fret::UiAppBuilder` -> `fret_bootstrap::UiAppBootstrapBuilder`
- `fret::UiAppDriver` -> `fret_bootstrap::ui_app_driver::UiAppDriver`
- `fret::advanced::run_native_with_fn_driver(...)` -> `fret_bootstrap::BootstrapBuilder::new_fn(...)`
- `fret::advanced::run_native_with_fn_driver_with_hooks(...)` -> `fret_bootstrap::BootstrapBuilder::new_fn_with_hooks(...)`
- `fret::advanced::run_native_with_configured_fn_driver(...)` -> `fret_bootstrap::BootstrapBuilder::new(...)` with a preconfigured `FnDriver`
- `fret::advanced::interop::run_native_with_compat_driver(...)` -> `fret_bootstrap::BootstrapBuilder::new(...)` for advanced low-level interop / retained driver cases
- `fret::advanced::kernel::*` -> `fret-framework::*`

The recommended manual-assembly entry point remains `fret-bootstrap`, keeping the underlying driver
hotpatch-friendly (function-pointer `FnDriver` surface, per ADR 0105 / 0110).
