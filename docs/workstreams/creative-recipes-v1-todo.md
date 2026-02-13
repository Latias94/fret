# Creative Recipes (v1) — TODO

This is a working TODO list for landing the creative recipes surface end-to-end.

See:

- Overview: `docs/workstreams/creative-recipes-v1.md`
- Milestones: `docs/workstreams/creative-recipes-v1-milestones.md`

## P0 — Foundation (ecosystem)

- [x] Define `ResolveCtx` + `ResolvedWithFallback<T>` + `DegradationReason` in `ecosystem/fret-ui-kit`.
- [x] Add a minimal diagnostics seam for `RecipeDegraded { label, reason }` (best-effort).
- [x] Migrate existing recipes (glass/pixelate) to the shared resolve/fallback/report shape without
      changing their public wrapper signatures.
- [x] Create a `VisualCatalog`/`MaterialCatalog` skeleton API and decide where it is stored (app model vs service).

Next (recommended order):

1. M8: `ecosystem/fret-ui-magic` Phase 0 (finish seed components + UI gallery + diag)
2. M6 gap: “bloom-like” Tier B recipe example (threshold -> blur -> add)
3. M9: external texture imports (contract-path demo + capability-gated backend)

## P0 — Kernel primitives (scene + renderer)

- [x] Paint v1 (ADR 0233):
  - [x] Add `Paint` types and wire to `SceneOp::Quad`.
  - [x] Add conformance tests for gradient mapping + sanitization.
- [x] Materials v1 (ADR 0235):
  - [x] Add `MaterialId` and registry API.
  - [x] Implement baseline kinds: `DotGrid`, `Grid`, `Checkerboard`, `Stripe`, `Noise`, `Beam`,
        `Sparkle`, `ConicSweep`.
  - [x] Add deterministic sanitization + seed/time rules (ADR 0244).
- [x] Masks v1 (ADR 0239):
  - [x] Add `PushMask/PopMask` ops and gradient mask evaluation.
  - [x] Add conformance tests for edge coverage and effect/clip interaction.
- [x] Composite groups v1 (ADR 0247):
  - [x] Add `PushCompositeGroup/PopCompositeGroup` and blend mode support.
  - [x] Wire budgets + deterministic degradation.

## P0 — Motion/pointer seams (UI runtime)

- [x] Frame clock snapshot read (ADR 0240) exposed to widget contexts (non-reactive).
- [x] Pointer motion snapshot + local mapping helper surface (ADR 0243 / ADR 0238).

## P0 — Authoring seams (UI mechanism surface)

These are the `fret-ui` authoring surfaces that make the kernel primitives (mask/composite) usable
from ecosystem recipes without falling back to ad-hoc canvas-only hacks.

- [x] Add a mask layer element in `crates/fret-ui` that emits `SceneOp::PushMask/PopMask` (ADR 0239).
- [x] Add a compositing group element in `crates/fret-ui` that emits
      `SceneOp::PushCompositeGroup/PopCompositeGroup` (ADR 0247).

## P1 — Recipes and demos

- [x] MagicUI parity recipes/wrappers:
  - [x] `MagicCard` (pointer-follow radial gradient fill/border).
  - [x] `Lens` (radial mask + content scale + reduced-motion behavior).
  - [x] `BorderBeam` (animated border highlight + glow; Phase 0 uses gradients + additive composite).
  - [x] Patterns:
    - [x] Static dot/grid/stripe backgrounds (Tier B materials) + UI gallery + diag script.
    - [x] Animated variants (phase/offset-driven; reduced-motion fallback).
  - [x] Sparkles text (seeded sparkle field; reduced-motion fallback).
- [ ] Add perf baselines for MagicUI parity pages (optional follow-up):
  - [x] Perf scripts:
    - `tools/diag-scripts/ui-gallery-magic-patterns-perf-steady.json`
    - `tools/diag-scripts/ui-gallery-magic-sparkles-text-perf-steady.json`
  - [x] Seed policy preset:
    - `docs/workstreams/perf-baselines/policies/ui-gallery-magic-recipes.v1.json`
  - [x] Windows-local baselines (initial):
    - `docs/workstreams/perf-baselines/ui-gallery-magic-patterns.windows-local.v1.json`
    - `docs/workstreams/perf-baselines/ui-gallery-magic-sparkles-text.windows-local.v1.json`
  - [ ] Extend coverage (optional):
    - Lens (`tools/diag-scripts/ui-gallery-magic-lens-*-perf-steady.json`)
    - BorderBeam (`tools/diag-scripts/ui-gallery-magic-border-beam-*-perf-steady.json`)
    - Dock (`tools/diag-scripts/ui-gallery-magic-dock-*-perf-steady.json`)
    - Marquee (`tools/diag-scripts/ui-gallery-magic-marquee-*-perf-steady.json`)

## P1 — Effect steps extension

- [x] Implement `ColorMatrix` + `AlphaThreshold` steps (ADR 0236).
- [x] Add a “bloom-like” recipe example (threshold -> blur -> add) once blend groups exist.

## P1 — Sampled materials (v2a, catalog textures)

This is the recommended first step for ADR 0242: sampled materials bind a renderer-owned catalog
texture selected at registration time (no per-instance `ImageId` yet).

- [x] Define `BindingShape::ParamsPlusCatalogTexture` in the renderer material registry and
      capability-gate it (ADR 0124).
