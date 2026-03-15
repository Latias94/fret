# Resource Loading Fearless Refactor v1 — TODO Tracker

Status: Draft

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `RESLOAD-{area}-{nnn}`

When completing an item, leave 1–3 evidence anchors and prefer small executable gates over prose.

## Design lock

- [x] RESLOAD-adr-010 Add or update the hard-contract ADR(s) for the portable asset model.
  - Minimum scope:
    - locator kinds,
    - bundle/key identity,
    - capability gating for file/url,
    - revision/invalidation semantics,
    - loader diagnostics contract.
  - Current landed slice:
    - `docs/adr/0065-icon-system-and-asset-packaging.md` now explicitly locks the hybrid icon
      ownership model:
      - vendor ids are namespaced,
      - semantic `ui.*` aliases are `alias_if_missing(...)` / first-writer-wins,
      - app/bootstrap code may explicitly override afterwards,
      - reusable crates keep icon semantics on `IconId` while non-icon shipped bytes live on the
        package asset contract.
    - `docs/adr/0317-portable-asset-locator-and-resolver-contract-v1.md` now locks:
      - the portable default story as logical `bundle + key`,
      - locator-kind semantics and truthful capability gating,
      - ordered host resolver precedence,
      - revision/invalidation expectations,
      - runtime-vs-startup diagnostics boundaries,
      - builder-lane vs installer-lane startup surfaces.

- [ ] RESLOAD-audit-020 Record the current incorrect logic explicitly in the relevant audits and
      workstreams so the migration does not drift back toward path-first design.
  - Minimum evidence:
    - `ImageSource::from_file_path(...)`
    - `SvgFileSource::from_file_path(...)`
    - half-wired `install()` semantics
    - divergent font baseline behavior

## Closure and truthfulness

