# Renderer Contracts (v1) — Contract Surface Summary

This repository is ADR-driven: authoritative contracts live in `docs/adr/`.
This document is a *navigational summary* of the renderer-facing contract surface that component
authors and subsystem maintainers can rely on.

If you are changing a “hard-to-change” behavior (ordering, compositing, clipping, text metrics,
effects, render targets, determinism), add/update an ADR and keep this summary in sync.

## Goals

- Provide a **portable, deterministic** display-list contract (`fret-core::Scene`) that can target
  native (wgpu) today and wasm (WebGPU) later.
- Support editor-grade UI needs: multi-window, multiple viewports, layered GPU rendering, and
  bounded multi-pass composition for effects.
- Keep “policy” (component sizing/spacing, dismissal rules, focus traps, hover intent, etc.) out of
  the renderer and out of `fret-ui` (mechanism-only).

## Non-goals (v1)

- Full CSS blend mode parity (destination sampling / complex Porter-Duff variants).
- Unbounded general render graphs with arbitrary dependencies.
- Arbitrary user-provided shaders by default (effects/materials are versioned, bounded, and gated).
- HDR / wide-gamut correctness as a default baseline (see **Color management** below).

## Known current limitations (implementation reality)

These are *not* contract guarantees; they are “what the default backend does today” notes to prevent
recipe authors from assuming behavior that is not yet closed.

- wasm/WebGPU: external texture import is frequently unavailable or backend-limited (capability-gated).
- Render target color encoding hints may be deterministically dropped when they conflict with the
  current portable RGB baseline.
- Blend mode support is intentionally restricted to a portable subset.

## Sources of truth (ADRs)

Core renderer contracts and semantics:

- Display list: `docs/adr/0002-display-list.md`
- Ordering/batching: `docs/adr/0009-renderer-ordering-and-batching.md`
- Shape semantics: `docs/adr/0030-shape-rendering-and-sdf-semantics.md`
- Color/compositing: `docs/adr/0040-color-management-and-compositing-contracts.md`
- RenderPlan substrate: `docs/adr/0116-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`
- Effect scene semantics: `docs/adr/0117-effect-layers-and-backdrop-filters-scene-semantics-v1.md`
- Budgets + degradation: `docs/adr/0118-renderer-intermediate-budgets-and-effect-degradation-v1.md`
- Streaming surfaces: `docs/adr/0119-streaming-images-and-video-surfaces.md`
- Capabilities: `docs/adr/0122-renderer-capabilities-and-optional-zero-copy-imports.md`
- Extensibility (materials/effects sandboxing): `docs/adr/0123-renderer-extensibility-materials-effects-and-sandboxing-v1.md`
- Effect clip masks / soft clipping: `docs/adr/0138-renderer-effect-clip-masks-and-soft-clipping-v1.md`
- Compositing groups: `docs/adr/0247-isolated-composite-groups.md` (and related)

## Layering and crate boundaries

- Contract types (portable, backend-agnostic): `crates/fret-core` and `crates/fret-render-core`
- Backend implementation (wgpu today): `crates/fret-render-wgpu`
- Public facade: `crates/fret-render`

Renderer backends must not leak backend handles into contract crates.

## Public contract surfaces (what downstream code should use)

### Display list

- `fret_core::scene::Scene` and `fret_core::scene::SceneOp`
- `fret_core::scene::{Paint, Mask, EffectChain, EffectStep, CompositeGroupDesc, BlendMode}`
- Resource handles: `ImageId`, `SvgId`, `TextBlobId`, `PathId`, `MaterialId`, `EffectId`,
  `RenderTargetId`

### Render targets (streaming / embedded viewports)

- Portable metadata types: `crates/fret-render-core/src/lib.rs`
  - `RenderTargetMetadata`, `RenderTargetIngestStrategy`, `RenderTargetColorEncoding`, etc.

### Extensibility: renderer-owned registries

- Materials: `fret_core::materials::{MaterialService, MaterialDescriptor, MaterialBindingShape}`
- Custom effects: `fret_core::effects::{CustomEffectService, CustomEffectDescriptorV1/V2/V3}`

These services return stable IDs (`MaterialId`, `EffectId`) rather than backend handles.

## Invariants and validation

- **Strict ordering is authoritative.** Renderers may batch only when doing so preserves the exact
  visual result implied by op order (adjacent batching).
- **DrawOrder is non-semantic.** It exists for diagnostics/authoring but does not override op order.
- Push/pop stacks must be balanced:
  - Transform, opacity, layer markers, clip, mask, effect, composite group, backdrop source group.
  - Scene validation exists and should be treated as a correctness gate.

## Primitives (v1)

The Scene supports a “UI-grade” primitive set that can cover most component library visuals:

