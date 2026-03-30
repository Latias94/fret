# Resource Loading Release Readiness Fearless Refactor v1

Status: active execution lane for pre-release cross-platform resource-loading closure

This workstream is a release-readiness follow-on to
`docs/workstreams/resource-loading-fearless-refactor-v1/`.

The earlier workstream reset the portable asset contract, capability vocabulary, startup surfaces,
and package-owned asset identity model. This follow-on focuses on the remaining places where the
current shipped runtime still says one thing and does another, especially on wasm and on
release-facing authoring surfaces.

This document is not an ADR. If this lane changes a hard contract, update the relevant ADR and
`docs/adr/IMPLEMENTATION_ALIGNMENT.md` separately.

Tracking files:

- `docs/workstreams/resource-loading-release-readiness-fearless-refactor-v1/README.md`
- `docs/workstreams/resource-loading-release-readiness-fearless-refactor-v1/TODO.md`
- `docs/workstreams/resource-loading-release-readiness-fearless-refactor-v1/MILESTONES.md`

Primary inputs:

- `docs/workstreams/resource-loading-fearless-refactor-v1/README.md`
- `docs/workstreams/resource-loading-fearless-refactor-v1/TODO.md`
- `docs/workstreams/resource-loading-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/font-system-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/font-mainline-fearless-refactor-v1/README.md`
- `docs/adr/0152-polychrome-glyphs-and-emoji-pipeline-v1.md`
- `docs/adr/0258-font-catalog-refresh-and-revisioning-v1.md`
- `docs/adr/0259-system-font-rescan-and-injected-font-retention-v1.md`
- `docs/adr/0317-portable-asset-locator-and-resolver-contract-v1.md`

## Why this workstream exists

Fret now has the right direction for portable resource loading, but release-readiness depends on
more than direction. It depends on whether the shipped runtime, diagnostics posture, and authoring
surfaces are truthful on every supported platform.

The highest-risk remaining gaps are no longer “design unknowns”; they are closure problems:

- wasm text startup still had a contract-vs-implementation mismatch,
- default product surfaces can still imply URL asset support without a first-party URL resolver,
- font asset identity and actual renderer load path are still split,
- bundled baseline guarantees are still incomplete for some generic families,
- web image loading still pays a byte-fetch + Rust decode path that should be explicit in release
  notes and capability docs.

This workstream exists to close or explicitly defer those gaps before release.

## Scope

In scope:

- release-facing cross-platform mismatches between documented asset/font contracts and shipped
  runtime behavior,
- wasm text/bootstrap truthfulness,
- truthful capability and authoring guidance for `AssetLocator::url(...)` and `AssetLocator::file(...)`,
- stage-1 convergence between bundled font asset identity and actual renderer bootstrap,
- documented generic-family guarantees for bundled profiles,
- explicit recording of web image decode tradeoffs.

Out of scope:

- a full rewrite of the general asset system,
- an editor/project asset database,
- redesigning every image path around browser-native decode in this lane,
- preserving misleading pre-release seams just because they already exist.

## Non-negotiable release invariants

1. wasm startup text must be bundled-only unless the framework has a truthful, first-party system
   font capability on that target.
2. `AssetLocator::url(...)` and `AssetLocator::file(...)` must remain capability-gated escape
   hatches, not default-path promises.
3. The framework-owned bundled baseline identity must not drift away from the actual renderer
   bootstrap path.
4. Bundled generic-family guarantees must be explicit, especially when a profile does not provide
   `serif`.
5. Release-facing docs must state when web image loading still falls back to byte fetch + Rust
   decode instead of browser-native decode.

## Current release-risk findings

Progress update (2026-03-30):

- `RLRR-001` is landed: wasm renderer bootstrap is now bundled-only by construction.
- `RLRR-002` is landed: the default AI attachment preview surface no longer implies built-in URL
  asset support unless the host resolver explicitly advertises `url` capability.
- `RLRR-003` is landed as a stage-1 closure: startup bundled-font injection now re-resolves bytes
  from the shared runtime asset resolver with no silent fallback path, and the ADR/runtime wording
  now explicitly distinguishes that startup baseline from the runtime `Effect::TextAddFonts` lane.
- `RLRR-004` is landed as an explicit denial: the default bundled profiles now say plainly that
  `serif` is not guaranteed on Web/WASM unless the app bundles it.
