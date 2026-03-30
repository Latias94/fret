# Resource Loading Fearless Refactor v1 — Capability Matrix

Status: Published current-truth matrix (updated 2026-03-30)

## Scope

This document records the current first-party truth for Fret resource loading across desktop,
web/WASM, and the current in-tree mobile posture.

It distinguishes three layers that are easy to conflate:

1. the general asset contract (`AssetLocator`, resolver stack, bundle identity, revisions),
2. app-facing builder/setup convenience lanes exposed by `fret`,
3. legacy UI escape hatches that bypass the general resolver contract.

## Portable default in one sentence

For cross-platform apps and reusable crates, ship logical bundle/embedded assets and mount them
through generated Rust modules or builder/setup installers; treat raw files and URLs as explicit
escape hatches.

## Current matrix

| Capability | Desktop native | Web/WASM | Mobile (current in-tree posture) | First-party guidance |
| --- | --- | --- | --- | --- |
| Memory assets | Yes | Yes | Yes | Safe contract surface everywhere through `InMemoryAssetResolver`. |
| Bundle assets | Yes | Yes | Yes | This is the portable default story. Use generated Rust modules, static bundle entries, or explicit resolvers. |
| Embedded assets | Yes | Yes | Yes | Supported through static embedded entries; useful for lower-level/package-owned bytes. |
| Raw files | Partial | No | No as a portable packaged story | On desktop, file-backed manifests/directories can mount files as logical bundle assets. Direct file-path UI helpers also exist, but they are legacy native/dev escape hatches. |
| URLs | Custom only | Partial | No first-party story | Web now has a first-party URL passthrough resolver for `AssetLocator::Url`, and the shared image bridge can consume resolver-provided URL image references on every platform. Desktop still needs a custom resolver, and first-party SVG/font URL lanes are still not shipped. |
| File watching / hot reload | Partial | No first-party lane | No first-party lane | Desktop native can opt into automatic invalidation for builder-mounted manifests/directories through `AssetReloadPolicy`; supported desktop hosts now prefer `NativeWatcher` with metadata polling fallback, publish the shared `AssetReloadEpoch`, and set `file_watch = true`. |
| System font scan | Yes | No | No documented first-party story yet | Desktop runners expose system font rescan/update flows. Web explicitly cannot access system font databases. |

## Detailed notes

### Memory assets

- First-party truth: supported everywhere through `InMemoryAssetResolver`.
- Practical use: transient runtime bytes, diagnostics captures, or custom resolvers that want the
  shared revision/error vocabulary without packaging implications.

### Bundle assets

- First-party truth:
  - desktop: yes through static entries and file-backed manifest/directory resolvers,
  - web: yes through static/generated bundle entries,
  - mobile: yes through static/generated bundle entries.
- Recommended user story:
  - app-owned bytes use `AssetBundleId::app(...)`,
  - reusable package-owned bytes use `AssetBundleId::package(...)`,
  - generated `Bundle` / `install(app)` / `mount(builder)` surfaces are the preferred publication
    story.

### Embedded assets

- First-party truth: supported everywhere through static embedded entry registration.
- Guidance:
  - `Embedded` is a stable lower-level owner-scoped lane,
  - but the default cross-package teaching surface should still prefer logical `BundleAsset`
    identity when the bytes are intended to be looked up publicly by bundle/key.

### Raw files

- Important distinction:
  - `FileAssetManifestResolver` does **not** resolve `AssetLocator::File`; it resolves logical
    `AssetLocator::bundle(...)` lookups backed by native files.
  - those same bundle locators can now expose an explicit native file-reference handoff through
    `resolve_reference(...)` / `resolve_locator_reference(...)` without dropping back to raw
    path-first widget code.
  - `fret-ui-assets::resolve_image_source*` now consumes that handoff on desktop, so logical
    bundle locators can recover path-based native/dev image loading without apps writing raw file
    paths directly.
  - native `fret-ui-assets::resolve_svg_file_source*` provides the same bridge for reloadable SVG
    file ergonomics while keeping the file-only shim explicit.
  - the old public direct file-path constructors were removed; the remaining escape hatch is the
    explicit locator/reference bridge, not widget-level raw paths.
- First-party truth today:
  - desktop: native/package-dev convenience lane exists,
  - web: no file-backed lane,
  - mobile: current in-tree code does not yet define a truthful packaged-mobile file-root story.
- Inference:
  - for mobile, generated Rust modules are the only trustworthy first-party packaged story today;
    raw file roots should not be taught as a portable contract.

### URLs

- First-party truth today:
  - the general asset contract has `AssetLocator::Url`,
  - `ImageSource::from_url(...)` is now a direct helper on every platform,
  - web/WASM now ships a first-party `UrlPassthroughAssetResolver` on the default launch host,
    which resolves URL locators as external URL references instead of pretending they are already
    byte-backed assets,
  - `resolve_image_source*` can now consume a resolver-provided URL reference on every platform:
    web/WASM keeps the browser-native URL handoff, while non-wasm targets fetch bytes off-thread
    and decode through the shared Rust image pipeline,
  - native image preview surfaces should not treat `AssetCapabilities { url: true, .. }` as a
    blanket promise that the host ships default URL support; resolver truthfulness still depends on
    whether the host actually installs a URL-capable resolver,
  - when no resolver layer returns a URL reference, web/WASM `resolve_image_source*` falls back to
    resolving bytes and feeding `ImageSource::from_resolved_asset_bytes(...)`, so the current
    first-party packaged/bundle lane is still a byte-fetch + Rust decode path rather than a
    browser-native decode lane,
  - desktop/native still has no first-party default resolver for `AssetLocator::Url`,
  - there is still no matching first-party SVG/font URL lane.
