# Resource Loading Fearless Refactor v1

Status: In progress (audit complete; contract reset started; capability matrix published)

Tracking files:

- `docs/workstreams/resource-loading-fearless-refactor-v1/TODO.md`
- `docs/workstreams/resource-loading-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/resource-loading-fearless-refactor-v1/CAPABILITY_MATRIX.md`

Related inputs:

- `docs/architecture.md`
- `docs/repo-structure.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/workstreams/ui-assets-image-loading-v1/ui-assets-image-loading-v1.md`
- `docs/workstreams/font-system-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/font-mainline-fearless-refactor-v1/README.md`

## Why this workstream exists

Fret does not currently have one cross-platform resource-loading system. It has three partially
overlapping stories:

- images in `ecosystem/fret-ui-assets`,
- SVG bytes in `crates/fret-ui` plus file helpers in `ecosystem/fret-ui-assets`,
- font bootstrap and publication in `crates/fret-launch` / `crates/fret-fonts` / renderer code.

That split is survivable for a prototype, but it is the wrong base for a desktop-first + wasm +
mobile framework that wants editor-grade reliability.

This workstream takes a fearless posture:

- Fret is still pre-release, so wrong public-looking seams should be replaced, not preserved.
- Raw filesystem paths should stop being the default authoring story.
- `crates/fret-ui` must remain a mechanism layer that consumes stable handles, not packaging logic.
- Asset loading must become truthful about platform capabilities and startup guarantees.

## Current status (practical)

- The first core asset contract slice exists in `crates/fret-assets`:
  - locator vocabulary,
  - revision/result payloads,
  - capability reporting,
  - resolver trait.
- The wasm portability honesty gate is now green:
  - `cargo check -p fret-launch --target wasm32-unknown-unknown`
- Runtime host attachment has started:
  - `crates/fret-runtime/src/asset_resolver.rs` now exposes a host-level
    `AssetResolverService`.
  - The host service is composable instead of replace-only:
    - multiple resolver layers can be registered,
    - static bundle/embedded entries can be registered incrementally by app or ecosystem code.
  - Host registrations now share one ordered resolver stack across:
    - `set_primary_resolver(...)`,
    - `register_resolver(...)`,
    - `register_bundle_entries(...)`,
    - `register_embedded_entries(...)`.
  - Static entries no longer bypass later resolver layers, and replacing the primary resolver keeps
    its existing stack slot instead of jumping ahead of newer registrations.
- The app-facing `fret` facade now exposes `fret::assets`:
  - logical asset vocabulary is reachable without importing `fret-assets` / `fret-runtime`
    directly,
  - bundle/embedded registration now has an app-facing authoring lane
    (`register_bundle_entries(...)`, `register_embedded_entries(...)`).
  - structured bundle ids now have first-party constructors:
    - `AssetBundleId::app(...)`
    - `AssetBundleId::package(...)`
- A first native/package-dev manifest resolver now exists:
  - `crates/fret-assets/src/file_manifest.rs` defines a file-backed bundle manifest format and
    resolver,
  - `FileAssetManifestBundleV1::scan_dir(...)` and
    `FileAssetManifestResolver::from_bundle_dir(...)` now provide the first generated-manifest
    convenience lane for one directory -> one logical bundle,
  - `fret::assets::register_file_bundle_dir(...)` exposes that directory-scanning convenience on
    the app-facing facade,
  - `fret::assets::register_file_manifest(...)` mounts that resolver on the app-facing facade
    without teaching repo-relative paths in widget code.
  - file-backed bundle locators can now also expose an explicit native file-reference handoff via
    `resolve_reference(...)` / `resolve_locator_reference(...)` on the shared resolver contract,
    so platform APIs can ask for a real file path without bypassing bundle identity.
  - `fretboard assets manifest write --dir ... --out ... --app-bundle ...` now emits an explicit
    file-backed manifest artifact from a scanned bundle directory,
  - `FretApp::asset_dir(...)` / `UiAppBuilder::with_asset_dir(...)` keep that generated-manifest
    convenience lane on the startup builder surface,
  - `FretApp::asset_manifest(...)` / `UiAppBuilder::with_asset_manifest(...)` now keep that
    manifest lane on the startup builder surface instead of app-local setup glue.
  - `FretApp` now preserves asset registration call order across `asset_dir(...)` and
    `asset_manifest(...)`, so later builder calls override earlier ones consistently with the
    composable resolver stack.
  - `fretboard new todo --ui-assets` / `fretboard new simple-todo --ui-assets` now scaffold:
    - an `assets/` directory for app-owned files,
    - a checked-in `src/generated_assets.rs` stub for the portable compile-time lane,
    - `generated_assets::mount(builder)` on the default `fret` builder path,
    - and an explicit regeneration command for `fretboard assets rust write ...`.
