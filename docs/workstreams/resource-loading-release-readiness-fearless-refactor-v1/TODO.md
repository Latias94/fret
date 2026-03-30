# Resource Loading Release Readiness Fearless Refactor v1 — TODO

Status: Active

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `RLRR-{nnn}`

## P0 release blockers

- [x] RLRR-001 Lock wasm startup to a bundled-only text baseline.
  - Required outcomes:
    - `crates/fret-render-wgpu/src/text/bootstrap.rs` chooses
      `ParleyShaper::new_without_system_fonts()` on wasm.
    - wasm startup no longer relies on `FRET_TEXT_SYSTEM_FONTS=*` to stay deterministic.
    - a regression test locks the renderer bootstrap split.
  - Evidence:
    - `crates/fret-launch/src/runner/web/gfx_init.rs`
    - `crates/fret-render-wgpu/src/text/bootstrap.rs`
    - `crates/fret-render-text/src/parley_shaper.rs`
    - `cargo nextest run -p fret-render-wgpu startup_parley_shaper_matches_platform_contract`
    - `cargo check -p fret-launch --target wasm32-unknown-unknown`

- [x] RLRR-002 Make URL asset truthfulness explicit on default product surfaces.
  - Required outcomes:
    - either land a first-party URL-capable resolver path,
    - or gate/remove default surfaces that imply built-in URL preview support.
  - Current landed slice:
    - the default AI attachment preview surface is still capability-gated instead of blindly
      emitting URL previews on hosts that cannot support them, and it now also requires the URL
      request to resolve into an actual image source before entering the image preview path,
    - the shipped web launch host now installs a first-party `UrlPassthroughAssetResolver`, so
      `AssetLocator::url(...)` can resolve through the general resolver contract there for
      browser-native image URL loading,
    - native/SVG/font URL lanes remain explicitly out of scope for the current first-party default.
  - Evidence:
    - `ecosystem/fret-ui-ai/src/elements/attachments.rs`
    - `ecosystem/fret-ui-assets/src/asset_resolver.rs`
    - `crates/fret-assets/src/lib.rs`
    - `crates/fret-launch/src/assets.rs`
    - `crates/fret-launch/src/runner/web/mod.rs`
    - `cargo nextest run -p fret-ui-ai attachment_preview`

- [x] RLRR-003 Close stage-1 drift between font asset identity and actual load path.
  - Required outcomes:
    - document one truthful owner story for startup and runtime font loading,
    - decide whether the short-term lane is “identity + byte injection” or a real asset-pipeline
      bridge,
    - leave diagnostics and API wording aligned with that choice.
  - Current landed slice:
    - startup bundled baseline still injects bytes into the renderer, but those bytes now resolve
      through the shared runtime asset resolver after baseline registration with no silent
      fallback path,
    - first-party runtime bundled-font top-ups can now use `Effect::TextAddFontAssets`, which
      resolves logical font assets through the shared asset resolver before renderer injection,
    - first-party UI Gallery bundled-font injection now also resolves default-profile faces through
      the shared runtime asset contract instead of masquerading as user-provided raw bytes,
    - repeated baseline installation now respects later runtime asset overrides for the same bundled
      font asset key, proving startup injection is no longer hard-wired to compiled-in bundled face
      bytes,
    - ADR/runtime/repo docs now explicitly describe startup bundled baseline loading as
      `asset identity -> resolver lookup -> renderer byte injection`, while
      `Effect::TextAddFontAssets` is the runtime asset-identity lane and `Effect::TextAddFonts`
      remains the runtime/user-provided raw-byte lane.
  - Evidence:
    - `crates/fret-launch/src/runner/font_catalog.rs`
    - `crates/fret-runtime/src/effect.rs`
    - `crates/fret-fonts/src/lib.rs`
    - `apps/fret-ui-gallery/src/driver/runtime_driver.rs`
    - `docs/adr/0147-font-stack-bootstrap-and-textfontstackkey-v1.md`
    - `docs/adr/0258-font-catalog-refresh-and-revisioning-v1.md`
    - `cargo nextest run -p fret-launch install_default_bundled_font_baseline`

- [x] RLRR-004 Decide and document the web `serif` guarantee boundary.
  - Required outcomes:
    - either add a bundled serif guarantee to the shipped profile,
    - or explicitly state that `serif` is not guaranteed on web in the release-facing docs.
  - Evidence:
    - `crates/fret-fonts/src/profiles.rs`
    - `crates/fret-runtime/src/font_bootstrap.rs`
    - `crates/fret-fonts/src/lib.rs`
    - `crates/fret-fonts/README.md`
    - `crates/fret-fonts/src/tests.rs`

- [x] RLRR-005 Publish the current web image decode limitation.
  - Required outcomes:
    - document the current “fetch bytes + Rust decode” path,
    - state its release tradeoff,
    - decide whether a browser-native decode follow-on is release-blocking or post-release.
  - Evidence:
    - `ecosystem/fret-ui-assets/src/asset_resolver.rs`
    - `ecosystem/fret-ui-assets/src/image_source.rs`
    - `ecosystem/fret-ui-assets/src/lib.rs`
    - `docs/workstreams/resource-loading-fearless-refactor-v1/CAPABILITY_MATRIX.md`

## P1 release hardening

- [x] RLRR-006 Refresh the capability matrix and release-readiness docs after each closure step.
  - Minimum outputs:
    - updated capability wording,
    - explicit platform notes for desktop vs wasm,
    - no stale “default URL support” or “web serif guarantee” wording left behind.
  - Current landed slice:
    - `CAPABILITY_MATRIX.md` now reflects the current web decode limitation and the explicit
      `configure_caches*` naming on the `fret-ui-assets` app-integration lane,
    - this release-readiness README now records the post-closure state for wasm fonts, URL
      truthfulness, font baseline ownership, web serif limits, and the partial-cache-setup naming
      cleanup,
    - the older resource-loading workstream now records that the `fret-ui-assets` `install*`
      compatibility aliases are deleted rather than merely deprecated.
  - Evidence:
    - `docs/workstreams/resource-loading-fearless-refactor-v1/CAPABILITY_MATRIX.md`
    - `docs/workstreams/resource-loading-release-readiness-fearless-refactor-v1/README.md`
    - `docs/workstreams/resource-loading-fearless-refactor-v1/README.md`
    - `docs/workstreams/resource-loading-fearless-refactor-v1/AUDIT.md`

- [x] RLRR-007 Run the release verification matrix for text/assets on native and wasm.
  - Minimum commands:
    - `cargo nextest run -p fret-launch runner::font_catalog`
    - `cargo nextest run -p fret-render-text --lib`
    - targeted `fret-render-wgpu` text bootstrap coverage if the bootstrap surface changes again
  - Current landed slice:
    - the `fret-launch` startup font-catalog matrix passes with the bundled-baseline/runtime
      resolver bridge in place,
    - the `fret-render-text` library matrix passes with the current bundled-only/web baseline and
      face-first test support surface,
    - the targeted `fret-render-wgpu` wasm/bootstrap split test still passes after the release
      truthfulness cleanup.
  - Evidence:
    - `cargo nextest run -p fret-launch runner::font_catalog`
    - `cargo nextest run -p fret-render-text --lib`
    - `cargo nextest run -p fret-render-wgpu startup_parley_shaper_matches_platform_contract`

## Completion rule

Do not close an item in this tracker with prose alone. Each closed item must leave behind at least
one executable gate or one precise evidence anchor that proves the runtime and the docs now say the
same thing.