- Guidance:
  - treat URL loading as a target-specific or custom-resolver escape hatch unless you are on the
    shipped web host or you explicitly own the non-wasm resolver that will back it,
  - do not teach it as the default framework asset story.

### File watching and hot reload

- First-party truth today:
  - desktop native can opt into automatic invalidation for builder-mounted manifests/directories
    through `AssetReloadPolicy`,
  - that automation bumps the shared `AssetReloadEpoch` and publishes `file_watch = true` through
    the host-level capability snapshot,
  - the shared runtime now also publishes `AssetReloadStatus`, so diagnostics can distinguish
    watcher-backed reload from metadata-poll fallback and capture the fallback reason,
  - first-party diagnostics can now gate that state in two ways:
    - live/scripted assertions through typed `UiPredicateV1` predicates on
      `debug.resource_loading.asset_reload`,
    - post-run bundle checks through `fretboard diag stats --check-asset-reload-*`,
  - supported desktop hosts now prefer a native watcher backend and fall back to metadata polling
    if the watcher backend or current watch roots cannot be installed,
  - wasm/mobile still have no first-party automatic reload lane,
  - the old UI-specific reload alias was removed; code should use the shared
    `AssetReloadEpoch`.
- Guidance:
  - the `file_watch` capability bit means “the host can automatically detect native file-backed
    asset changes and publish invalidation,” not necessarily “the host uses OS file events,”
  - any future watcher-backed lane must still publish new `AssetRevision` values through the
    shared contract instead of inventing a separate reload protocol.

### System font scan

- First-party truth today:
  - desktop: yes, via runner-owned system font rescan/update flows,
  - web/WASM: no, explicitly unavailable,
  - mobile: no documented first-party story is locked yet.
- Guidance:
  - system font scan is an optional augmentation capability, not the baseline portable text story,
  - portable apps should not depend on it for their first-frame text environment.

## Authoring guidance by audience

### App authors

- Portable default:
  - generate Rust asset modules with `fretboard assets rust write ...`,
  - mount them with `generated_assets::mount(builder)?` or install them with a named bundle.
  - first-party generated modules now also publish `preferred_startup_plan()` /
    `preferred_startup_mode()` so native debug can stay file-backed while packaged targets stay on
    compiled bundle bytes, and those helpers now lower onto shared `fret` facade defaults
    (`AssetStartupPlan::development_bundle_dir_if_native(...)` /
    `AssetStartupMode::preferred()`).
  - native desktop startup can also opt into automatic dev reload for builder-mounted file assets
    through `AssetReloadPolicy` on `FretApp`, `UiAppBuilder`, or `BootstrapBuilder`; supported
    desktop hosts now prefer a native watcher with metadata polling fallback.
  - direct bootstrap apps can use the same named startup contract through
    `fret_bootstrap::assets::{AssetStartupPlan, AssetStartupMode}` plus
    `BootstrapBuilder::with_asset_startup(...)`.
- Native/package-dev convenience:
  - `FretApp::asset_dir(...)`,
  - `UiAppBuilder::with_asset_dir(...)`,
  - `fret::assets::register_file_bundle_dir(...)`,
  - `FretApp::asset_manifest(...)`,
  - `UiAppBuilder::with_asset_manifest(...)`,
  - `fret::assets::register_file_manifest(...)`.
- Escape hatches:
  - direct file-path UI helpers are for native/dev compatibility only,
  - direct `fret-ui-assets` app wiring is intentionally named `configure_caches*` because cache
    setup and event-driving are separate responsibilities,
  - URL loading is target-specific or custom-resolver territory.

### Ecosystem authors

- Publish package-owned bytes under `AssetBundleId::package(...)`.
- Expose named installer/bundle surfaces instead of asking apps to replay low-level registrations.
- If the crate also ships icons:
  - keep icon semantics on `IconRegistry` / `IconId`,
  - publish non-icon bytes through the general asset contract,
  - compose both behind one installer/bundle when appropriate.

## Evidence anchors

- `crates/fret-assets/src/lib.rs`
- `crates/fret-assets/src/file_manifest.rs`
- `crates/fret-runtime/src/asset_resolver.rs`
- `crates/fret-launch/src/assets.rs`
- `crates/fret-launch/src/runner/web/mod.rs`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/src/app_entry.rs`
- `apps/fretboard/src/assets.rs`
- `ecosystem/fret-ui-assets/src/image_source.rs`
- `ecosystem/fret-ui-assets/src/ui.rs`
- `ecosystem/fret-ui-assets/src/reload.rs`
- `crates/fret-launch/src/runner/web/gfx_init.rs`
- `crates/fret-launch/src/runner/web/effects.rs`
- `crates/fret-launch/src/runner/desktop/runner/effects.rs`
- `docs/adr/0317-portable-asset-locator-and-resolver-contract-v1.md`