- A first compile-time embedded artifact lane now exists for packaged/web/mobile-friendly builds:
  - `fretboard assets rust write --dir ... --out ... --app-bundle ...` emits a generated Rust
    module with:
    - `StaticAssetEntry` registrations,
    - stable content-based revisions,
    - media-type inference for common asset kinds,
    - `include_bytes!`-backed bytes owned by the package build.
  - The generated module supports two consumption surfaces:
    - `--surface fret` for apps using the `fret` facade,
    - `--surface framework` for lower-level/runtime-facing crates that want direct
      `fret-assets` / `fret-runtime` integration.
  - The `--surface fret` module now exposes both:
    - `register(app)` for direct host/app registration,
    - `mount(builder)` for `UiAppBuilder` startup-path mounting via
      `with_bundle_asset_entries(...)`.
  - `FretApp::asset_entries(...)`, `FretApp::bundle_asset_entries(...)`,
    `FretApp::embedded_asset_entries(...)`, `UiAppBuilder::with_bundle_asset_entries(...)`, and
    `UiAppBuilder::with_embedded_asset_entries(...)` now keep compile-time/static registrations on
    the same ordered builder/startup surface as `asset_dir(...)` and `asset_manifest(...)`.
- `fret-ui-assets` now consumes the shared resolver contract for both bytes and explicit external
  references:
  - image helpers prefer target-appropriate reference handoff first (native file paths, wasm URL
    references) and fall back to bytes when the current winning layer cannot provide a usable
    external reference; `ImageSourceElementContextExt::use_image_source_state_from_asset_request(...)`
    now keeps the UI-facing app/widget story locator-first instead of forcing app code to resolve
    `ImageSource` eagerly,
  - native SVG helpers can now bridge logical bundle locators into `SvgFileSource` for reloadable
    file-backed development ergonomics without teaching raw app paths as the primary authoring
    story, and `fret-ui-assets::ui::SvgAssetElementContextExt` now keeps the UI-facing app/widget
    story locator-first instead of exposing `SvgFileSource` directly in ordinary render code,
  - byte-based SVG loading and the existing async image invalidation ergonomics remain intact.
- The general asset contract now also models explicit external-reference handoff:
  - `ResolvedAssetReference` / `AssetExternalReference` in `crates/fret-assets`,
  - host-level resolution via `crates/fret-runtime/src/asset_resolver.rs`,
  - app-facing facade helpers via `fret::assets::resolve_reference(...)` and
    `fret::assets::resolve_locator_reference(...)`.
- `crates/fret-fonts` now also exposes a package-scoped asset-contract surface for bundled
  framework fonts:
  - `bundled_asset_bundle()` returns the logical package bundle id,
  - bundled font faces now carry stable logical keys/media types,
  - bundled profiles now expose `asset_entries()` so the framework-owned font baseline can mount
    through `StaticAssetEntry` instead of existing only as raw byte arrays.
- startup publication now also has an explicit runtime-global slot for that baseline identity:
  - `fret_runtime::BundledFontBaselineSnapshot` records which bundled profile/bundle/asset keys
    the runner chose,
  - startup now also mounts `fret-fonts::default_profile().asset_entries()` into the shared
    runtime asset resolver, so the framework-owned default font baseline is backed by the same
    package-owned `bundle + key` contract that it publishes diagnostically,
  - web and the current non-wasm winit startup path now both publish the current
    `fret-fonts::default_profile()` contract before startup font-environment initialization,
  - native startup still keeps `FontFamilyDefaultsPolicy::None`, so system-font augmentation stays
    separate from the framework-owned baseline identity,
  - local evidence now includes `cargo check -p fret-launch --target aarch64-apple-ios`,
    while Android target verification is currently blocked by missing NDK clang tooling in the
    local environment.
- Accepted ADR coverage now exists for both:
  - icon ownership/package composition (`docs/adr/0065-icon-system-and-asset-packaging.md`),
  - the general portable locator/resolver contract
    (`docs/adr/0317-portable-asset-locator-and-resolver-contract-v1.md`).
