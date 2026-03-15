# Resource Loading Fearless Refactor v1 — TODO Tracker

Status: Draft

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `RESLOAD-{area}-{nnn}`

When completing an item, leave 1–3 evidence anchors and prefer small executable gates over prose.

## Design lock

- [ ] RESLOAD-adr-010 Add or update the hard-contract ADR(s) for the portable asset model.
  - Minimum scope:
    - locator kinds,
    - bundle/key identity,
    - capability gating for file/url,
    - revision/invalidation semantics,
    - loader diagnostics contract.

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

- [ ] RESLOAD-cap-040 Publish a first-class asset capability matrix for desktop/web/mobile.
  - Minimum capability axes:
    - bundled assets
    - embedded assets
    - raw files
    - URLs
    - file watching / hot reload
    - system font scan

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
    - `StaticAssetEntry` plus bulk `InMemoryAssetResolver::insert_*_entries(...)`
    - runtime host mounting via `crates/fret-runtime/src/asset_resolver.rs`
    - UI bridge helpers via `ecosystem/fret-ui-assets/src/asset_resolver.rs`
  - Remaining:
    - explicit external URI/path handoff contract
    - structured diagnostics surface
    - non-UI first-party resolver implementations

- [ ] RESLOAD-core-120 Decide the authoritative bundle identity model.
  - Required outcomes:
    - app bundle story,
    - ecosystem/library bundle story,
    - no collision-prone global string soup.

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

- [ ] RESLOAD-svg-400 Replace dedicated SVG file helpers with the unified asset locator story.

- [ ] RESLOAD-svg-410 Decide the short-term SVG text policy and enforce it in docs/tests.
  - Preferred truthful baseline:
    - outlines for UI icons/illustrations,
    - no silent promises for arbitrary `<text>`.

- [ ] RESLOAD-svg-420 Plan the long-term shared SVG-text font environment path.
  - The SVG renderer should not permanently own an unrelated `fontdb` universe.

- [ ] RESLOAD-img-430 Move image loading onto the shared locator/resolver contract while preserving
      the existing async/UI invalidation ergonomics.

## Public API cleanup

- [x] RESLOAD-api-500 Rename or replace misleading install surfaces in `fret-ui-assets`.
  - `install()` must either perform complete wiring or stop being called `install()`.
  - Evidence:
    - `ecosystem/fret-ui-assets/src/app.rs`
    - `ecosystem/fret-ui-assets/src/advanced.rs`

- [ ] RESLOAD-api-510 Design the golden-path authoring API for app and ecosystem authors.
  - Target qualities:
    - logical-key first,
    - no filesystem assumptions,
    - easy escape hatches for file/url when explicitly needed.

- [~] RESLOAD-api-520 Remove path-first asset loading from cookbook/gallery/bootstrap teaching
      surfaces.
  - Current landed slice:
    - `apps/fret-cookbook/examples/icons_and_assets_basics.rs` now teaches
      `bundle locator + host resolver + static asset entries`
  - Remaining:
    - UI Gallery
    - bootstrap templates
    - dedicated dev/native escape-hatch examples should be labeled as such

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
    - shadcn ecosystem recipes that ship icons/images.

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
