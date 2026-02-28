# Renderer Parity Audit (2026-02) — Fret vs Reference Sources

This is an audit note (non-authoritative). Contracts live in `docs/adr/`.

Scope:

- Renderer-facing semantics and portability constraints.
- What the renderer contract can support for “typical component ecosystems” (Radix/shadcn/Base UI/
  Material-like recipes) and editor-grade UI surfaces.
- Not in scope: overlay/input policy details (those belong in `ecosystem/*` and dedicated ADRs).

## Reference sources (local, pinned)

- Zed/GPUI: `repo-ref/zed`, `repo-ref/gpui-component`
- Flutter (Skia/Impeller layer model reference): `repo-ref/flutter`, `repo-ref/flutter_liquid_glass`
- Vello (scene encoding + blend/layer vocabulary): `repo-ref/vello`
- wgpu / WebGPU constraints: `repo-ref/wgpu`

## What “renderer parity” means for Fret

For Fret, parity is outcome-driven and bounded:

- The contract must be implementable on **wgpu + WebGPU**, not just desktop GPUs.
- Effects and layers must have **explicit bounds** and **deterministic degradation** (budgets).
- The display list must be stable: `fret-core::Scene` is the mechanism substrate; policy stays out.

## Current Fret renderer contract coverage (high level)

Fret already covers the majority of “self-rendered UI framework” requirements:

- Strictly ordered display list with stack scopes and validation.
- Core primitives: quads (fill + border + radii), paths, images, SVG, text, viewport surfaces.
- Masks and clip paths (bounded).
- Multi-pass effects substrate (RenderPlan) with budgets + degradation.
- Extensibility: bounded materials + bounded custom effects (versioned WGSL prelude).

### Capability status snapshot (coarse)

This is a coarse “what should a component ecosystem assume is available?” snapshot:

| Capability | Status | Notes |
| --- | --- | --- |
| Strict op ordering | ✅ | Adjacent batching only. |
| Rect/RRect clip | ✅ | Portable baseline. |
| Clip-path | ✅ (bounded) | May deterministically degrade under budgets/caps. |
| Gradient fills | ✅ | Includes Oklab option in paint vocabulary. |
| Text shaping + caret/hit-test | ✅ | Renderer-owned; editor-grade APIs exist. |
| Drop shadow | ✅ (bounded) | Recipe policy stays in ecosystem. |
| Backdrop blur/warp (“glass”) | ✅ (bounded) | Requires explicit bounds + budget-aware authoring. |
| Viewport surfaces (render targets) | ✅ | Contract vocabulary exists. |
| Streaming video import | ⚠️ partial | Native paths vary; wasm external import is constrained. |
| Blend mode breadth | ⚠️ limited | Fixed-function-friendly subset (not CSS parity). |
| HDR / wide gamut correctness | ❌ (v1 baseline) | Treat as explicit future workstream + gates. |

## Alignment notes by capability

### 1) Layers / saveLayer / isolated opacity groups

Reference vocabulary:

- Flutter’s `saveLayer` is the canonical “protective offscreen layer” mechanism for opacity and
  filters (`repo-ref/flutter/engine/src/flutter/flow/layers/layer_state_stack.h`).
- Vello has explicit `push_layer` / `push_clip_layer` with blend modes (`repo-ref/vello/vello/src/scene.rs`).

Fret mapping:

- `CompositeGroupDesc` + group opacity is the direct “isolated group + saveLayerAlpha-like” surface.
- `EffectMode::FilterContent` maps to “render children to an offscreen intermediate, filter, composite”.

Gap / caution:

- Fret intentionally restricts blend modes to a portable subset. This is compatible with most UI
  component ecosystems, but is not CSS parity.

### 2) Drop shadows (BoxShadow-style)

Reference vocabulary:

- GPUI exposes CSS-like `BoxShadow { color, offset, blur_radius, spread_radius }`
  (`repo-ref/zed/crates/gpui/src/style.rs`, `repo-ref/zed/crates/gpui/examples/shadow.rs`).

Fret mapping:

- `EffectStep::DropShadowV1` covers the common UI “shadow stack” need (offset + blur + color) with
  bounded parameters; recipes belong in ecosystem crates.

Gap / caution:

- Spread radius is not a first-class mechanism-level shadow parameter in Fret v1; if parity-sensitive
  recipes require spread, this likely belongs in policy (inflate bounds + adjust blur radius) or a v2
  contract step with explicit boundedness.

### 3) Backdrop filters / “liquid glass”

Reference vocabulary:

- Flutter’s `BackdropFilter` layer model motivates explicit bounds + clip semantics
  (see `repo-ref/flutter_liquid_glass` for real-world “glass” stacks).
- Flutter’s engine notes emphasize that backdrop behavior interacts with clip cull rects and
  performance regressions (historically breaking changes around cull rect shrinking).