- A first capability matrix is now published in
  `docs/workstreams/resource-loading-fearless-refactor-v1/CAPABILITY_MATRIX.md`:
  - it distinguishes the general asset contract from legacy UI escape hatches,
  - it marks raw-file and URL lanes truthfully per platform,
  - and it records the current mobile inference: generated embedded/bundle modules are the only
    trustworthy first-party packaged story today.
- Legacy file-path helpers still exist only as migration/dev/native compatibility shims.
- Generated directory scanning is still only a native/package-dev convenience lane today; the new
  generated Rust module is the first packaged/web/mobile-friendly lane, but it does not yet cover
  hashed web output rewriting or mobile platform-native bundle mapping.
- Font startup still remains split across mobile/SVG text and is not fully solved by the current
  slice.

## Current incorrect logic (must be corrected, not preserved)

### 1) There is no single resource contract

Images, SVGs, and fonts follow different identities, loading rules, invalidation rules, and startup
rules.

Examples:

- Images:
  - `ecosystem/fret-ui-assets/src/image_source.rs`
- SVG file helpers:
  - `ecosystem/fret-ui-assets/src/svg_file.rs`
- Font startup/bootstrap:
  - `crates/fret-launch/src/runner/font_catalog.rs`
  - `crates/fret-launch/src/runner/web/gfx_init.rs`
  - `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`

That makes it too easy for each asset class to drift into a different platform truth.

### 2) `path` is treated like a primary authoring surface

Current resource helpers still expose file-path loading and older workstream/docs history taught it
as a primary authoring path:

- `ImageSource::from_file_path(...)` in `ecosystem/fret-ui-assets/src/image_source.rs`
- `SvgFileSource::from_file_path(...)` in `ecosystem/fret-ui-assets/src/svg_file.rs`
- cookbook examples:
  - `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`
  - `apps/fret-cookbook/examples/icons_and_assets_basics.rs`

But the runtime capability matrix explicitly says `real_paths = false` on wasm and mobile:

- `crates/fret-launch/src/runner/desktop/runner/mod.rs`

So the current story is structurally wrong:

- a raw OS path is not a portable resource identity,
- it cannot be the golden path for app authors,
- and it should not be used to teach ecosystem authors how to ship assets.

At most, `path` is a development/native capability escape hatch.

### 3) `install()` is only half an install

`fret-ui-assets::app::install()` currently only configures cache budgets:

- `ecosystem/fret-ui-assets/src/app.rs`
- `ecosystem/fret-ui-assets/src/ui_assets.rs`

But image readiness propagation still depends on wiring `UiAssets::handle_event(...)` into the app
driver:

- `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
- `apps/fret-ui-gallery/src/driver/runtime_driver.rs`

This is too easy to misuse. A public-looking `install()` API must either perform complete runtime
wiring or be renamed to describe its actual effect.

### 4) Font baseline semantics are still under-verified across platforms

Web and the current native winit startup path now inject the same bundled default font baseline as
soon as the renderer is ready:

- `crates/fret-launch/src/runner/web/gfx_init.rs`
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`

The remaining gap is now narrower:

- native still keeps `FontFamilyDefaultsPolicy::None` and layers optional system scanning on top,
- mobile shell/toolchain verification is still incomplete even though the current non-wasm runner
  path routes through the same startup helper,
- and Android local verification currently depends on an external NDK clang toolchain that is not
  present in this environment.

- `crates/fret-launch/src/runner/font_catalog.rs`

So the framework now guarantees one deterministic bundled baseline on web and on the current native
runner path before platform-specific augmentation, but mobile-specific diagnostics and Android CI
toolchain evidence are still missing.

### 5) SVG text does not share the text system font environment

The SVG renderer builds its own `usvg fontdb` and loads system fonts directly:

- `crates/fret-render-wgpu/src/svg.rs`

So even when the main text system has a known bundled/system font state, SVG `<text>` resolution
can still diverge.

For a framework that wants truthful cross-platform text behavior, that is the wrong architecture.

### 6) SVG file loading is sync filesystem I/O with epoch polling

`read_svg_file_cached(...)` does a synchronous `std::fs::read(...)` behind an epoch cache:

- `ecosystem/fret-ui-assets/src/svg_file.rs`

Images already have an async decode/event-driven story. SVG file loading should not remain a
completely different native-only pipeline.

### 7) The wasm build is not currently closed

The current resource/runtime graph is not even compile-closed for wasm because
`render_plan_dump_assemble.rs` imports non-wasm helper functions that are compiled out in
`render_plan_dump_summary.rs`:

- `crates/fret-render-wgpu/src/renderer/render_plan_dump_assemble.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan_dump_summary.rs`

