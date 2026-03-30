# Resource Loading Fearless Refactor v1 — Milestones

## M0 — Truthfulness and design lock

Deliverables:

- The current incorrect logic is documented explicitly.
- The wasm compile break is fixed.
- The portability/capability matrix is written down in a way that matches reality.
- The ADR/design documents for the new asset contract are ready for implementation.
- Accepted ADR coverage now includes the icon ownership bridge (`0065`) and the general portable
  locator/resolver contract (`0317`).
- The current capability truth is published in
  `docs/workstreams/resource-loading-fearless-refactor-v1/CAPABILITY_MATRIX.md`.

Exit criteria:

- A contributor can answer “what is the default portable asset story in Fret?” with one sentence.
- `cargo check -p fret-launch --target wasm32-unknown-unknown` passes.

## M1 — Core asset contract

Deliverables:

- A core asset contract exists in a core crate.
- The contract defines:
  - locator kinds,
  - bundle identity,
  - revision semantics,
  - loader/resolver responsibilities,
  - capability reporting.

Exit criteria:

- Images, SVGs, and fonts can all point at the same conceptual asset identity model.
- `path` and `url` are explicitly modeled as escape hatches, not the main story.

## M2 — Deterministic font baseline

Deliverables:

- Every platform starts from a framework-owned bundled font baseline.
- Desktop/web/mobile share one conceptual font-environment publication flow.
- System font scanning is layered on top as an optional capability, not the baseline identity.

Status note (2026-03-30):

- web and the shared native winit startup path now both install the same framework-owned bundled
  baseline before startup font-environment publication,
- the runtime now also publishes `RendererFontEnvironmentSnapshot` so bundled startup, runtime
  asset-request injection, and raw runtime font bytes share one renderer-visible source inventory
  with a monotonic revision and diagnostics-facing source records,
- `fret-render-wgpu` can now also rebuild a `usvg::fontdb::Database` from the renderer's current
  approved text collection plus current generic mappings, so Stage 2 no longer depends on
  publishing extra raw font bytes into runtime globals,
- current local evidence includes `cargo check -p fret-launch --target aarch64-apple-ios`, so the
  same startup contract is at least compile-validated on iOS,
- desktop system-font refresh now has a focused runner-level gate proving it augments the live
  catalog without replacing the bundled baseline snapshot or seeding desktop family defaults.

Exit criteria:

- The same app can rely on one documented baseline text environment across desktop and web.
- The remaining platform differences are explicit capability differences, not startup accidents.
- Any remaining SVG-text gap is reduced to raster wiring/cache invalidation and supported-scope
  decisions, not to missing runner-visible font provenance or a missing backend bridge seed.

## M3 — Unified image and SVG loading

Deliverables:

- Image loading uses the shared asset contract.
- SVG loading uses the shared asset contract.
- Dedicated file-only SVG helper concepts are deleted or reduced to thin compatibility shims.
- Revision/invalidation semantics are shared.

Status note (2026-03-30):

- image loading and the current SVG bridge now both sit on the shared locator/resolver contract,
- the first-party SVG raster path no longer silently loads system fonts for SVG `<text>`,
- the renderer backend now has a bridge seed that rebuilds `usvg fontdb` from the current text
  collection and generic mappings without reopening a hidden host-font lane,
- renderer-local SVG bridge diagnostics now record explicit font-family misses, fallback hops, and
  missing glyphs under bundled-only gates,
- the renderer-owned shipped SVG raster path now admits text-bearing SVGs only when those bridge
  diagnostics are clean and otherwise still fails closed, and
- that long-term bridge is now documented in
  `docs/workstreams/resource-loading-fearless-refactor-v1/SVG_TEXT_FONT_ENVIRONMENT_PLAN.md`.

Exit criteria:

- There is no separate “image path story” and “SVG path story”.
- Missing asset / decode failure / unsupported capability are diagnosable through one model.

## M4 — Authoring surface reset

Deliverables:

- The golden-path authoring API is bundle/key based.
- Development manifests/directories and compile-time embedded assets can all mount through the
  same builder/startup surface with one ordering model.
- `fret-launch` owns the lowest-level asset startup contract, and higher facades reuse it instead
  of forking separate startup-policy types.
- The first-party startup surface can name the development-vs-packaged choice explicitly
  (`AssetStartupPlan` + `AssetStartupMode`) instead of teaching ad-hoc branching at each app
  entry point.
- Desktop-native startup can also opt into explicit development reload automation through the same
  startup family (`AssetReloadPolicy`) instead of hiding invalidation behind UI-local helper
  globals.
- Generated `--surface fret` asset modules can participate on both the builder lane
  (`mount(builder)`) and the app setup lane (`Bundle` / `install(app)`).
- Ecosystem libraries have one documented ownership rule for package resources and icon-pack
  participation.
- Cookbook, gallery, and bootstrap examples teach the portable path.
- Misleading install/setup APIs are renamed, removed, or completed.

Exit criteria:

- New users are no longer taught repo-relative asset paths as the default Fret story.
- Packaged/web/mobile-friendly embedded assets do not require dropping to ad-hoc setup hooks when
  the app is otherwise using the `fret` builder surface.
- App authors can compose ecosystem installers without having to know whether a dependency's
  shipped bytes are mounted through icon registries, package bundles, or both.
- Ecosystem authors can ship namespaced assets without runtime packaging knowledge.

## M5 — Cleanup, deprecation, and hardening

Deliverables:

- Legacy path-first helpers are deprecated or removed.
- Legacy partial-install compatibility aliases are removed; partial cache setup keeps explicit
  `configure_caches*` naming until a fully wired bootstrap surface exists.
- Deprecated UI-specific reload aliases are removed once first-party migration is complete.
- The explicit M5 cleanup checklist for the former compatibility surfaces is burned down:
  - `UiAssetsReloadEpoch`
  - `bump_ui_assets_reload_epoch(...)`
  - `FRET_DEV_RELOAD_UI_ASSETS_TRIGGER_PATH`
  - `.fret/ui_assets.touch`
- Diagnostics and regression gates cover:
  - capability mismatches,
  - startup baseline drift,
  - revision-driven invalidation,
  - missing bundle assets.

Exit criteria:

- The new model is simpler than the old model for app authors.
- The framework no longer has three independent resource-loading stories.

## Done means

This workstream is done only when Fret has all of the following:

- one portable default asset identity model,
- one truthful capability story,
- one deterministic font baseline strategy,
- one unified image/SVG loading pipeline,
- and user-facing docs/examples that teach the correct model first.