- Solid and gradient fills; rounded rects with borders (`Quad`)
- Rounded-rect strokes (`StrokeRRect`)
- Paths (filled/stroked) via prepared `PathId`
- Images (full + region) and mask images
- SVGs (mask icon or RGBA image path)
- Text via prepared `TextBlobId` + paint/outline/shadow v1 surfaces
- Embedded viewport surfaces: `ViewportSurface { target: RenderTargetId }`

## Clipping and masking

### Clips

- Clip stack supports:
  - Axis-aligned rect and rrect clips
  - Path-based clips (`PushClipPath`) with explicit computation bounds
- `bounds` are **computation bounds**, not an implicit clip. Renderers use bounds to:
  - bound GPU work,
  - enable deterministic budgeting/degradation.

Degradation policy (high level):

- Clip-path may degrade deterministically (e.g. to a scissor-only bound) under budgets or
  unsupported capabilities. Degradation should be observable in diagnostics/perf counters.

### Masks

- Mask stack supports gradient and image masks.
- Masks are intended to be composable with effects and composite groups under bounded semantics.

## Effects and compositing

### Effect stack

- Effects are applied to a bounded region with a bounded chain (`EffectChain::MAX_STEPS`).
- `EffectMode` distinguishes:
  - `FilterContent` (render children to an intermediate, filter, composite)
  - `Backdrop` (sample already-rendered backdrop, filter, then draw children)
- `EffectQuality` is a hint used for deterministic budgeting/degradation.

Effect steps include (non-exhaustive):

- Blur, drop shadow, noise, color adjust/matrix, alpha threshold, pixelate, dither
- Backdrop warp (for “liquid glass”-style refraction recipes)
- Custom effects (v1/v2/v3) — bounded parameters + versioned WGSL prelude

### Composite groups

`CompositeGroupDesc` represents an isolated group rendered to an offscreen intermediate and then
composited back using a restricted, portable `BlendMode` subset and a group-level opacity
multiplier (saveLayerAlpha-like semantics).

### Backdrop source groups (v1)

Backdrop source groups exist to share a single backdrop snapshot (and optional bounded pyramid)
across multiple CustomV3 surfaces, enabling “glass stacks” without redundant backdrop captures.

## Extensibility surfaces

### Materials

Materials are framework-controlled “Tier B” stylization primitives:

- Stable kinds (`MaterialKind`) and stable binding shapes (`MaterialBindingShape`)
- Optional renderer-owned catalog textures (e.g. dither/noise) referenced by stable IDs
- Backend capability gating is explicit and deterministic (registration may fail as `Unsupported`)

### Custom effects

Custom effects are versioned, bounded WGSL snippets compiled into renderer-owned pipelines.

- v1: params-only
- v2: params + optional user image input
- v3: params + up to two user images + renderer-provided sources (`src_raw` + optional pyramid)

All versions include `max_sample_offset_px` (bounded chain padding contract).

## Text contract (renderer-owned)

The renderer owns shaping/wrapping and exposes prepared `TextBlobId` handles with metrics and
hit-testing surfaces.

Contract expectations:

- Deterministic shaping and line-breaking for a given font configuration and constraints.
- Stable caret and selection rectangle APIs for editor-grade widgets.
- Subpixel rendering is an implementation detail but must remain deterministic and capability-aware.

## Render targets and streaming surfaces

Render targets represent external producer surfaces (engine viewport, video frame, remote desktop
tile, etc.) painted into UI via `ViewportSurface`.

Renderer-visible metadata:

- Alpha mode and orientation
- Best-effort color encoding hints
- Requested vs effective ingest strategy (`Owned`, `ExternalZeroCopy`, `GpuCopy`, `CpuUpload`)
- Optional frame timestamps for diagnostics

Important portability note (current baseline):

- The default wgpu backend assumes a “portable RGB” baseline. If `RenderTargetColorEncoding`
  conflicts with this assumption, the backend may deterministically drop encoding hints and report
  the degradation through diagnostics counters.

## Color management (v1 baseline)

Baseline goal is correctness for typical UI content in `sRGB` and `Linear` workflows.

- sRGB vs linear render target selection is explicit at the contract surface.
- Wide-gamut / HDR / transfer-function correctness is not guaranteed as a default baseline.

If you need HDR correctness for a product surface, treat it as a contract expansion workstream and
add an ADR + conformance gates.

## Determinism, budgets, and degradation

Renderers must provide deterministic outcomes under resource pressure:

- Intermediate target budgets are bounded and capability-aware.
- When budgets are exceeded, effects/clips may degrade in *specified* ways (no random behavior).
- Degradation must be observable (counters + diagnostics snapshots).

## Observability and conformance gates

Recommended gates (non-exhaustive):

- Scene validation: stack balance + finite data invariants.
- Shader conformance (WGSL parse + WebGPU validator): `crates/fret-render-wgpu/src/renderer/tests.rs`
- Golden/diag coverage for:
  - nested clips (rect/rrect/path) + transforms
  - composite groups + group opacity
  - backdrop effects + forced budget degradation
  - text shaping + caret/selection rects + subpixel variants
