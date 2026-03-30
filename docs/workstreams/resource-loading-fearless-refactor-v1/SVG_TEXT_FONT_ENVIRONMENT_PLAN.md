# SVG Text Font Environment Plan

## Status

Current truthful baseline:

- the first-party SVG raster pipeline rejects text-bearing SVG assets,
- outline/icon/illustration SVGs remain supported,
- long-term SVG text support is deferred until it can share the framework font publication story.

This document defines that long-term path.

## Problem statement

Fret now has a deterministic startup font baseline story for main text:

- runner startup installs the framework-owned bundled baseline,
- runtime font loading can flow through `Effect::TextAddFontAssets`,
- raw user/runtime bytes can still flow through `Effect::TextAddFonts`,
- and runtime globals publish coarse font-environment metadata such as
  `BundledFontBaselineSnapshot`, `FontCatalogMetadata`, and `TextFontStackKey`.

What is still missing for SVG text is the authoritative "which exact fonts are currently usable for
rendering?" surface.

Today:

- `FontCatalogMetadata` is a best-effort family picker/diagnostics view,
- `TextFontStackKey` is a stable invalidation key,
- but neither one is enough to rebuild an SVG text font environment from the same bytes/identity
  the renderer is actually using.

That is why SVG text must stay rejected for now.

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
  - actual font bytes or a renderer-owned handle that can reproduce them.

Why this is required:

- `FontCatalogMetadata` only publishes family names and coarse axis hints,
- family names are not enough to rebuild deterministic SVG text rendering,
- SVG needs the same byte-level font authority as main text, not a second resolver path.

### Stage 2: build an SVG font bridge from the published inventory

Once Stage 1 exists, add a renderer-internal `SvgTextFontBridge` that:

- rebuilds a `usvg fontdb` only from the published renderer font inventory,
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
3. Add an SVG font-bridge revision surface in `fret-render-wgpu`.
4. Add diagnostics/tests before enabling any text-bearing SVG support.
5. Keep `SvgRenderError::TextNodesUnsupported` as the shipped baseline until the above exists.

## Evidence anchors

- `crates/fret-runtime/src/effect.rs`
- `crates/fret-runtime/src/font_catalog.rs`
- `crates/fret-launch/src/runner/font_catalog.rs`
- `crates/fret-render-wgpu/src/svg.rs`