- [x] RESLOAD-build-030 Make `cargo check -p fret-launch --target wasm32-unknown-unknown` green.
  - This is not the full refactor, but it is the minimum portability honesty gate.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/render_plan_dump_summary.rs`
    - `cargo check -p fret-launch --target wasm32-unknown-unknown`

- [x] RESLOAD-cap-040 Publish a first-class asset capability matrix for desktop/web/mobile.
  - Minimum capability axes:
    - bundled assets
    - embedded assets
    - raw files
    - URLs
    - file watching / hot reload
    - system font scan
  - Evidence:
    - `docs/workstreams/resource-loading-fearless-refactor-v1/CAPABILITY_MATRIX.md`
    - `crates/fret-assets/src/file_manifest.rs`
    - `ecosystem/fret-ui-assets/src/image_source.rs`
    - `ecosystem/fret-ui-assets/src/svg_file.rs`
    - `crates/fret-launch/src/runner/web/gfx_init.rs`
    - `crates/fret-launch/src/runner/web/effects.rs`
    - `crates/fret-launch/src/runner/desktop/runner/effects.rs`

## Core asset contract

- [x] RESLOAD-core-100 Introduce a core asset contract crate (recommended: `crates/fret-assets`).
  - Minimum types:
    - `AssetLocator`
    - `AssetKey`
    - `AssetBundleId`
    - `AssetRevision`
    - `AssetCapabilities`
    - `AssetLoadError`
  - Evidence:
    - `crates/fret-assets/src/lib.rs`
    - `cargo test -p fret-assets`

- [~] RESLOAD-core-110 Define resolver/loader traits and result payloads.
  - The design must support:
    - bytes resolution,
    - explicit external URI/path handoff for system APIs,
    - diagnostics,
    - revision tracking.
  - Current landed slice:
    - `AssetResolver` / `ResolvedAssetBytes` in `crates/fret-assets`
    - `ResolvedAssetReference` / `AssetExternalReference` in `crates/fret-assets`
    - `StaticAssetEntry` plus bulk `InMemoryAssetResolver::insert_*_entries(...)`
    - native/package-dev file-backed manifest resolver in
      `crates/fret-assets/src/file_manifest.rs`
    - explicit bundle-locator -> native file-reference handoff in
      `crates/fret-assets/src/file_manifest.rs`
    - composable runtime host mounting via `crates/fret-runtime/src/asset_resolver.rs`
      (`register_asset_resolver`, `register_bundle_asset_entries`,
      `register_embedded_asset_entries`, `resolve_asset_reference`)
    - UI bridge helpers via `ecosystem/fret-ui-assets/src/asset_resolver.rs`, including:
      - reference-aware image resolution that prefers target-appropriate external handoff and
        falls back to bytes when the winning layer cannot provide a usable external reference,
      - native bundle-locator -> `SvgFileSource` bridging for reloadable file-backed SVGs
    - app-facing facade reference helpers via `ecosystem/fret/src/lib.rs`
      (`resolve_reference`, `resolve_locator_reference`)
  - Remaining:
    - structured diagnostics surface
    - non-UI first-party resolver implementations

- [~] RESLOAD-core-120 Decide the authoritative bundle identity model.
  - Required outcomes:
    - app bundle story,
    - ecosystem/library bundle story,
    - no collision-prone global string soup.
  - Current landed slice:
    - `AssetBundleId::app(...)`
    - `AssetBundleId::package(...)`
    - `asset_app_bundle_id!()` / `asset_package_bundle_id!()` for caller-package defaults
  - Remaining:
    - final deprecation/removal posture for opaque legacy bundle strings
    - packaged/web/mobile tooling defaults that pick app vs package ownership automatically

- [~] RESLOAD-core-125 Define ecosystem ownership rules for package resources and icon packs.
  - Required outcomes:
    - package-owned images/SVGs/fonts default to `AssetBundleId::package(...)`,
    - apps compose ecosystem installer surfaces instead of redoing internal resource mounts,
    - one documented relationship between package asset bundles and the current `IconRegistry`
      model,
    - documented conflict policy for semantic icon ids (`ui.*`) vs vendor ids (`lucide.*`,
      `radix.*`, ...).
  - Minimum evidence:
    - `ecosystem/fret-icons/src/lib.rs`
    - `ecosystem/fret-icons-lucide/src/app.rs`
    - `crates/fret-assets/src/lib.rs`
    - `crates/fret-runtime/src/asset_resolver.rs`
  - Current landed slice:
    - cookbook asset basics now models a reusable app-facing bundle that installs package-owned
      logical assets plus an icon pack behind one `.setup(...)` value
    - `fretboard assets rust write --surface fret ...` now emits a generated `Bundle` type that
      implements `fret::integration::InstallIntoApp`, so reusable crates can publish namespaced
      asset installers without hand-writing the boilerplate
    - icon-pack semantic alias conflicts now have a central tested rule:
      `IconRegistry::alias_if_missing(...)` makes `ui.*` aliases first-writer-wins, vendor ids stay
      namespaced (`lucide.*`, `radix.*`, ...), and app/bootstrap code can still intentionally
      override with an explicit follow-up alias
  - Remaining:
    - first-party docs should show when bundle ownership belongs in the generated asset module vs a
      hand-written higher-level recipe bundle that also composes icon packs
    - first-party docs still need a cleaner “when to publish `Embedded` vs when to publish
      `BundleAsset`” authoring note for reusable crates

## Packaging and startup

- [ ] RESLOAD-pack-200 Define development vs packaged asset-bundle behavior.
  - Development:
    - real file roots,
    - watchers,
    - hot reload revisions.
  - Packaged:
    - emitted/embedded/mobile-bundled asset lookup by logical key.

- [ ] RESLOAD-pack-210 Define the bootstrap/build-tool integration point.
  - Candidates:
    - `fret-launch`
    - `fret-bootstrap`
    - future `fretboard` asset manifest tooling

## Font baseline unification

- [ ] RESLOAD-font-300 Make bundled font baseline deterministic on every platform before first-frame
      text work.

- [ ] RESLOAD-font-310 Ensure desktop/web/mobile all publish the same conceptual font-environment
      snapshot shape, even when capabilities differ.

- [ ] RESLOAD-font-320 Define bundled font profiles/manifests as a real product surface.
  - Minimum guarantees:
    - UI sans/serif/monospace roles,
    - emoji fallback,
    - any promised CJK fallback coverage.

- [ ] RESLOAD-font-330 Make system-font scan an optional augmentation layer, not the baseline
      identity of the framework.

## SVG and image pipeline unification

- [~] RESLOAD-svg-400 Replace dedicated SVG file helpers with the unified asset locator story.
  - Current landed slice:
    - `resolve_svg_file_source(...)` /
      `resolve_svg_file_source_from_host(...)` in
      `ecosystem/fret-ui-assets/src/asset_resolver.rs` now bridge logical bundle locators into
      `SvgFileSource` through the shared external-reference contract instead of teaching raw
      widget-level file paths.
    - `fret-ui-assets::ui::SvgAssetElementContextExt::svg_source_state_from_asset_request(...)`
      now lets ordinary UI code stay on logical bundle locators while the native/dev file-backed
      compatibility shim remains hidden behind the UI helper.
  - Remaining:
    - `SvgFileSource` still exists as a native/dev compatibility shim because `fret_ui::SvgSource`
      is currently bytes-only.

- [ ] RESLOAD-svg-410 Decide the short-term SVG text policy and enforce it in docs/tests.
  - Preferred truthful baseline:
    - outlines for UI icons/illustrations,
    - no silent promises for arbitrary `<text>`.

- [ ] RESLOAD-svg-420 Plan the long-term shared SVG-text font environment path.
  - The SVG renderer should not permanently own an unrelated `fontdb` universe.

- [~] RESLOAD-img-430 Move image loading onto the shared locator/resolver contract while preserving
      the existing async/UI invalidation ergonomics.
  - Current landed slice:
    - `resolve_image_source(...)` / `resolve_image_source_from_host(...)` in
      `ecosystem/fret-ui-assets/src/asset_resolver.rs` now prefer target-appropriate external
      references first and fall back to bytes only when the winning layer cannot provide a usable
      external reference handoff.
    - `fret-ui-assets::ui::ImageSourceElementContextExt::use_image_source_state_from_asset_request(...)`
      now lets ordinary UI code stay on logical bundle locators while reusing the existing async
      image decode/upload state machine and ViewCache invalidation wiring.
  - Remaining:
    - broader first-party docs/examples still need to stop teaching direct file-path constructors
      as the default app story.

## Public API cleanup

- [x] RESLOAD-api-500 Rename or replace misleading install surfaces in `fret-ui-assets`.
  - `install()` must either perform complete wiring or stop being called `install()`.
  - Evidence:
    - `ecosystem/fret-ui-assets/src/app.rs`
    - `ecosystem/fret-ui-assets/src/advanced.rs`

- [~] RESLOAD-api-510 Design the golden-path authoring API for app and ecosystem authors.
  - Target qualities:
    - logical-key first,
    - no filesystem assumptions,
    - easy escape hatches for file/url when explicitly needed.
  - Current landed slice:
    - `ecosystem/fret/src/lib.rs` now exposes `fret::assets`
    - app-facing registration helpers exist on the facade
      (`register_bundle_entries`, `register_embedded_entries`, `register_resolver`)
    - `register_file_bundle_dir(...)` now exposes the first generated-manifest directory lane on
      the app-facing facade
    - `register_file_manifest(...)` now exposes the first native/package-dev manifest lane on the
      app-facing facade
    - `fretboard assets manifest write ...` now emits an explicit manifest artifact from a scanned
      bundle directory
    - `fretboard assets rust write ...` now emits a compile-time embedded Rust module for
      packaged/web/mobile-friendly bundle assets
    - the generated Rust module supports both `--surface fret` and `--surface framework`
      consumption lanes
    - the generated `--surface fret` module now exposes `mount(builder)` so compile-time embedded
      assets can stay on the `UiAppBuilder` startup path instead of falling back to ad-hoc setup
      hooks
    - the generated `--surface fret` module now exposes a named `Bundle` type that implements
      `fret::integration::InstallIntoApp`, so reusable crates can stay on the `.setup(...)`
      surface without rewriting the generated registration boilerplate
    - `FretApp::asset_dir(...)` / `UiAppBuilder::with_asset_dir(...)` keep the directory-scanning
      convenience lane on the builder/startup surface
    - `FretApp::asset_manifest(...)` / `UiAppBuilder::with_asset_manifest(...)` keep that manifest
      lane on the builder/startup surface
    - `FretApp::{asset_entries, bundle_asset_entries, embedded_asset_entries}` now keep static
      bundle/embedded registrations on the same builder/startup surface
    - `UiAppBuilder::{with_bundle_asset_entries, with_embedded_asset_entries}` now keep
      compile-time static asset registration on the same ordered startup surface as manifest/dir
      mounts
    - `FretApp` now preserves mixed `asset_dir(...)` / `asset_manifest(...)` call order so later
      builder calls override earlier ones consistently
    - host-level resolver precedence is now unified across
      `set_primary_resolver(...)`, `register_resolver(...)`,
      `register_bundle_entries(...)`, and `register_embedded_entries(...)`
    - static bundle/embedded registrations no longer bypass later resolver layers
    - replacing the primary resolver now keeps its existing stack slot, so it does not silently
      jump ahead of newer registrations
    - `fretboard` todo/simple-todo scaffolds now create `assets/` plus a checked-in
      `src/generated_assets.rs` stub and mount it through `generated_assets::mount(builder)` when
      `--ui-assets` is enabled
    - scaffold READMEs now teach the regeneration command
      `fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle ...`
      instead of teaching `FretApp::asset_dir("assets")` as the default first-contact story
    - cookbook asset basics now teaches the facade lane instead of direct
      `fret-assets` / `fret-runtime` imports
  - Remaining:
    - broader first-party packaged/web/mobile manifest tooling story
    - packaged/web/mobile manifest layering guidance beyond generated Rust embedding
      (hashed web outputs, mobile bundle/resource mapping, hybrid packaged + remote lanes)
    - final documented story for ecosystem package resources vs icon-pack installers
    - ecosystem-oriented bundle identity guidance
    - final migration of remaining first-party gallery/bootstrap surfaces

- [~] RESLOAD-api-520 Remove path-first asset loading from cookbook/gallery/bootstrap teaching
      surfaces.
  - Current landed slice:
    - `apps/fret-cookbook/examples/icons_and_assets_basics.rs` now teaches
      `bundle locator + composable host resolver + static asset entries`
    - `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs` is explicitly labeled as the
      native/dev file-path escape hatch instead of the portable default story
  - Remaining:
    - UI Gallery
    - bootstrap templates
    - any remaining dedicated dev/native escape-hatch examples should be labeled as such

- [ ] RESLOAD-api-530 Decide deprecation/removal sequencing for:
  - `ImageSource::from_file_path(...)`
  - `SvgFileSource::from_file_path(...)`

## Diagnostics and gates

- [ ] RESLOAD-diag-600 Add diagnostics for:
  - missing bundle asset,
  - unsupported file/url capability,
  - stale/missing manifest mapping,
  - font baseline source,
  - revision transitions.

- [ ] RESLOAD-test-610 Add portable contract tests for asset capability and fallback behavior.

- [ ] RESLOAD-test-620 Add startup gates for the bundled-font baseline on desktop and web.

- [ ] RESLOAD-test-630 Add regression coverage proving that hot reload / invalidation works via
      revision changes rather than widget re-execution accidents.

## Migration and cleanup

- [ ] RESLOAD-mig-700 Migrate first-party users onto the new bundle-based asset story.
  - Minimum surfaces:
    - cookbook examples,
    - UI Gallery,
    - bootstrap templates,
    - shadcn ecosystem recipes that ship icons/images,
    - ecosystem icon-pack install stories that currently rely on implicit global registry behavior.

- [ ] RESLOAD-mig-710 Remove or archive superseded one-off resource helpers once the unified path is
      verified.
  - Deprecated compatibility names should remain only during migration and be deleted after:
    - the unified asset contract lands,
    - first-party callers move over,
    - docs/examples stop teaching the old names.

- [ ] RESLOAD-mig-720 Update workstream/docs alignment after the contract lands.
  - At minimum:
    - `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
    - `docs/shadcn-declarative-progress.md` if authoring guidance changes