Fret mapping:

- `EffectMode::Backdrop` + `BackdropWarpV1/V2` + `PushBackdropSourceGroupV1` + CustomEffectV3 sources
  (`src_raw` + optional pyramid request) provide a bounded substrate for glass/refraction recipes.

Gap / caution:

- Deep nesting of backdrop effects will hit intermediate budgets and deterministically degrade; recipes
  must be authored with bounded nesting (and conformance gates should cover the “TargetExhausted”
  behavior).

### 4) Clip paths and mask layers

Reference vocabulary:

- Flutter: clip stack includes `clipPath` (AA vs non-AA) and often interacts with saveLayer.
- Vello: clipping/layer pushes are explicit and can act as blend/mask layers
  (`push_clip_layer`, luminance mask layers).

Fret mapping:

- `PushClipRect`, `PushClipRRect`, `PushClipPath` and `PushMask` cover typical UI needs; bounded
  computation `bounds` exist specifically to keep the GPU work deterministic.

Gap / caution:

- Fret does not currently model an explicit “AA on/off” knob at the contract surface. That is usually
  acceptable for UI, but if a future backend requires it for portability, it should be an explicit ADR
  decision (to avoid implicit behavior drift).

### 5) Text rendering (editor-grade)

Reference vocabulary:

- Zed exposes a subpixel text rendering mode and has platform-specific subpixel support constraints
  (`repo-ref/zed/assets/settings/default.json`, `repo-ref/zed/crates/gpui_macos/src/text_system.rs`).

Fret mapping:

- Renderer-owned shaping/wrapping (Parley) + caret/selection/hit-testing surfaces already align with
  “editor first” requirements. Subpixel binning + gamma correction are explicitly modeled in the
  renderer implementation.

Gap / caution:

- Keep text policy (typography presets, fallback injection) in ecosystem or dedicated workstreams.
  The renderer contract should remain stable and bounded.

### 6) Video / streaming surfaces (NV12/I420, external textures)

Reference vocabulary:

- Zed handles platform-specific remote video frames and does I420→NV12 conversion in some paths
  (`repo-ref/zed/crates/livekit_client/src/livekit_client/playback.rs`).
- wgpu/WebGPU defines `ExternalTexture` with multi-plane formats (e.g. NV12/YU12) and transfer
  function metadata (`repo-ref/wgpu/wgpu-types/src/texture/external_texture.rs`).

Fret mapping:

- Fret has an explicit ingest strategy vocabulary (`Owned`, `ExternalZeroCopy`, `GpuCopy`, `CpuUpload`)
  and a capability surface for streaming image paths (NV12 GPU convert, optional external import).

Gap / caution:

- On wasm/WebGPU, external texture import is frequently constrained. Today the wgpu backend reports
  external texture import as unsupported for wasm; this is a real ecosystem limitation for “zero-copy
  video” and should be treated as an explicit capability gate in recipes and demos.

### 7) Blend modes and compositing breadth

Reference vocabulary:

- Vello and Flutter have broad blend mode catalogs (including destination-aware compositing modes).

Fret mapping:

- Fret intentionally restricts `BlendMode` to a fixed-function-friendly subset to keep portability and
  avoid destination sampling dependencies.

Gap / caution:

- If a component ecosystem (or editor surface) truly needs destination-aware blend modes, that is a
  contract expansion that should be gated behind capabilities and conformance tests (not silently
  emulated).

### 8) Color spaces beyond sRGB (Oklab, wide gamut, transfer functions)

Reference vocabulary:

- GPUI includes Oklab conversions in platform shaders (`repo-ref/zed/crates/gpui_macos/src/shaders.metal`).

Fret mapping:

- Fret supports Oklab as a paint color space choice (for gradients/material-driven effects) and keeps
  the baseline render target story to `sRGB` vs `Linear`.

Gap / caution:

- While Fret has `RenderTargetColorEncoding` metadata, the default backend currently assumes a portable
  RGB baseline and may drop non-baseline encodings deterministically. Treat HDR/wide-gamut correctness
  as a dedicated workstream with explicit contracts and gates.

## Recommended next actions (contract + gates)

1) Add a “budgeted effects nesting” conformance scene (golden/diag) that:
   - forces intermediate exhaustion,
   - verifies deterministic degradation (no flicker / no non-deterministic aliasing),
   - is runnable on native and wasm (with wasm skips where capabilities are unavailable).

2) Document the current color-encoding behavior explicitly in user-facing docs:
   - what metadata is honored vs dropped today,
   - how callers should interpret the degradation counters.

3) Expand the renderer capability surface where needed for authoring ergonomics:
   - ensure every contract surface that can fail/degrade has a discoverable capability and/or
     deterministic fallback story.
