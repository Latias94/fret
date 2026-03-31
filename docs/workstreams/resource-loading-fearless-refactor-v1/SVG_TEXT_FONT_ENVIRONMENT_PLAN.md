# SVG Text Font Environment Plan

## Status

Current truthful baseline:

- the renderer-owned first-party SVG raster path can now rasterize text-bearing SVGs when a
  bridge-backed parse is diagnostics-clean against the current approved text environment,
- unresolved text-bearing SVGs still fail closed instead of guessing a host font,
- the low-level `SvgRenderer::render_*_fit_mode(...)` helpers still reject text-bearing SVG assets
  unless they are explicitly fed from the bridge,
- outline/icon/illustration SVGs remain supported,
- shared diagnostics/export now exist under `debug.resource_loading.svg_text_bridge`,
- and broader SVG text support is still deferred until deterministic end-to-end gates and
  supported-scope decisions exist.

This document defines that long-term path.

## Problem statement

Fret now has a deterministic startup font baseline story for main text:

- runner startup installs the framework-owned bundled baseline,
- runtime font loading can flow through `Effect::TextAddFontAssets`,
- and runtime globals publish coarse font-environment metadata such as
  `BundledFontBaselineSnapshot`, `FontCatalogMetadata`, and `TextFontStackKey`.

What was missing for SVG text was the authoritative "which exact fonts are currently usable for
rendering?" surface.

That gap now has a first landed slice:

- runtime globals publish the identity/revision side through
  `RendererFontEnvironmentSnapshot`,
- and the renderer can now rebuild a `usvg fontdb` from the current approved text collection
  without loading system fonts independently,
- and the renderer-owned SVG raster path now consumes that bridge for text-bearing SVGs only when
  the bridge diagnostics are clean.

What is still missing now is the promotion/hardening layer:

- low-level direct `svg.rs` helpers still reject `<text>` unless explicitly bridged,
- deterministic SVG-text gates beyond the current bundled-only subset do not exist yet,
- and advanced SVG text scope (`textPath`, broader shaping parity, cross-platform golden outputs)
  is still intentionally unresolved.

That is why SVG text support must still stay narrow, diagnostics-gated, and fail-closed for now.

## Non-negotiable contract rules

If Fret re-enables SVG text in the future, it must obey all of the following:

1. SVG text must not load system fonts independently inside the SVG pipeline.
2. SVG text must be invalidated by the same effective font-environment revision as main text.
3. SVG text must only see framework/runtime-approved font bytes, not a second host-specific font
   universe.
4. wasm/mobile/native must keep the same conceptual contract:
   supported fonts come from the published Fret text environment, not hidden platform discovery.
5. Missing or unsupported SVG text fonts must remain diagnosable through the same resource-loading
   vocabulary instead of silently degrading to host guesses.

## Recommended staged plan

### Stage 1: publish a real renderer font inventory

Status note (2026-03-30):

