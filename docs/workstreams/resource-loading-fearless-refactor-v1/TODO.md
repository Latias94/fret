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

- [x] RESLOAD-audit-020 Record the current incorrect logic explicitly in the relevant audits and
      workstreams so the migration does not drift back toward path-first design.
  - Evidence:
    - `docs/workstreams/resource-loading-fearless-refactor-v1/AUDIT.md`
    - `ecosystem/fret-ui-assets/src/image_source.rs`
    - `ecosystem/fret-ui-assets/src/ui.rs`
    - `ecosystem/fret-ui-assets/src/app.rs`
    - `ecosystem/fret-ui-assets/src/advanced.rs`
    - `crates/fret-launch/src/runner/font_catalog.rs`

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
    - `ecosystem/fret-ui-assets/src/{asset_resolver.rs,ui.rs}`
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
      - shared URL image bridging on every platform when the winning resolver returns
        `AssetExternalReference::Url`,
      - native SVG request helpers keeping reloadable file-reference caching internal while
        ordinary UI code stays on logical bundle locators
    - runtime file-read failures now stay typed as `AssetLoadError::Io { operation, path, message }`
      instead of collapsing into free-form strings, and `fret-diag` now exposes the corresponding
      `debug.resource_loading.asset_load.io_requests` counter through both stats and post-run gates
      (`--check-asset-load-io-max`, `asset_load_io_max`)
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

- [x] RESLOAD-core-125 Define ecosystem ownership rules for package resources and icon packs.
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
    - `docs/workstreams/resource-loading-fearless-refactor-v1/ECOSYSTEM_INSTALLER_COMPOSITION.md`
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
    - `ECOSYSTEM_INSTALLER_COMPOSITION.md` now documents the expected publication/composition model:
      ecosystem crates own internal icon/bundle registration, apps compose one installer/bundle
      surface, logical package assets stay on `AssetBundleId::package(...)`, and app follow-up
      installers can override semantic icon aliases without replaying dependency asset mounts
    - `fret::integration` now has an integration test proving that one ecosystem installer can
      publish package-owned assets plus semantic/vendor icons together, and that a later app
      installer can override `ui.*` aliases while leaving dependency bundle assets untouched
    - first-party public docs now keep icon-pack ownership explicit across the main component and
      pack surfaces:
      `ecosystem/fret-ui-kit/README.md`, `ecosystem/fret-ui-shadcn/README.md`,
      `ecosystem/fret-ui-material3/README.md`, `ecosystem/fret-icons-lucide/README.md`, and
      `ecosystem/fret-icons-radix/README.md`
    - first-party docs now distinguish:
      - when the generated `--surface fret` asset module is sufficient,
      - when a hand-written higher-level installer/bundle surface should wrap it,
      - and when reusable crates should publish `BundleAsset` vs keep bytes on the lower-level
        `Embedded` lane
    - cookbook teaching surfaces now model both sides:
      - generated-module/public-bundle lane in
        `apps/fret-cookbook/examples/app_owned_bundle_assets_basics.rs`
      - hand-written higher-level installer composition lane in
        `apps/fret-cookbook/examples/icons_and_assets_basics.rs`

## Packaging and startup

- [~] RESLOAD-pack-200 Define development vs packaged asset-bundle behavior.
  - Development:
    - real file roots,
    - watchers,
    - hot reload revisions.
  - Packaged:
    - emitted/embedded/mobile-bundled asset lookup by logical key.
  - Current landed slice:
    - `fret-launch::assets::{AssetStartupPlan, AssetStartupMode}` plus
      `WinitAppBuilder::{with_bundle_asset_entries, with_embedded_asset_entries,
      with_asset_startup}` now define the lowest-level native startup contract directly on the
      runner-facing builder surface,
    - `fret::assets::{AssetStartupPlan, AssetStartupMode}` now gives the `fret` facade one named
      startup policy object for selecting development vs packaged publication on the builder path,
    - `fret-bootstrap::assets::{AssetStartupPlan, AssetStartupMode}` now re-exports the same
      launch-owned contract and keeps bootstrap startup on `BootstrapBuilder::with_asset_startup(...)`
      plus `AssetStartupPlan::{development_dir,development_manifest,packaged_entries,
      packaged_bundle_entries,packaged_embedded_entries}(...)`,
    - `FretApp::asset_startup(...)` lowers that policy onto the default app-facing startup
      surface while preserving fail-early builder semantics,
    - `UiAppBuilder::with_asset_startup(...)` keeps the same policy available on the explicit
      advanced builder lane,
    - the plan intentionally lowers to the existing ordered startup registrations for
      file-backed development inputs plus `with_bundle_asset_entries(...)` /
      `with_embedded_asset_entries(...)`,
    - `AssetStartupMode::preferred()` now provides one shared app-facing heuristic for
      `native+debug => Development` vs packaged/web/mobile/release => `Packaged`,
    - `AssetStartupPlan::development_bundle_dir_if_native(...)` now lets app/tooling code keep one
      portable startup-plan expression instead of repeating per-call-site `cfg` branches,
    - generated `--surface fret` modules now publish `preferred_startup_plan()` /
      `preferred_startup_mode()` and route `mount(builder)?` through `with_asset_startup(...)`,
      so native debug startup automatically uses the file-backed development lane while
      packaged/web/mobile keeps the compiled bundle lane,
    - `fret_runtime::{AssetReloadEpoch, AssetReloadSupport}` now define one runtime-global asset
      invalidation contract shared by UI and non-UI consumers,
    - `fret_runtime::{AssetReloadStatus, AssetReloadBackendKind, AssetReloadFallbackReason}` now
      also publish the currently active automatic reload backend plus watcher fallback details for
      diagnostics/runtime consumers, and
      `crates/fret-runtime/src/asset_resolver.rs` now merges that support into aggregated
      `AssetCapabilities.file_watch`,
    - `fret-launch::assets::AssetReloadPolicy` plus
      `WinitAppBuilder::with_asset_reload_policy(...)` now define an explicit development reload
      automation surface for file-backed startup mounts,
    - the current first-party desktop implementation now prefers `AssetReloadPolicy::NativeWatcher`
      on supported native hosts, falls back to metadata polling when watcher installation fails,
      watches builder-mounted manifests/directories, bumps the shared `AssetReloadEpoch`, publishes
      `AssetReloadSupport { file_watch: true }`, publishes the effective reload backend/fallback
      through `AssetReloadStatus`, and requests redraws for each tracked native window,
    - `BootstrapBuilder::with_asset_reload_policy(...)`,
      `UiAppBuilder::with_asset_reload_policy(...)`, and `FretApp::asset_reload_policy(...)` now
      reuse that same policy on higher authoring surfaces,
    - missing selected lanes now fail as startup configuration errors instead of silently falling
      through to ad-hoc runtime glue.
  - Remaining:
    - wasm/mobile still have no first-party automatic reload lane,
    - desktop watcher-backed reload still only exists on the native winit runner path today,
    - broader first-party docs/examples still need to migrate from UI-specific reload nouns.