Before changing author-facing resource APIs, we needed a truthful platform-closure baseline.

This specific compile break has now been fixed, but the broader cross-platform asset/runtime story
is still being refactored.

## Non-negotiable constraints

1. `crates/fret-ui` stays a mechanism layer
   - UI code should resolve to stable render/runtime handles (`ImageId`, `SvgId`, text-facing
     font environment state), not raw paths, URLs, or packaging conventions.

2. The primary authoring surface must be portable
   - The main app/ecosystem story must work on desktop, wasm, and mobile without changing widget
     code.

3. Packaging identity must be logical, not filesystem-relative
   - Resource identity should survive moving from dev checkout, to packaged desktop app, to web
     output, to mobile bundle.

4. Capability-gated escape hatches are allowed, but must be explicit
   - File paths, remote URLs, and external platform handles are valid escape hatches only when the
     capability matrix says so.

5. Startup baseline must be deterministic
   - Text, images, and SVGs should all start from a known framework-owned baseline before optional
     platform-specific augmentation.

6. Revision/invalidation must be first-class
   - Resource loading is not just “open bytes”; the system must expose revision and invalidation so
     hot reload, caching, diagnostics, and external tooling stay coherent.

7. Ecosystem authors need namespaced assets without runtime packaging knowledge
   - A component crate should be able to ship icons/fonts/images without knowing whether the final
     app is embedded, packaged, web-served, or mobile-bundled.

## Recommended target shape

### 1) Introduce one portable asset contract

Fret should gain one cross-cutting asset contract owned by a core layer, not by
`ecosystem/fret-ui-assets`.

Recommended ownership:

- add a new core crate such as `crates/fret-assets` for:
  - `AssetLocator`
  - `AssetKey` / `AssetBundleId`
  - `AssetRevision`
  - `AssetCapabilities`
  - `AssetLoadError`
  - resolver/loader traits
- keep `ecosystem/fret-ui-assets` as the UI-facing ergonomics layer on top of that contract

Recommended locator set:

- `Memory`
  - runtime-provided bytes or RGBA payloads
- `Embedded`
  - compile-time embedded bytes owned by code or generated manifests
- `BundleAsset`
  - logical asset key inside a named asset bundle
  - this should be the default app/ecosystem authoring path
- `File`
  - explicit native/dev escape hatch
- `Url`
  - explicit network/capability escape hatch

Important rule:

- `File` and `Url` are not the main authoring model.
- `BundleAsset` is the main authoring model.

### 2) Use bundle/key identity as the primary resource model

The portable identity should be “bundle + key”, not “current working directory + relative path”.

The bundle side should not stay an unstructured global string forever. The recommended direction is
to namespace it explicitly:

- app-owned bundles:
  - `AssetBundleId::app(...)`
- ecosystem/package-owned bundles:
  - `AssetBundleId::package(...)`

Examples of what this enables:

- an app bundle can ship `textures/test.jpg`,
- `fret-ui-shadcn` can ship its own icons under a separate bundle namespace,
- web packaging can rewrite the actual emitted file name or hash without changing UI code,
- mobile packaging can map the same logical key into APK/iOS bundle resources.

Recommended ecosystem ownership rules:

- app-owned resources live under `AssetBundleId::app(...)`,
- ecosystem/package-owned images, SVGs, fonts, and similar shipped bytes live under
  `AssetBundleId::package(...)`,
- reusable component crates should not require app authors to understand the final packaging
  layout in order to consume those resources,
- app authors should compose installer/setup surfaces instead of manually reproducing each
  ecosystem crate's internal asset mounting steps.

Important current gap:

- generic resources now have a package-bundle story, but icon packs still primarily install through
  the global `IconRegistry` surface,
- that is workable today, but the long-term story still needs one documented ownership bridge
  between `IconRegistry` and package-owned shipped bytes.

Current explicit icon-pack conflict policy:

- vendor ids (`lucide.*`, `radix.*`, ...) are namespaced and do not conflict,
- semantic ids (`ui.*`) use a stable first-successful-alias-wins rule via
  `IconRegistry::alias_if_missing(...)`,
- app/bootstrap code may intentionally override a semantic alias afterwards with
  `IconRegistry::alias(...)` or `register(...)`,
- the default first-party preference remains Lucide on the `fret` batteries lane because bootstrap
  only enables Radix semantic aliases when Lucide semantic aliases are not already selected.

This is the pattern used by mature cross-platform systems:

