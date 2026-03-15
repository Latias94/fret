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
    - `ecosystem/fret-ui-assets/src/svg_file.rs`
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
    - `fret::assets::{AssetStartupPlan, AssetStartupMode}` now gives the `fret` facade one named
      startup policy object for selecting development vs packaged publication on the builder path,
    - `fret-bootstrap::assets::{AssetStartupPlan, AssetStartupMode}` now gives direct bootstrap
      users the same named policy object and keeps `BootstrapBuilder::with_asset_startup(...)` on
      the non-`fret` builder path,
    - `FretApp::asset_startup(...)` lowers that policy onto the default app-facing startup
      surface while preserving fail-early builder semantics,
    - `UiAppBuilder::with_asset_startup(...)` keeps the same policy available on the explicit
      advanced builder lane,
    - the plan intentionally lowers to the existing ordered builder registrations:
      `asset_dir(...)`, `asset_manifest(...)`, `with_bundle_asset_entries(...)`, and
      `with_embedded_asset_entries(...)`,
    - `AssetStartupMode::preferred()` now provides one shared app-facing heuristic for
      `native+debug => Development` vs packaged/web/mobile/release => `Packaged`,
    - `AssetStartupPlan::development_bundle_dir_if_native(...)` now lets app/tooling code keep one
      portable startup-plan expression instead of repeating per-call-site `cfg` branches,
    - generated `--surface fret` modules now publish `preferred_startup_plan()` /
      `preferred_startup_mode()` and route `mount(builder)?` through `with_asset_startup(...)`,
      so native debug startup automatically uses the file-backed development lane while
      packaged/web/mobile keeps the compiled bundle lane,
    - missing selected lanes now fail as startup configuration errors instead of silently falling
      through to ad-hoc runtime glue.
  - Remaining:
    - watcher/hot-reload policy is still implicit in the native file-manifest resolver layer,
    - `fret-launch` still does not publish an equivalent shared build-profile contract for the
      lowest-level non-bootstrap startup surface.

- [~] RESLOAD-pack-210 Define the bootstrap/build-tool integration point.
  - Candidates:
    - `fret-launch`
    - `fret-bootstrap`
    - future `fretboard` asset manifest tooling
  - Current landed slice:
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
    - `fretboard new {simple-todo,todo} --ui-assets` now consumes that generated startup surface
      directly, so the first-party scaffold owns the preferred mode choice without bespoke app
      code.
  - Remaining:
    - decide whether `fret-launch` should adopt the same preferred-mode policy for the lowest-level
      non-bootstrap startup surfaces,
    - wire the same story through any future `fret-bootstrap` / `fret-launch` packaging helpers.

## Font baseline unification

- [ ] RESLOAD-font-300 Make bundled font baseline deterministic on every platform before first-frame
      text work.

- [~] RESLOAD-font-310 Ensure desktop/web/mobile all publish the same conceptual font-environment
      snapshot shape, even when capabilities differ.
  - Current landed slice:
    - `fret_runtime::BundledFontBaselineSnapshot` now gives runners one explicit runtime global for
      the framework-owned bundled baseline contract.
    - web and the current native winit startup path now both install bundled default fonts and
      publish the current `fret-fonts::default_profile()` identity (profile name, bundle id,
      logical asset keys, declared roles, guaranteed generics) before startup font-environment
      initialization.
    - native intentionally keeps `FontFamilyDefaultsPolicy::None`, so system-font augmentation
      remains an additive capability instead of redefining the baseline identity.
    - local iOS target evidence now exists:
      - `cargo check -p fret-launch --target aarch64-apple-ios`
  - Remaining:
    - add stable Android target evidence once the local/CI environment provides NDK clang
      toolchains
    - add mobile-specific diagnostics or startup gates beyond shared native runner wiring

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
      `StaticAssetEntry` instead of existing only as ad-hoc byte bags for `Effect::TextAddFonts`.
    - runner startup now consumes the same default profile manifest for both:
      - mounting package-owned bundled font assets into the shared runtime resolver,
      - injecting renderer font bytes from `BundledFontProfile::font_bytes()`.
    - regression coverage now proves the startup helper resolves bundled font faces through the
      runtime asset resolver and does not register duplicate font-asset resolver layers on
      repeated baseline installation.
  - Remaining:
    - define how mobile/package builds surface framework-owned bundled fonts on the builder/startup
      lane

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
      `src/generated_assets.rs` stub and mount it through `generated_assets::mount(builder)?` when
      `--ui-assets` is enabled
    - those generated modules now also publish `preferred_startup_plan()` /
      `preferred_startup_mode()` so the scaffold owns native-debug vs packaged startup behavior
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
      `bundle locator + UI helper + static asset entries` for package-owned ecosystem resources
    - `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs` is explicitly labeled as the
      native/dev file-path escape hatch instead of the portable default story
  - Remaining:
    - UI Gallery
    - bootstrap templates
    - any remaining dedicated dev/native escape-hatch examples should be labeled as such