- landed slice:
  - `fret_runtime::RendererFontEnvironmentSnapshot` now publishes a monotonic `revision`,
    `text_font_stack_key`, and accepted renderer source records for bundled startup and
    asset-request-backed runtime injection,
  - `crates/fret-launch/src/runner/font_catalog.rs` now records those source records from
    `install_default_bundled_font_baseline` and `TextAddFontAssets`,
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/debug_snapshot_{impl,types}.rs` now exports the
    same inventory under `debug.resource_loading.font_environment.renderer_font_*`.
- remaining gap:
  - the current snapshot intentionally stays identity/fingerprint-first at runtime-global scope,
    while the actual SVG bridge rehydration still lives renderer-locally.

Add a runtime-visible snapshot that answers:

- which font sources are currently installed in the renderer,
- which logical asset identities produced them when available,
- and which monotonic revision should invalidate SVG text caches.

Minimum shape:

- a new runtime global such as `RendererFontEnvironmentSnapshot`,
- monotonic `revision`,
- per-font source records containing:
  - source lane (`bundled_startup`, `asset_request`),
  - optional logical identity (`AssetRequest` or equivalent asset key/bundle pair),
  - stable byte fingerprint,
  - enough information for diagnostics and provenance; actual bridge rehydration may stay
    renderer-local instead of being copied into runtime globals.

Why this is required:

- `FontCatalogMetadata` only publishes family names and coarse axis hints,
- family names are not enough to rebuild deterministic SVG text rendering,
- SVG needs the same font authority as main text, not a second resolver path,
- and the bridge decision should preserve layering by keeping bridge rehydration in the renderer
  instead of pushing more font bytes through runtime globals.

### Stage 2: build an SVG font bridge from the published inventory

Status note (2026-03-30):

- landed slice:
  - `fret-render-text::ParleyShaper::{family_name_for_id,for_each_font_environment_blob}` can now
    enumerate deduped blobs from the current approved text collection and map the current generic
    family ids back to names,
  - `fret-render-wgpu::TextSystem::build_svg_text_font_db()` and
    `Renderer::build_svg_text_font_db_for_bridge()` now rebuild a `usvg::fontdb::Database` only
    from that live renderer text collection,
  - `crates/fret-render-wgpu/src/svg.rs` now also has an internal bridge-backed render helper and
    an end-to-end test proving that text-bearing SVG can render when explicitly fed from the
    renderer-built bridge `fontdb`,
  - registered text-bearing SVGs now also thread the current `text_font_stack_key` into
    `SvgRasterKey`, while outline-only SVGs keep a zero key, so cache identity is already aligned
    with the renderer text environment before shipped `<text>` support is turned on,
  - generic sans/serif/monospace mapping in the bridge now follows the renderer's current text
    policy instead of host/system discovery,
  - `crates/fret-render-wgpu/src/svg.rs` now also has structured diagnostics for
    bridge-backed SVG text parses, recording:
    - explicit font-family selection misses,
    - successful fallback hops, and
    - post-layout missing glyphs,
  - focused renderer coverage now locks both the bridge seed and those diagnostics to a
    bundled-only environment so host system-font drift cannot change the expected outcome,
  - the renderer-owned shipped SVG raster path now consumes that bridge for text-bearing SVGs, but
    only admits parses whose bridge diagnostics are clean,
  - `fret-launch` now publishes the most recently observed bridge snapshot into
    `fret_runtime::RendererSvgTextBridgeDiagnosticsSnapshot`,
  - `fret-bootstrap` now exports that state under `debug.resource_loading.svg_text_bridge`, and
    resource-loading predicates can now assert clean-vs-dirty bridge outcomes, selection misses,
    missing glyphs, and fallback hops.
- remaining gap:
  - the low-level `render_*_fit_mode(...)` SVG helpers still keep the text-free baseline,
  - `textPath` / advanced shaping cases remain out of scope,
  - and broader deterministic runtime/mobile/web gates are still missing.

Once Stage 1 exists, add a renderer-internal `SvgTextFontBridge` that:

- rebuilds a `usvg fontdb` only from the renderer's current approved text collection and the
  shared font-environment publication model,
- never calls `load_system_fonts()`,
- keys rebuilds and raster cache invalidation off the shared font-environment revision,
- remains empty/disabled when no SVG-text support is enabled.

This keeps `usvg` as an implementation detail while removing the hidden platform font lane.

### Stage 3: re-enable a limited, tested SVG text subset

Do not jump from "rejected" to "arbitrary SVG text works".

Status note (2026-03-30):

- landed slice:
  - the renderer-owned SVG raster path now supports ordinary text-bearing SVGs when the bridge
    parse is diagnostics-clean,
  - unresolved font-family or missing-glyph cases still fail closed,
  - focused renderer tests now cover both admitted and rejected paths under the bundled-only gate,
  - and the shared diagnostics surface now preserves the last observed bridge result for scripts
    and bundles even after subsequent raster-cache hits.
- remaining gap:
  - the low-level direct SVG helpers still keep the old text-free contract,
  - and the admitted subset is still intentionally narrow.

Promote support only after focused gates exist for:

- bundled-only SVG text on wasm and native producing deterministic results,
- runtime-added font assets participating in SVG text after the shared revision bumps,
- missing-font cases failing explicitly instead of silently choosing a host font,
- cache invalidation across `TextAddFontAssets` and any system-font augmentation
  that the main text environment already exposes.

Initial supported scope should stay conservative:

- ordinary `<text>` and basic `<tspan>` composition first,
- `textPath`, advanced shaping edge cases, and browser-grade document rendering remain out of
  scope until proven.

### Stage 4: decide whether the bridge is enough

After Stage 3 lands, explicitly evaluate whether mirroring the font inventory into `usvg fontdb`
is good enough.

Promotion criteria:

- if the bridge provides acceptable layout/shaping parity for the supported SVG subset, keep it,
- if the bridge still drifts too far from Fret's main text behavior, move to a stronger
  "shape-with-Fret, then emit outlines" architecture for SVG text.

Important rule:

- the bridge can be a temporary implementation strategy,
- but it must still be fed by the shared Fret font environment, not by host font discovery.

## Anti-goals

This plan does not aim to:

- make Fret promise full browser SVG text compatibility,
- keep legacy "whatever system fonts happen to exist" behavior,
- expose raw filesystem or host-font assumptions in UI/widget code,
- or make SVG text a backdoor around the main text asset contract.

## Concrete follow-up items

1. Add the runtime/global font-inventory snapshot described in Stage 1.
2. Thread successful font injection (`install_default_bundled_font_baseline`,
   `TextAddFontAssets`) through that snapshot.
3. Keep the renderer-owned bridge snapshot aligned with the shared diagnostics vocabulary as new
   SVG text cases are admitted.
4. Add broader deterministic wasm/mobile/runtime-font gates before widening supported scope.
5. Keep unresolved text-bearing SVGs fail-closed until a stronger cross-platform parity story
   exists for any newly admitted SVG text features.

Progress note:

- items 1 through 5 now have first landed slices,
- the remaining work is no longer "make shipped SVG text possible at all", but "broaden the now
  shared-contract diagnostics-gated slice with deterministic cross-platform evidence".

## Evidence anchors

- `crates/fret-runtime/src/effect.rs`
- `crates/fret-runtime/src/font_catalog.rs`
- `crates/fret-launch/src/runner/font_catalog.rs`
- `crates/fret-render-text/src/{parley_font_db.rs,parley_shaper.rs}`
- `crates/fret-render-wgpu/src/{renderer/config.rs,renderer/svg/{mod.rs,prepare.rs},svg.rs,text/fonts.rs,text/tests.rs}`
- `crates/fret-render-wgpu/src/svg.rs`