- Flutter packages assets into an `AssetBundle` and encourages `DefaultAssetBundle` indirection
  instead of raw file access:
  - https://api.flutter.dev/flutter/services/rootBundle.html
  - https://api.flutter.dev/flutter/widgets/DefaultAssetBundle-class.html
- Compose Multiplatform uses generated `Res` accessors as the primary authoring surface and only
  exposes `getUri()` when an external API specifically needs a platform path/URI:
  - https://www.jetbrains.com/help/kotlin-multiplatform-dev/compose-multiplatform-resources.html
  - https://www.jetbrains.com/help/kotlin-multiplatform-dev/compose-multiplatform-resources-usage.html
- GPUI/Zed keeps bundled assets behind an `AssetSource` trait, while local file and remote URL
  images are separate inputs:
  - `repo-ref/zed/crates/gpui/src/assets.rs`
  - `repo-ref/zed/crates/gpui/examples/image/image.rs`

### 3) Introduce a unified loader/resolver service

The runtime should expose one loader/resolver service that turns `AssetLocator` into resolved data.

Minimum responsibilities:

- resolve locator to bytes or a platform handle/URI when explicitly requested,
- report revision/invalidation,
- expose capability failures explicitly,
- surface structured diagnostics,
- normalize caching semantics across images, SVGs, and fonts.

Recommended result model:

- resolved payload
  - bytes for image/SVG/font decode
  - optional external URI/path handle for platform APIs that require it
- content metadata
  - media type if known
  - declared asset key / bundle origin
- revision metadata
  - monotonic revision or content fingerprint
- diagnostics
  - unsupported capability
  - missing asset
  - network denied
  - decode failure

This is also the right place to centralize hot reload, file watching, and dev-only refresh logic.

### 3.5) Define icon-pack participation explicitly

Icons should remain ergonomic for component authors, but their ownership model must be documented
instead of inferred from install order.

Recommended direction:

- components depend on `IconId` or semantic `ui.*` ids, not raw file paths,
- ecosystem libraries that need non-semantic vendor ids should keep those requirements explicit,
- icon packs should provide explicit installers/bundles so app authors compose one install surface
  per ecosystem dependency instead of hand-registering loose resources,
- the project should decide whether icon pack bytes eventually remain on `IconRegistry`, move onto
  the general asset contract, or keep a hybrid model with a documented bridge.

### 4) Keep `fret-ui` handle-based

`crates/fret-ui` should continue consuming only resolved, stable UI/render handles:

- `ImageId`
- `SvgId`
- font-environment state and text APIs already owned by the renderer/runtime

It should not learn:

- bundle naming,
- manifest layout,
- dev file watchers,
- platform URL rules,
- mobile sandbox path details.

### 5) Make the user-facing API explicit and ergonomic

Recommended golden-path authoring should look like logical asset selection, not filesystem plumbing.

Representative examples:

```rust
let app_assets = AssetBundleId::app("my-app");
let photo = ImageAsset::bundle(app_assets.clone(), "photos/avatar.png");
let icon = SvgAsset::bundle(app_assets.clone(), "icons/search.svg");
let font = FontAsset::bundle(app_assets, "fonts/InterVariable.ttf");
```

Escape hatches should stay explicit:

```rust
let dev_only = ImageAsset::file("assets/textures/test.jpg");
let remote = ImageAsset::url("https://example.com/banner.png");
let memory = ImageAsset::bytes(bytes);
```

If Fret wants an even more ergonomic facade, it should be sugar over the same contract, for example:

```rust
let logo = cx.assets().image("icons/logo.png");
```

But the underlying model should still resolve to a logical bundle/key, not a hidden repository path.

### 6) Fonts must adopt the same asset and revision story

Font startup should be redefined as:

1. load a deterministic bundled font profile on every platform before first-frame text work,
2. publish one renderer font environment snapshot,
3. optionally layer in system font scanning on platforms that support it,
4. bump revisions only when the effective environment changes.

This means:

- desktop should stop being “system-first by default” while web is “bundled-first by necessity”,
- the bundled baseline should be explicit and shared,
- `fret-fonts` should provide manifest-quality bundle/profile data,
- `fret-launch` should only orchestrate publication and platform capabilities.

### 7) SVG must stop having a separate file/font universe

SVG should converge on the same loader contract as images:

- file-based SVG becomes `AssetLocator::File`, not a dedicated helper type,
- bundle-based SVG becomes `AssetLocator::BundleAsset`,
- SVG bytes flow through the same revision/diagnostics story.