- `RLRR-005` is landed as an explicit limitation note: `fret-ui-assets` now documents that web
  logical-asset image loading still falls back to bytes + Rust decode unless a resolver provides a
  URL reference.
- the `fret-ui-assets` app-integration surface now keeps only `configure_caches*` names, so
  release-facing docs no longer imply that a partial cache setup call is a fully wired install.
- A first-party URL resolver is still not shipped; the remaining work is documentation and any
  future intentional productization of that lane.

### 1) wasm font contract drift

The web runner already documents “no system fonts” as the startup contract:

- `crates/fret-launch/src/runner/web/gfx_init.rs`
- `crates/fret-runtime/src/effect.rs`
- `docs/adr/0259-system-font-rescan-and-injected-font-retention-v1.md`

But the renderer bootstrap still constructed `ParleyShaper::new()` on every platform, which only
disabled system fonts if an environment variable happened to force it:

- `crates/fret-render-wgpu/src/text/bootstrap.rs`
- `crates/fret-render-text/src/parley_shaper.rs`

Release consequence:

- wasm could appear “bundled-only” in docs while still depending on a non-contractual code path.

### 2) URL asset support is not a default host capability

Some product surfaces already emit `AssetLocator::url(...)` requests:

- `ecosystem/fret-ui-ai/src/elements/attachments.rs`

But the current default host path still resolves through capability-gated asset resolvers:

- `ecosystem/fret-ui-assets/src/asset_resolver.rs`
- `crates/fret-assets/src/lib.rs`
- `crates/fret-assets/src/file_manifest.rs`

Current first-party resolvers do not advertise `url: true`, so the URL lane is closer to an
extension point than to a release-ready default.

Release consequence:

- default surfaces can imply a portable remote-preview story that the built-in host stack does not
  actually close.

Status note (2026-03-30):

- the default AI attachment preview surface is now capability-gated, so this mismatch no longer
  leaks through the shipped default surface;
- the unresolved part is still the absence of a first-party URL-capable resolver.

### 3) Font asset identity and actual renderer load path are still split

Startup now publishes bundled fonts into the runtime asset resolver and re-resolves those same
logical assets before renderer injection:

- `crates/fret-launch/src/runner/font_catalog.rs`
- `crates/fret-runtime/src/effect.rs`
- `crates/fret-fonts/src/lib.rs`

Status note (2026-03-30):

- stage-1 closure is now landed: startup bundled baselines have one truthful owner story
  (`asset identity -> resolver lookup -> renderer byte injection`),
- `Effect::TextAddFonts` is now documented as the runtime/user-provided raw-byte lane after
  startup,
- full asset-pipeline unification is still future work rather than an implied current guarantee.

### 4) Web serif guarantees are still open

The default bundled profiles currently guarantee `sans` and `monospace`, not `serif`:

- `crates/fret-fonts/src/profiles.rs`
- `crates/fret-runtime/src/font_bootstrap.rs`

Release consequence:

- generic `serif` behavior is not something the framework can honestly promise on wasm/web today.

### 5) Web image decode still needs an explicit release note

The current web image path resolves asset bytes and decodes through the Rust-side pipeline rather
than a browser-native decode lane:

- `ecosystem/fret-ui-assets/src/asset_resolver.rs`
- `ecosystem/fret-ui-assets/src/image_source.rs`

Release consequence:

- long tables, attachment-heavy UIs, and lower-memory devices may pay higher CPU and memory costs
  than a browser-native decode path would.

## First execution slice

The first release-readiness slice is intentionally small and high-leverage, and it is now landed:

- wasm renderer bootstrap now uses `ParleyShaper::new_without_system_fonts()`
  unconditionally in `crates/fret-render-wgpu/src/text/bootstrap.rs`,
- a focused regression test now locks that platform split,
- the release-readiness tracker can treat this as the first closed item before widening into URL or
  asset-pipeline follow-up work.

Verification:

- `cargo nextest run -p fret-render-wgpu startup_parley_shaper_matches_platform_contract`
- `cargo nextest run -p fret-render-text --lib`
- `cargo nextest run -p fret-launch runner::font_catalog`
- `cargo check -p fret-launch --target wasm32-unknown-unknown`

## Done means

This workstream is done only when every release-facing mismatch above is either:

- closed with code + gate + evidence, or
- explicitly deferred with a named limitation in the release notes and capability docs.