- [x] Add a small catalog texture set (blue-noise/dither) and wire upload + lifetime to the renderer.
- [x] Implement at least one sampled baseline material kind (e.g. a higher quality noise/dither
      overlay) that uses the catalog texture in the shader.
- [x] Add a conformance test for sampled material rendering and deterministic fallbacks.

## P1 — `fret-ui-magic` (Phase 0)

Land a MagicUI-aligned ecosystem crate that composes the existing kernel primitives via
`fret-ui-kit` recipes. The goal is fast “creative baseline” parity with stable fallbacks and
diagnostics, not perfect CSS parity.

- [x] Create `ecosystem/fret-ui-magic` (crate + minimal public surface).
- [x] Implement 3–5 seed components (Phase 0):
  - [x] `Lens`
  - [x] `MagicCard`
  - [x] `BorderBeam`
  - [x] `Marquee`
  - [x] `Dock`
- [x] Add UI gallery entries + `fretboard diag` scripts for each seed component:
  - [x] `Marquee`
  - [x] `Lens`
  - [x] `MagicCard`
  - [x] `BorderBeam`
  - [x] `Dock`
- [x] Verify deterministic behavior under `--fixed-frame-delta-ms` (diag-controlled time).
  - Evidence (scripts):
    - `tools/diag-scripts/ui-gallery-magic-patterns-fixed-frame-delta.json`
    - `tools/diag-scripts/ui-gallery-magic-marquee-fixed-frame-delta.json`
    - `tools/diag-scripts/ui-gallery-magic-border-beam-fixed-frame-delta.json`
    - `tools/diag-scripts/ui-gallery-magic-bloom-fixed-frame-delta.json`
    - `tools/diag-scripts/ui-gallery-magic-sparkles-text-fixed-frame-delta.json`
  - Recommended invocation (example):
    - `FRET_DIAG_SCREENSHOTS=1 cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-magic-patterns-fixed-frame-delta.json --fixed-frame-delta-ms 16 --check-pixels-changed ui-gallery-magic-pattern-stripe --include-screenshots --launch -- cargo run -p fret-ui-gallery`

## P1 — External texture imports (v1)

This closes the loop for “real import and run” beyond `ImageId` uploads: platform decoders and/or
external systems produce GPU textures and the runner imports them without leaking backend handles
to `fret-ui` (ADR 0234).

- [x] Land a “contract-path demo” that imports a per-frame `wgpu::TextureView` via runner deltas
      (`EngineFrameUpdate.target_updates`) and shows it in the UI as a `ViewportSurface`:
  - [x] Provide a small helper (`fret-launch`) that owns a stable `RenderTargetId` but updates the
        registry via `RenderTargetUpdate::Update` (not direct `renderer.update_render_target` calls).
  - [x] Demo app: `external_texture_imports_demo` (`apps/fret-demo --bin external_texture_imports_demo`)
        with:
    - [x] resize coverage (target reallocates on window resize)
    - [x] fit coverage (contain/cover/stretch panels)
    - [x] lifecycle coverage (toggle unregister/register via `V`)
  - [x] Diagnostics evidence (script v2 + screenshots):
    - [x] `fretboard diag run` works in `--launch` mode. Recommended build: `--features devtools-ws`.
    - [x] Script is verified to produce bundles + screenshots:
      - `tools/diag-scripts/external-texture-imports-contract-path.json`
  - [x] Perf evidence (steady-state baseline):
    - [x] `fretboard diag perf` steady-state script:
      - `tools/diag-scripts/external-texture-imports-contract-path-perf-steady.json`
    - [x] Seed policy preset is committed:
      - `docs/workstreams/perf-baselines/policies/external-texture-imports-contract-path.v1.json`
    - [x] A windows-local baseline JSON is committed:
      - `docs/workstreams/perf-baselines/external-texture-imports-contract-path.windows-local.v1.json`
- [ ] Add capability gating for a first “true external import” backend path (optional v1 follow-up):
  - [x] web (v0 copy path): `ExternalImageSource` → `Queue::copy_external_image_to_texture` →
        `RenderTargetUpdate::Update` (GPU copy, no CPU readback).
    - Evidence: `apps/fret-examples/src/external_texture_imports_web_demo.rs`,
      `apps/fret-demo-web/src/wasm.rs` (`demo=external_texture_imports_web_demo`)
  - [ ] web (v1 zero-copy): WebCodecs `VideoFrame` → WebGPU external texture / `ExternalTexture`
        (capability-gated) with deterministic fallback.
        Note: currently blocked on wgpu's WebGPU backend implementing `ExternalTexture`
        (wgpu v28: `wgpu/src/backend/webgpu.rs` contains `unimplemented!("ExternalTexture not implemented for web")`).
  - [x] native (v1 copy path): software decode → CPU upload (`Queue::write_texture`) →
        `RenderTargetUpdate::Update` with deterministic fallback.
    - Evidence: `apps/fret-examples/src/external_texture_imports_demo.rs` (`I` toggles source)
- [x] Add a concrete per-frame keepalive mechanism for truly ephemeral imported resources (ADR 0234 D3).
- [x] Decide and implement the minimal render target descriptor metadata seam needed by real imports:
      alpha semantics (`premul` vs `straight`), orientation/transform metadata, and frame timing hints
      for diagnostics (ADR 0234 D4).