- [~] RESLOAD-api-530 Decide deprecation/removal sequencing for:
  - `ImageSource::from_file_path(...)`
  - `SvgFileSource::from_file_path(...)`
  - Current landed slice:
    - both direct file-path constructors are now explicitly deprecated in
      `ecosystem/fret-ui-assets/src/{image_source.rs,svg_file.rs}` while internal bridge code
      moved onto crate-private constructors so the compatibility seam stays available without
      teaching it as the default authoring path
  - Remaining:
    - decide whether the lower-level host locator bridge helpers should also be deprecated, or stay
      as the long-term non-UI compatibility seam
    - delete the deprecated constructors after first-party migration and non-UI bridge guidance are
      complete

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
      `debug.resource_loading` instead of requiring stringly JSON-pointer predicates.
    - `ecosystem/fret-bootstrap/src/ui_diagnostics/debug_snapshot_predicates.rs` now centralizes
      recent debug-snapshot predicate evaluation for both docking and resource-loading surfaces, and
      `assert` / `wait` / `drag` script steps can consume the new resource-loading predicates from
      recent snapshots.
    - `crates/fret-assets/src/file_manifest.rs` now reports a typed runtime
      `AssetLoadError::StaleManifestMapping` when a file-backed manifest entry still resolves the
      logical bundle/key mapping but its mapped host file path is gone.
    - `crates/fret-runtime/src/asset_resolver.rs` now keeps `stale_manifest_requests` and
      `stale_manifest` recent outcomes distinct from true missing bundle assets.
    - `crates/fret-diag/src/stats/resource_loading.rs` plus
      `crates/fret-diag/src/registry/checks/builtin_post_run/resource_loading.rs` now add
      post-run `fretboard diag` gates for:
      - asset-load missing-bundle counter max,
      - asset-load stale-manifest counter max,
      - unsupported file/url counter max,
      - external-reference-unavailable counter max,
      - revision-change counter max,
      - bundled font baseline source equality.

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
      - `SvgFileSource` staying truthful about requiring an external file reference instead of
        silently treating byte-only bundle assets as native files.
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
      `.asset_dir("assets")` as the default first-contact story.
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
    - `docs/crate-usage-guide.md`
    - `docs/component-author-guide.md`
    - `docs/workstreams/resource-loading-fearless-refactor-v1/ECOSYSTEM_INSTALLER_COMPOSITION.md`
    - `ecosystem/fret-ui-shadcn/README.md`
    - `ecosystem/fret-ui-material3/README.md`
    - `apps/fretboard/src/scaffold/templates.rs`
      (`todo_template_mounts_generated_assets_when_ui_assets_are_enabled`,
      `simple_todo_template_mounts_generated_assets_when_ui_assets_are_enabled`)
  - Remaining:
    - continue migrating UI Gallery snippets/pages that still rely on inline demo image buffers
      when the intent is to teach shipped asset ownership rather than deterministic in-memory
      rendering, prioritizing AI/media/file-preview surfaces over pure image-fit/aspect-ratio demos,
    - audit shadcn ecosystem recipes that ship icons/images so package-owned installers stay the
      default app integration story,
      and keep first-party docs/examples aligned when new ecosystem bundles are added.
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