- [~] RESLOAD-pack-210 Define the bootstrap/build-tool integration point.
  - Candidates:
    - `fret-launch`
    - `fret-bootstrap`
    - future `fretboard` asset manifest tooling
  - Current landed slice:
    - `fret-launch::assets` is now the canonical owner for the host/startup asset contract, so
      advanced integrations no longer need `bootstrap` to access the shared development-vs-packaged
      builder surface,
    - the first app-facing integration point is now explicit in `ecosystem/fret`:
      `AssetStartupPlan` + `AssetStartupMode` on the builder surface,
    - `ecosystem/fret` now also publishes shared preferred-mode helpers
      (`AssetStartupMode::preferred()` and
      `AssetStartupPlan::development_bundle_dir_if_native(...)`) so non-scaffolded `fret` apps
      can reuse the same native-debug vs packaged defaults as generated modules,
    - `ecosystem/fret-bootstrap` now exposes the same startup contract for direct bootstrap users
      through `fret_bootstrap::assets::{AssetStartupPlan, AssetStartupMode}` and
      `BootstrapBuilder::with_asset_startup(...)`,
    - generated `--surface fret` modules now expose startup-policy helpers
      (`preferred_startup_plan()`, `preferred_startup_mode()`, `mount(builder)?`) in addition to
      the packaged-lane ingredients (`ENTRIES`, `bundle_id()`, `Bundle`, `install(app)`),
    - `fretboard-dev new {simple-todo,todo} --ui-assets` now consumes that generated startup surface
      directly, so the first-party scaffold owns the preferred mode choice without bespoke app
      code.
  - Remaining:
    - wire the same story through any future `fret-bootstrap` / `fret-launch` packaging helpers.

## Font baseline unification