For text inside SVG, Fret needs an explicit policy:

- short-term truthful rule:
  - UI SVGs are expected to avoid `<text>` unless the required fonts are guaranteed and verified,
  - icon and illustration assets should prefer outlines
- long-term correct rule:
  - SVG text resolution should share the renderer text/font environment instead of loading system
    fonts independently inside `usvg`

The short-term restriction is acceptable.
The current silent divergence is not.

## Platform mapping

The contract should make platform truth obvious:

| Locator kind | Desktop | Web | Mobile | Notes |
| --- | --- | --- | --- | --- |
| `Memory` | Yes | Yes | Yes | Portable baseline. |
| `Embedded` | Yes | Yes | Yes | Deterministic and packaging-independent. |
| `BundleAsset` | Yes | Yes | Yes | Primary authoring surface. |
| `File` | Yes (capability-gated) | No | No by default | Dev/native escape hatch only. |
| `Url` | Yes (policy-gated) | Yes (policy/CORS-gated) | Yes (policy-gated) | Never assume availability or caching semantics. |

Important note:

- “supported” does not mean “identical transport”.
- It means the same authoring surface remains valid and platform differences are handled by the
  loader/resolver, not leaked into widget code.

## Packaging and build-tool implications

The correct cross-platform story is not just runtime API design. It also needs a packaging model.

Recommended direction:

- development mode
  - asset bundle roots can map to real files with watchers and hot reload
- production mode
  - the same logical keys map to packaged files, embedded blobs, or web-emitted assets
- web
  - logical keys may map to hashed output paths or URL-rewritten assets
- mobile
  - logical keys may map to APK/iOS bundle resources without exposing raw app-internal paths

This is the right place for future `fretboard` / bootstrap integration.

The wrong model is “teach users to call `from_path()` and hope packaging catches up later”.

## Ownership map after the refactor

### Core contract layer

Recommended new ownership:

- `crates/fret-assets` (new)
  - portable asset identity, capabilities, revisions, resolver traits
- `crates/fret-runtime`
  - globals/publication hooks as needed
- `crates/fret-launch`
  - platform resolver implementation, startup wiring, build/runtime integration

### Mechanism layer

- `crates/fret-ui`
  - stable image/SVG/text consumption only
- `crates/fret-render-*`
  - decode/raster/register using resolved bytes/handles only

### Ecosystem layer

- `ecosystem/fret-ui-assets`
  - authoring ergonomics for UI image/SVG loading
  - view-cache-safe state helpers
  - no longer owns the foundational asset contract
- ecosystem crates
  - reference logical bundled assets, not repository-relative files

## Immediate delete-or-demote candidates

These surfaces should not remain part of the “recommended Fret way”:

- `ImageSource::from_file_path(...)`
- `SvgFileSource::from_file_path(...)`
- cookbook examples that teach repo-relative asset paths as the normal story
- `install()` naming that implies complete installation while only configuring budgets

Acceptable transitional rule:

- keep native/dev escape hatches temporarily,
- remove them from teaching surfaces immediately,
- deprecate or rename them while the portable bundle-based path is landing,
- then delete the deprecated compatibility names once the unified asset contract, first-party
  migration, and replacement authoring/docs are complete.

## Recommended execution order

1. Fix truthfulness and closure first
   - close the wasm build break,
   - document the real platform capability matrix,
   - stop teaching path-first authoring in docs/examples.

2. Lock the new asset contract
   - add the core locator/resolver/revision design,
   - decide bundle identity and naming.

3. Unify the font baseline
   - deterministic bundled profile on every platform,
   - system font scan as optional augmentation,
   - shared text environment publication.

4. Move images and SVGs onto the same loader
   - delete dedicated file helper concepts,
   - unify invalidation/diagnostics/cache expectations.

5. Clean up public-facing ergonomics
   - rename/remove misleading install surfaces,
   - update cookbook, gallery, and bootstrap guidance,
   - leave explicit escape hatches for file/url usage.

## Definition of done

The workstream is complete when all of the following are true:

- app authors have one portable default asset story based on logical bundle keys,
- ecosystem crates can ship namespaced assets without filesystem assumptions,
- `fret-ui` remains handle-based and packaging-agnostic,
- fonts, images, and SVGs share one truthful revision/capability model,
- desktop, web, and mobile startup semantics are explicitly documented and testable,
- path and URL loading remain only as explicit capability-gated escape hatches,
- `cargo check -p fret-launch --target wasm32-unknown-unknown` is green again.
