# SVG Text Font Environment Plan

## Status

Current truthful baseline:

- the first-party SVG raster pipeline rejects text-bearing SVG assets,
- the renderer backend now has an internal bridge seed that can rebuild `usvg fontdb` from the
  current approved text collection and generic mappings, but the shipped SVG raster path does not
  consume it yet,
- outline/icon/illustration SVGs remain supported,
- long-term SVG text support is deferred until shared invalidation and deterministic gates exist.

This document defines that long-term path.

## Problem statement

Fret now has a deterministic startup font baseline story for main text:

- runner startup installs the framework-owned bundled baseline,
- runtime font loading can flow through `Effect::TextAddFontAssets`,
- raw user/runtime bytes can still flow through `Effect::TextAddFonts`,
- and runtime globals publish coarse font-environment metadata such as
  `BundledFontBaselineSnapshot`, `FontCatalogMetadata`, and `TextFontStackKey`.

What was missing for SVG text was the authoritative "which exact fonts are currently usable for
rendering?" surface.

That gap now has a first landed slice:

- runtime globals publish the identity/revision side through
  `RendererFontEnvironmentSnapshot`,
- and the renderer can now rebuild a `usvg fontdb` from the current approved text collection
  without loading system fonts independently.

What is still missing is the end-to-end raster wiring:

- the shipped SVG raster path still rejects `<text>`,
- bridge rebuilds are not yet keyed into SVG raster/cache invalidation,
- and deterministic SVG-text gates do not exist yet.

That is why SVG text must still stay rejected for now.

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
    `text_font_stack_key`, and accepted renderer source records for bundled startup,
    asset-request-backed runtime injection, and raw runtime font bytes,
  - `crates/fret-launch/src/runner/font_catalog.rs` now records those source records from
    `install_default_bundled_font_baseline`, `TextAddFontAssets`, and `TextAddFonts`,
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/debug_snapshot_{impl,types}.rs` now exports the
    same inventory under `debug.resource_loading.font_environment.renderer_font_*`.
- remaining gap:
  - the current snapshot intentionally stays identity/fingerprint-first at runtime-global scope,
    while the actual SVG bridge rehydration now lives renderer-locally.

Add a runtime-visible snapshot that answers:

- which font sources are currently installed in the renderer,
- which logical asset identities produced them when available,
- and which monotonic revision should invalidate SVG text caches.

Minimum shape:

- a new runtime global such as `RendererFontEnvironmentSnapshot`,
- monotonic `revision`,
- per-font source records containing:
  - source lane (`bundled_startup`, `asset_request`, `raw_runtime_bytes`),
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
  - generic sans/serif/monospace mapping in the bridge now follows the renderer's current text
    policy instead of host/system discovery,
  - focused renderer coverage now locks the bridge seed to export bundled-only `Inter`,
    `JetBrains Mono`, and matching generic mappings.
- remaining gap:
  - the bridge is not yet wired into the shipped `render_*_fit_mode(...)` SVG raster path,
  - SVG raster/cache invalidation is not yet keyed to the shared font-environment revision,
  - and `<text>` remains rejected in the shipped raster path.

Once Stage 1 exists, add a renderer-internal `SvgTextFontBridge` that:

- rebuilds a `usvg fontdb` only from the renderer's current approved text collection and the
  shared font-environment publication model,
- never calls `load_system_fonts()`,
- keys rebuilds and raster cache invalidation off the shared font-environment revision,
- remains empty/disabled when no SVG-text support is enabled.

This keeps `usvg` as an implementation detail while removing the hidden platform font lane.

### Stage 3: re-enable a limited, tested SVG text subset

Do not jump from "rejected" to "arbitrary SVG text works".

Promote support only after focused gates exist for:

- bundled-only SVG text on wasm and native producing deterministic results,
- runtime-added font assets participating in SVG text after the shared revision bumps,
- missing-font cases failing explicitly instead of silently choosing a host font,
- cache invalidation across `TextAddFontAssets`, `TextAddFonts`, and any system-font augmentation
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
   `TextAddFontAssets`, `TextAddFonts`) through that snapshot.
3. Wire the existing SVG font-bridge seed into the actual SVG raster/cache invalidation path in
   `fret-render-wgpu`.
4. Add diagnostics/tests before enabling any text-bearing SVG support.
5. Keep `SvgRenderError::TextNodesUnsupported` as the shipped baseline until the above exists.

Progress note:

- items 1 through 3 now have first landed slices,
- items 4 and 5 remain open at shipped-path scope because the actual raster path still rejects
  `<text>`.

## Evidence anchors

- `crates/fret-runtime/src/effect.rs`
- `crates/fret-runtime/src/font_catalog.rs`
- `crates/fret-launch/src/runner/font_catalog.rs`
- `crates/fret-render-text/src/{parley_font_db.rs,parley_shaper.rs}`
- `crates/fret-render-wgpu/src/{renderer/config.rs,svg.rs,text/fonts.rs,text/tests.rs}`
- `crates/fret-render-wgpu/src/svg.rs`