- [x] RESLOAD-font-300 Make bundled font baseline deterministic on every platform before first-frame
      text work.
  - Current landed slice:
    - web startup installs the framework-owned bundled baseline immediately when the renderer
      becomes available, before startup font-environment publication.
    - the shared native winit startup path does the same for desktop and current mobile targets
      before first-frame text work.
    - runner-level tests now lock that web and desktop startup helpers both inject the same
      bundled baseline before publishing renderer font-environment state, and local iOS target
      compile evidence proves the shared native path still builds for mobile.
  - Evidence:
    - `crates/fret-launch/src/runner/web/gfx_init.rs`
    - `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
    - `crates/fret-launch/src/runner/font_catalog.rs`
      (`initialize_web_startup_font_environment_installs_baseline_and_seeds_missing_families`,
      `initialize_desktop_startup_font_environment_installs_baseline_for_sync_and_async_modes`,
      `platform_startup_helpers_share_bundled_baseline_but_keep_distinct_defaults_policy`)
    - `cargo nextest run -p fret-launch runner::font_catalog`
    - `cargo check -p fret-launch --target aarch64-apple-ios`

- [~] RESLOAD-font-310 Ensure desktop/web/mobile all publish the same conceptual font-environment
      snapshot shape, even when capabilities differ.
  - Current landed slice:
    - `fret_runtime::BundledFontBaselineSnapshot` now gives runners one explicit runtime global for
      the framework-owned bundled baseline contract.
    - `fret_runtime::RendererFontEnvironmentSnapshot` now gives runners one shared runtime-visible
      renderer font inventory:
      - monotonic `revision`,
      - `text_font_stack_key`,
      - accepted renderer source records for `bundled_startup` and `asset_request`,
      - stable byte fingerprints and logical `AssetRequest` identity when available.
    - web and the current native winit startup path now both install bundled default fonts and
      publish the current `fret-fonts::default_profile()` identity (profile name, bundle id,
      logical asset keys, declared roles, guaranteed generics) before startup font-environment
      initialization.
    - startup bundled baseline injection plus runtime `TextAddFontAssets` now all feed that same
      shared source inventory instead of keeping startup-vs-runtime font provenance on separate
      publication paths.
    - the UI gallery's deterministic bundled-font diagnostics switch
      (`FRET_UI_GALLERY_BOOTSTRAP_FONTS=1`) now republishes the already-installed startup baseline
      through live renderer catalog metadata instead of re-injecting duplicate bundled font bytes
      through an app-local side lane.
    - first-party manifest-driven development font reload now also stays on the asset-identity
      lane:
      - `fret-bootstrap::dev_reload` publishes stable bundle asset locators for manifest entries,
      - repeated reloads reuse one mutable resolver layer instead of stacking duplicate
        registrations,
      - and the watcher now emits `TextAddFontAssets` instead of bypassing identity with a
        separate runtime raw-byte path.
    - first-party local font import flows now also stay on the asset-identity lane:
      - `fret_fonts::build_imported_font_asset_batch(...)` prepares stable memory locators plus
        `Font`-hinted requests for user-selected files,
      - `fret_fonts::ImportedFontAssetResolver` provides one mutable memory resolver layer for
        session-local imports, and
      - `apps/fret-ui-gallery` plus `apps/fret-examples` now stage local file-dialog imports into
        that resolver before emitting `TextAddFontAssets`.
    - `fret-bootstrap` diagnostics now export that renderer inventory through
      `debug.resource_loading.font_environment.renderer_font_*`, so diagnostics bundles can see
      the same revision/source records the future SVG-text bridge will depend on.
    - `UiPredicateV1` resource-loading gates can now also assert renderer-font inventory revision,
      source lane, and asset key, so scripted diagnostics can fail on provenance regressions
      without hand-reading debug snapshots.
    - the renderer now keeps the actual SVG bridge rehydration local:
      `fret-render-text` can enumerate deduped blobs from the current approved text collection,
      and `fret-render-wgpu` can rebuild a `usvg fontdb` from that live collection plus the
      current generic-family mapping without pushing more raw font bytes into runtime globals.
    - native intentionally keeps `FontFamilyDefaultsPolicy::None`, so system-font augmentation
      remains an additive capability instead of redefining the baseline identity.
    - local iOS target evidence now exists:
      - `cargo check -p fret-launch --target aarch64-apple-ios`
  - Remaining:
    - add stable Android target evidence once the local/CI environment provides NDK clang
      toolchains
    - add mobile-specific diagnostics or startup gates beyond shared native runner wiring
    - audit whether any remaining internal APIs still talk about a runtime raw-byte lane even
      though the shipped runtime surface now loads fonts only through asset requests
  - Evidence:
    - `crates/fret-fonts/src/{lib.rs,tests.rs}`
    - `crates/fret-runtime/src/font_catalog.rs`
    - `crates/fret-diag-protocol/src/lib.rs`
    - `crates/fret-launch/src/runner/font_catalog.rs`
      (`install_default_bundled_font_baseline_adds_fonts_and_publishes_snapshot`,
      `inject_font_asset_batch_and_refresh_catalog_records_asset_sources`,
      `publish_renderer_font_environment_sets_key_after_locale_application`)
    - `apps/fret-ui-gallery/src/driver/runtime_driver.rs`
      (`ui_gallery_default_profile_font_requests_stay_on_asset_request_lane`,
      `ui_gallery_local_font_import_installs_memory_asset_requests`)
    - `apps/fret-examples/src/components_gallery.rs`
      (`components_gallery_local_font_import_installs_memory_asset_requests`)
    - `crates/fret-render-text/src/{parley_font_db.rs,parley_shaper.rs}`
    - `crates/fret-render-wgpu/src/{renderer/config.rs,renderer/svg/{mod.rs,raster.rs},text/fonts.rs,text/tests.rs}`
    - `ecosystem/fret-bootstrap/src/dev_reload.rs`
    - `ecosystem/fret-bootstrap/src/ui_diagnostics/{debug_snapshot_impl.rs,debug_snapshot_predicates.rs,debug_snapshot_types.rs}`
    - `cargo nextest run -p fret-launch`
    - `cargo nextest run -p fret-bootstrap --features ui-app-driver --lib`
    - `cargo nextest run -p fret-bootstrap --features 'ui-app-driver diagnostics' debug_snapshot_types_tests::font_environment_snapshot_from_runtime_keeps_renderer_font_sources`
    - `cargo check -p fret-bootstrap --features 'ui-app-driver diagnostics'`
    - `cargo check -p fret-ui-gallery --target wasm32-unknown-unknown --features gallery-dev`

- [~] RESLOAD-font-320 Define bundled font profiles/manifests as a real product surface.
  - Minimum guarantees:
    - UI sans/serif/monospace roles,
    - emoji fallback,
    - any promised CJK fallback coverage.
  - Current landed slice:
    - `crates/fret-fonts` now publishes package-scoped logical asset identity alongside the
      existing profile manifest:
      - `bundled_asset_bundle()`,
      - `BundledFontFaceSpec::{asset_key,asset_locator(),asset_request(),asset_entry()}`,
      - `BundledFontProfile::asset_entries()`.
    - bundled font profiles can now be mounted into the shared asset contract through
      `StaticAssetEntry` instead of existing only as ad-hoc byte bags for post-startup font
      injection.
    - runner startup now consumes the same default profile manifest for both:
      - mounting package-owned bundled font assets into the shared runtime resolver,
      - injecting renderer font bytes from bundled face byte payloads.
    - regression coverage now proves the startup helper resolves bundled font faces through the
      runtime asset resolver and does not register duplicate font-asset resolver layers on
      repeated baseline installation.
  - Remaining:
    - define how mobile/package builds surface framework-owned bundled fonts on the builder/startup
      lane

- [x] RESLOAD-font-330 Make system-font scan an optional augmentation layer, not the baseline
      identity of the framework.
  - Current landed slice:
    - desktop startup still publishes the framework-owned bundled baseline snapshot first and keeps
      `FontFamilyDefaultsPolicy::None`, so empty family config remains empty instead of being
      synthesized from the current system catalog.
    - explicit system-font refresh paths on desktop continue to refresh the live renderer/runtime
      catalog with `FontFamilyDefaultsPolicy::None`, so system enumeration augments available
      families without redefining the startup baseline contract.
    - regression coverage now proves a desktop system-font refresh can replace the live catalog
      entries while leaving both the startup bundled-baseline snapshot and the empty desktop family
      config unchanged.
  - Evidence:
    - `crates/fret-launch/src/runner/desktop/runner/effects.rs`
    - `crates/fret-launch/src/runner/font_catalog.rs`
      (`initialize_desktop_startup_font_environment_installs_baseline_for_sync_and_async_modes`,
      `desktop_system_font_refresh_augments_catalog_without_redefining_baseline`)
    - `crates/fret-runtime/src/font_bootstrap.rs`
    - `docs/adr/0258-font-catalog-refresh-and-revisioning-v1.md`
    - `docs/adr/0259-system-font-rescan-and-injected-font-retention-v1.md`
    - `cargo nextest run -p fret-launch runner::font_catalog`

## SVG and image pipeline unification

- [x] RESLOAD-svg-400 Replace dedicated SVG file helpers with the unified asset locator story.
  - Current landed slice:
    - the public `resolve_svg_file_source(...)` /
      `resolve_svg_file_source_from_host(...)` compatibility seams are deleted, so the SVG app/UI
      surface no longer exposes a separate native file handoff object.
    - `fret-ui-assets::ui::SvgAssetElementContextExt::svg_source_state_from_asset_request(...)`
      now lets ordinary UI code stay on logical bundle locators while the native/dev file-backed
      reload cache lives only as an internal implementation detail in `ecosystem/fret-ui-assets/src/ui.rs`.

- [x] RESLOAD-svg-410 Decide the short-term SVG text policy and enforce it in docs/tests.
  - Current landed slice:
    - `crates/fret-render-wgpu/src/svg.rs` no longer builds a private `usvg fontdb` or loads
      system fonts inside the first-party SVG raster path.
    - the low-level direct SVG helpers still reject text-bearing assets via
      `SvgRenderError::TextNodesUnsupported` instead of silently diverging from the framework text
      baseline.
    - the renderer-owned shipped raster path now only admits text-bearing SVGs when the
      bridge-backed diagnostics are clean, and otherwise still fails closed.
    - focused renderer tests now lock both the reject-by-default low-level path and the admitted
      vs rejected renderer-owned raster path.
  - Evidence:
    - `crates/fret-render-wgpu/src/svg.rs`
      (`svg_text_nodes_are_rejected_for_alpha_and_rgba_rasterization`)
    - `crates/fret-render-wgpu/src/renderer/svg/{mod.rs,raster.rs}`
    - `docs/workstreams/resource-loading-fearless-refactor-v1/README.md`
    - `cargo nextest run -p fret-render-wgpu svg::tests`

- [x] RESLOAD-svg-420 Plan the long-term shared SVG-text font environment path.
  - Current landed slice:
    - `docs/workstreams/resource-loading-fearless-refactor-v1/SVG_TEXT_FONT_ENVIRONMENT_PLAN.md`
      now documents the long-term contract in staged form:
      - publish a real renderer font inventory,
      - build any future `usvg` bridge only from that shared inventory,
      - keep SVG `<text>` rejected until deterministic tests exist,
      - and treat any eventual `usvg fontdb` bridge as an implementation detail instead of a
        second host font universe.
  - Evidence:
    - `docs/workstreams/resource-loading-fearless-refactor-v1/SVG_TEXT_FONT_ENVIRONMENT_PLAN.md`
    - `crates/fret-runtime/src/effect.rs`
    - `crates/fret-launch/src/runner/font_catalog.rs`

- [~] RESLOAD-svg-425 Seed a renderer-owned SVG-text font bridge from the live text environment.
  - Current landed slice:
    - `fret-render-text::ParleyShaper::{family_name_for_id,for_each_font_environment_blob}` now
      expose the current approved text collection as:
      - deduped live font blobs,
      - current family-name lookup for injected generic ids.
    - `fret-render-wgpu::TextSystem::build_svg_text_font_db()` and
      `Renderer::build_svg_text_font_db_for_bridge()` now rebuild a `usvg::fontdb::Database`
      only from that live renderer text collection.
    - `crates/fret-render-wgpu/src/svg.rs` now has an internal bridge-backed render helper and a
      focused end-to-end test proving text-bearing SVG can render when fed from that renderer-built
      `fontdb`, while the shipped path still rejects `<text>`.
    - registered text-bearing SVGs now also carry the current `text_font_stack_key` in
      `SvgRasterKey`, while outline-only SVGs keep a zero key, so future SVG-text cache
      invalidation follows the renderer text environment instead of introducing a separate cache
      epoch.
    - sans/serif/monospace generic-family mapping inside the bridge now follows the renderer's
      current text policy instead of host/system discovery.
    - focused coverage now locks the bundled-only bridge seed to export `Inter`,
      `JetBrains Mono`, and matching generic mappings.
    - `crates/fret-render-wgpu/src/svg.rs` now also records structured bridge diagnostics for
      explicit font-family selection misses, fallback hops, and post-layout missing glyphs.
    - focused SVG bridge diagnostics tests now run on the bundled-only lane by forcing
      `FRET_TEXT_SYSTEM_FONTS=0`, so the expected fallback/missing-glyph outcomes are no longer
      host-system-font-dependent.
    - the renderer-owned shipped SVG raster path now consumes that bridge for text-bearing SVGs,
      but only allows parses whose bridge diagnostics are clean; unresolved SVG text still fails
      closed.
    - `fret-launch` now publishes the renderer-owned bridge snapshot into
      `fret_runtime::RendererSvgTextBridgeDiagnosticsSnapshot`.
    - `fret-bootstrap` now exports that state under `debug.resource_loading.svg_text_bridge`, and
      resource-loading predicates can now assert:
      - selection misses,
      - missing glyphs,
      - clean-vs-dirty bridge results,
      - fallback hops.
  - Remaining:
    - decide whether the low-level `SvgRenderer::render_*_fit_mode(...)` surface should stay
      text-free permanently or grow an explicit bridge-backed sibling API
    - broaden deterministic end-to-end SVG-text gates beyond the current bundled-only subset
  - Evidence:
    - `crates/fret-runtime/src/font_catalog.rs`
    - `crates/fret-launch/src/runner/{font_catalog.rs,desktop/runner/app_handler.rs,web/render_loop.rs}`
    - `crates/fret-diag-protocol/src/lib.rs`
    - `crates/fret-render-text/src/{parley_font_db.rs,parley_shaper.rs,lib.rs}`
    - `crates/fret-render-wgpu/src/{renderer/config.rs,renderer/svg/{mod.rs,prepare.rs,raster.rs},svg.rs,text/fonts.rs,text/tests.rs}`
    - `ecosystem/fret-bootstrap/src/ui_diagnostics/{debug_snapshot_impl.rs,debug_snapshot_predicates.rs,debug_snapshot_types.rs}`
    - `cargo nextest run -p fret-render-wgpu text::tests::svg_text_font_db_uses_current_collection_fonts_and_generic_mappings`
    - `cargo nextest run -p fret-render-wgpu -p fret-launch -p fret-bootstrap -p fret-diag-protocol`

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
  - Current landed slice:
    - `ecosystem/fret-ui-assets::app` now exposes only `configure_caches(...)` and
      `configure_caches_with_budgets(...)`
    - `ecosystem/fret-ui-assets::advanced` now exposes only
      `configure_caches_with_ui_services(...)` and
      `configure_caches_with_ui_services_and_budgets(...)`
    - the deprecated `install*` compatibility aliases are deleted instead of being kept as
      pre-release baggage
  - Evidence:
    - `ecosystem/fret-ui-assets/src/app.rs`
    - `ecosystem/fret-ui-assets/src/advanced.rs`
    - `ecosystem/fret-ui-assets/src/lib.rs`

- [~] RESLOAD-api-510 Design the golden-path authoring API for app and ecosystem authors.
  - Target qualities:
    - logical-key first,
    - no filesystem assumptions,
    - easy escape hatches for file/url when explicitly needed.
  - Current landed slice:
    - `ecosystem/fret/src/lib.rs` now exposes `fret::assets`
    - app-facing registration helpers exist on the facade
      (`register_bundle_entries`, `register_embedded_entries`, `register_resolver`)
    - native/package-dev host code now uses
      `FileAssetManifestResolver::{from_bundle_dir(...), from_manifest_path(...)}` plus
      `register_resolver(...)` instead of duplicate facade wrappers
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
    - `FretApp::asset_startup(...)` / `UiAppBuilder::with_asset_startup(...)` now keep
      file-backed `development_dir(...)` / `development_manifest(...)` selection on one named
      builder/startup contract instead of duplicating facade-specific dir/manifest helpers
    - `FretApp::{asset_entries, bundle_asset_entries, embedded_asset_entries}` now keep static
      bundle/embedded registrations on the same builder/startup surface
    - `UiAppBuilder::{with_bundle_asset_entries, with_embedded_asset_entries}` now keep
      compile-time static asset registration on the same ordered startup surface as
      `asset_startup(...)`
    - `FretApp` now preserves mixed `asset_startup(...)` / static-entry call order so later
      builder calls override earlier ones consistently
    - host-level resolver precedence is now unified across
      `set_primary_resolver(...)`, `register_resolver(...)`,
      `register_bundle_entries(...)`, and `register_embedded_entries(...)`
    - static bundle/embedded registrations no longer bypass later resolver layers
    - replacing the primary resolver now keeps its existing stack slot, so it does not silently
      jump ahead of newer registrations
    - `fretboard` todo/simple-todo scaffolds now create `assets/` plus a checked-in
      `src/generated_assets.rs` stub and mount it through `generated_assets::mount(builder)?` when
      `--ui-assets` is enabled
    - those generated modules now also publish `preferred_startup_plan()` /
      `preferred_startup_mode()` so the scaffold owns native-debug vs packaged startup behavior
    - scaffold READMEs now teach the regeneration command
      `fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle ...`
      instead of teaching dedicated file-backed builder helpers as the default first-contact story
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
      `bundle locator + UI helper + static asset entries` for package-owned ecosystem resources
    - `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs` is explicitly labeled as the
      native/dev file-path escape hatch instead of the portable default story
  - Remaining:
    - UI Gallery
    - bootstrap templates
    - any remaining dedicated dev/native escape-hatch examples should be labeled as such

- [x] RESLOAD-api-530 Remove deprecated direct file-path constructors after first-party migration.
  - `ImageSource::from_file_path(...)`
  - historical `SvgFileSource::from_file_path(...)`
  - Current landed slice:
    - internal bridge code stays on crate-private native constructors in
      `ecosystem/fret-ui-assets/src/image_source.rs` plus the internal native SVG request cache in
      `ecosystem/fret-ui-assets/src/ui.rs`.
    - the public deprecated direct file-path constructors are removed from the image/SVG authoring
      surface.
    - the lower-level UI-ready locator bridge helpers remain explicit:
      - `resolve_image_source_from_host_locator(...)`
      - `resolve_svg_source_from_host_locator(...)`
    - non-UI integrations that truly need raw external references now use
      `fret::assets::resolve_reference(...)` / `resolve_locator_reference(...)` directly.

- [x] RESLOAD-api-540 Remove UI-specific reload naming after migration.
  - Current landed slice:
    - `fret_runtime::{AssetReloadEpoch, AssetReloadSupport}` are now the canonical runtime-global
      asset reload/invalidation nouns.
    - first-party callers now use the generic names instead of the deprecated UI-specific ones:
      - `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`
      - `ecosystem/fret-bootstrap/src/dev_reload.rs`
    - the dedicated `fret-ui-assets::reload` shim is now deleted, so first-party code only uses
      the shared runtime/app-facing reload surfaces:
      - `crates/fret-runtime/src/asset_reload.rs`
      - `ecosystem/fret/src/lib.rs`
      - `ecosystem/fret-bootstrap/src/dev_reload.rs`
  - Follow-up record:
    - `docs/workstreams/resource-loading-fearless-refactor-v1/M5_DEPRECATION_CLEANUP.md`

## Diagnostics and gates

- [x] RESLOAD-diag-600 Add diagnostics for:
  - missing bundle asset,
  - unsupported file/url capability,
  - stale/missing manifest mapping,
  - font baseline source,
  - revision transitions.
  - Current landed slice:
    - `crates/fret-runtime/src/asset_resolver.rs` now keeps a bounded diagnostics snapshot on the
      shared resolver service, including:
      - recent bytes/reference resolution outcomes,
      - missing bundle asset counts,
      - unsupported file/url capability counts,
      - external-reference-unavailable counts,
      - per-locator revision transition classification (`initial` / `stable` / `changed`).
    - `ecosystem/fret-bootstrap/src/ui_diagnostics/debug_snapshot_{types,impl}.rs` now exposes
      `debug.resource_loading` in bundle snapshots, including:
      - recent asset-load diagnostic events,
      - current bundled font baseline source/profile/bundle/key contract,
      - current font catalog revision/family count,
      - current `TextFontStackKey`,
      - current native system-font rescan state.
    - `crates/fret-diag-protocol/src/lib.rs` now exposes typed `UiPredicateV1` variants over
      `debug.resource_loading` instead of requiring stringly JSON-pointer predicates, including
      the current asset-reload epoch/configured-backend/active-backend/fallback-reason surface.
    - `ecosystem/fret-bootstrap/src/ui_diagnostics/debug_snapshot_predicates.rs` now centralizes
      recent debug-snapshot predicate evaluation for both docking and resource-loading surfaces, and
      `assert` / `wait` / `drag` script steps can consume the new resource-loading predicates from
      recent snapshots.
    - `crates/fret-assets/src/file_manifest.rs` now reports a typed runtime
      `AssetLoadError::StaleManifestMapping` when a file-backed manifest entry still resolves the
      logical bundle/key mapping but its mapped host file path is gone.
    - the shared asset contract no longer uses a generic `AssetLoadError::Message` bucket; runtime
      manifest file-read failures now surface as typed `AssetLoadError::Io { operation, path,
      message }`.
    - `crates/fret-runtime/src/asset_resolver.rs` now keeps `stale_manifest_requests` and
      `stale_manifest` recent outcomes distinct from true missing bundle assets, and it now also
      publishes typed `io_requests` / `io` outcomes instead of routing those failures through a
      string-only bucket.
    - `crates/fret-diag/src/stats/resource_loading.rs` plus
      `crates/fret-diag/src/registry/checks/builtin_post_run/resource_loading.rs` now add
      post-run `fretboard-dev diag` gates for:
      - asset-load missing-bundle counter max,
      - asset-load stale-manifest counter max,
      - unsupported file/url counter max,
      - external-reference-unavailable counter max,
      - revision-change counter max,
      - bundled font baseline source equality,
      - asset-reload epoch minimum,
      - asset-reload configured/active backend equality,
      - asset-reload fallback-reason equality (with `none`/absent as a first-class expectation).

- [~] RESLOAD-test-610 Add portable contract tests for asset capability and fallback behavior.
  - Current landed slice:
    - `crates/fret-runtime/src/asset_resolver.rs` now locks the resolver-service contract for:
      - capability union across layered resolvers,
      - `UnsupportedLocatorKind` vs `NotFound` truthfulness for file-capability lanes,
      - bytes/reference fallback to earlier layers when a later supporting layer is simply missing
        the requested locator.
    - `ecosystem/fret-ui-assets/src/asset_resolver.rs` now locks the UI bridge contract for:
      - image helpers propagating unsupported file-locator capability errors instead of swallowing
        them,
      - byte-backed URL resolvers remaining consumable through the shared image bridge,
      - reference-only URL resolvers remaining consumable for images while SVG URL requests still
        fail truthfully as reference-only locators,
      - SVG UI helpers keeping native file-reference reload caching internal while byte-backed
        bundle assets and URL locators remain truthful about whether they are immediately SVG-ready.
    - `ecosystem/fret-ui-ai/src/elements/attachments.rs` now locks the first-party attachment
      preview call site for:
      - generating image-only URL asset requests,
      - accepting the shipped `UrlPassthroughAssetResolver` lane used by the default launch hosts.
  - Remaining:
    - add broader packaged/mobile-facing capability tests once builder/package startup lanes are
      defined
    - add revision-transition / hot-reload contract coverage via unified locator keys

- [x] RESLOAD-test-620 Add startup gates for the bundled-font baseline on desktop and web.
  - Evidence:
    - `crates/fret-launch/src/runner/font_catalog.rs`
      (`initialize_web_startup_font_environment_installs_baseline_and_seeds_missing_families`,
      `initialize_desktop_startup_font_environment_installs_baseline_for_sync_and_async_modes`,
      `platform_startup_helpers_share_bundled_baseline_but_keep_distinct_defaults_policy`)
    - `crates/fret-launch/src/runner/web/gfx_init.rs`
    - `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`

- [x] RESLOAD-test-630 Add regression coverage proving that hot reload / invalidation works via
      revision changes rather than widget re-execution accidents.
  - Evidence:
    - `ecosystem/fret-ui-assets/src/image_source.rs`
      (`bundle_asset_request_same_revision_reuses_signal_model_and_request_key`,
      `bundle_asset_request_revision_change_creates_new_signal_model_and_request_key`)
  - Locked invariants:
    - same logical locator + same revision must reuse the same signal model/request key even if the
      resolver is reinstalled or the widget helper re-executes,
    - same logical locator + changed revision must create a new signal model/request key so the UI
      asset helper path observes the change through revision semantics instead of accidental
      subtree rebuilds.

## Migration and cleanup

- [~] RESLOAD-mig-700 Migrate first-party users onto the new bundle-based asset story.
  - Current landed slice:
    - `fretboard -- new {simple-todo,todo} --ui-assets` already starts on generated
      `src/generated_assets.rs` modules and `generated_assets::mount(builder)?` instead of teaching
      dedicated file-backed builder helpers as the default first-contact story.
    - cookbook examples now cover all three first-party lanes explicitly:
      - app-owned compile-time bundle assets,
      - package-owned reusable bundle assets,
      - native/package-dev reload via bundle-dir mounting.
    - UI Gallery `Card / Image` now resolves its cover through a gallery-owned logical package
      bundle request installed during app startup, so at least one first-party docs page teaches
      shipped asset ownership via `AssetRequest` instead of only inline RGBA demo buffers.
    - UI Gallery `AI Attachments` preview-bearing variants (`Usage`, `Grid`, `Inline`, `List`) now
      resolve thumbnails from gallery-owned logical package bundle requests rather than synthesizing
      inline RGBA previews inside each snippet.
    - UI Gallery `AI Image`, `AI Queue`, and `AI Chain of Thought` now resolve their preview media
      from the shared gallery demo asset bundle instead of synthesizing inline RGBA sources inside
      each snippet.
    - UI Gallery `Aspect Ratio` now resolves landscape/portrait/square teaching media through
      gallery-owned logical bundle requests, so the snippet family no longer synthesizes per-file
      RGBA previews while still avoiding the old per-window
      `Model<Option<ImageId>>` bootstrap lane.
    - UI Gallery `Avatar`, `Card`, and `Hover Card` now reuse gallery-owned logical bundle
      requests for their shipped demo images instead of synthesizing local RGBA buffers inside each
      helper module.
    - UI Gallery `Image Object Fit` now resolves its square/wide/tall/sampling teaching media from
      the gallery demo asset bundle, and the sampling demo ships an explicit tiny PNG asset rather
      than constructing pixels inline.
    - Public guidance now links the transitive icon-pack + package-bundle composition story back to
      one ecosystem-owned installer surface instead of teaching apps to replay low-level icon or
      asset registration manually.
  - Evidence:
    - `apps/fret-cookbook/examples/app_owned_bundle_assets_basics.rs`
    - `apps/fret-cookbook/examples/icons_and_assets_basics.rs`
    - `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`
    - `apps/fret-ui-gallery/src/driver/demo_assets.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/card/image.rs`
    - `apps/fret-ui-gallery/src/ui/pages/card.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/ai/attachments_usage.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/ai/attachments_grid.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/ai/attachments_inline.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/ai/attachments_list.rs`
    - `apps/fret-ui-gallery/src/ui/pages/ai_attachments_demo.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/ai/image_demo.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/ai/prompt_input_docs_demo.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/ai/queue_demo.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/ai/chain_of_thought_demo.rs`
    - `apps/fret-ui-gallery/src/driver/demo_assets.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/aspect_ratio/images.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/aspect_ratio/demo.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/avatar/mod.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/card/mod.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/image_object_fit/mod.rs`
    - `docs/crate-usage-guide.md`
    - `docs/component-author-guide.md`
    - `docs/workstreams/resource-loading-fearless-refactor-v1/ECOSYSTEM_INSTALLER_COMPOSITION.md`
    - `ecosystem/fret-ui-shadcn/README.md`
    - `ecosystem/fret-ui-material3/README.md`
    - `apps/fretboard/src/scaffold/templates.rs`
      (`todo_template_mounts_generated_assets_when_ui_assets_are_enabled`,
      `simple_todo_template_mounts_generated_assets_when_ui_assets_are_enabled`)
  - Remaining:
    - audit remaining first-party docs/examples outside the newly cleaned UI Gallery surfaces so
      shipped-media teaching stays on logical bundle requests and only truly deterministic harnesses
      keep inline image construction,
    - audit shadcn ecosystem recipes that ship icons/images so package-owned installers stay the
      default app integration story,
      and keep first-party docs/examples aligned when new ecosystem bundles are added.
  - Minimum surfaces:
    - cookbook examples,
    - UI Gallery,
    - bootstrap templates,
    - shadcn ecosystem recipes that ship icons/images,
    - ecosystem icon-pack install stories that currently rely on implicit global registry behavior.

- [~] RESLOAD-mig-710 Remove or archive superseded one-off resource helpers once the unified path is
      verified.
  - Current landed slice:
    - `ecosystem/fret-bootstrap::BootstrapBuilder` now keeps file-backed startup on the canonical
      `with_asset_startup(...) + AssetStartupPlan::{development_dir,development_manifest}(...)`
      path instead of duplicating native/package-dev dir/manifest one-off helpers.
    - `ecosystem/fret::FretApp` and `UiAppBuilder` now keep app-facing file-backed startup on that
      same `asset_startup(...) + AssetStartupPlan::{development_dir,development_manifest}(...)`
      contract and no longer advertise separate dir/manifest helpers.
    - `crates/fret-launch::WinitAppBuilder` now keeps low-level native startup on
      `with_asset_startup(...) + AssetStartupPlan::{development_dir,development_manifest}(...)`
      and no longer advertises separate dir/manifest helpers.
    - first-party usage docs now teach that bootstrap lane as one named startup contract instead of
      advertising duplicate builder helpers.
  - Remaining:
    - delete or archive any other one-off helper surfaces once they are proven redundant by the
      unified locator/startup story.
    - keep docs/examples from reintroducing helper-specific teaching for capabilities already
      modeled by `AssetStartupPlan`.
  - Deprecated compatibility names should remain only during migration and be deleted after:
    - the unified asset contract lands,
    - first-party callers move over,
    - docs/examples stop teaching the old names.

- [ ] RESLOAD-mig-720 Update workstream/docs alignment after the contract lands.
  - At minimum:
    - `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
    - `docs/shadcn-declarative-progress.md` if authoring guidance changes
